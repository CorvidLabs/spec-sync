use super::*;

// ─── MCP Server Tests ──────────────────────────────────────────────────────

#[test]
fn mcp_initialize_returns_capabilities() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "specsync-test", "version": "1.0.0" }
            }
        })],
    );

    assert_eq!(responses.len(), 1);
    let result = &responses[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "specsync");
    assert!(result["capabilities"]["tools"].is_object());
}

#[test]
fn mcp_initialize_rejects_missing_or_wrong_typed_negotiation_params() {
    let tmp = TempDir::new().unwrap();
    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize"
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "initialize", "params": []
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "initialize",
            "params": { "protocolVersion": 42, "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": "initialize",
            "params": { "protocolVersion": "2024-11-05", "capabilities": [], "clientInfo": { "name": "test", "version": "1" } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 5, "method": "initialize",
            "params": { "protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": { "name": "", "version": 1 } }
        }),
    ];

    let responses = mcp_request(tmp.path(), &requests);

    assert_eq!(responses.len(), requests.len());
    for response in responses {
        assert_eq!(response["error"]["code"], -32602);
        assert!(response.get("result").is_none());
    }
}

#[test]
fn mcp_tools_list_defaults_to_read_only_tools() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        })],
    );

    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"specsync_check"));
    assert!(tool_names.contains(&"specsync_coverage"));
    assert!(tool_names.contains(&"specsync_list_specs"));
    assert!(tool_names.contains(&"specsync_score"));
    assert!(tool_names.contains(&"specsync_issues"));
    assert!(!tool_names.contains(&"specsync_generate"));
    assert!(!tool_names.contains(&"specsync_init"));
    assert_eq!(tools.len(), 5);
    assert!(
        tools
            .iter()
            .all(|tool| tool["inputSchema"]["additionalProperties"] == false)
    );
}

#[test]
fn mcp_allow_write_lists_mutating_tools_with_exact_schemas() {
    let tmp = TempDir::new().unwrap();
    let responses = mcp_request_with_write(
        tmp.path(),
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        })],
    );

    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(tools.len(), 7);
    assert!(tool_names.contains(&"specsync_generate"));
    assert!(tool_names.contains(&"specsync_init"));
    for tool in tools {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    }
}

#[test]
fn mcp_generate_rejects_retired_ai_arguments_without_echoing_credentials() {
    let tmp = TempDir::new().unwrap();
    let secret = "sk-never-echo-this";
    let responses = mcp_request_with_write(
        tmp.path(),
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_generate",
                "arguments": { "apiKey": secret }
            }
        })],
    );

    assert_eq!(responses[0]["error"]["code"], -32602);
    let message = responses[0]["error"]["message"].as_str().unwrap();
    assert!(message.contains("removed in spec-sync 5.0"));
    assert!(!message.contains(secret));
}

#[test]
fn mcp_tools_call_rejects_shape_type_and_unknown_key_errors() {
    let tmp = TempDir::new().unwrap();
    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": []
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {}, "extra": true }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": 42, "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": [] }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 5, "method": "tools/call",
            "params": { "name": "specsync_check", "arguments": { "strict": "yes" } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 6, "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": { "root": false } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": { "name": "specsync_score", "arguments": { "unexpected": true } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 8, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": { "root": "." } }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {}
        }),
    ];

    let responses = mcp_request_with_write(tmp.path(), &requests);

    assert_eq!(responses.len(), requests.len());
    for response in responses {
        assert_eq!(response["error"]["code"], -32602);
        assert!(response.get("result").is_none());
    }
    assert!(!tmp.path().join("specsync.json").exists());
}

#[test]
fn mcp_invalid_json_rpc_envelopes_return_invalid_request_without_mutating() {
    let tmp = TempDir::new().unwrap();
    let mutator = serde_json::json!({
        "id": 1,
        "method": "tools/call",
        "params": { "name": "specsync_init", "arguments": {} }
    });
    let requests = [
        mutator.clone(),
        serde_json::json!({
            "jsonrpc": "1.0", "id": 2, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": 2, "id": 3, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": ["tools/call"],
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": { "invalid": true }, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": [5], "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": null, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 6.5, "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": "invalid"
        }),
        serde_json::json!([mutator]),
    ];

    let responses = mcp_request_with_write(tmp.path(), &requests);

    assert_eq!(responses.len(), requests.len());
    for response in responses {
        assert_eq!(response["id"], serde_json::Value::Null);
        assert_eq!(response["error"]["code"], -32600);
        assert!(response.get("result").is_none());
    }
    assert!(!tmp.path().join("specsync.json").exists());
}

#[test]
fn mcp_rejects_oversized_request_ids_before_dispatching_mutators() {
    let tmp = TempDir::new().unwrap();
    let requests = [serde_json::json!({
        "jsonrpc": "2.0",
        "id": "x".repeat(4 * 1024 + 1),
        "method": "tools/call",
        "params": { "name": "specsync_init", "arguments": {} }
    })];

    let responses = mcp_request_with_write(tmp.path(), &requests);

    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], serde_json::Value::Null);
    assert_eq!(responses[0]["error"]["code"], -32600);
    assert!(!tmp.path().join("specsync.json").exists());
}

#[test]
fn mcp_resources_read_requires_exact_params_and_confines_direct_uris() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let secret = "outside-resource-secret";
    fs::write(outside.join("secret.spec.md"), secret).unwrap();

    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "resources/read", "params": []
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "resources/read", "params": {}
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "resources/read",
            "params": { "uri": 42 }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": "resources/read",
            "params": { "uri": "specsync:///config", "extra": true }
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "id": 5, "method": "resources/read",
            "params": { "uri": "specsync:///specs/../../outside/secret.spec.md" }
        }),
    ];

    let responses = mcp_request(&root, &requests);

    assert_eq!(responses.len(), requests.len());
    for response in &responses[..4] {
        assert_eq!(response["error"]["code"], -32602);
        assert!(response.get("result").is_none());
    }
    assert_eq!(responses[4]["error"]["code"], -32602);
    assert!(responses[4].get("result").is_none());
    assert!(!responses[4].to_string().contains(secret));
}

#[test]
fn mcp_notifications_never_respond_or_mutate() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"notification victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request_with_write(
        &root,
        &[
            serde_json::json!({ "jsonrpc": "2.0", "method": "initialize", "params": {} }),
            serde_json::json!({ "jsonrpc": "2.0", "method": "tools/list", "params": {} }),
            serde_json::json!({ "jsonrpc": "2.0", "method": "unknown/method" }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({ "jsonrpc": "2.0", "id": 99, "method": "ping" }),
        ],
    );

    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 99);
    assert!(!root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_parse_error_still_returns_json_rpc_error() {
    let tmp = TempDir::new().unwrap();
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(tmp.path())
        .write_stdin("{not-json\n")
        .output()
        .expect("failed to run mcp");
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(response["id"], serde_json::Value::Null);
    assert_eq!(response["error"]["code"], -32700);
}

#[test]
fn mcp_rejects_an_oversized_line_and_processes_the_next_request() {
    let tmp = TempDir::new().unwrap();
    let oversized = "x".repeat(1024 * 1024 + 1);
    let input = format!("{oversized}\n{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}}\n");
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(tmp.path())
        .write_stdin(input)
        .output()
        .expect("failed to run mcp");
    let responses: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], serde_json::Value::Null);
    assert_eq!(responses[0]["error"]["code"], -32700);
    assert!(
        responses[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("1 MiB")
    );
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[1]["result"], serde_json::json!({}));
}

#[test]
fn mcp_rejects_an_attacker_controlled_oversized_id_with_a_bounded_error() {
    let tmp = TempDir::new().unwrap();
    let method = "unknown";
    let probe = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "",
        "method": method
    });
    let probe_len = serde_json::to_string(&probe).unwrap().len();
    let id = "x".repeat(1024 * 1024 - probe_len);
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method
    });
    let encoded = serde_json::to_string(&request).unwrap();
    assert_eq!(encoded.len(), 1024 * 1024);

    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(tmp.path())
        .write_stdin(format!("{encoded}\n"))
        .output()
        .expect("failed to run mcp");

    assert!(output.stdout.len() <= 1024 * 1024);
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(response["id"].is_null());
    assert_eq!(response["error"]["code"], -32600);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("4 KiB")
    );
}

#[test]
fn mcp_help_documents_explicit_write_authorization() {
    let output = specsync()
        .arg("mcp")
        .arg("--help")
        .output()
        .expect("failed to read mcp help");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("--allow-write"));
    assert!(stdout.contains("configured project root"));
}

#[test]
fn mcp_unknown_tool_returns_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        })],
    );

    let result = &responses[0]["result"];
    assert!(result["isError"].as_bool().unwrap());
}

#[test]
fn mcp_ping_returns_empty_result() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        })],
    );

    assert_eq!(responses.len(), 1);
    assert!(responses[0]["result"].is_object());
}
