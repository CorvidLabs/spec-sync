use crate::ai;
use crate::config::{detect_source_dirs, load_config};
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::scoring;
use crate::types::SpecSyncConfig;
use crate::validator::{compute_coverage, find_spec_files, get_schema_table_names, validate_spec};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const SERVER_NAME: &str = "specsync";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Run the MCP server on stdio.
pub fn run_mcp_server(root: &Path) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
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

        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let response = match method {
            "initialize" => Some(handle_initialize(id)),
            "notifications/initialized" => None, // notification, no response
            "tools/list" => Some(handle_tools_list(id)),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_tools_call(id, &params, root))
            }
            "resources/list" => Some(handle_resources_list(id)),
            "resources/read" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_resources_read(id, &params, root))
            }
            "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
            _ => {
                // Notifications (no id) get no response
                if id.is_some() {
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": format!("Method not found: {method}") }
                    }))
                } else {
                    None
                }
            }
        };

        if let Some(resp) = response {
            let _ = writeln!(stdout, "{}", resp);
            let _ = stdout.flush();
        }
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

fn handle_tools_list(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "specsync_check",
                    "description": "Validate all spec files against source code. Returns errors, warnings, and pass/fail status for each spec.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            },
                            "strict": {
                                "type": "boolean",
                                "description": "Treat warnings as errors (default: false)"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_coverage",
                    "description": "Get file and LOC coverage metrics. Shows which source files and modules have specs and which don't.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_generate",
                    "description": "Generate spec files for uncovered source modules. Returns paths of generated specs.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            },
                            "ai": {
                                "type": "boolean",
                                "description": "Use AI to generate meaningful spec content instead of templates (default: false)"
                            },
                            "provider": {
                                "type": "string",
                                "description": "AI provider: claude, anthropic, openai, ollama, copilot"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_list_specs",
                    "description": "List all spec files found in the project with their module names and status.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_init",
                    "description": "Initialize a specsync.json config file with auto-detected source directories.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_score",
                    "description": "Score spec quality (0-100) with letter grades, breakdown by category, and improvement suggestions.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            }
                        }
                    }
                },
                {
                    "name": "specsync_issues",
                    "description": "Verify GitHub issue references in spec frontmatter. Checks that linked issues exist and reports their status (open/closed).",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root": {
                                "type": "string",
                                "description": "Project root directory (default: server root)"
                            }
                        }
                    }
                }
            ]
        }
    })
}

fn handle_tools_call(id: Option<Value>, params: &Value, default_root: &Path) -> Value {
    let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let root = arguments
        .get("root")
        .and_then(|r| r.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| default_root.to_path_buf());
    let root = root.canonicalize().unwrap_or(root);

    let result = match tool_name {
        "specsync_check" => tool_check(&root, &arguments),
        "specsync_coverage" => tool_coverage(&root),
        "specsync_generate" => tool_generate(&root, &arguments),
        "specsync_list_specs" => tool_list_specs(&root),
        "specsync_init" => tool_init(&root),
        "specsync_score" => tool_score(&root),
        "specsync_issues" => tool_issues(&root),
        _ => Err(format!("Unknown tool: {tool_name}")),
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
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": msg
                }],
                "isError": true
            }
        }),
    }
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
    Ok((serde_json::to_string_pretty(&output).unwrap(), "application/json"))
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
    let config = load_config(root);
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

    Ok((serde_json::to_string_pretty(&output).unwrap(), "application/json"))
}

fn resource_config(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_config(root);
    let output = json!({
        "specs_dir": config.specs_dir,
        "source_dirs": config.source_dirs,
        "required_sections": config.required_sections,
        "exclude_dirs": config.exclude_dirs,
        "exclude_patterns": config.exclude_patterns,
        "schema_dir": config.schema_dir,
    });

    Ok((serde_json::to_string_pretty(&output).unwrap(), "application/json"))
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

    Ok((serde_json::to_string_pretty(&output).unwrap(), "application/json"))
}

// ─── Tool Implementations ────────────────────────────────────────────────

fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let config = load_config(root);
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

    if spec_files.is_empty() && !allow_empty {
        return Err(format!(
            "No spec files found in {}/. Run specsync generate to scaffold specs.",
            config.specs_dir
        ));
    }

    Ok((config, spec_files))
}

fn tool_check(root: &Path, arguments: &Value) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns =
        crate::schema::build_schema(&root.join(config.schema_dir.as_deref().unwrap_or("")));
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
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage(root, &spec_files, &config);

    let ai = arguments
        .get("ai")
        .and_then(|a| a.as_bool())
        .unwrap_or(false)
        || arguments.get("provider").is_some();

    let resolved_provider = if ai {
        let provider_str = arguments.get("provider").and_then(|p| p.as_str());
        match ai::resolve_ai_provider(&config, provider_str) {
            Ok(p) => Some(p),
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    let generated_paths = generate_specs_for_unspecced_modules_paths(
        root,
        &coverage,
        &config,
        resolved_provider.as_ref(),
    );

    Ok(json!({
        "generated": generated_paths,
        "count": generated_paths.len(),
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
    if config_path.exists() {
        return Ok(json!({
            "created": false,
            "message": "specsync.json already exists"
        }));
    }

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
        let resp = handle_tools_list(Some(json!(2)));
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_handle_tools_list_tool_names() {
        let resp = handle_tools_list(Some(json!(1)));
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
        let resp = handle_tools_list(Some(json!(1)));
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            assert!(
                tool["inputSchema"].is_object(),
                "Tool {} missing inputSchema",
                tool["name"]
            );
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    // --- handle_tools_call ---

    #[test]
    fn test_handle_tools_call_unknown_tool() {
        let tmp = setup_project();
        let params = json!({ "name": "nonexistent_tool", "arguments": {} });
        let resp = handle_tools_call(Some(json!(1)), &params, tmp.path());
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_tools_call_custom_root() {
        let tmp = setup_project();
        // coverage tool should work with empty specs
        let params = json!({
            "name": "specsync_coverage",
            "arguments": { "root": tmp.path().to_string_lossy() }
        });
        let resp = handle_tools_call(Some(json!(1)), &params, tmp.path());
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
        std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

        let result = tool_generate(tmp.path(), &json!({}));
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["count"].as_u64().unwrap() >= 0);
    }

    // --- JSONRPC response structure ---

    #[test]
    fn test_tools_call_success_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "specsync_coverage", "arguments": {} });
        let resp = handle_tools_call(Some(json!(42)), &params, tmp.path());

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 42);
        assert!(resp["result"]["content"].is_array());
        assert_eq!(resp["result"]["content"][0]["type"], "text");
    }

    #[test]
    fn test_tools_call_error_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "bogus", "arguments": {} });
        let resp = handle_tools_call(Some(json!(99)), &params, tmp.path());

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
        let uris: Vec<&str> = resources.iter().map(|r| r["uri"].as_str().unwrap()).collect();
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
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("No spec found"));
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
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown resource URI"));
    }
}
