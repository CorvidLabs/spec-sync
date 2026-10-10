use super::*;

#[test]
fn mcp_tool_check_validates_specs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_check",
                "arguments": {}
            }
        })],
    );

    assert_eq!(responses.len(), 1);
    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["passed"].as_bool().unwrap());
    assert_eq!(result["specs_checked"].as_u64().unwrap(), 1);
}

#[test]
fn mcp_tool_coverage_returns_metrics() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_coverage",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["files_total"].as_u64().unwrap() > 0);
    assert!(result["file_coverage"].is_number());
}

#[test]
fn mcp_tool_init_creates_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    let responses = mcp_request_with_write(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_init",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["created"].as_bool().unwrap());
    assert!(root.join("specsync.json").exists());
}

#[test]
fn mcp_generate_reports_destination_collisions_and_io_failures_as_tool_errors() {
    let tmp = TempDir::new().unwrap();

    let collision_root = tmp.path().join("collision-server");
    fs::create_dir_all(collision_root.join("src")).unwrap();
    fs::create_dir_all(collision_root.join("specs/collision/collision.spec.md")).unwrap();
    fs::write(
        collision_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        collision_root.join("src/collision.rs"),
        "pub fn collision() {}\n",
    )
    .unwrap();

    let collision_responses = mcp_request_with_write(
        &collision_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_generation_failed_without_zero_count(&collision_responses[0]);

    let blocked_root = tmp.path().join("blocked-server");
    fs::create_dir_all(blocked_root.join("src")).unwrap();
    fs::create_dir_all(blocked_root.join("specs")).unwrap();
    fs::write(
        blocked_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(blocked_root.join("src/blocked.rs"), "pub fn blocked() {}\n").unwrap();
    fs::write(blocked_root.join("specs/blocked"), "not a directory\n").unwrap();

    let blocked_responses = mcp_request_with_write(
        &blocked_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_generation_failed_without_zero_count(&blocked_responses[0]);
}

#[test]
fn mcp_read_only_rejects_direct_mutators_and_preserves_outside_victim() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"outside victim bytes\0must stay exact";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request(
        &root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": {}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": {}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
        ],
    );

    assert_eq!(responses.len(), 4);
    for response in &responses[..2] {
        assert_eq!(response["result"]["isError"], true);
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("--allow-write")
        );
    }
    assert_eq!(responses[2]["error"]["code"], -32602);
    assert_eq!(responses[3]["error"]["code"], -32602);
    assert!(!root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert!(!outside.join("specs").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_allow_write_uses_server_root_and_rejects_root_overrides() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"write override victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request_with_write(
        &root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_init", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_init",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "specsync_generate",
                    "arguments": { "root": outside.to_string_lossy() }
                }
            }),
        ],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    assert_eq!(responses[1]["error"]["code"], -32602);
    assert_eq!(responses[2]["error"]["code"], -32602);
    assert!(root.join("specsync.json").exists());
    assert!(!outside.join("specsync.json").exists());
    assert!(!outside.join("specs").exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(windows)]
#[test]
fn mcp_write_tools_reject_windows_junction_destinations_without_touching_outside_bytes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    let junction = root.join("specs").join("evil");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src").join("evil.rs"), "pub fn evil() {}\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"windows write-junction victim\0exact";
    fs::write(&victim, victim_bytes).unwrap();
    create_windows_junction(&junction, &outside)
        .unwrap_or_else(|error| panic!("failed to create Windows junction fixture: {error}"));
    let canonical_outside = fs::canonicalize(&outside)
        .unwrap_or_else(|error| panic!("failed to canonicalize outside fixture: {error}"));
    assert_eq!(
        fs::canonicalize(&junction)
            .unwrap_or_else(|error| panic!("failed to canonicalize junction fixture: {error}")),
        canonical_outside,
        "Windows junction fixture does not target the outside directory"
    );

    let responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );

    assert_eq!(
        responses.len(),
        1,
        "unexpected write-enabled MCP responses: {responses:#?}"
    );
    let response = &responses[0];
    assert_eq!(
        response["result"]["isError"], true,
        "generation unexpectedly accepted an outside junction destination: {response}"
    );
    let error = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("generation rejection omitted diagnostic text: {response}"));
    let normalized_error = error.replace('\\', "/");
    let rejected_during_snapshot = normalized_error.contains("specs/evil")
        && (normalized_error
            .contains("Cannot inspect MCP project input specs/evil through its root capability")
            || normalized_error.contains(
                "Cannot read MCP project directory specs/evil through its root capability",
            )
            || normalized_error.contains(
                "MCP project input must be a regular file or directory and must not be a symlink or reparse point",
            ));
    let rejected_at_destination =
        error.contains("generation destination escapes the configured server root");
    assert!(
        rejected_during_snapshot || rejected_at_destination,
        "generation failed for the wrong reason: {response}"
    );
    assert_eq!(
        fs::read(&victim).unwrap_or_else(|error| panic!(
            "failed to read outside victim after rejection: {error}"
        )),
        victim_bytes,
        "generation changed outside victim bytes through the junction"
    );
    let outside_entries: Vec<String> = fs::read_dir(&outside)
        .unwrap_or_else(|error| panic!("failed to inspect outside directory: {error}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to inspect outside entry: {error}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        outside_entries,
        vec!["victim.bin".to_string()],
        "generation wrote through the junction or left staging debris outside the server root"
    );
}

#[cfg(windows)]
#[test]
fn mcp_init_rejects_a_windows_junction_at_its_destination() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"windows init-junction victim\0exact";
    fs::write(&victim, victim_bytes).unwrap();
    create_windows_junction(&root.join("specsync.json"), &outside)
        .unwrap_or_else(|error| panic!("failed to create Windows junction fixture: {error}"));

    let responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"], true);
    assert_eq!(fs::read(&victim).unwrap(), victim_bytes);
    assert!(!outside.join("specsync.json").exists());
}

#[test]
fn mcp_tool_list_specs_returns_spec_info() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_list_specs",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["count"].as_u64().unwrap() >= 1);
    let specs = result["specs"].as_array().unwrap();
    assert!(specs[0]["module"].is_string());
    assert!(specs[0]["path"].is_string());
}

// ─── MCP Score Tool Tests ────────────────────────────────────────────────

#[test]
fn mcp_score_tool_returns_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "specsync-test", "version": "1.0.0" }
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_score",
                    "arguments": {}
                }
            }),
        ],
    );

    assert_eq!(responses.len(), 2);
    let score_result = &responses[1]["result"]["content"][0]["text"];
    let score_json: serde_json::Value =
        serde_json::from_str(score_result.as_str().unwrap()).unwrap();
    assert!(score_json["average_score"].is_number());
    assert!(score_json["grade"].is_string());
}

fn assert_generation_failed_without_zero_count(response: &serde_json::Value) {
    assert_eq!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.contains("\"count\": 0"));
}
