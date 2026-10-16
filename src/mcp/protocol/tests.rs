use super::super::test_support::{open_test_directory, setup_project, setup_project_with_spec};
use super::*;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use tempfile::TempDir;

#[test]
fn issue_reference_field_validation_accepts_supported_list_forms() {
    let content = "---\nmodule: valid\nimplements: [41, 42]\ntracks:\n  - 43\n  - 44 # retained comment\n---\n\n# Purpose\nTest\n";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );
}

#[test]
fn issue_reference_field_validation_ignores_nested_extensions_and_block_scalars() {
    let content = "\
---
module: valid
extensions:
  implements: [900]
  nested:
    tracks: invalid
extension_sequence:
  - implements: [901]
    tracks: [902]
notes: |
  implements: invalid
  tracks:
    - 903
folded: >
  tracks: [904]
implements: [41, 42]
tracks:
  - 43
  - 44
---

# Purpose
Test
";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );
}

#[test]
fn issue_reference_field_validation_keeps_top_level_known_fields_strict() {
    let content = "\
---
module: invalid
extensions:
  implements: [900]
implements:
  nested: invalid
---

# Purpose
Test
";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap_err(),
        "`implements` must be a list of unsigned issue numbers"
    );
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

#[test]
fn generation_rejects_an_oversized_result_during_response_preflight() {
    let result = json!({
        "generated": ["specs/a/very-long-generated-spec-name.spec.md"],
        "count": 1,
    });

    let error = validate_tool_content_response_size_with_limit(&result, 64).unwrap_err();
    assert!(error.contains("bounded response limit"));
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
    assert_eq!(parsed["git_freshness_available"], false);
    assert_eq!(parsed["specs"][0]["git_freshness_available"], false);
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
    // This test used to assert the bug: an empty project has no source
    // file, so there is no percentage to report. `100.0` told an agent the
    // project was fully specced when nothing had been looked at (#582).
    assert!(parsed["file_coverage_percent"].is_null(), "{parsed}");
    assert!(parsed["loc_coverage_percent"].is_null(), "{parsed}");
    assert_eq!(parsed["files_total"], 0, "{parsed}");
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

#[cfg(unix)]
#[test]
fn test_retained_server_capability_survives_root_path_swap() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let moved = tmp.path().join("server-moved");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
    fs::write(outside.join("victim"), "outside\n").unwrap();
    let directory = open_test_directory(&root);
    fs::rename(&root, &moved).unwrap();
    symlink(&outside, &root).unwrap();

    let response = handle_tools_call_with_directory(
        Some(json!(1)),
        &json!({
            "name": "specsync_init",
            "arguments": {}
        }),
        &root,
        &root,
        &directory,
        true,
    );

    assert_eq!(response["result"]["isError"].as_bool(), None);
    assert!(moved.join("specsync.json").is_file());
    assert!(!outside.join("specsync.json").exists());
    assert_eq!(
        fs::read_to_string(outside.join("victim")).unwrap(),
        "outside\n"
    );
}

#[cfg(unix)]
#[test]
fn retained_read_root_resolution_ignores_a_replacement_server_path() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let moved = tmp.path().join("server-moved");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("child/src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        root.join("child/specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(root.join("child/src/lib.rs"), "pub fn retained() {}\n").unwrap();
    let directory = open_test_directory(&root);
    fs::rename(&root, &moved).unwrap();
    symlink(&outside, &root).unwrap();

    let retained = handle_tools_call_with_directory(
        Some(json!(1)),
        &json!({
            "name": "specsync_coverage",
            "arguments": { "root": "child" }
        }),
        &root,
        &root,
        &directory,
        false,
    );
    assert_eq!(retained["result"]["isError"], Value::Null);

    let missing_before = handle_tools_call_with_directory(
        Some(json!(2)),
        &json!({
            "name": "specsync_coverage",
            "arguments": { "root": "probe" }
        }),
        &root,
        &root,
        &directory,
        false,
    );
    fs::create_dir_all(outside.join("probe/src")).unwrap();
    fs::write(outside.join("probe/src/outside.rs"), "outside\n").unwrap();
    let missing_after = handle_tools_call_with_directory(
        Some(json!(3)),
        &json!({
            "name": "specsync_coverage",
            "arguments": { "root": "probe" }
        }),
        &root,
        &root,
        &directory,
        false,
    );
    assert_eq!(missing_before["result"]["isError"], true);
    assert_eq!(missing_after["result"]["isError"], true);
    assert_eq!(
        missing_before["result"]["content"][0]["text"],
        missing_after["result"]["content"][0]["text"]
    );
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

#[test]
fn test_resources_read_rejects_non_exact_arguments() {
    let tmp = setup_project();
    for params in [
        json!([]),
        json!({}),
        json!({"uri": 7}),
        json!({"uri": "specsync:///config", "extra": true}),
    ] {
        let response = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert_eq!(response["error"]["code"], -32602);
    }
}
