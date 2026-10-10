use super::*;

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

// Drives the product's read-root rendezvous (`wait_for_mcp_read_root_revalidation_test_barrier`),
// which is `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it
// synchronises (`ConfinedReadRoot::revalidate_before_success`) is compiled unconditionally
// and is present in the shipped binary.
//
// That guard has no release-runnable test; this test is its only coverage. Note the
// rendezvous sits inside `revalidate_before_success`, which runs on every read-success
// path — compiling it unconditionally would mean a repeated 30s stall per operation, not
// a one-shot one.
#[cfg(any(unix, windows))]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "MCP read-root rendezvous is #[cfg(debug_assertions)]"
)]
fn mcp_read_root_route_rejects_or_os_blocks_link_replacement_before_success() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_READ_ROOT_REVALIDATION_BARRIER";

    let tmp = TempDir::new().unwrap();
    let control = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let selected = root.join("selected");
    let retained = root.join("retained-selected");
    let outside = tmp.path().join("outside");
    let barrier = control.path().join("read-root-route");
    setup_minimal_mcp_project_at(&selected.join("leaf"));
    fs::create_dir_all(outside.join("leaf")).unwrap();
    let victim = outside.join("victim.bin");
    let victim_bytes = b"read-root route replacement victim";
    fs::write(&victim, victim_bytes).unwrap();
    fs::create_dir_all(&barrier).unwrap();

    let binary = specsync().get_program().to_os_string();
    let mut child = Command::new(binary)
        .arg("mcp")
        .arg("--root")
        .arg(&root)
        .env(BARRIER_ENV, &barrier)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("read-root route MCP server must start");
    {
        let mut stdin = child.stdin.take().expect("MCP stdin must be piped");
        serde_json::to_writer(
            &mut stdin,
            &coverage_request(1, serde_json::json!("selected/leaf")),
        )
        .unwrap();
        stdin.write_all(b"\n").unwrap();
    }

    let ready = barrier.join("operation-complete");
    let ready_deadline = Instant::now() + Duration::from_secs(10);
    while !ready.is_file() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "MCP server exited before read-root route revalidation"
        );
        assert!(
            Instant::now() < ready_deadline,
            "MCP server did not reach read-root route revalidation"
        );
        thread::sleep(Duration::from_millis(10));
    }

    let route_replaced = match fs::rename(&selected, &retained) {
        Ok(()) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;

                symlink(&outside, &selected).unwrap();
            }
            #[cfg(windows)]
            create_windows_junction(&selected, &outside)
                .unwrap_or_else(|error| panic!("failed to create replacement junction: {error}"));
            true
        }
        Err(error) => {
            #[cfg(unix)]
            panic!("selected route parent must be replaceable: {error}");
            #[cfg(windows)]
            {
                assert_eq!(
                    error.kind(),
                    std::io::ErrorKind::PermissionDenied,
                    "Windows may block the route replacement while its descendant capability is retained, but no other rename failure is expected: {error}"
                );
                assert!(
                    selected.is_dir() && !retained.exists(),
                    "a denied Windows rename must leave the original selected route intact"
                );
                false
            }
        }
    };
    fs::write(barrier.join("resume"), b"resume\n").unwrap();

    let exit_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= exit_deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "read-root route replacement blocked; stdout: {}; stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "read-root route MCP server failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    if route_replaced {
        assert_eq!(response["result"]["isError"], true, "{response}");
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|message| {
                    message.contains("read-root route changed before operation success")
                }),
            "{response}"
        );
    } else {
        assert_eq!(
            response["result"]["isError"],
            serde_json::Value::Null,
            "an OS-denied replacement must leave the retained original route usable: {response}"
        );
    }
    assert_eq!(fs::read(&victim).unwrap(), victim_bytes);

    #[cfg(unix)]
    fs::remove_file(&selected).unwrap();
    #[cfg(windows)]
    if route_replaced {
        fs::remove_dir(&selected).unwrap();
    }
}

#[test]
fn mcp_read_roots_reject_git_metadata_as_operation_authority() {
    for (id, requested_root) in [".git", ".GIT", ".GiT", "child/.gIt"]
        .into_iter()
        .enumerate()
    {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let selected = root.join(requested_root);
        fs::create_dir_all(selected.join("src")).unwrap();
        fs::write(
            selected.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            selected.join("src/lib.rs"),
            "pub fn hidden_authority() {}\n",
        )
        .unwrap();

        let responses = mcp_request(
            &root,
            &[coverage_request(
                u64::try_from(id + 1).unwrap(),
                serde_json::json!(requested_root),
            )],
        );

        assert_eq!(responses[0]["result"]["isError"], true);
        assert_eq!(
            responses[0]["result"]["content"][0]["text"],
            "Read root override must not select Git metadata"
        );
    }
}

#[cfg(unix)]
#[test]
fn mcp_read_roots_reject_a_symlink_alias_to_git_metadata() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    fs::create_dir_all(root.join(".git/src")).unwrap();
    fs::write(
        root.join(".git/specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    symlink(root.join(".git"), root.join("innocent-alias")).unwrap();

    let responses = mcp_request(
        &root,
        &[coverage_request(1, serde_json::json!("innocent-alias"))],
    );

    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|message| message.contains("regular confined directories"))
    );
}

#[cfg(windows)]
#[test]
fn mcp_read_roots_reject_a_junction_alias_to_git_metadata() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    fs::create_dir_all(root.join(".git/src")).unwrap();
    fs::write(
        root.join(".git/specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    create_windows_junction(&root.join("innocent-alias"), &root.join(".git"))
        .unwrap_or_else(|error| panic!("failed to create Git metadata junction alias: {error}"));

    let responses = mcp_request(
        &root,
        &[coverage_request(1, serde_json::json!("innocent-alias"))],
    );

    assert_eq!(responses[0]["result"]["isError"], true);
    assert!(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|message| message.contains("regular confined directories"))
    );
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

// Drives the product's startup rendezvous (`wait_for_mcp_startup_test_barrier`), which is
// `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it synchronises
// (the identity comparison in `open_server_root_capability`) is compiled unconditionally.
//
// This guard does keep release coverage:
// `mcp::tests::server_root_capability_rejects_a_root_replaced_before_canonicalization`
// (src/mcp.rs) calls `open_server_root_capability` with a swap closure at the same seam the
// rendezvous occupies, and is gated only on `unix`, so it runs under `--release`.
//
// Gated with `ignore` rather than `cfg` so the test stays compiled — and therefore
// type-checked — in release builds, and stays visible in the run output.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "MCP startup rendezvous is #[cfg(debug_assertions)]"
)]
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
