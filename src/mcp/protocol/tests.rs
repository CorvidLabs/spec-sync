use super::*;
use crate::mcp::handle_tools_call;
use crate::mcp::tests::setup_project;
use std::io::Read;

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
fn test_read_mcp_line_propagates_reader_failures() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("reader failed"))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("reader failed"))
        }

        fn consume(&mut self, _amount: usize) {}
    }

    let error = read_mcp_line(&mut FailingReader).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "reader failed");
}

#[test]
fn test_invalid_json_rpc_envelopes_fail_before_dispatch() {
    for request in [
        json!([]),
        json!({"id": 1, "method": "ping"}),
        json!({"jsonrpc": "1.0", "id": 1, "method": "ping"}),
        json!({"jsonrpc": "2.0", "id": 1, "method": 7}),
        json!({"jsonrpc": "2.0", "id": true, "method": "ping"}),
        json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": true}),
    ] {
        let response = validate_request_envelope(&request).unwrap_err();
        assert_eq!(response["error"]["code"], -32600);
        assert!(response["id"].is_null());
    }

    let notification = json!({"jsonrpc": "2.0", "method": "ping"});
    assert_eq!(validate_request_envelope(&notification).unwrap(), None);
    let request = json!({"jsonrpc": "2.0", "id": "safe", "method": "ping"});
    assert_eq!(
        validate_request_envelope(&request).unwrap(),
        Some(json!("safe"))
    );
}

#[test]
fn test_oversized_response_is_replaced_with_a_bounded_error() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 19,
        "result": "x".repeat(MAX_JSON_RPC_RESPONSE_BYTES + 1)
    });
    let mut output = Vec::new();

    write_mcp_response(&mut output, &response).unwrap();

    assert!(output.len() < MAX_JSON_RPC_RESPONSE_BYTES);
    let bounded: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(bounded["id"], 19);
    assert_eq!(bounded["error"]["code"], -32603);
}

#[test]
fn test_response_writer_failures_are_propagated() {
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("intentional writer failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let error = write_mcp_response(
        &mut FailingWriter,
        &json!({"jsonrpc": "2.0", "id": 1, "result": {}}),
    )
    .unwrap_err();
    assert!(error.contains("intentional writer failure"));
}
