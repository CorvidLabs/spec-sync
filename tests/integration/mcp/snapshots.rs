use super::*;

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

// Drives the product's snapshot-file rendezvous (`wait_for_mcp_snapshot_file_test_barrier`),
// which is `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it
// synchronises is compiled unconditionally, and
// `mcp::tests::generic_snapshot_reader_rejects_fifo_symlink_and_regular_replacements`
// covers that same guard in-process (via a `#[cfg(test)]` hook) under `--release`.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "MCP snapshot-file rendezvous is #[cfg(debug_assertions)]"
)]
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

// Drives the product's snapshot-directory rendezvous (`wait_for_mcp_snapshot_directory_test_barrier`),
// which is `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it
// synchronises is compiled unconditionally, and
// `mcp::tests::snapshot_rejects_directory_replacement_after_enumeration_before_recursion`
// covers that same guard in-process (via a `#[cfg(test)]` hook) under `--release`.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "MCP snapshot-directory rendezvous is #[cfg(debug_assertions)]"
)]
fn mcp_tool_and_resource_reject_directory_replacement_after_enumeration() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_IDENTITY_BARRIER";
    const TARGET_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_PATH";

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
        let tmp = TempDir::new().unwrap();
        let control = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let barrier = control.path().join(format!("barrier-{request_kind}"));
        fs::create_dir_all(&barrier).unwrap();

        let binary = specsync().get_program().to_os_string();
        let mut child = Command::new(binary)
            .arg("mcp")
            .arg("--root")
            .arg(&root)
            .env(BARRIER_ENV, &barrier)
            .env(TARGET_ENV, ".")
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

        let ready = barrier.join("enumerated-open");
        let ready_deadline = Instant::now() + Duration::from_secs(5);
        while !ready.is_file() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "{request_kind} exited before the directory-enumeration barrier"
            );
            assert!(
                Instant::now() < ready_deadline,
                "{request_kind} did not reach the directory-enumeration barrier"
            );
            thread::sleep(Duration::from_millis(10));
        }

        fs::rename(root.join("src"), root.join("retained-src")).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("src/attacker.rs"),
            "pub fn post_enumeration_attacker() {}\n",
        )
        .unwrap();
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
                    "{request_kind} directory replacement blocked; stdout: {}; stderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(10));
        }

        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{request_kind}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        if request_kind == "tool" {
            assert_eq!(response["result"]["isError"], true, "{response}");
        } else {
            assert_eq!(response["error"]["code"], -32602, "{response}");
        }
        let rendered = serde_json::to_string(&response).unwrap();
        assert!(
            rendered.contains("changed during snapshot traversal"),
            "{request_kind}: {rendered}"
        );
        assert!(
            !rendered.contains("post_enumeration_attacker"),
            "{request_kind}: {rendered}"
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
