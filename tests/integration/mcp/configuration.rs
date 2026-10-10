use super::*;

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
