use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

#[cfg(unix)]
fn mcp_request_with_timeout(
    root: &std::path::Path,
    requests: &[serde_json::Value],
    label: &str,
) -> Vec<serde_json::Value> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    let binary = specsync().get_program().to_os_string();
    let mut child = Command::new(binary)
        .arg("mcp")
        .arg("--root")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("failed to spawn {label}: {error}"));
    {
        let mut stdin = child.stdin.take().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut stdin, request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child
            .try_wait()
            .unwrap_or_else(|error| panic!("failed to poll {label}: {error}"))
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            child
                .kill()
                .unwrap_or_else(|error| panic!("failed to terminate blocked {label}: {error}"));
            let output = child
                .wait_with_output()
                .unwrap_or_else(|error| panic!("failed to collect blocked {label}: {error}"));
            panic!(
                "{label} blocked; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }

    let output = child
        .wait_with_output()
        .unwrap_or_else(|error| panic!("failed to collect {label}: {error}"));
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}

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
            "params": {}
        })],
    );

    assert_eq!(responses.len(), 1);
    let result = &responses[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "specsync");
    assert!(result["capabilities"]["tools"].is_object());
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

#[test]
fn mcp_read_roots_allow_existing_children_and_reject_escapes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let child = root.join("child");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(child.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"read confinement victim";
    fs::write(&victim, victim_bytes).unwrap();
    let nonexistent_outside = outside.join("missing");

    let responses = mcp_request(
        &root,
        &[
            coverage_request(1, serde_json::json!("child")),
            coverage_request(2, serde_json::json!(outside.to_string_lossy())),
            coverage_request(3, serde_json::json!(nonexistent_outside.to_string_lossy())),
            coverage_request(4, serde_json::json!("../outside")),
        ],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    for response in &responses[1..] {
        assert_eq!(response["result"]["isError"], true);
    }
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_absolute_outside_roots_do_not_disclose_existence() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let existing_outside = tmp.path().join("existing-outside");
    let missing_outside = tmp.path().join("missing-outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&existing_outside).unwrap();

    let responses = mcp_request(
        &root,
        &[
            coverage_request(1, serde_json::json!(existing_outside.to_string_lossy())),
            coverage_request(2, serde_json::json!(missing_outside.to_string_lossy())),
        ],
    );

    let existing_error = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let missing_error = responses[1]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(responses[0]["result"]["isError"], true);
    assert_eq!(responses[1]["result"]["isError"], true);
    assert_eq!(existing_error, missing_error);
    assert!(existing_error.contains("escapes the configured server root"));
    assert!(!existing_error.contains(existing_outside.to_string_lossy().as_ref()));
    assert!(!missing_error.contains(missing_outside.to_string_lossy().as_ref()));
}

#[cfg(unix)]
#[test]
fn mcp_read_root_rejects_symlink_escape_and_preserves_referent() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("escape")).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"symlink escape victim";
    fs::write(&victim, victim_bytes).unwrap();

    let responses = mcp_request(&root, &[coverage_request(1, serde_json::json!("escape"))]);

    assert_eq!(responses[0]["result"]["isError"], true);
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(windows)]
#[test]
fn mcp_read_root_rejects_windows_junction_escape_and_preserves_referent() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    let junction = root.join("escape");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"junction escape victim";
    fs::write(&victim, victim_bytes).unwrap();

    create_windows_junction(&junction, &outside)
        .unwrap_or_else(|error| panic!("failed to create Windows junction fixture: {error}"));
    assert_eq!(
        fs::canonicalize(&junction).unwrap(),
        fs::canonicalize(&outside).unwrap()
    );

    let responses = mcp_request(&root, &[coverage_request(1, serde_json::json!("escape"))]);

    assert_eq!(responses[0]["result"]["isError"], true);
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

#[cfg(windows)]
#[test]
fn mcp_windows_read_roots_accept_absolute_children_and_reject_ambiguous_prefixes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let child = root.join("child");
    let child_source = child.join("src").join("auth").join("service.ts");
    let child_spec = child.join("specs").join("auth").join("auth.spec.md");
    fs::create_dir_all(child_source.parent().unwrap()).unwrap();
    fs::create_dir_all(child_spec.parent().unwrap()).unwrap();
    write_config(&child, "specs", &["src"]);
    fs::write(
        &child_source,
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::write(&child_spec, valid_spec("auth", &["src/auth/service.ts"])).unwrap();
    let absolute_sibling_prefix = root.with_file_name("server-sibling").join("child");

    let responses = mcp_request(
        &root,
        &[
            coverage_request(1, serde_json::json!(child.to_string_lossy())),
            coverage_request(2, serde_json::json!(r"\Windows")),
            coverage_request(3, serde_json::json!(r"C:relative")),
            coverage_request(
                4,
                serde_json::json!(absolute_sibling_prefix.to_string_lossy()),
            ),
        ],
    );

    assert_eq!(
        responses.len(),
        4,
        "unexpected Windows read-root responses: {responses:#?}"
    );
    let accepted = &responses[0];
    assert_eq!(
        accepted["result"]["isError"].as_bool(),
        None,
        "absolute child root should be accepted before coverage execution: {accepted}"
    );
    let coverage_text = accepted["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("accepted child-root response omitted coverage text: {accepted}")
        });
    let coverage: serde_json::Value = serde_json::from_str(coverage_text)
        .unwrap_or_else(|error| panic!("invalid child-root coverage JSON ({error}): {accepted}"));
    assert_eq!(
        coverage["files_total"], 1,
        "coverage did not execute against the configured child project: {accepted}"
    );
    assert_eq!(
        coverage["files_covered"], 1,
        "child fixture should be fully covered: {accepted}"
    );

    for (label, response) in [
        ("rooted path", &responses[1]),
        ("drive-relative path", &responses[2]),
    ] {
        assert_eq!(
            response["result"]["isError"], true,
            "{label} unexpectedly passed Windows read-root validation: {response}"
        );
        let error = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("{label} rejection omitted diagnostic text: {response}"));
        assert!(
            error.contains("must not use a rooted or drive-relative path"),
            "{label} failed for the wrong reason: {response}"
        );
    }

    let sibling_response = &responses[3];
    assert_eq!(
        sibling_response["result"]["isError"], true,
        "absolute sibling-prefix path unexpectedly passed Windows read-root validation: \
         {sibling_response}"
    );
    assert!(
        sibling_response["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|error| error.contains("escapes the configured server root")),
        "absolute sibling-prefix path failed for the wrong reason: {sibling_response}"
    );
}

#[test]
fn mcp_rejects_configured_read_and_write_path_escapes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(outside.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "pub fn local() {}\n").unwrap();
    fs::write(outside.join("src/secret.rs"), "pub fn secret() {}\n").unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"configured path victim";
    fs::write(&victim, victim_bytes).unwrap();

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"../outside/specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let write_responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(write_responses[0]["result"]["isError"], true);
    assert!(
        write_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("specs_dir")
    );
    assert!(!outside.join("specs").exists());

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["../outside/src"]}"#,
    )
    .unwrap();
    let read_responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(read_responses[0]["result"]["isError"], true);
    assert!(
        read_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("source_dirs")
    );
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_allow_empty_tool_and_resource_reject_malformed_selected_config() {
    for (format, relative_config, config) in [
        (
            "JSON",
            ".specsync/config.json",
            r#"{"specsDir":"custom-specs","sourceDirs":["custom-source"]"#,
        ),
        (
            "TOML",
            ".specsync/config.toml",
            "specs_dir = \"custom-specs\"\nsource_dirs = [\"custom-source\"]\n[broken\n",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(root.join("custom-source")).unwrap();
        fs::create_dir_all(root.join("custom-specs/custom")).unwrap();
        fs::write(root.join("custom-source/lib.rs"), "pub fn custom() {}\n").unwrap();
        fs::write(
            root.join("custom-specs/custom/custom.spec.md"),
            valid_spec("custom", &["custom-source/lib.rs"]),
        )
        .unwrap();
        fs::write(root.join(relative_config), config).unwrap();

        assert_mcp_allow_empty_reads_reject_config(
            root,
            &format!("malformed {format}"),
            &format!("malformed {format}"),
        );
    }
}

#[test]
fn mcp_allow_empty_tool_and_resource_reject_invalid_utf8_selected_config() {
    for relative_config in [".specsync/config.json", ".specsync/config.toml"] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        let mut config = if relative_config.ends_with(".json") {
            br#"{"specsDir":"custom-specs","sourceDirs":["custom-source"]}"#.to_vec()
        } else {
            b"specs_dir = \"custom-specs\"\nsource_dirs = [\"custom-source\"]\n".to_vec()
        };
        config.push(0xff);
        fs::write(root.join(relative_config), config).unwrap();

        assert_mcp_allow_empty_reads_reject_config(
            root,
            "not valid UTF-8",
            &format!("invalid UTF-8 {relative_config}"),
        );
    }
}

#[test]
fn mcp_allow_empty_tool_and_resource_reject_wrong_typed_selected_path_fields() {
    for (relative_config, config, selector) in [
        (
            ".specsync/config.json",
            r#"{"specsDir":42,"sourceDirs":["custom-source"]}"#,
            "specsDir",
        ),
        (
            ".specsync/config.json",
            r#"{"specsDir":"custom-specs","sourceDirs":"custom-source"}"#,
            "sourceDirs",
        ),
        (
            ".specsync/config.toml",
            "specs_dir = 42\nsource_dirs = [\"custom-source\"]\n",
            "specs_dir",
        ),
        (
            ".specsync/config.toml",
            "specs_dir = \"custom-specs\"\nsource_dirs = \"custom-source\"\n",
            "source_dirs",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(root.join("custom-source")).unwrap();
        fs::create_dir_all(root.join("custom-specs")).unwrap();
        fs::write(root.join(relative_config), config).unwrap();

        assert_mcp_allow_empty_reads_reject_config(
            root,
            selector,
            &format!("wrong-typed path selector {selector} in {relative_config}"),
        );
    }
}

#[test]
fn mcp_allow_empty_tool_and_resource_reject_non_object_json_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("specsync.json"), "[]\n").unwrap();

    assert_mcp_allow_empty_reads_reject_config(
        root,
        "malformed JSON",
        "non-object JSON configuration",
    );
}

#[test]
fn mcp_allow_empty_reads_reject_wrong_typed_github_repo() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":42}}"#,
    )
    .unwrap();

    assert_mcp_allow_empty_reads_reject_config(
        root,
        "github.repo",
        "wrong-typed GitHub repository configuration",
    );
}

#[cfg(unix)]
#[test]
fn mcp_rejects_selected_config_symlinks_and_fifos_without_blocking() {
    use std::io::Write;
    use std::os::unix::fs::symlink;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    let symlink_tmp = TempDir::new().unwrap();
    let symlink_root = symlink_tmp.path();
    fs::create_dir_all(symlink_root.join(".specsync")).unwrap();
    fs::write(
        symlink_root.join("real-config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    symlink(
        symlink_root.join("real-config.toml"),
        symlink_root.join(".specsync/config.toml"),
    )
    .unwrap();
    let symlink_output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(symlink_root)
        .write_stdin(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\
             \"params\":{\"name\":\"specsync_list_specs\",\"arguments\":{}}}\n",
        )
        .output()
        .expect("failed to run MCP symlink-config rejection");
    assert!(symlink_output.status.success());
    let symlink_response: serde_json::Value =
        serde_json::from_slice(&symlink_output.stdout).unwrap();
    assert_eq!(symlink_response["result"]["isError"], true);
    assert!(
        symlink_response["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|error| error.contains("regular file"))
    );

    let fifo_tmp = TempDir::new().unwrap();
    let fifo_root = fifo_tmp.path();
    fs::create_dir_all(fifo_root.join(".specsync")).unwrap();
    let fifo = fifo_root.join(".specsync/config.toml");
    let mkfifo_status = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("failed to invoke mkfifo");
    assert!(mkfifo_status.success());

    let binary = specsync().get_program().to_os_string();
    let mut command = Command::new(binary);
    command
        .arg("mcp")
        .arg("--root")
        .arg(fifo_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("failed to spawn MCP FIFO probe");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\
              \"params\":{\"name\":\"specsync_list_specs\",\"arguments\":{}}}\n",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if child
            .try_wait()
            .expect("failed to poll MCP FIFO probe")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            child
                .kill()
                .expect("failed to terminate blocked MCP FIFO probe");
            let output = child
                .wait_with_output()
                .expect("failed to collect blocked MCP FIFO probe");
            panic!(
                "MCP blocked while opening a selected FIFO config; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let fifo_output = child
        .wait_with_output()
        .expect("failed to collect MCP FIFO rejection");
    assert!(fifo_output.status.success());
    let fifo_response: serde_json::Value = serde_json::from_slice(&fifo_output.stdout).unwrap();
    assert_eq!(fifo_response["result"]["isError"], true);
    assert!(
        fifo_response["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|error| error.contains("regular file"))
    );
}

#[cfg(unix)]
#[test]
fn mcp_tools_and_resources_reject_generic_fifo_and_socket_sources_without_blocking() {
    use std::os::unix::net::UnixListener;
    use std::process::Command;

    for special in ["fifo", "socket"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let path = root.join("src/special.rs");
        let _listener = match special {
            "fifo" => {
                assert!(
                    Command::new("mkfifo")
                        .arg(&path)
                        .status()
                        .unwrap()
                        .success()
                );
                None
            }
            "socket" => match UnixListener::bind(&path) {
                Ok(listener) => Some(listener),
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => continue,
                Err(error) => panic!("cannot create generic MCP socket fixture: {error}"),
            },
            _ => unreachable!(),
        };
        let responses = mcp_request_with_timeout(
            &root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_coverage", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///coverage" }
                }),
            ],
            &format!("generic MCP {special} source probe"),
        );

        assert_eq!(responses.len(), 2, "{special}: {responses:#?}");
        assert_eq!(responses[0]["result"]["isError"], true, "{special}");
        assert_eq!(responses[1]["error"]["code"], -32602, "{special}");
        let rendered = serde_json::to_string(&responses).unwrap();
        assert!(
            rendered.contains("regular file or directory"),
            "{special}: {rendered}"
        );
    }
}

#[cfg(unix)]
#[test]
fn mcp_tool_and_resource_snapshot_races_reject_fifo_symlink_and_regular_replacements() {
    use std::io::Write;
    use std::os::unix::fs::symlink;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_IDENTITY_BARRIER";
    const TARGET_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_PATH";
    const ATTACKER_BYTES: &str = "MCP_GENERIC_SNAPSHOT_ATTACKER_BYTES";

    let requests = [
        (
            "tool",
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
        ),
        (
            "resource",
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "resources/read",
                "params": { "uri": "specsync:///coverage" }
            }),
        ),
    ];

    for (request_kind, request) in requests {
        for replacement in ["fifo", "symlink", "regular"] {
            let tmp = TempDir::new().unwrap();
            let control = TempDir::new().unwrap();
            let root = setup_minimal_project(&tmp);
            let barrier = control
                .path()
                .join(format!("barrier-{request_kind}-{replacement}"));
            let attacker = control
                .path()
                .join(format!("attacker-{request_kind}-{replacement}.rs"));
            fs::create_dir_all(&barrier).unwrap();
            fs::write(&attacker, ATTACKER_BYTES).unwrap();

            let binary = specsync().get_program().to_os_string();
            let mut child = Command::new(binary)
                .arg("mcp")
                .arg("--root")
                .arg(&root)
                .env(BARRIER_ENV, &barrier)
                .env(TARGET_ENV, "src/auth/service.ts")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            {
                let mut stdin = child.stdin.take().unwrap();
                serde_json::to_writer(&mut stdin, &request).unwrap();
                stdin.write_all(b"\n").unwrap();
            }

            let ready = barrier.join("retained-open");
            let ready_deadline = Instant::now() + Duration::from_secs(5);
            while !ready.is_file() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{request_kind}/{replacement} exited before the retained-open barrier"
                );
                assert!(
                    Instant::now() < ready_deadline,
                    "{request_kind}/{replacement} did not reach the retained-open barrier"
                );
                thread::sleep(Duration::from_millis(10));
            }

            let target = root.join("src/auth/service.ts");
            fs::rename(&target, root.join("src/auth/retained.ts")).unwrap();
            match replacement {
                "fifo" => {
                    assert!(
                        Command::new("mkfifo")
                            .arg(&target)
                            .status()
                            .unwrap()
                            .success()
                    );
                }
                "symlink" => symlink(&attacker, &target).unwrap(),
                "regular" => fs::rename(&attacker, &target).unwrap(),
                _ => unreachable!(),
            }
            fs::write(barrier.join("resume"), b"resume\n").unwrap();

            let exit_deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                if Instant::now() >= exit_deadline {
                    child.kill().unwrap();
                    let output = child.wait_with_output().unwrap();
                    panic!(
                        "{request_kind}/{replacement} blocked; stdout: {}; stderr: {}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                thread::sleep(Duration::from_millis(10));
            }
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "{request_kind}/{replacement}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            if request_kind == "tool" {
                assert_eq!(
                    response["result"]["isError"], true,
                    "{replacement}: {response}"
                );
            } else {
                assert_eq!(
                    response["error"]["code"], -32602,
                    "{replacement}: {response}"
                );
            }
            let rendered = serde_json::to_string(&response).unwrap();
            assert!(
                rendered.contains("changed during inspection"),
                "{request_kind}/{replacement}: {rendered}"
            );
            assert!(
                !rendered.contains(ATTACKER_BYTES),
                "{request_kind}/{replacement}: {rendered}"
            );
            let attacker_location = if replacement == "regular" {
                &target
            } else {
                &attacker
            };
            assert_eq!(
                fs::read(attacker_location).unwrap(),
                ATTACKER_BYTES.as_bytes(),
                "{request_kind}/{replacement} changed attacker bytes"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn mcp_tools_and_resources_reject_unsafe_gradle_build_manifests() {
    use std::os::unix::fs::symlink;
    use std::process::Command;

    for case in ["symlink", "fifo", "oversized"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        match case {
            "symlink" => {
                let outside = tmp.path().join("outside-build.gradle.kts");
                fs::write(&outside, "plugins { id(\"GRADLE_BUILD_SECRET\") }\n").unwrap();
                symlink(&outside, root.join("build.gradle.kts")).unwrap();
            }
            "fifo" => {
                assert!(
                    Command::new("mkfifo")
                        .arg(root.join("build.gradle.kts"))
                        .status()
                        .unwrap()
                        .success()
                );
            }
            "oversized" => {
                fs::write(
                    root.join("build.gradle.kts"),
                    vec![b' '; 4 * 1024 * 1024 + 1],
                )
                .unwrap();
            }
            _ => unreachable!(),
        }

        let responses = mcp_request(
            &root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_list_specs", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///config" }
                }),
            ],
        );

        assert_eq!(responses[0]["result"]["isError"], true, "{case}");
        assert_eq!(responses[1]["error"]["code"], -32602, "{case}");
        let rendered = serde_json::to_string(&responses).unwrap();
        assert!(rendered.contains("manifest"), "{case}: {rendered}");
        assert!(!rendered.contains("GRADLE_BUILD_SECRET"));
        if case == "oversized" {
            assert!(rendered.contains("4 MiB"), "{rendered}");
        } else {
            assert!(rendered.contains("regular file"), "{rendered}");
        }
    }
}

#[test]
fn mcp_allow_empty_tool_and_resource_preserve_valid_selected_config_compatibility() {
    for (relative_config, config) in [
        (
            ".specsync/config.json",
            r#"{"specsDir":"custom-specs","sourceDirs":["custom-source"]}"#,
        ),
        (
            ".specsync/config.toml",
            "specs_dir = \"custom-specs\"\nsource_dirs = [\"custom-source\"]\n",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(root.join("custom-source")).unwrap();
        fs::create_dir_all(root.join("custom-specs/custom")).unwrap();
        fs::write(root.join("custom-source/lib.rs"), "pub fn custom() {}\n").unwrap();
        fs::write(
            root.join("custom-specs/custom/custom.spec.md"),
            valid_spec("custom", &["custom-source/lib.rs"]),
        )
        .unwrap();
        fs::write(root.join(relative_config), format!("\u{feff}{config}")).unwrap();

        let responses = mcp_request(
            root,
            &[
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": { "name": "specsync_list_specs", "arguments": {} }
                }),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "resources/read",
                    "params": { "uri": "specsync:///specs" }
                }),
            ],
        );

        let tool_result: serde_json::Value = serde_json::from_str(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let resource_result: serde_json::Value = serde_json::from_str(
            responses[1]["result"]["contents"][0]["text"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            tool_result["count"], 1,
            "valid config failed: {responses:#?}"
        );
        assert_eq!(
            resource_result["count"], 1,
            "valid config failed: {responses:#?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn mcp_rejects_configured_symlink_trees_and_dangling_write_destinations() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"configured symlink victim";
    fs::write(&victim, victim_bytes).unwrap();

    let read_root = tmp.path().join("read-server");
    fs::create_dir_all(read_root.join("specs")).unwrap();
    fs::create_dir_all(read_root.join("src/nested")).unwrap();
    symlink(&victim, read_root.join("src/nested/escape.rs")).unwrap();
    fs::write(
        read_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let read_responses = mcp_request(
        &read_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(read_responses[0]["result"]["isError"], true);

    let write_root = tmp.path().join("write-server");
    fs::create_dir_all(write_root.join("src")).unwrap();
    fs::create_dir_all(write_root.join("specs")).unwrap();
    symlink(&outside, write_root.join("specs/evil")).unwrap();
    fs::write(write_root.join("src/main.rs"), "pub fn main() {}\n").unwrap();
    fs::write(
        write_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"modules":{"evil":{"files":["src/main.rs"]}}}"#,
    )
    .unwrap();
    let write_responses = mcp_request_with_write(
        &write_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(write_responses[0]["result"]["isError"], true);

    let init_root = tmp.path().join("init-server");
    fs::create_dir_all(&init_root).unwrap();
    let outside_config = outside.join("new-config.json");
    symlink(&outside_config, init_root.join("specsync.json")).unwrap();
    let init_responses = mcp_request_with_write(
        &init_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        })],
    );
    assert_eq!(init_responses[0]["result"]["isError"], true);
    assert!(!outside_config.exists());
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[test]
fn mcp_rejects_unsafe_module_names_and_spec_file_mappings() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/local")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/main.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("victim.rs");
    let victim_bytes = b"pub fn outside_secret() {}\n";
    fs::write(&victim, victim_bytes).unwrap();

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"modules":{"bad\\name":{"files":["src/main.rs"]}}}"#,
    )
    .unwrap();
    let module_responses = mcp_request_with_write(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
    );
    assert_eq!(module_responses[0]["result"]["isError"], true);
    assert!(
        module_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("module name")
    );

    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1.0.0\nstatus: stable\nfiles:\n  - ../outside/victim.rs\n---\n\n# Local\n",
    )
    .unwrap();
    let mapping_responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );
    assert_eq!(mapping_responses[0]["result"]["isError"], true);
    assert!(
        mapping_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("spec file mapping")
    );
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
}

#[cfg(unix)]
#[test]
fn mcp_rejects_metadata_symlinks_and_traversing_dependency_references() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let outside_manifest = outside.join("package.json");
    fs::write(&outside_manifest, r#"{"name":"outside"}"#).unwrap();

    let manifest_root = tmp.path().join("manifest-server");
    fs::create_dir_all(manifest_root.join("src")).unwrap();
    symlink(&outside_manifest, manifest_root.join("package.json")).unwrap();
    let manifest_responses = mcp_request(
        &manifest_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(manifest_responses[0]["result"]["isError"], true);

    let cache_root = tmp.path().join("cache-server");
    fs::create_dir_all(cache_root.join("src")).unwrap();
    fs::create_dir_all(cache_root.join("specs/local")).unwrap();
    fs::create_dir_all(cache_root.join(".specsync")).unwrap();
    fs::write(cache_root.join("src/local.rs"), "pub fn local() {}\n").unwrap();
    fs::write(
        cache_root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1\nstatus: stable\nfiles:\n  - src/local.rs\n---\n\n# Local\n",
    )
    .unwrap();
    symlink(&outside_manifest, cache_root.join(".specsync/hashes.json")).unwrap();
    let cache_responses = mcp_request(
        &cache_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_check", "arguments": {} }
        })],
    );
    assert_eq!(cache_responses[0]["result"]["isError"], true);

    fs::remove_file(cache_root.join(".specsync/hashes.json")).unwrap();
    fs::write(
        cache_root.join("specs/local/local.spec.md"),
        "---\nmodule: local\nversion: 1\nstatus: stable\nfiles:\n  - src/local.rs\ndepends_on:\n  - ../../outside/package.json\n---\n\n# Local\n\n### Consumed By\n\n| Module | Purpose |\n|---|---|\n| `../../outside/package.json` | outside |\n",
    )
    .unwrap();
    let dependency_responses = mcp_request(
        &cache_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );
    assert_eq!(dependency_responses[0]["result"]["isError"], true);
    assert!(
        dependency_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("dependency reference")
    );
}

#[test]
fn mcp_issues_requires_explicit_repo_before_redirected_git_metadata() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        &spec_path,
        spec.replacen(
            "status: active\n",
            "status: active\nimplements:\n  - 1\n",
            1,
        ),
    )
    .unwrap();
    let outside_git = tmp.path().join("outside-git");
    fs::create_dir_all(outside_git.join("objects")).unwrap();
    fs::create_dir_all(outside_git.join("refs/heads")).unwrap();
    fs::write(outside_git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::write(
        outside_git.join("config"),
        "[core]\n\trepositoryformatversion = 0\n\tbare = false\n\
         [remote \"origin\"]\n\turl = git@github.com:outside/metadata.git\n",
    )
    .unwrap();
    fs::write(
        root.join(".git"),
        format!("gitdir: {}\n", outside_git.display()),
    )
    .unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_issues", "arguments": {} }
        })],
    );

    let error = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(error.contains("explicit `github.repo`"));
    assert!(!error.contains("outside/metadata"));
}

#[test]
fn mcp_issues_without_references_skips_repository_resolution() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_issues", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    let result: serde_json::Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["repo"], serde_json::Value::Null);
    assert_eq!(result["total_valid"], 0);
    assert_eq!(result["total_closed"], 0);
    assert_eq!(result["total_not_found"], 0);
    assert_eq!(result["specs"], serde_json::json!([]));
}

#[cfg(any(unix, windows))]
#[test]
fn mcp_issue_diagnostics_preserve_unix_backslashes_and_normalize_windows_separators() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let diagnostic_dir = root.join("specs").join("diagnostics");
    #[cfg(unix)]
    let spec_path = diagnostic_dir.join(r"literal\backslash.spec.md");
    #[cfg(windows)]
    let spec_path = diagnostic_dir.join("windows").join("path.spec.md");
    fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
    fs::write(&spec_path, "# Missing frontmatter\n").unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_issues", "arguments": {} }
        })],
    );

    let error = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(responses[0]["result"]["isError"], true);
    #[cfg(unix)]
    assert!(error.contains(r"specs/diagnostics/literal\backslash.spec.md"));
    #[cfg(windows)]
    assert!(error.contains("specs/diagnostics/windows/path.spec.md"));
}

#[cfg(unix)]
#[test]
fn mcp_issues_rejects_a_git_symlink_to_outside_metadata() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        &spec_path,
        spec.replacen(
            "status: active\n",
            "status: active\nimplements:\n  - 1\n",
            1,
        ),
    )
    .unwrap();
    let outside_git = tmp.path().join("outside-git");
    fs::create_dir_all(&outside_git).unwrap();
    fs::write(
        outside_git.join("config"),
        "[remote \"origin\"]\n\turl = git@github.com:outside/symlink.git\n",
    )
    .unwrap();
    symlink(&outside_git, root.join(".git")).unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_issues", "arguments": {} }
        })],
    );

    let error = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(!error.contains("outside/symlink"));
}

#[cfg(unix)]
#[test]
fn mcp_confinement_scan_honors_configured_excluded_directories() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(root.join("ignored")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("main.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("ignored.rs");
    fs::write(&victim, "pub fn ignored() {}\n").unwrap();
    symlink(&victim, root.join("ignored/escape.rs")).unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["."],"excludeDirs":["ignored"]}"#,
    )
    .unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
}

#[cfg(unix)]
#[test]
fn mcp_manifest_autodetection_rejects_workspace_escapes() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let outside_cargo = tmp.path().join("outside-cargo");
    let outside_packages = tmp.path().join("outside-packages/pkg");
    fs::create_dir_all(outside_cargo.join("src")).unwrap();
    fs::create_dir_all(outside_packages.join("src")).unwrap();
    let cargo_manifest_bytes = b"[package]\nname = \"outside-cargo\"\nversion = \"0.1.0\"\n";
    fs::write(outside_cargo.join("Cargo.toml"), cargo_manifest_bytes).unwrap();
    let package_manifest_bytes = br#"{"name":"outside-package"}"#;
    fs::write(
        outside_packages.join("package.json"),
        package_manifest_bytes,
    )
    .unwrap();

    let cargo_root = tmp.path().join("cargo-server");
    fs::create_dir_all(cargo_root.join("src")).unwrap();
    fs::write(cargo_root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
    symlink(&outside_cargo, cargo_root.join("linked-member")).unwrap();
    fs::write(
        cargo_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"../outside-cargo\", \"linked-member\"]\n",
    )
    .unwrap();
    fs::write(
        cargo_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let cargo_responses = mcp_request(
        &cargo_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(cargo_responses[0]["result"]["isError"], true);
    fs::remove_file(cargo_root.join("specsync.json")).unwrap();
    let cargo_init_responses = mcp_request_with_write(
        &cargo_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_init", "arguments": {} }
        })],
    );
    assert_eq!(cargo_init_responses[0]["result"]["isError"], true);

    let package_root = tmp.path().join("package-server");
    fs::create_dir_all(package_root.join("src")).unwrap();
    fs::write(
        package_root.join("src/index.ts"),
        "export const local = 1;\n",
    )
    .unwrap();
    fs::write(
        package_root.join("package.json"),
        r#"{"name":"local","workspaces":["../outside-packages/*"]}"#,
    )
    .unwrap();
    fs::write(
        package_root.join("specsync.json"),
        r#"{"specsDir":"specs"}"#,
    )
    .unwrap();
    let package_responses = mcp_request(
        &package_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(package_responses[0]["result"]["isError"], true);

    let linked_package_root = tmp.path().join("linked-package-server");
    fs::create_dir_all(linked_package_root.join("src")).unwrap();
    fs::create_dir_all(linked_package_root.join("packages")).unwrap();
    fs::write(
        linked_package_root.join("src/index.ts"),
        "export const local = 1;\n",
    )
    .unwrap();
    symlink(
        &outside_packages,
        linked_package_root.join("packages/linked"),
    )
    .unwrap();
    fs::write(
        linked_package_root.join("package.json"),
        r#"{"name":"local","workspaces":["packages/*"]}"#,
    )
    .unwrap();
    fs::write(
        linked_package_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let linked_package_responses = mcp_request(
        &linked_package_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(linked_package_responses[0]["result"]["isError"], true);

    let scan_root = tmp.path().join("scan-server");
    fs::create_dir_all(&scan_root).unwrap();
    symlink(&outside_cargo, scan_root.join("linked-source")).unwrap();
    let scan_responses = mcp_request(
        &scan_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(scan_responses[0]["result"]["isError"], true);

    assert_eq!(
        fs::read(outside_cargo.join("Cargo.toml")).unwrap(),
        cargo_manifest_bytes
    );
    assert_eq!(
        fs::read(outside_packages.join("package.json")).unwrap(),
        package_manifest_bytes
    );
}

#[test]
fn mcp_manifest_autodetection_rejects_gradle_and_python_path_escapes() {
    let tmp = TempDir::new().unwrap();

    let outside_gradle = tmp.path().join("outside-gradle");
    fs::create_dir_all(outside_gradle.join("src/main/kotlin")).unwrap();
    let gradle_root = tmp.path().join("gradle-server");
    fs::create_dir_all(&gradle_root).unwrap();
    fs::write(gradle_root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        gradle_root.join("settings.gradle.kts"),
        "include ':outside'\nproject(':outside').projectDir = file('../outside-gradle')\n",
    )
    .unwrap();
    fs::write(
        gradle_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let gradle_responses = mcp_request(
        &gradle_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(gradle_responses[0]["result"]["isError"], true);

    let outside_python = tmp.path().join("outside-python");
    fs::create_dir_all(&outside_python).unwrap();
    let python_root = tmp.path().join("python-server");
    fs::create_dir_all(&python_root).unwrap();
    fs::write(
        python_root.join("pyproject.toml"),
        "[project]\nname = \"../outside-python\"\n",
    )
    .unwrap();
    fs::write(python_root.join("specsync.json"), r#"{"specsDir":"specs"}"#).unwrap();
    let python_responses = mcp_request(
        &python_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(python_responses[0]["result"]["isError"], true);
}

#[test]
fn mcp_gradle_set_project_dir_escapes_fail_closed_without_outside_access() {
    let tmp = TempDir::new().unwrap();
    let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_SET_PROJECT_DIR_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();

    for (label, project_dir) in [
        ("traversal", "../outside"),
        ("drive", "C:/outside"),
        ("unc", "//server/share/outside"),
    ] {
        let root = tmp.path().join(format!("{label}-server"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!(
                "include(\":outside\")\nproject(\":outside\").setProjectDir(file(\"{project_dir}\"))\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();

        assert_mcp_gradle_discovery_rejected(
            &root,
            &outside_source,
            outside_bytes,
            "outside",
            label,
        );
    }
}

#[test]
fn mcp_gradle_interpolated_project_dirs_fail_closed_without_outside_access() {
    let tmp = TempDir::new().unwrap();
    let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_GRADLE_INTERPOLATION_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();

    for (label, override_statement) in [
        (
            "assignment-unbraced",
            r#"project(":member").projectDir = file("$outside")"#,
        ),
        (
            "setter-unbraced",
            r#"project(":member").setProjectDir(file("$outside"))"#,
        ),
        (
            "assignment-braced",
            r#"project(":member").projectDir = file("${outside}")"#,
        ),
        (
            "setter-braced",
            r#"project(":member").setProjectDir(file("${outside}"))"#,
        ),
        (
            "indirect-assignment",
            "val member = project(\":member\")\nmember.projectDir = file(\"../outside\")",
        ),
        (
            "indirect-setter",
            "val member = project(\":member\")\nmember.setProjectDir(file(\"../outside\"))",
        ),
        (
            "closure-assignment",
            "project(\":member\") {\nprojectDir = file(\"../outside\")\n}",
        ),
        (
            "whitespace-setter",
            "project(\":member\") . setProjectDir(file(\"../outside\"))",
        ),
        (
            "concatenated-constructor",
            "project(\":member\").projectDir = newFile(rootDir, \"../outside\")",
        ),
        (
            "drive-relative-selector",
            "project(\"C:outside\").projectDir = file(\"modules/member\")",
        ),
        (
            "multiline-conditional-include",
            "if (enabled) {\ninclude(\":member\")\n}",
        ),
        ("triple-quoted-include", "include(\"\"\":member\"\"\")"),
    ] {
        let root = tmp.path().join(format!("{label}-server"));
        setup_minimal_mcp_project_at(&root);
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!("val outside = \"../outside\"\ninclude(\":member\")\n{override_statement}\n"),
        )
        .unwrap();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();

        assert_mcp_gradle_discovery_rejected(
            &root,
            &outside_source,
            outside_bytes,
            "member",
            label,
        );
    }
}

#[cfg(unix)]
#[test]
fn mcp_gradle_symlink_module_escape_fails_closed_without_outside_access() {
    use std::os::unix::fs::symlink;

    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = project_tmp.path();
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"MCP_GRADLE_SYMLINK_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    symlink(outside_tmp.path(), root.join("linked")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();

    assert_mcp_gradle_discovery_rejected(root, &outside_source, outside_bytes, "linked", "symlink");
}

fn setup_minimal_mcp_project_at(root: &std::path::Path) {
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
}

fn assert_mcp_gradle_discovery_rejected(
    root: &std::path::Path,
    outside_source: &std::path::Path,
    outside_bytes: &[u8],
    module: &str,
    label: &str,
) {
    let responses = mcp_request_with_write(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_check", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": { "name": "specsync_score", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": { "name": "specsync_generate", "arguments": {} }
            }),
        ],
    );

    assert_eq!(
        responses.len(),
        4,
        "Gradle {label} discovery returned incomplete MCP responses: {responses:#?}"
    );
    for response in &responses {
        assert_eq!(
            response["result"]["isError"], true,
            "Gradle {label} discovery produced an MCP false green: {response}"
        );
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(
            text.contains("Gradle")
                || text.contains("root capability")
                || text.contains("symlink")
                || text.contains("reparse point"),
            "Gradle {label} rejection did not explicitly identify manifest or path confinement: {response}"
        );
        assert!(
            !text.contains("SECRET"),
            "Gradle {label} rejection disclosed outside source bytes: {response}"
        );
    }
    assert!(
        !root.join("specs").join(module).exists(),
        "Gradle {label} rejection allowed MCP generation to create partial output"
    );
    assert_eq!(
        fs::read(outside_source).unwrap(),
        outside_bytes,
        "Gradle {label} rejection changed outside source bytes"
    );
}

#[test]
fn mcp_gradle_preflight_rejects_malformed_settings_without_partial_results() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":member\"\n").unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Gradle")
    );
}

#[cfg(unix)]
#[test]
fn mcp_gradle_preflight_rejects_in_root_manifest_symlinks() {
    use std::os::unix::fs::symlink;

    for manifest in ["build.gradle.kts", "settings.gradle.kts"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let real_manifest = root.join("real-gradle-manifest");
        fs::write(&real_manifest, "plugins {}\n").unwrap();
        symlink(&real_manifest, root.join(manifest)).unwrap();

        let responses = mcp_request(
            &root,
            &[serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            })],
        );

        assert_eq!(responses[0]["result"]["isError"], true);
        assert!(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|error| error.contains("symlink or reparse point")),
            "{manifest}: {responses:#?}"
        );
    }
}

#[test]
fn mcp_gradle_preflight_rejects_every_manifest_above_four_mib() {
    for manifest in [
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
    ] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        fs::write(root.join(manifest), vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();

        let responses = mcp_request(
            &root,
            &[serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            })],
        );

        assert_eq!(responses[0]["result"]["isError"], true);
        assert!(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|error| error.contains("4 MiB")),
            "{manifest}: {responses:#?}"
        );
    }
}

#[test]
fn mcp_gradle_preflight_accepts_comments_and_escaped_paths() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::create_dir_all(root.join("modules/member/src/main/kotlin")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        root.join("settings.gradle.kts"),
        r#"val kotlinDocumentation = """
include(":phantom")
project(":member").projectDir = file("../outside")
"""
def groovyDocumentation = '''
include(":groovy-phantom")
project(":member").setProjectDir(file("../outside"))
'''
/* outer ignored directive:
   include(":outer-phantom")
   /* include(":nested-phantom") */
*/
include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
    )
    .unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_list_specs", "arguments": {} }
        })],
    );

    assert_ne!(responses[0]["result"]["isError"], true);
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(result["count"], 1);
}

#[test]
fn mcp_manifest_preflight_rejects_cycles_and_excessive_configured_paths() {
    let tmp = TempDir::new().unwrap();

    let cycle_root = tmp.path().join("cycle-server");
    fs::create_dir_all(&cycle_root).unwrap();
    fs::write(
        cycle_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\".\"]\n",
    )
    .unwrap();
    let cycle_responses = mcp_request(
        &cycle_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(
        cycle_responses[0]["result"]["isError"], true,
        "unexpected cycle response: {}",
        cycle_responses[0]
    );
    assert!(
        cycle_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("cycle")
    );

    let bounded_root = tmp.path().join("bounded-server");
    fs::create_dir_all(&bounded_root).unwrap();
    let source_dirs: Vec<String> = (0..1_001).map(|index| format!("src-{index}")).collect();
    fs::write(
        bounded_root.join("specsync.json"),
        serde_json::to_vec(&serde_json::json!({
            "specsDir": "specs",
            "sourceDirs": source_dirs
        }))
        .unwrap(),
    )
    .unwrap();
    let bounded_responses = mcp_request(
        &bounded_root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(bounded_responses[0]["result"]["isError"], true);
    assert!(
        bounded_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("input paths")
    );
}

#[test]
fn mcp_manifest_traversal_accepts_duplicate_cargo_and_node_workspaces_once() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_minimal_mcp_project_at(root);
    fs::create_dir_all(root.join("crates/member/src")).unwrap();
    fs::create_dir_all(root.join("packages/member")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("packages/member/package.json"),
        r#"{"name":"member"}"#,
    )
    .unwrap();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_ne!(
        responses[0]["result"]["isError"], true,
        "duplicate workspace declarations must not trigger replay or a false failure: {}",
        responses[0]
    );
}

#[cfg(unix)]
#[test]
fn mcp_autodetection_preflight_honors_builtin_ignored_directories() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("src/local.rs"), "pub fn local() {}\n").unwrap();
    let victim = outside.join("secret.rs");
    let victim_bytes = b"pub fn outside_secret() {}\n";
    fs::write(&victim, victim_bytes).unwrap();
    symlink(&victim, root.join("target/escape.rs")).unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );
    assert_eq!(responses[0]["result"]["isError"].as_bool(), None);
    assert_eq!(fs::read(victim).unwrap(), victim_bytes);
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

#[cfg(unix)]
#[test]
fn mcp_resources_read_rejects_a_spec_symlink_to_an_outside_resource() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let secret = "outside-symlink-resource-secret";
    let outside_spec = outside.join("outside.spec.md");
    fs::write(
        &outside_spec,
        format!("---\nmodule: outside\nversion: 1\nstatus: stable\nfiles: []\n---\n\n{secret}\n"),
    )
    .unwrap();
    symlink(&outside_spec, root.join("specs/outside.spec.md")).unwrap();

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "resources/read",
            "params": { "uri": "specsync:///specs/outside" }
        })],
    );

    assert_eq!(responses[0]["error"]["code"], -32602);
    assert!(responses[0].get("result").is_none());
    assert!(!responses[0].to_string().contains(secret));
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
fn mcp_bounds_per_file_and_cumulative_project_inputs() {
    let tmp = TempDir::new().unwrap();

    let per_file_root = tmp.path().join("per-file-server");
    fs::create_dir_all(per_file_root.join("src")).unwrap();
    fs::write(
        per_file_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::File::create(per_file_root.join("src/oversized.rs"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();

    let per_file_responses = mcp_request(
        &per_file_root,
        &[coverage_request(1, serde_json::json!("."))],
    );
    let per_file_error = per_file_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(per_file_responses[0]["result"]["isError"], true);
    assert!(per_file_error.contains("8 MiB per-file limit"));

    let cumulative_root = tmp.path().join("cumulative-server");
    fs::create_dir_all(cumulative_root.join("src")).unwrap();
    fs::write(
        cumulative_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    for index in 0..9 {
        fs::File::create(cumulative_root.join(format!("src/input-{index}.rs")))
            .unwrap()
            .set_len(8 * 1024 * 1024)
            .unwrap();
    }

    let cumulative_responses = mcp_request(
        &cumulative_root,
        &[coverage_request(2, serde_json::json!("."))],
    );
    let cumulative_error = cumulative_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(cumulative_responses[0]["result"]["isError"], true);
    assert!(cumulative_error.contains("64 MiB cumulative limit"));

    let config_cumulative_root = tmp.path().join("config-cumulative-server");
    fs::create_dir_all(config_cumulative_root.join("src")).unwrap();
    let mut padded_config = br#"{"specsDir":"specs","sourceDirs":["src"]}"#.to_vec();
    padded_config.resize(4 * 1024 * 1024, b' ');
    fs::write(config_cumulative_root.join("specsync.json"), padded_config).unwrap();
    for index in 0..8 {
        fs::File::create(config_cumulative_root.join(format!("src/input-{index}.rs")))
            .unwrap()
            .set_len(8 * 1024 * 1024)
            .unwrap();
    }

    let config_cumulative_responses = mcp_request(
        &config_cumulative_root,
        &[coverage_request(3, serde_json::json!("."))],
    );
    let config_cumulative_error = config_cumulative_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(config_cumulative_responses[0]["result"]["isError"], true);
    assert!(config_cumulative_error.contains("64 MiB cumulative limit"));

    let explicit_source_root = tmp.path().join("explicit-source-server");
    fs::create_dir_all(explicit_source_root.join("src/docs")).unwrap();
    fs::write(
        explicit_source_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::File::create(explicit_source_root.join("src/docs/oversized.rs"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();

    let explicit_source_responses = mcp_request(
        &explicit_source_root,
        &[coverage_request(4, serde_json::json!("."))],
    );
    let explicit_source_error = explicit_source_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(explicit_source_responses[0]["result"]["isError"], true);
    assert!(explicit_source_error.contains("8 MiB per-file limit"));

    let normally_ignored_root = tmp.path().join("explicit-vendor-server");
    fs::create_dir_all(normally_ignored_root.join("vendor")).unwrap();
    fs::write(
        normally_ignored_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["vendor"]}"#,
    )
    .unwrap();
    fs::File::create(normally_ignored_root.join("vendor/oversized.rs"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();

    let normally_ignored_responses = mcp_request(
        &normally_ignored_root,
        &[coverage_request(5, serde_json::json!("."))],
    );
    let normally_ignored_error = normally_ignored_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(normally_ignored_responses[0]["result"]["isError"], true);
    assert!(normally_ignored_error.contains("8 MiB per-file limit"));

    for git_name in [".git", ".GIT"] {
        let git_input_root = tmp
            .path()
            .join(format!("git-input-server-{}", &git_name[1..]));
        fs::create_dir_all(git_input_root.join(git_name)).unwrap();
        fs::write(
            git_input_root.join("specsync.json"),
            format!(r#"{{"specsDir":"specs","sourceDirs":["{git_name}"]}}"#),
        )
        .unwrap();
        fs::write(
            git_input_root.join(git_name).join("config"),
            "outside = false\n",
        )
        .unwrap();

        let git_input_responses = mcp_request(
            &git_input_root,
            &[coverage_request(6, serde_json::json!("."))],
        );
        let git_input_error = git_input_responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert_eq!(git_input_responses[0]["result"]["isError"], true);
        assert!(git_input_error.contains("must not use Git metadata"));
    }

    let manifest_root = tmp.path().join("manifest-cumulative-server");
    fs::create_dir_all(manifest_root.join("src")).unwrap();
    fs::write(
        manifest_root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    for index in 0..9 {
        let member = manifest_root.join(format!("member-{index}"));
        fs::create_dir_all(&member).unwrap();
        fs::File::create(member.join("Cargo.toml"))
            .unwrap()
            .set_len(8 * 1024 * 1024)
            .unwrap();
    }

    let manifest_responses = mcp_request(
        &manifest_root,
        &[coverage_request(7, serde_json::json!("."))],
    );
    let manifest_error = manifest_responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert_eq!(manifest_responses[0]["result"]["isError"], true);
    assert!(manifest_error.contains("64 MiB cumulative limit"));
}

#[test]
fn mcp_bounds_outbound_resource_responses() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/large")).unwrap();
    fs::write(root.join("src/large.rs"), "pub fn large() {}\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    let mut spec = valid_spec("large", &["src/large.rs"]);
    spec.push_str(&"x".repeat(1024 * 1024));
    fs::write(root.join("specs/large/large.spec.md"), spec).unwrap();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "resources/read",
            "params": { "uri": "specsync:///specs/large" }
        })],
    );

    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(responses[0]["error"]["code"], -32603);
    assert!(
        responses[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("1 MiB output limit")
    );
    assert!(responses[0].to_string().len() < 1024);
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
fn mcp_rejects_a_nonexistent_server_root() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("missing");
    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(&missing)
        .write_stdin("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n")
        .output()
        .expect("failed to run mcp with missing root");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not exist or is not a directory")
    );
    assert!(!missing.exists());
}

#[cfg(all(unix, debug_assertions))]
#[test]
fn mcp_real_cli_rejects_requested_root_replacement_after_identity_binding() {
    use std::io::Write;
    use std::os::unix::fs::symlink;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const STARTUP_BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_STARTUP_IDENTITY_BARRIER";

    let tmp = TempDir::new().unwrap();
    let requested_root = tmp.path().join("requested-root");
    let original_root = tmp.path().join("original-root");
    let replacement_root = tmp.path().join("replacement-root");
    let barrier = tmp.path().join("startup-barrier");
    fs::create_dir_all(&original_root).unwrap();
    fs::create_dir_all(&replacement_root).unwrap();
    fs::create_dir_all(&barrier).unwrap();
    symlink(&original_root, &requested_root).unwrap();

    let binary = specsync().get_program().to_os_string();
    let mut command = Command::new(binary);
    command
        .arg("mcp")
        .arg("--root")
        .arg(&requested_root)
        .env(STARTUP_BARRIER_ENV, &barrier)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, _) in std::env::vars() {
        if key.starts_with("SPECSYNC_") && key != STARTUP_BARRIER_ENV {
            command.env_remove(key);
        }
    }

    let mut child = command.spawn().expect("failed to spawn real MCP CLI");
    let mut stdin = child.stdin.take().expect("MCP CLI stdin was not piped");
    stdin
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n")
        .unwrap();
    drop(stdin);

    let ready_path = barrier.join("identity-bound");
    let deadline = Instant::now() + Duration::from_secs(30);
    while !ready_path.is_file() {
        if let Some(status) = child.try_wait().expect("failed to poll MCP CLI") {
            let output = child
                .wait_with_output()
                .expect("failed to collect early MCP CLI output");
            panic!(
                "MCP CLI exited before identity binding with {status}; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        if Instant::now() >= deadline {
            child.kill().expect("failed to terminate stalled MCP CLI");
            let output = child
                .wait_with_output()
                .expect("failed to collect stalled MCP CLI output");
            panic!(
                "timed out waiting for MCP root identity binding; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(5));
    }

    fs::remove_file(&requested_root).unwrap();
    symlink(&replacement_root, &requested_root).unwrap();
    fs::write(barrier.join("resume"), b"resume\n").unwrap();

    let output = child
        .wait_with_output()
        .expect("failed to collect MCP CLI output");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("MCP server root changed while its identity was being resolved"),
        "unexpected MCP startup rejection: {}",
        String::from_utf8_lossy(&output.stderr)
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
                "params": { "capabilities": {} }
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

fn coverage_request(id: u64, root: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "specsync_coverage",
            "arguments": { "root": root }
        }
    })
}

fn assert_mcp_allow_empty_reads_reject_config(root: &std::path::Path, expected: &str, label: &str) {
    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": "specsync_list_specs", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "resources/read",
                "params": { "uri": "specsync:///specs" }
            }),
        ],
    );

    assert_eq!(
        responses[0]["result"]["isError"], true,
        "{label} produced an allow-empty tool success: {responses:#?}"
    );
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|error| error.contains(expected)),
        "{label} tool rejection was not explicit: {responses:#?}"
    );
    assert_eq!(
        responses[1]["error"]["code"], -32602,
        "{label} produced an allow-empty resource success: {responses:#?}"
    );
    assert!(
        responses[1]["error"]["message"]
            .as_str()
            .is_some_and(|error| error.contains(expected)),
        "{label} resource rejection was not explicit: {responses:#?}"
    );
}

fn mcp_request_with_write(
    root: &std::path::Path,
    requests: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    let input = requests
        .iter()
        .map(|request| serde_json::to_string(request).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let output = specsync()
        .arg("mcp")
        .arg("--allow-write")
        .arg("--root")
        .arg(root)
        .write_stdin(input)
        .output()
        .expect("failed to run write-enabled mcp");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("invalid JSON-RPC response"))
        .collect()
}

fn assert_generation_failed_without_zero_count(response: &serde_json::Value) {
    assert_eq!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.contains("\"count\": 0"));
}

#[cfg(windows)]
fn create_windows_junction(
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    let output = std::process::Command::new("cmd")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(junction)
        .arg(target)
        .output()
        .map_err(|error| format!("failed to launch cmd /C mklink /J: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "cmd /C mklink /J exited with {:?}; stdout: {}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}
