use crate::ai;
use crate::config::{detect_source_dirs, load_config};
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
                "tools": {}
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
    let strict = arguments
        .get("strict")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<Value> = Vec::new();
    let mut all_warnings: Vec<Value> = Vec::new();
    let mut spec_results: Vec<Value> = Vec::new();

    for spec_file in &spec_files {
        let result = validate_spec(spec_file, root, &schema_tables, &config);
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

    let coverage = compute_coverage(root, &spec_files, &config);
    let overall_passed = total_errors == 0 && (!strict || total_warnings == 0);

    Ok(json!({
        "passed": overall_passed,
        "specs_checked": spec_files.len(),
        "specs_passed": passed,
        "total_errors": total_errors,
        "total_warnings": total_warnings,
        "errors": all_errors,
        "warnings": all_warnings,
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
