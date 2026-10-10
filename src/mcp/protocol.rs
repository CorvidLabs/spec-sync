use serde_json::Value;
use serde_json::json;
use std::io;
use std::io::BufRead;
use std::io::Write;

const SERVER_NAME: &str = "specsync";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_JSON_RPC_LINE_BYTES: usize = 1024 * 1024;
pub(super) const MAX_JSON_RPC_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_JSON_RPC_ID_BYTES: usize = 4 * 1024;

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

pub(super) fn handle_initialize_request(id: Option<Value>, params: Option<&Value>) -> Value {
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

pub(super) fn validate_tool_arguments(
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

pub(super) fn invalid_params(id: Option<Value>, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32602, "message": message.into() }
    })
}

pub(super) fn tool_error(id: Option<Value>, message: impl Into<String>) -> Value {
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

#[cfg(test)]
mod tests;
