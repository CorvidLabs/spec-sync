---
spec: mcp.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/mcp.rs` | cargo test mcp:: | `test_handle_initialize_response_format`, `test_handle_initialize_null_id`, `test_handle_initialize_string_id`, `test_handle_tools_list_returns_all_tools`, `test_handle_tools_list_tool_names`, `test_handle_tools_list_all_have_schemas` |
| `tests/integration.rs` | cargo test --test integration mcp_initialize_returns_capabilities | End-to-end fixture: `mcp_initialize_returns_capabilities` |
| `tests/integration.rs` | cargo test --test integration mcp_tools_list_returns_all_tools | End-to-end fixture: `mcp_tools_list_returns_all_tools` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_check_validates_specs | End-to-end fixture: `mcp_tool_check_validates_specs` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_coverage_returns_metrics | End-to-end fixture: `mcp_tool_coverage_returns_metrics` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_init_creates_config | End-to-end fixture: `mcp_tool_init_creates_config` |
| `tests/integration.rs` | cargo test --test integration mcp_tool_list_specs_returns_spec_info | End-to-end fixture: `mcp_tool_list_specs_returns_spec_info` |
| `tests/integration.rs` | cargo test --test integration mcp_unknown_tool_returns_error | End-to-end fixture: `mcp_unknown_tool_returns_error` |
| `tests/integration.rs` | cargo test --test integration mcp_ping_returns_empty_result | End-to-end fixture: `mcp_ping_returns_empty_result` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Initialize MCP session | a client sends `{"jsonrpc":"2.0","id":1,"method":"initialize"}` | the server processes the request | responds with protocol version, capabilities, and server info |
| Call specsync_check tool | a client sends a `tools/call` request with `name: "specsync_check"` | the server processes the request | responds with validation results including passed/failed status, errors, and warnings |
| List available resources | a client sends `{"jsonrpc":"2.0","id":2,"method":"resources/list"}` | the server processes the request | responds with 4 static resources and 1 resource template |
| Read a spec by module name | a client sends `{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"specsync:///specs/auth"}}` | the module "auth" exists in the project | responds with the full spec content as text/markdown |
| Unknown method | a client sends a request with `method: "unknown/method"` and an `id` | the server processes the request | responds with JSON-RPC error code -32601 "Method not found" |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Malformed JSON input | JSON-RPC error -32700 "Parse error" | Keep or add a focused assertion before changing this behavior |
| Unknown method with id | JSON-RPC error -32601 "Method not found" | Keep or add a focused assertion before changing this behavior |
| Unknown tool name | Tool error: "Unknown tool: {name}" | Keep or add a focused assertion before changing this behavior |
| Unknown resource URI | JSON-RPC error -32602 "Unknown resource URI: {uri}" | Keep or add a focused assertion before changing this behavior |
| Spec module not found | JSON-RPC error -32602 "No spec found for module: {name}" | Keep or add a focused assertion before changing this behavior |
| No spec files found | Tool error with suggestion to run `specsync generate` | Keep or add a focused assertion before changing this behavior |
| stdin EOF | Server exits gracefully | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/mcp.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
