mod capabilities;
mod confinement;
mod issues;
mod manifest_paths;
mod protocol;
mod publication;
mod read_root;
mod resources;
mod snapshot;
#[cfg(debug_assertions)]
mod test_barriers;
mod tools;

#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use capabilities::open_server_root_capability;
use issues::tool_issues;
use protocol::MAX_JSON_RPC_RESPONSE_BYTES;
use protocol::handle_initialize_request;
use protocol::handle_tools_list;
use protocol::invalid_params;
use protocol::read_mcp_line;
use protocol::tool_error;
use protocol::validate_request_envelope;
use protocol::validate_tool_arguments;
use protocol::write_mcp_response;
use read_root::open_confined_read_root;
use read_root::resolve_read_root;
use resources::handle_resources_list;
use resources::handle_resources_read_with_directory;
use serde_json::Value;
use serde_json::json;
use snapshot::ProjectSnapshot;
use snapshot::with_isolated_snapshot_git;
use std::io;
use std::path::Path;
use std::path::PathBuf;
#[cfg(debug_assertions)]
use test_barriers::wait_for_mcp_startup_test_barrier;
use tools::tool_check;
use tools::tool_coverage;
use tools::tool_generate;
use tools::tool_init;
use tools::tool_list_specs;
use tools::tool_score;

const MAX_PROJECT_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PROJECT_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CONFINEMENT_ENTRIES: usize = 100_000;
const MAX_MANIFEST_PREFLIGHTS: usize = 1_000;
const MAX_CONFIGURED_PATHS: usize = 1_000;
const MAX_GENERATED_SPECS: usize = 1_000;
const MAX_GENERATED_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOOL_CONTENT_RESPONSE_BYTES: usize = MAX_JSON_RPC_RESPONSE_BYTES - 16 * 1024;

/// Run the MCP server on stdio.
pub fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
    let (server_root, server_directory) = open_server_root_capability(root, || {
        #[cfg(debug_assertions)]
        wait_for_mcp_startup_test_barrier()?;
        Ok(())
    })?;
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
                write_mcp_response(&mut stdout, &error)?;
                continue;
            }
            Ok(None) => break,
            Err(error) => return Err(format!("Failed to read MCP stdin: {error}")),
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
                write_mcp_response(&mut stdout, &err)?;
                continue;
            }
        };

        let id = match validate_request_envelope(&request) {
            Ok(id) => id,
            Err(response) => {
                write_mcp_response(&mut stdout, &response)?;
                continue;
            }
        };

        // JSON-RPC notifications never receive a response. Suppress them before
        // dispatch so even mutating tools cannot execute without an acknowledged ID.
        if id.is_none() {
            continue;
        }

        let method = request
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| "Validated MCP request lost its method".to_string())?;

        let response = match method {
            "initialize" => Some(handle_initialize_request(id, request.get("params"))),
            "tools/list" => Some(handle_tools_list(id, allow_write)),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_tools_call_with_directory(
                    id,
                    &params,
                    &server_root,
                    root,
                    &server_directory,
                    allow_write,
                ))
            }
            "resources/list" => Some(handle_resources_list(id)),
            "resources/read" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_resources_read_with_directory(
                    id,
                    &params,
                    &server_root,
                    &server_directory,
                ))
            }
            "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
            _ => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {method}") }
            })),
        };

        if let Some(resp) = response {
            write_mcp_response(&mut stdout, &resp)?;
        }
    }

    Ok(())
}

#[cfg(test)]
fn handle_tools_call(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
    allow_write: bool,
) -> Value {
    let server_directory = match Dir::open_ambient_dir(server_root, ambient_authority()) {
        Ok(directory) => directory,
        Err(error) => {
            return tool_error(
                id,
                format!(
                    "Cannot open MCP server root capability {}: {error}",
                    server_root.display()
                ),
            );
        }
    };
    handle_tools_call_with_directory(
        id,
        params,
        server_root,
        server_root,
        &server_directory,
        allow_write,
    )
}

fn handle_tools_call_with_directory(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
    requested_server_root: &Path,
    server_directory: &Dir,
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

    let operation_relative = if is_mutating {
        PathBuf::from(".")
    } else {
        match resolve_read_root(
            server_root,
            requested_server_root,
            arguments.get("root").and_then(Value::as_str),
        ) {
            Ok(relative) => relative,
            Err(message) => return tool_error(id, message),
        }
    };
    let operation_route = match open_confined_read_root(server_directory, &operation_relative) {
        Ok(route) => route,
        Err(error) => return tool_error(id, error),
    };
    let arguments = Value::Object(arguments.clone());
    let snapshot = match ProjectSnapshot::create_from_directory(operation_route.directory()) {
        Ok(snapshot) => snapshot,
        Err(message) => return tool_error(id, message),
    };
    let result = with_isolated_snapshot_git(&snapshot, || {
        let operation_root = snapshot.root();
        match tool_name {
            "specsync_check" => tool_check(operation_root, &arguments),
            "specsync_coverage" => tool_coverage(operation_root),
            "specsync_generate" => tool_generate(operation_root, server_directory, &arguments),
            "specsync_list_specs" => tool_list_specs(operation_root),
            "specsync_init" => tool_init(operation_root, server_directory),
            "specsync_score" => tool_score(operation_root, snapshot.git_freshness_available()),
            "specsync_issues" => tool_issues(operation_root),
            _ => unreachable!("known tool was validated before dispatch"),
        }
    });

    match result {
        Ok(content) => {
            if let Err(message) = operation_route.revalidate_before_success() {
                return tool_error(id, message);
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                    }]
                }
            })
        }
        Err(msg) => tool_error(id, msg),
    }
}

/// Same tool-definition list the MCP server registers for `tools/list`.
/// One catalog — lock/diff must call this rather than duplicating schemas.
pub fn mcp_tool_definitions(allow_write: bool) -> Vec<Value> {
    protocol::mcp_tool_definitions(allow_write)
}

#[cfg(test)]
mod tests;
