use crate::config::{detect_source_dirs, load_config};
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::scoring;
use crate::types::SpecSyncConfig;
use crate::validator::{compute_coverage, find_spec_files, get_schema_table_names, validate_spec};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

const SERVER_NAME: &str = "specsync";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_JSON_RPC_LINE_BYTES: usize = 1024 * 1024;
const MAX_CONFINEMENT_ENTRIES: usize = 100_000;
const MAX_MANIFEST_PREFLIGHTS: usize = 1_000;
const MAX_CONFIGURED_PATHS: usize = 1_000;
const AUTODETECT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "build",
    "out",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "coverage",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    "env",
    ".env",
    ".idea",
    ".vscode",
    ".DS_Store",
    "specs",
    "docs",
    "doc",
    ".github",
    ".gitlab",
    "migrations",
    "Pods",
    ".dart_tool",
    ".gradle",
    "bin",
    "obj",
];

/// Run the MCP server on stdio.
pub fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
    let server_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    if !server_root.is_dir() {
        return Err(format!(
            "MCP server root is not a directory: {}",
            server_root.display()
        ));
    }
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        let line = match read_mcp_line(&mut stdin) {
            Ok(Some(Ok(line))) => line,
            Ok(Some(Err(message))) => {
                let error = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": message }
                });
                let _ = writeln!(stdout, "{}", error);
                let _ = stdout.flush();
                continue;
            }
            Ok(None) | Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                let _ = writeln!(stdout, "{}", err);
                let _ = stdout.flush();
                continue;
            }
        };

        // JSON-RPC notifications never receive a response. Suppress them before
        // dispatch so even mutating tools cannot execute without an acknowledged ID.
        if request.get("id").is_none() {
            continue;
        }

        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let response = match method {
            "initialize" => Some(handle_initialize(id)),
            "tools/list" => Some(handle_tools_list(id, allow_write)),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_tools_call(id, &params, &server_root, allow_write))
            }
            "resources/list" => Some(handle_resources_list(id)),
            "resources/read" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_resources_read(id, &params, &server_root))
            }
            "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
            _ => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {method}") }
            })),
        };

        if let Some(resp) = response {
            let _ = writeln!(stdout, "{}", resp);
            let _ = stdout.flush();
        }
    }

    Ok(())
}

fn read_mcp_line<Reader: BufRead>(
    reader: &mut Reader,
) -> io::Result<Option<Result<String, &'static str>>> {
    let mut bytes = Vec::new();
    let mut saw_input = false;
    let mut too_large = false;

    loop {
        let (consumed, reached_newline) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                if !saw_input {
                    return Ok(None);
                }
                break;
            }
            saw_input = true;

            let newline = available.iter().position(|byte| *byte == b'\n');
            let content_end = newline.unwrap_or(available.len());
            if !too_large {
                let remaining = MAX_JSON_RPC_LINE_BYTES.saturating_sub(bytes.len());
                let copy_len = content_end.min(remaining);
                bytes.extend_from_slice(&available[..copy_len]);
                too_large = content_end > remaining;
            }

            (
                newline.map_or(available.len(), |index| index + 1),
                newline.is_some(),
            )
        };
        reader.consume(consumed);
        if reached_newline {
            break;
        }
    }

    if too_large {
        return Ok(Some(Err("JSON-RPC request exceeds the 1 MiB line limit")));
    }
    match String::from_utf8(bytes) {
        Ok(line) => Ok(Some(Ok(line))),
        Err(_) => Ok(Some(Err("JSON-RPC request is not valid UTF-8"))),
    }
}

fn handle_initialize(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        }
    })
}

fn handle_tools_list(id: Option<Value>, allow_write: bool) -> Value {
    let root_property = json!({
        "type": "string",
        "description": "Existing project directory at or below the server root"
    });
    let mut tools = vec![
        json!({
            "name": "specsync_check",
            "description": "Validate all spec files against source code. Returns errors, warnings, and pass/fail status for each spec.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "root": root_property.clone(),
                    "strict": {
                        "type": "boolean",
                        "description": "Treat warnings as errors (default: false)"
                    }
                },
                "additionalProperties": false
            }
        }),
        read_tool_schema(
            "specsync_coverage",
            "Get file and LOC coverage metrics. Shows which source files and modules have specs and which don't.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_list_specs",
            "List all spec files found in the project with their module names and status.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_score",
            "Score spec quality (0-100) with letter grades, breakdown by category, and improvement suggestions.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_issues",
            "Verify GitHub issue references in spec frontmatter. Checks that linked issues exist and reports their status (open/closed).",
            &root_property,
        ),
    ];

    if allow_write {
        tools.push(write_tool_schema(
            "specsync_generate",
            "Deterministically scaffold spec files for uncovered source modules at the server root. Returns paths of generated specs.",
        ));
        tools.push(write_tool_schema(
            "specsync_init",
            "Initialize a specsync.json config file at the server root with auto-detected source directories.",
        ));
    }

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn read_tool_schema(name: &str, description: &str, root_property: &Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": { "root": root_property },
            "additionalProperties": false
        }
    })
}

fn write_tool_schema(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }
    })
}

fn handle_tools_call(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
    allow_write: bool,
) -> Value {
    let Some(params) = params.as_object() else {
        return invalid_params(id, "tools/call params must be an object");
    };
    if let Some(key) = params
        .keys()
        .find(|key| key.as_str() != "name" && key.as_str() != "arguments")
    {
        return invalid_params(id, format!("Unknown tools/call parameter `{key}`"));
    }
    let Some(tool_name) = params.get("name").and_then(Value::as_str) else {
        return invalid_params(id, "tools/call parameter `name` must be a string");
    };
    let empty_arguments = serde_json::Map::new();
    let arguments = match params.get("arguments") {
        Some(Value::Object(arguments)) => arguments,
        Some(_) => {
            return invalid_params(id, "tools/call parameter `arguments` must be an object");
        }
        None => &empty_arguments,
    };

    let is_mutating = matches!(tool_name, "specsync_generate" | "specsync_init");
    let is_known = matches!(
        tool_name,
        "specsync_check"
            | "specsync_coverage"
            | "specsync_generate"
            | "specsync_list_specs"
            | "specsync_init"
            | "specsync_score"
            | "specsync_issues"
    );
    if !is_known {
        return tool_error(id, format!("Unknown tool: {tool_name}"));
    }
    if let Err(message) = validate_tool_arguments(tool_name, arguments) {
        return invalid_params(id, message);
    }
    if is_mutating && !allow_write {
        return tool_error(
            id,
            format!("Tool `{tool_name}` requires starting the MCP server with --allow-write"),
        );
    }

    let root = if is_mutating {
        server_root.to_path_buf()
    } else {
        match resolve_read_root(server_root, arguments.get("root").and_then(Value::as_str)) {
            Ok(root) => root,
            Err(message) => return tool_error(id, message),
        }
    };
    let arguments = Value::Object(arguments.clone());

    let result = match tool_name {
        "specsync_check" => tool_check(&root, &arguments),
        "specsync_coverage" => tool_coverage(&root),
        "specsync_generate" => tool_generate(&root, &arguments),
        "specsync_list_specs" => tool_list_specs(&root),
        "specsync_init" => tool_init(&root),
        "specsync_score" => tool_score(&root),
        "specsync_issues" => tool_issues(&root),
        _ => unreachable!("known tool was validated before dispatch"),
    };

    match result {
        Ok(content) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                }]
            }
        }),
        Err(msg) => tool_error(id, msg),
    }
}

fn validate_tool_arguments(
    tool_name: &str,
    arguments: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    const RETIRED_GENERATE_ARGUMENTS: &[&str] = &[
        "ai",
        "provider",
        "aiProvider",
        "ai_provider",
        "model",
        "aiModel",
        "ai_model",
        "apiKey",
        "api_key",
        "aiApiKey",
        "ai_api_key",
        "credential",
        "credentials",
        "baseUrl",
        "base_url",
        "aiBaseUrl",
        "ai_base_url",
        "timeout",
        "timeoutSecs",
        "timeout_secs",
        "aiTimeout",
        "ai_timeout",
        "command",
        "aiCommand",
        "ai_command",
    ];

    if tool_name == "specsync_generate"
        && let Some(name) = RETIRED_GENERATE_ARGUMENTS
            .iter()
            .find(|name| arguments.contains_key(**name))
    {
        return Err(format!(
            "MCP argument `{name}` was removed in spec-sync 5.0; `specsync_generate` is deterministic"
        ));
    }

    let allowed: &[(&str, &str)] = match tool_name {
        "specsync_check" => &[("root", "string"), ("strict", "boolean")],
        "specsync_coverage" | "specsync_list_specs" | "specsync_score" | "specsync_issues" => {
            &[("root", "string")]
        }
        "specsync_generate" | "specsync_init" => &[],
        _ => &[],
    };

    for (name, value) in arguments {
        let Some((_, expected_type)) = allowed.iter().find(|(allowed, _)| allowed == name) else {
            return Err(format!("Unknown argument `{name}` for tool `{tool_name}`"));
        };
        let valid = match *expected_type {
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            _ => false,
        };
        if !valid {
            return Err(format!(
                "Argument `{name}` for tool `{tool_name}` must be a {expected_type}"
            ));
        }
    }

    Ok(())
}

fn resolve_read_root(server_root: &Path, requested_root: Option<&str>) -> Result<PathBuf, String> {
    let Some(requested_root) = requested_root else {
        return Ok(server_root.to_path_buf());
    };
    let requested_path = Path::new(requested_root);
    if requested_path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err("Read root override must not contain parent traversal".to_string());
    }

    let candidate = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        server_root.join(requested_path)
    };
    let canonical = candidate.canonicalize().map_err(|_| {
        format!(
            "Read root override does not resolve to an existing directory: {}",
            candidate.display()
        )
    })?;
    if !canonical.is_dir() {
        return Err(format!(
            "Read root override is not a directory: {}",
            canonical.display()
        ));
    }
    if !canonical.starts_with(server_root) {
        return Err("Read root override escapes the configured server root".to_string());
    }

    Ok(canonical)
}

fn invalid_params(id: Option<Value>, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32602, "message": message.into() }
    })
}

fn tool_error(id: Option<Value>, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": message.into()
            }],
            "isError": true
        }
    })
}

// ─── Resource Handlers ──────────────────────────────────────────────────

fn handle_resources_list(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resources": [
                {
                    "uri": "specsync:///specs",
                    "name": "All Specs",
                    "description": "List all spec modules with metadata (name, path, version, status, score)",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///graph",
                    "name": "Dependency Graph",
                    "description": "Cross-module dependency graph with edges, cycles, and topological order",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///config",
                    "name": "Configuration",
                    "description": "Current specsync.json configuration",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///coverage",
                    "name": "Coverage Report",
                    "description": "File and LOC coverage metrics — which modules have specs and which don't",
                    "mimeType": "application/json"
                }
            ],
            "resourceTemplates": [
                {
                    "uriTemplate": "specsync:///specs/{module}",
                    "name": "Spec by Module",
                    "description": "Read a specific spec's full content with parsed frontmatter and score",
                    "mimeType": "text/markdown"
                }
            ]
        }
    })
}

fn handle_resources_read(id: Option<Value>, params: &Value, root: &Path) -> Value {
    let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");

    let result = match uri {
        "specsync:///specs" => resource_specs_list(root),
        "specsync:///graph" => resource_graph(root),
        "specsync:///config" => resource_config(root),
        "specsync:///coverage" => resource_coverage(root),
        _ if uri.starts_with("specsync:///specs/") => {
            let module = &uri["specsync:///specs/".len()..];
            resource_spec_by_module(root, module)
        }
        _ => Err(format!("Unknown resource URI: {uri}")),
    };

    match result {
        Ok((content, mime_type)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "contents": [{
                    "uri": uri,
                    "mimeType": mime_type,
                    "text": content
                }]
            }
        }),
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": msg }
        }),
    }
}

// ─── Resource Implementations ───────────────────────────────────────────

fn resource_specs_list(root: &Path) -> Result<(String, &'static str), String> {
    let (config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| {
            let content = std::fs::read_to_string(f).unwrap_or_default();
            let parsed = crate::parser::parse_frontmatter(&content);
            let score = scoring::score_spec(f, root, &config);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                    "depends_on": fm.depends_on,
                    "score": score.total,
                    "grade": score.grade,
                })
            } else {
                json!({
                    "path": relative,
                    "module": null,
                    "score": score.total,
                    "grade": score.grade,
                })
            }
        })
        .collect();

    let output = json!({ "specs": specs, "count": specs.len() });
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_spec_by_module(root: &Path, module: &str) -> Result<(String, &'static str), String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    // Find the spec file matching this module name
    for f in &spec_files {
        let content = match std::fs::read_to_string(f) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed = crate::parser::parse_frontmatter(&content);
        let matches = parsed
            .as_ref()
            .and_then(|p| p.frontmatter.module.as_deref())
            .map(|m| m == module)
            .unwrap_or(false);

        if matches {
            return Ok((content, "text/markdown"));
        }
    }

    Err(format!("No spec found for module: {module}"))
}

fn resource_graph(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let graph = build_dep_graph(root, &config.specs_dir);

    let nodes: Vec<Value> = graph
        .values()
        .map(|node| {
            json!({
                "module": node.module,
                "spec_path": node.spec_path,
                "depends_on": node.declared_deps,
                "files": node.files,
            })
        })
        .collect();

    // Build edges list
    let mut edges: Vec<Value> = Vec::new();
    for node in graph.values() {
        for dep in &node.declared_deps {
            edges.push(json!({
                "from": node.module,
                "to": dep,
            }));
        }
    }

    // Detect cycles
    let cycles = crate::deps::validate_deps(root, &config.specs_dir).cycles;
    let cycle_values: Vec<Value> = cycles.iter().map(|c| json!(c)).collect();

    // Topological order
    let topo = crate::deps::topological_sort(&graph);

    let output = json!({
        "modules": nodes,
        "edges": edges,
        "module_count": graph.len(),
        "edge_count": edges.len(),
        "cycles": cycle_values,
        "topological_order": topo,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_config(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let output = json!({
        "specs_dir": config.specs_dir,
        "source_dirs": config.source_dirs,
        "required_sections": config.required_sections,
        "exclude_dirs": config.exclude_dirs,
        "exclude_patterns": config.exclude_patterns,
        "schema_dir": config.schema_dir,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_coverage(root: &Path) -> Result<(String, &'static str), String> {
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage(root, &spec_files, &config);

    let file_coverage = if coverage.total_source_files == 0 {
        100.0
    } else {
        (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
    };

    let loc_coverage = if coverage.total_loc == 0 {
        100.0
    } else {
        (coverage.specced_loc as f64 / coverage.total_loc as f64) * 100.0
    };

    let uncovered_modules: Vec<Value> = coverage
        .unspecced_modules
        .iter()
        .map(|m| json!({ "name": m }))
        .collect();

    let uncovered_files: Vec<Value> = coverage
        .unspecced_file_loc
        .iter()
        .map(|(f, loc)| json!({ "file": f, "loc": loc }))
        .collect();

    let output = json!({
        "file_coverage_percent": (file_coverage * 100.0).round() / 100.0,
        "files_covered": coverage.specced_file_count,
        "files_total": coverage.total_source_files,
        "loc_coverage_percent": (loc_coverage * 100.0).round() / 100.0,
        "loc_covered": coverage.specced_loc,
        "loc_total": coverage.total_loc,
        "uncovered_modules": uncovered_modules,
        "uncovered_files": uncovered_files,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

// ─── Tool Implementations ────────────────────────────────────────────────

fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let config = load_confined_config(root)?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files: Vec<PathBuf> = find_spec_files(&specs_dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs)?;

    if spec_files.is_empty() && !allow_empty {
        return Err(format!(
            "No spec files found in {}/. Run specsync generate to scaffold specs.",
            config.specs_dir
        ));
    }

    Ok((config, spec_files))
}

fn load_confined_config(root: &Path) -> Result<SpecSyncConfig, String> {
    validate_known_config_files(root)?;
    validate_manifest_inputs(root)?;
    if config_requires_source_autodetection(root) {
        validate_autodetection_tree(root)?;
    }
    let config = load_config(root);

    let configured_path_count = 1usize
        + config.source_dirs.len()
        + usize::from(config.schema_dir.is_some())
        + config
            .modules
            .values()
            .map(|module| module.files.len())
            .sum::<usize>();
    if configured_path_count > MAX_CONFIGURED_PATHS {
        return Err(format!(
            "MCP configuration exceeds {MAX_CONFIGURED_PATHS} path entries"
        ));
    }

    let no_exclusions = HashSet::new();
    let source_exclusions: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let mut confinement_entries_seen = 0usize;

    validate_configured_path_with_budget(
        root,
        &config.specs_dir,
        "specs_dir",
        Some(&no_exclusions),
        &mut confinement_entries_seen,
    )?;
    for source_dir in &config.source_dirs {
        validate_configured_path_with_budget(
            root,
            source_dir,
            "source_dirs entry",
            Some(&source_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        validate_configured_path_with_budget(
            root,
            schema_dir,
            "schema_dir",
            Some(&no_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    for (module_name, module) in &config.modules {
        validate_module_name(module_name)?;
        for file in &module.files {
            validate_configured_path_with_budget(
                root,
                file,
                "configured module file",
                Some(&source_exclusions),
                &mut confinement_entries_seen,
            )?;
        }
    }

    Ok(config)
}

fn validate_known_config_files(root: &Path) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync/config.local.toml",
        ".specsync.toml",
        "specsync.json",
        ".specsync/hashes.json",
        "Cargo.toml",
        "Package.swift",
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
        "package.json",
        "pubspec.yaml",
        "go.mod",
        "pyproject.toml",
    ] {
        let candidate = root.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => validate_existing_path(root, &candidate, "project metadata file", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP configuration path {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Ok(())
}

fn selected_config_path(root: &Path) -> Option<PathBuf> {
    [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ]
    .iter()
    .map(|relative| root.join(relative))
    .find(|path| path.exists())
}

fn config_requires_source_autodetection(root: &Path) -> bool {
    let Some(config_path) = selected_config_path(root) else {
        return true;
    };
    let Some(content) = crate::config::read_config_file(&config_path) else {
        return false;
    };
    if config_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("json")
    {
        return serde_json::from_str::<Value>(&content).is_ok()
            && !content.contains("\"sourceDirs\"");
    }

    let mut top_level = true;
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            top_level = false;
            continue;
        }
        if top_level
            && line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "source_dirs")
        {
            return false;
        }
    }
    true
}

fn validate_manifest_inputs(root: &Path) -> Result<(), String> {
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0usize;
    validate_cargo_workspace_manifest(
        root,
        root,
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
    )?;
    validate_package_workspaces(root)?;
    validate_gradle_modules(root)?;
    validate_python_package_path(root)?;
    Ok(())
}

fn validate_autodetection_tree(root: &Path) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} during source autodetection preflight: {error}",
            root.display()
        )
    })?;
    let ignored: HashSet<&str> = AUTODETECT_IGNORED_DIRS.iter().copied().collect();
    let mut entries_seen = 0usize;
    let entries = WalkDir::new(&canonical_root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_str().unwrap_or("");
            if entry.depth() == 1 && (name.starts_with('.') || ignored.contains(name)) {
                return false;
            }
            !entry.file_type().is_dir() || (!name.starts_with('.') && !ignored.contains(name))
        });

    for entry in entries {
        entries_seen += 1;
        if entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP source autodetection preflight exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry = entry
            .map_err(|error| format!("Cannot inspect MCP source autodetection input: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink used by MCP source autodetection {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Symlink used by MCP source autodetection escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

fn validate_cargo_workspace_manifest(
    server_root: &Path,
    manifest_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
    validated: &mut HashSet<PathBuf>,
    manifests_seen: &mut usize,
) -> Result<(), String> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    match fs::symlink_metadata(&manifest_path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Cargo workspace manifest {}: {error}",
                manifest_path.display()
            ));
        }
    }
    validate_existing_path(
        server_root,
        &manifest_path,
        "Cargo workspace manifest",
        None,
    )?;
    let canonical_manifest = manifest_path.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP Cargo workspace manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    if validated.contains(&canonical_manifest) {
        return Ok(());
    }
    if !visiting.insert(canonical_manifest.clone()) {
        return Err(format!(
            "MCP Cargo workspace manifest cycle detected at {}",
            manifest_path.display()
        ));
    }
    *manifests_seen += 1;
    if *manifests_seen > MAX_MANIFEST_PREFLIGHTS {
        return Err(format!(
            "MCP Cargo workspace preflight exceeds {MAX_MANIFEST_PREFLIGHTS} manifests"
        ));
    }

    let content = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "Cannot read MCP Cargo workspace manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    for member in extract_manifest_toml_array(&content, "members", "[workspace]") {
        let member_root = validate_manifest_relative_candidate(
            server_root,
            manifest_dir,
            &member,
            "Cargo workspace member",
        )?;
        let nested_manifest = member_root.join("Cargo.toml");
        match fs::symlink_metadata(&nested_manifest) {
            Ok(_) => validate_cargo_workspace_manifest(
                server_root,
                &member_root,
                visiting,
                validated,
                manifests_seen,
            )?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP Cargo workspace member manifest {}: {error}",
                    nested_manifest.display()
                ));
            }
        }
    }

    visiting.remove(&canonical_manifest);
    validated.insert(canonical_manifest);
    Ok(())
}

fn validate_package_workspaces(root: &Path) -> Result<(), String> {
    let package_path = root.join("package.json");
    let content = match fs::read_to_string(&package_path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Ok(()),
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(_) => return Ok(()),
    };
    let workspace_patterns: Vec<&str> = match json.get("workspaces") {
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        Some(Value::Object(object)) => object
            .get("packages")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    let mut entries_seen = 0usize;
    for pattern in workspace_patterns {
        let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
        let base_dir = if base.is_empty() {
            root.to_path_buf()
        } else {
            validate_manifest_relative_candidate(root, root, base, "package workspace base")?
        };
        match fs::symlink_metadata(&base_dir) {
            Ok(_) => validate_existing_path(root, &base_dir, "package workspace base", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP package workspace base {}: {error}",
                    base_dir.display()
                ));
            }
        }
        if !base_dir.canonicalize().is_ok_and(|path| path.is_dir()) {
            continue;
        }
        let entries = match fs::read_dir(&base_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries {
            entries_seen += 1;
            if entries_seen > MAX_CONFINEMENT_ENTRIES {
                return Err(format!(
                    "MCP package workspace preflight exceeds {MAX_CONFINEMENT_ENTRIES} entries"
                ));
            }
            let entry = entry.map_err(|error| {
                format!(
                    "Cannot inspect entry beneath MCP package workspace base {}: {error}",
                    base_dir.display()
                )
            })?;
            validate_existing_path(root, &entry.path(), "package workspace entry", None)?;
            let canonical_entry = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP package workspace entry {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical_entry.is_dir() {
                continue;
            }
            let nested_package = entry.path().join("package.json");
            match fs::symlink_metadata(&nested_package) {
                Ok(_) => validate_existing_path(
                    root,
                    &nested_package,
                    "package workspace manifest",
                    None,
                )?,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "Cannot inspect MCP package workspace manifest {}: {error}",
                        nested_package.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_gradle_modules(root: &Path) -> Result<(), String> {
    let settings_path = ["settings.gradle.kts", "settings.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    let Some(settings_path) = settings_path else {
        return Ok(());
    };
    let settings = match fs::read_to_string(&settings_path) {
        Ok(settings) => settings,
        Err(_) => return Ok(()),
    };
    for line in settings
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("include"))
    {
        let mut search = line;
        while let Some(quote_start) = search.find('"') {
            let rest = &search[quote_start + 1..];
            let Some(quote_end) = rest.find('"') else {
                break;
            };
            let module = rest[..quote_end].trim_start_matches(':');
            if !module.is_empty() {
                validate_manifest_relative_candidate(root, root, module, "Gradle module path")?;
            }
            search = &rest[quote_end + 1..];
        }
    }
    Ok(())
}

fn validate_python_package_path(root: &Path) -> Result<(), String> {
    let src = root.join("src");
    match fs::symlink_metadata(&src) {
        Ok(_) => {
            validate_existing_path(root, &src, "Python source directory", None)?;
            return Ok(());
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Python source directory {}: {error}",
                src.display()
            ));
        }
    }

    let pyproject_path = root.join("pyproject.toml");
    let content = match fs::read_to_string(&pyproject_path) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };
    let name = extract_manifest_toml_value(&content, "name", "[project]")
        .or_else(|| extract_manifest_toml_value(&content, "name", "[tool.poetry]"));
    if let Some(name) = name {
        validate_manifest_relative_candidate(root, root, &name, "Python package path")?;
    }
    Ok(())
}

fn validate_manifest_relative_candidate(
    server_root: &Path,
    base: &Path,
    configured: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(configured);
    if configured.is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path without traversal: {configured}"
        ));
    }
    let candidate = base.join(relative);
    validate_path_or_ancestor(server_root, &candidate, label, None)?;
    Ok(candidate)
}

fn manifest_toml_section<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    let end = after
        .find("\n[")
        .map(|position| position + 1)
        .unwrap_or(after.len());
    Some(&after[..end])
}

fn extract_manifest_toml_array(content: &str, key: &str, section: &str) -> Vec<String> {
    let Some(section) = manifest_toml_section(content, section) else {
        return Vec::new();
    };
    for line in section.lines().map(str::trim) {
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        if candidate_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if !value.starts_with('[') || !value.ends_with(']') {
            return Vec::new();
        }
        return value[1..value.len() - 1]
            .split(',')
            .map(str::trim)
            .map(|item| {
                if item.starts_with('"') && item.ends_with('"') && item.len() >= 2 {
                    item[1..item.len() - 1].to_string()
                } else {
                    item.to_string()
                }
            })
            .filter(|item| !item.is_empty())
            .collect();
    }
    Vec::new()
}

fn extract_manifest_toml_value(content: &str, key: &str, section: &str) -> Option<String> {
    let section = manifest_toml_section(content, section)?;
    for line in section.lines().map(str::trim) {
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        if candidate_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
        return Some(value.to_string());
    }
    None
}

fn validate_configured_path_with_budget(
    root: &Path,
    configured: &str,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let path = Path::new(configured);
    if configured.is_empty() || path.is_absolute() {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path: {configured}"
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "MCP {label} must not contain traversal or an absolute prefix: {configured}"
        ));
    }

    validate_path_or_ancestor_with_budget(
        root,
        &root.join(path),
        label,
        tree_exclusions,
        confinement_entries_seen,
    )
}

fn validate_path_or_ancestor(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_path_or_ancestor_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_path_or_ancestor_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut current = candidate;
    loop {
        match fs::symlink_metadata(current) {
            Ok(_) => {
                let exclusions = (current == candidate).then_some(tree_exclusions).flatten();
                validate_existing_path_with_budget(
                    root,
                    current,
                    label,
                    exclusions,
                    confinement_entries_seen,
                )?;
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                current = current.parent().ok_or_else(|| {
                    format!(
                        "MCP {label} has no resolvable ancestor: {}",
                        candidate.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP {label} {}: {error}",
                    current.display()
                ));
            }
        }
    }
}

fn validate_existing_path(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_existing_path_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_existing_path_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} while validating {label}: {error}",
            root.display()
        )
    })?;
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP {label} {} (including symlink targets): {error}",
            candidate.display()
        )
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "MCP {label} escapes the configured server root: {}",
            candidate.display()
        ));
    }
    if let Some(exclusions) = tree_exclusions
        && canonical.is_dir()
    {
        validate_tree_confinement(
            &canonical_root,
            &canonical,
            label,
            exclusions,
            confinement_entries_seen,
        )?;
    }
    Ok(())
}

fn validate_tree_confinement(
    root: &Path,
    start: &Path,
    label: &str,
    exclusions: &HashSet<String>,
    entries_seen: &mut usize,
) -> Result<(), String> {
    let entries = WalkDir::new(start)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| exclusions.contains(name))
        });

    for entry in entries {
        *entries_seen += 1;
        if *entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP {label} confinement scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry =
            entry.map_err(|error| format!("Cannot inspect entry beneath MCP {label}: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink beneath MCP {label} {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "Symlink beneath MCP {label} escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

fn validate_spec_file_mappings(
    root: &Path,
    spec_files: &[PathBuf],
    exclude_dirs: &[String],
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    let exclusions: HashSet<String> = exclude_dirs.iter().cloned().collect();
    for spec_file in spec_files {
        validate_existing_path(root, spec_file, "spec file", None)?;
        let content = fs::read_to_string(spec_file)
            .map_err(|error| format!("Cannot read spec {}: {error}", spec_file.display()))?;
        let Some(parsed) = crate::parser::parse_frontmatter(&content) else {
            continue;
        };
        for file in &parsed.frontmatter.files {
            validate_configured_path_with_budget(
                root,
                file,
                "spec file mapping",
                Some(&exclusions),
                &mut confinement_entries_seen,
            )?;
        }
        for dependency in &parsed.frontmatter.depends_on {
            if crate::validator::is_cross_project_ref(dependency) {
                continue;
            }
            if !dependency.contains('/') && !dependency.contains('.') {
                validate_module_name(dependency)?;
            } else {
                validate_configured_path_with_budget(
                    root,
                    dependency,
                    "dependency reference",
                    None,
                    &mut confinement_entries_seen,
                )?;
            }
        }
        validate_consumed_by_paths(root, &parsed.body, &mut confinement_entries_seen)?;
    }
    Ok(())
}

fn validate_consumed_by_paths(
    root: &Path,
    body: &str,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut in_section = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "### Consumed By" {
            in_section = true;
            continue;
        }
        if in_section && (trimmed.starts_with("## ") || trimmed.starts_with("### ")) {
            break;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }
        for (index, value) in line.split('`').enumerate() {
            if index % 2 == 1 && value.rsplit_once('.').is_some() {
                validate_configured_path_with_budget(
                    root,
                    value,
                    "Consumed By file reference",
                    None,
                    confinement_entries_seen,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_module_name(module_name: &str) -> Result<(), String> {
    crate::commands::validate_module_name(module_name)
        .map_err(|error| format!("MCP module name is unsafe: {error}"))
}

fn validate_generated_module_names(module_names: &[String]) -> Result<(), String> {
    for module_name in module_names {
        validate_module_name(module_name)?;
    }
    Ok(())
}

fn tool_check(root: &Path, arguments: &Value) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = config
        .schema_dir
        .as_deref()
        .map(|schema_dir| crate::schema::build_schema(&root.join(schema_dir)))
        .unwrap_or_default();
    let strict = arguments
        .get("strict")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    // Classify changes for staleness detection
    let cache = crate::hash_cache::HashCache::load(root);
    let classifications = crate::hash_cache::classify_all_changes(root, &spec_files, &cache);
    let mut stale_entries: Vec<Value> = Vec::new();
    for classification in &classifications {
        let spec_rel = classification
            .spec_path
            .strip_prefix(root)
            .unwrap_or(&classification.spec_path)
            .to_string_lossy()
            .to_string();
        if classification.has(&crate::hash_cache::ChangeKind::Requirements) {
            stale_entries.push(json!({
                "spec": spec_rel,
                "reason": "requirements_changed",
                "message": "requirements changed — spec may need re-validation"
            }));
        }
    }

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<Value> = Vec::new();
    let mut all_warnings: Vec<Value> = Vec::new();
    let mut spec_results: Vec<Value> = Vec::new();

    for spec_file in &spec_files {
        let result = validate_spec(spec_file, root, &schema_tables, &schema_columns, &config);
        let spec_passed = result.errors.is_empty();

        spec_results.push(json!({
            "spec": result.spec_path,
            "passed": spec_passed,
            "errors": result.errors,
            "warnings": result.warnings,
            "export_summary": result.export_summary,
        }));

        for e in &result.errors {
            all_errors.push(json!(format!("{}: {e}", result.spec_path)));
        }
        for w in &result.warnings {
            all_warnings.push(json!(format!("{}: {w}", result.spec_path)));
        }

        total_errors += result.errors.len();
        total_warnings += result.warnings.len();
        if spec_passed {
            passed += 1;
        }
    }

    // Add staleness warnings into the warnings array for consistency
    for entry in &stale_entries {
        if let Some(msg) = entry["message"].as_str() {
            let spec = entry["spec"].as_str().unwrap_or("unknown");
            all_warnings.push(json!(format!("{spec}: {msg}")));
        }
    }

    let coverage = compute_coverage(root, &spec_files, &config);
    let staleness_warnings = stale_entries.len();
    let effective_warnings = total_warnings + staleness_warnings;
    let overall_passed = total_errors == 0 && (!strict || effective_warnings == 0);

    Ok(json!({
        "passed": overall_passed,
        "specs_checked": spec_files.len(),
        "specs_passed": passed,
        "total_errors": total_errors,
        "total_warnings": effective_warnings,
        "errors": all_errors,
        "warnings": all_warnings,
        "stale": stale_entries,
        "specs": spec_results,
        "coverage": {
            "file_percent": coverage.coverage_percent,
            "loc_percent": coverage.loc_coverage_percent,
        }
    }))
}

fn tool_coverage(root: &Path) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage(root, &spec_files, &config);

    let file_coverage = if coverage.total_source_files == 0 {
        100.0
    } else {
        (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
    };

    let loc_coverage = if coverage.total_loc == 0 {
        100.0
    } else {
        (coverage.specced_loc as f64 / coverage.total_loc as f64) * 100.0
    };

    let modules: Vec<Value> = coverage
        .unspecced_modules
        .iter()
        .map(|m| json!({ "name": m, "has_spec": false }))
        .collect();

    let uncovered_files: Vec<Value> = coverage
        .unspecced_file_loc
        .iter()
        .map(|(f, loc)| json!({ "file": f, "loc": loc }))
        .collect();

    Ok(json!({
        "file_coverage": (file_coverage * 100.0).round() / 100.0,
        "files_covered": coverage.specced_file_count,
        "files_total": coverage.total_source_files,
        "loc_coverage": (loc_coverage * 100.0).round() / 100.0,
        "loc_covered": coverage.specced_loc,
        "loc_total": coverage.total_loc,
        "uncovered_modules": modules,
        "uncovered_files": uncovered_files,
    }))
}

fn tool_generate(root: &Path, arguments: &Value) -> Result<Value, String> {
    const RETIRED_ARGUMENTS: &[&str] = &[
        "ai",
        "provider",
        "aiProvider",
        "ai_provider",
        "model",
        "aiModel",
        "ai_model",
        "apiKey",
        "api_key",
        "aiApiKey",
        "ai_api_key",
        "credential",
        "credentials",
        "baseUrl",
        "base_url",
        "aiBaseUrl",
        "ai_base_url",
        "timeout",
        "timeoutSecs",
        "timeout_secs",
        "aiTimeout",
        "ai_timeout",
        "command",
        "aiCommand",
        "ai_command",
    ];
    if let Some(name) = RETIRED_ARGUMENTS
        .iter()
        .find(|name| arguments.get(**name).is_some())
    {
        return Err(format!(
            "MCP argument `{name}` was removed in spec-sync 5.0. `specsync_generate` is deterministic; use your coding agent to enrich the generated spec."
        ));
    }
    if let Some(name) = arguments
        .as_object()
        .and_then(|object| object.keys().find(|name| name.as_str() != "root"))
    {
        return Err(format!(
            "Unknown MCP generate argument `{name}`. `specsync_generate` accepts only `root`."
        ));
    }

    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage(root, &spec_files, &config);
    validate_generated_module_names(&coverage.unspecced_modules)?;
    for module_name in &coverage.unspecced_modules {
        let destination = root
            .join(&config.specs_dir)
            .join(module_name)
            .join(format!("{module_name}.spec.md"));
        validate_path_or_ancestor(root, &destination, "generation destination", None)?;
    }
    let outcome = generate_specs_for_unspecced_modules_paths(root, &coverage, &config);

    Ok(json!({
        "generated": outcome.generated_paths,
        "count": outcome.generated,
    }))
}

fn tool_list_specs(root: &Path) -> Result<Value, String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| {
            let content = std::fs::read_to_string(f).unwrap_or_default();
            let parsed = crate::parser::parse_frontmatter(&content);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                })
            } else {
                json!({
                    "path": relative,
                    "module": null,
                    "version": null,
                    "status": null,
                    "files": [],
                })
            }
        })
        .collect();

    Ok(json!({
        "specs": specs,
        "count": specs.len(),
    }))
}

fn tool_init(root: &Path) -> Result<Value, String> {
    let config_path = root.join("specsync.json");
    validate_path_or_ancestor(root, &config_path, "initialization destination", None)?;
    validate_known_config_files(root)?;
    if config_path.exists() {
        return Ok(json!({
            "created": false,
            "message": "specsync.json already exists"
        }));
    }

    validate_autodetection_tree(root)?;
    validate_manifest_inputs(root)?;
    let detected_dirs = detect_source_dirs(root);

    let default = json!({
        "specsDir": "specs",
        "sourceDirs": detected_dirs,
        "requiredSections": [
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
            "Dependencies",
            "Change Log"
        ],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
    });

    let content = serde_json::to_string_pretty(&default).unwrap() + "\n";
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write specsync.json: {e}"))?;

    Ok(json!({
        "created": true,
        "source_dirs": detected_dirs,
        "message": "Created specsync.json"
    }))
}

fn tool_score(root: &Path) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;

    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| scoring::score_spec(f, root, &config))
        .collect();
    let project = scoring::compute_project_score(scores);

    let specs_json: Vec<Value> = project
        .spec_scores
        .iter()
        .map(|s| {
            json!({
                "spec": s.spec_path,
                "total": s.total,
                "grade": s.grade,
                "frontmatter": s.frontmatter_score,
                "sections": s.sections_score,
                "api": s.api_score,
                "depth": s.depth_score,
                "freshness": s.freshness_score,
                "suggestions": s.suggestions,
            })
        })
        .collect();

    Ok(json!({
        "average_score": (project.average_score * 10.0).round() / 10.0,
        "grade": project.grade,
        "total_specs": project.total_specs,
        "distribution": {
            "A": project.grade_distribution[0],
            "B": project.grade_distribution[1],
            "C": project.grade_distribution[2],
            "D": project.grade_distribution[3],
            "F": project.grade_distribution[4],
        },
        "specs": specs_json,
    }))
}

fn tool_issues(root: &Path) -> Result<Value, String> {
    use crate::github;
    use crate::parser::parse_frontmatter;

    let (config, spec_files) = load_and_discover(root, false)?;

    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = github::resolve_repo(repo_config, root)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;

    for spec_path in &spec_files {
        let content = match std::fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let fm = &parsed.frontmatter;
        if fm.implements.is_empty() && fm.tracks.is_empty() {
            continue;
        }

        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();

        let verification = github::verify_spec_issues(&repo, &rel_path, &fm.implements, &fm.tracks);

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();

        results.push(json!({
            "spec": rel_path,
            "valid": verification.valid.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
                "state": i.state,
            })).collect::<Vec<_>>(),
            "closed": verification.closed.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
            })).collect::<Vec<_>>(),
            "not_found": verification.not_found,
        }));
    }

    Ok(json!({
        "repo": repo,
        "total_valid": total_valid,
        "total_closed": total_closed,
        "total_not_found": total_not_found,
        "specs": results,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let config = json!({
            "specsDir": "specs",
            "sourceDirs": ["src"],
            "requiredSections": ["Purpose", "Public API"]
        });
        std::fs::write(
            tmp.path().join("specsync.json"),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        tmp
    }

    fn setup_project_with_spec(spec_name: &str, spec_content: &str) -> TempDir {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join(spec_name);
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join(format!("{spec_name}.spec.md")), spec_content).unwrap();
        tmp
    }

    // --- handle_initialize ---

    #[test]
    fn test_handle_initialize_response_format() {
        let resp = handle_initialize(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "specsync");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert!(resp["result"]["capabilities"]["resources"].is_object());
    }

    #[test]
    fn test_handle_initialize_null_id() {
        let resp = handle_initialize(None);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"]["protocolVersion"].is_string());
    }

    #[test]
    fn test_handle_initialize_string_id() {
        let resp = handle_initialize(Some(json!("req-42")));
        assert_eq!(resp["id"], "req-42");
    }

    // --- handle_tools_list ---

    #[test]
    fn test_handle_tools_list_returns_all_tools() {
        let resp = handle_tools_list(Some(json!(2)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_handle_tools_list_defaults_to_read_only_tools() {
        let resp = handle_tools_list(Some(json!(2)), false);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5);
        assert!(
            tools
                .iter()
                .all(|tool| tool["name"] != "specsync_generate" && tool["name"] != "specsync_init")
        );
    }

    #[test]
    fn test_handle_tools_list_tool_names() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"specsync_check"));
        assert!(names.contains(&"specsync_coverage"));
        assert!(names.contains(&"specsync_generate"));
        assert!(names.contains(&"specsync_list_specs"));
        assert!(names.contains(&"specsync_init"));
        assert!(names.contains(&"specsync_score"));
        assert!(names.contains(&"specsync_issues"));
    }

    #[test]
    fn test_handle_tools_list_all_have_schemas() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            assert!(
                tool["inputSchema"].is_object(),
                "Tool {} missing inputSchema",
                tool["name"]
            );
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        }
    }

    // --- handle_tools_call ---

    #[test]
    fn test_handle_tools_call_unknown_tool() {
        let tmp = setup_project();
        let params = json!({ "name": "nonexistent_tool", "arguments": {} });
        let resp = handle_tools_call(Some(json!(1)), &params, tmp.path(), false);
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_tools_call_custom_root() {
        let tmp = setup_project();
        let server_root = tmp.path().canonicalize().unwrap();
        // coverage tool should work with empty specs
        let params = json!({
            "name": "specsync_coverage",
            "arguments": { "root": server_root.to_string_lossy() }
        });
        let resp = handle_tools_call(Some(json!(1)), &params, &server_root, false);
        assert!(!resp["result"]["isError"].as_bool().unwrap_or(false));
    }

    // --- load_and_discover ---

    #[test]
    fn test_load_and_discover_empty_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (config, specs) = result.unwrap();
        assert_eq!(config.specs_dir, "specs");
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_empty_not_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No spec files found"));
    }

    #[test]
    fn test_load_and_discover_filters_private_specs() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("_private");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("_private.spec.md"),
            "---\nmodule: private\n---",
        )
        .unwrap();

        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        // Private specs (starting with _) should be filtered out
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_finds_real_specs() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        assert_eq!(specs.len(), 1);
    }

    // --- tool_init ---

    #[test]
    fn test_tool_init_creates_config() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();

        let result = tool_init(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["created"], true);
        assert!(tmp.path().join("specsync.json").exists());
    }

    #[test]
    fn test_tool_init_already_exists() {
        let tmp = setup_project();
        let result = tool_init(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["created"], false);
        assert!(val["message"].as_str().unwrap().contains("already exists"));
    }

    // --- tool_coverage ---

    #[test]
    fn test_tool_coverage_empty_project() {
        let tmp = setup_project();
        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["file_coverage"], 100.0);
        assert_eq!(val["files_total"], 0);
    }

    #[test]
    fn test_tool_coverage_with_unspecced_files() {
        let tmp = setup_project();
        // Create a source file without a spec
        std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["files_total"].as_u64().unwrap() > 0);
    }

    // --- tool_list_specs ---

    #[test]
    fn test_tool_list_specs_empty() {
        let tmp = setup_project();
        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 0);
        assert!(val["specs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_tool_list_specs_with_frontmatter() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert_eq!(spec["module"], "auth");
        assert_eq!(spec["version"], "2.0.0");
        assert_eq!(spec["status"], "stable");
    }

    #[test]
    fn test_tool_list_specs_malformed_frontmatter() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("bad");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("bad.spec.md"), "no frontmatter here").unwrap();

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert!(spec["module"].is_null());
    }

    // --- tool_check ---

    #[test]
    fn test_tool_check_strict_mode() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result_normal = tool_check(tmp.path(), &json!({ "strict": false }));
        assert!(result_normal.is_ok());

        let result_strict = tool_check(tmp.path(), &json!({ "strict": true }));
        assert!(result_strict.is_ok());
    }

    #[test]
    fn test_tool_check_no_specs_error() {
        let tmp = setup_project();
        let result = tool_check(tmp.path(), &json!({}));
        assert!(result.is_err());
    }

    // --- tool_score ---

    #[test]
    fn test_tool_score_no_specs_error() {
        let tmp = setup_project();
        let result = tool_score(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_score_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n\n# Invariants\nNone\n\n# Behavioral Examples\nNone\n\n# Error Cases\nNone\n\n# Dependencies\nNone\n\n# Change Log\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result = tool_score(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["average_score"].as_f64().unwrap() >= 0.0);
        assert!(val["grade"].is_string());
        assert_eq!(val["total_specs"], 1);
        assert!(val["distribution"].is_object());
    }

    // --- tool_generate ---

    #[test]
    fn test_tool_generate_no_uncovered() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result = tool_generate(tmp.path(), &json!({}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_generate_creates_spec() {
        let tmp = setup_project();
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result = tool_generate(tmp.path(), &json!({}));
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"].as_u64(), Some(1));
        let generated = val["generated"].as_array().unwrap();
        assert_eq!(generated.len(), 1);
        let generated_path = std::path::Path::new(generated[0].as_str().unwrap());
        assert!(
            generated_path.ends_with(
                std::path::Path::new("specs")
                    .join("auth")
                    .join("auth.spec.md")
            )
        );
    }

    #[test]
    fn test_tool_generate_rejects_retired_ai_arguments_without_echoing_values() {
        let tmp = setup_project();
        let secret = "sk-do-not-echo";
        let error = tool_generate(tmp.path(), &json!({ "apiKey": secret })).unwrap_err();
        assert!(error.contains("removed in spec-sync 5.0"));
        assert!(!error.contains(secret));
    }

    #[test]
    fn test_tool_generate_rejects_unknown_arguments() {
        let tmp = setup_project();
        let error = tool_generate(tmp.path(), &json!({ "unexpected": true })).unwrap_err();
        assert!(error.contains("Unknown MCP generate argument `unexpected`"));
    }

    // --- JSONRPC response structure ---

    #[test]
    fn test_tools_call_success_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "specsync_coverage", "arguments": {} });
        let resp = handle_tools_call(Some(json!(42)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 42);
        assert!(resp["result"]["content"].is_array());
        assert_eq!(resp["result"]["content"][0]["type"], "text");
    }

    #[test]
    fn test_tools_call_error_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "bogus", "arguments": {} });
        let resp = handle_tools_call(Some(json!(99)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 99);
        assert_eq!(resp["result"]["isError"], true);
    }

    // --- handle_resources_list ---

    #[test]
    fn test_handle_resources_list_returns_resources() {
        let resp = handle_resources_list(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        let resources = resp["result"]["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 4);
        let uris: Vec<&str> = resources
            .iter()
            .map(|r| r["uri"].as_str().unwrap())
            .collect();
        assert!(uris.contains(&"specsync:///specs"));
        assert!(uris.contains(&"specsync:///graph"));
        assert!(uris.contains(&"specsync:///config"));
        assert!(uris.contains(&"specsync:///coverage"));
    }

    #[test]
    fn test_handle_resources_list_has_templates() {
        let resp = handle_resources_list(Some(json!(1)));
        let templates = resp["result"]["resourceTemplates"].as_array().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(
            templates[0]["uriTemplate"].as_str().unwrap(),
            "specsync:///specs/{module}"
        );
    }

    // --- handle_resources_read ---

    #[test]
    fn test_resource_specs_list_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert_eq!(resp["jsonrpc"], "2.0");
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    #[test]
    fn test_resource_specs_list_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 1);
        assert_eq!(parsed["specs"][0]["module"], "auth");
        assert!(parsed["specs"][0]["score"].is_number());
    }

    #[test]
    fn test_resource_spec_by_module() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs/auth" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("module: auth"));
        assert_eq!(
            resp["result"]["contents"][0]["mimeType"].as_str().unwrap(),
            "text/markdown"
        );
    }

    #[test]
    fn test_resource_spec_by_module_not_found() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs/nonexistent" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("No spec found")
        );
    }

    #[test]
    fn test_resource_graph_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///graph" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["module_count"], 0);
        assert_eq!(parsed["edge_count"], 0);
    }

    #[test]
    fn test_resource_config() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///config" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["specs_dir"], "specs");
    }

    #[test]
    fn test_resource_coverage_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///coverage" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["file_coverage_percent"], 100.0);
    }

    #[test]
    fn test_resource_unknown_uri() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///bogus" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown resource URI")
        );
    }

    #[test]
    fn test_run_mcp_server_rejects_a_non_directory_root() {
        let tmp = TempDir::new().unwrap();
        let file_root = tmp.path().join("not-a-directory");
        fs::write(&file_root, "not a project root").unwrap();

        let error = run_mcp_server(&file_root, false).unwrap_err();
        assert!(error.contains("not a directory"));
    }

    #[test]
    fn test_read_mcp_line_rejects_and_drains_an_oversized_request() {
        let mut input = vec![b'x'; MAX_JSON_RPC_LINE_BYTES + 1];
        input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n");
        let mut reader = std::io::Cursor::new(input);

        let first = read_mcp_line(&mut reader).unwrap().unwrap().unwrap_err();
        assert!(first.contains("1 MiB"));
        let second = read_mcp_line(&mut reader).unwrap().unwrap().unwrap();
        assert!(second.contains("\"id\":2"));
        assert!(read_mcp_line(&mut reader).unwrap().is_none());
    }

    #[test]
    fn test_repeated_tree_scans_share_one_confinement_budget() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
        let exclusions = HashSet::new();
        let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

        validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap();
        let error = validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap_err();

        assert!(error.contains("confinement scan exceeds"));
    }

    #[test]
    fn test_cargo_manifest_preflight_enforces_manifest_bound() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = MAX_MANIFEST_PREFLIGHTS;

        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
        )
        .unwrap_err();
        assert!(error.contains("preflight exceeds"));
    }
}
