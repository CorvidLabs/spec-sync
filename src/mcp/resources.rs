use super::confinement::load_and_discover;
use super::confinement::load_confined_config;
use super::confinement::read_file_bounded;
use super::protocol::invalid_params;
use super::read_root::open_confined_read_root;
use super::snapshot::ProjectSnapshot;
use super::snapshot::with_isolated_snapshot_git;
use super::tools::mcp_coverage_payload;
use super::tools::score_spec_for_mcp;
use crate::deps::build_dep_graph;
#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde_json::Value;
use serde_json::json;
use std::path::Path;

pub(super) fn handle_resources_list(id: Option<Value>) -> Value {
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

#[cfg(test)]
fn handle_resources_read(id: Option<Value>, params: &Value, root: &Path) -> Value {
    let directory = match Dir::open_ambient_dir(root, ambient_authority()) {
        Ok(directory) => directory,
        Err(error) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32602,
                    "message": format!("Cannot open MCP server root capability {}: {error}", root.display())
                }
            });
        }
    };
    handle_resources_read_with_directory(id, params, root, &directory)
}

pub(super) fn handle_resources_read_with_directory(
    id: Option<Value>,
    params: &Value,
    _root: &Path,
    directory: &Dir,
) -> Value {
    let Some(arguments) = params.as_object() else {
        return invalid_params(id, "resources/read params must be an object");
    };
    if let Some(key) = arguments.keys().find(|key| key.as_str() != "uri") {
        return invalid_params(id, format!("Unknown resources/read parameter `{key}`"));
    }
    let Some(uri) = arguments.get("uri").and_then(Value::as_str) else {
        return invalid_params(id, "resources/read parameter `uri` must be a string");
    };

    let operation_route = match open_confined_read_root(directory, Path::new(".")) {
        Ok(route) => route,
        Err(message) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": message }
            });
        }
    };
    let snapshot = match ProjectSnapshot::create_from_directory(operation_route.directory()) {
        Ok(snapshot) => snapshot,
        Err(message) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": message }
            });
        }
    };
    let result = with_isolated_snapshot_git(&snapshot, || {
        let operation_root = snapshot.root();
        match uri {
            "specsync:///specs" => {
                resource_specs_list(operation_root, snapshot.git_freshness_available())
            }
            "specsync:///graph" => resource_graph(operation_root),
            "specsync:///config" => resource_config(operation_root),
            "specsync:///coverage" => resource_coverage(operation_root),
            _ if uri.starts_with("specsync:///specs/") => {
                let module = &uri["specsync:///specs/".len()..];
                resource_spec_by_module(operation_root, module)
            }
            _ => Err(format!("Unknown resource URI: {uri}")),
        }
    });

    match result {
        Ok((content, mime_type)) => {
            if let Err(message) = operation_route.revalidate_before_success() {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": message }
                });
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": [{
                        "uri": uri,
                        "mimeType": mime_type,
                        "text": content
                    }]
                }
            })
        }
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": msg }
        }),
    }
}

// ─── Resource Implementations ───────────────────────────────────────────

fn resource_specs_list(
    root: &Path,
    git_freshness_available: bool,
) -> Result<(String, &'static str), String> {
    let (config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| -> Result<Value, String> {
            let content = read_file_bounded(root, f, "spec file")?;
            let parsed = crate::parser::parse_frontmatter(&content);
            let score = score_spec_for_mcp(f, root, &config, git_freshness_available);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                Ok(json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                    "depends_on": fm.depends_on,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            } else {
                Ok(json!({
                    "path": relative,
                    "module": null,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            }
        })
        .collect::<Result<_, _>>()?;

    let output = json!({
        "specs": specs,
        "count": specs.len(),
        "git_freshness_available": git_freshness_available,
    });
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_spec_by_module(root: &Path, module: &str) -> Result<(String, &'static str), String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    // Find the spec file matching this module name
    for f in &spec_files {
        let content = read_file_bounded(root, f, "spec file")?;

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

pub(super) fn resource_coverage(root: &Path) -> Result<(String, &'static str), String> {
    // Byte-identical to the `specsync_coverage` tool: one constructor, one
    // validation pass, one payload. This resource is what an agent reads when
    // it asks the project how it is doing, and it used to answer with
    // percentages alone — no findings, no `passed`. An agent has no exit code
    // and no second channel, so "nothing reported" is the only thing it can
    // read as "nothing wrong".
    let output = mcp_coverage_payload(root)?;
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

#[cfg(test)]
mod tests;
