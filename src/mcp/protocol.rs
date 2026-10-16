//! JSON-RPC stdio transport, request envelopes, and MCP method dispatch.

use super::confinement::{open_confined_read_root, open_server_root_capability, resolve_read_root};
use super::snapshot::{ProjectSnapshot, with_isolated_snapshot_git};
use super::tools::{
    resource_config, resource_coverage, resource_graph, resource_spec_by_module,
    resource_specs_list, tool_check, tool_coverage, tool_generate, tool_init, tool_issues,
    tool_list_specs, tool_score,
};
#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde_json::{Value, json};
#[cfg(debug_assertions)]
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const SERVER_NAME: &str = "specsync";

const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(super) const PROTOCOL_VERSION: &str = "2024-11-05";

pub(super) const MAX_JSON_RPC_LINE_BYTES: usize = 1024 * 1024;

pub(super) const MAX_JSON_RPC_RESPONSE_BYTES: usize = 1024 * 1024;

const MAX_JSON_RPC_ID_BYTES: usize = 4 * 1024;

const MAX_TOOL_CONTENT_RESPONSE_BYTES: usize = MAX_JSON_RPC_RESPONSE_BYTES - 16 * 1024;

#[cfg(debug_assertions)]
const MCP_STARTUP_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_STARTUP_IDENTITY_BARRIER";

#[cfg(debug_assertions)]
pub(super) const MCP_STARTUP_TEST_BARRIER_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(30);

/// Run the MCP server on stdio.
pub(super) fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
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

#[cfg(debug_assertions)]
fn wait_for_mcp_startup_test_barrier() -> Result<(), String> {
    let Some(barrier_directory) = std::env::var_os(MCP_STARTUP_TEST_BARRIER_ENV) else {
        return Ok(());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("identity-bound");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP startup test barrier marker: {error}"))?;
    ready_file
        .write_all(b"identity-bound\n")
        .map_err(|error| format!("Cannot write MCP startup test barrier marker: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err("MCP startup test barrier resume marker is not a file".to_string());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP startup test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP startup test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

pub(super) fn validate_request_envelope(request: &Value) -> Result<Option<Value>, Value> {
    let Some(object) = request.as_object() else {
        return Err(invalid_request("JSON-RPC request must be an object"));
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(invalid_request("JSON-RPC version must be exactly `2.0`"));
    }
    if !object.get("method").is_some_and(Value::is_string) {
        return Err(invalid_request("JSON-RPC method must be a string"));
    }
    if let Some(id) = object.get("id")
        && !(id.is_string() || id.as_i64().is_some() || id.as_u64().is_some())
    {
        return Err(invalid_request("JSON-RPC id must be a string or integer"));
    }
    if let Some(id) = object.get("id")
        && serde_json::to_vec(id).is_ok_and(|encoded| encoded.len() > MAX_JSON_RPC_ID_BYTES)
    {
        return Err(invalid_request("JSON-RPC id exceeds the 4 KiB limit"));
    }
    if let Some(params) = object.get("params")
        && !(params.is_object() || params.is_array())
    {
        return Err(invalid_request(
            "JSON-RPC params must be an object or array",
        ));
    }
    Ok(object.get("id").cloned())
}

fn invalid_request(message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -32600, "message": message.into() }
    })
}

pub(super) fn write_mcp_response(writer: &mut impl Write, response: &Value) -> Result<(), String> {
    let mut encoded = serde_json::to_vec(response)
        .map_err(|error| format!("Failed to encode MCP response: {error}"))?;
    if encoded.len() > MAX_JSON_RPC_RESPONSE_BYTES {
        let id = response.get("id").cloned().unwrap_or(Value::Null);
        encoded = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": "MCP response exceeds the 1 MiB output limit"
            }
        }))
        .map_err(|error| format!("Failed to encode bounded MCP response: {error}"))?;
        if encoded.len() > MAX_JSON_RPC_RESPONSE_BYTES {
            encoded = serde_json::to_vec(&json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32603,
                    "message": "MCP response exceeds the 1 MiB output limit"
                }
            }))
            .map_err(|error| format!("Failed to encode null-ID MCP response: {error}"))?;
        }
    }
    writer
        .write_all(&encoded)
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|error| format!("Failed to write MCP stdout: {error}"))
}

pub(super) fn read_mcp_line<Reader: BufRead>(
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

pub(super) fn handle_initialize(id: Option<Value>) -> Value {
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

fn handle_initialize_request(id: Option<Value>, params: Option<&Value>) -> Value {
    let Some(Value::Object(params)) = params else {
        return invalid_params(id, "initialize params must be an object");
    };
    if params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return invalid_params(
            id,
            "initialize parameter `protocolVersion` must be a non-empty string",
        );
    }
    if !params.get("capabilities").is_some_and(Value::is_object) {
        return invalid_params(id, "initialize parameter `capabilities` must be an object");
    }
    let Some(client_info) = params.get("clientInfo").and_then(Value::as_object) else {
        return invalid_params(id, "initialize parameter `clientInfo` must be an object");
    };
    for field in ["name", "version"] {
        if client_info
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return invalid_params(
                id,
                format!("initialize parameter `clientInfo.{field}` must be a non-empty string"),
            );
        }
    }
    handle_initialize(id)
}

/// Same tool-definition list the MCP server registers for `tools/list`.
/// One catalog — lock/diff must call this rather than duplicating schemas.
pub(super) fn mcp_tool_definitions(allow_write: bool) -> Vec<Value> {
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

    tools
}

pub(super) fn handle_tools_list(id: Option<Value>, allow_write: bool) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": mcp_tool_definitions(allow_write) }
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

#[cfg(test)]
pub(super) fn handle_tools_call(
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

pub(super) fn handle_tools_call_with_directory(
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
pub(super) fn handle_resources_read(id: Option<Value>, params: &Value, root: &Path) -> Value {
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

fn handle_resources_read_with_directory(
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

pub(super) fn validate_tool_content_response_size(content: &Value) -> Result<(), String> {
    validate_tool_content_response_size_with_limit(content, MAX_TOOL_CONTENT_RESPONSE_BYTES)
}

pub(super) fn validate_tool_content_response_size_with_limit(
    content: &Value,
    max_response_bytes: usize,
) -> Result<(), String> {
    let text = serde_json::to_string_pretty(content)
        .map_err(|error| format!("Cannot encode MCP tool result: {error}"))?;
    let response = json!({
        "jsonrpc": "2.0",
        "id": null,
        "result": { "content": [{ "type": "text", "text": text }] }
    });
    let encoded = serde_json::to_vec(&response)
        .map_err(|error| format!("Cannot preflight MCP tool response: {error}"))?;
    if encoded.len() > max_response_bytes {
        return Err("MCP generated-spec result exceeds the bounded response limit".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
