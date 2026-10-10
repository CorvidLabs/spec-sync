use super::*;

#[cfg(unix)]
fn mcp_request_with_low_fd_limit(
    root: &std::path::Path,
    requests: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    mcp_request_with_fd_limit(root, requests, 128, false)
}

#[cfg(unix)]
fn mcp_request_with_fd_limit(
    root: &std::path::Path,
    requests: &[serde_json::Value],
    descriptor_limit: u64,
    allow_write: bool,
) -> Vec<serde_json::Value> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let binary = specsync().get_program().to_os_string();
    let script = if allow_write {
        r#"ulimit -n "$3"; exec "$1" mcp --root "$2" --allow-write"#
    } else {
        r#"ulimit -n "$3"; exec "$1" mcp --root "$2""#
    };
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg("specsync-fd-limit")
        .arg(binary)
        .arg(root)
        .arg(descriptor_limit.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("descriptor-constrained MCP server must start");
    {
        let mut stdin = child.stdin.take().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut stdin, request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }

    let output = child
        .wait_with_output()
        .expect("descriptor-constrained MCP server must terminate");
    assert!(
        output.status.success(),
        "descriptor-constrained MCP server failed: {}",
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
fn mcp_snapshot_bounds_open_directory_handles_for_broad_trees() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_minimal_mcp_project_at(root);
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src","broad"]}"#,
    )
    .unwrap();
    for index in 0..200 {
        let directory = root.join(format!("broad/module-{index:03}"));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("module.rs"), "pub fn bounded() {}\n").unwrap();
    }

    let responses = mcp_request_with_low_fd_limit(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_coverage", "arguments": {} }
        })],
    );

    assert_eq!(responses.len(), 1);
    assert_ne!(
        responses[0]["result"]["isError"], true,
        "a valid broad tree must fit beneath a 128-descriptor process limit: {}",
        responses[0]
    );
}

#[cfg(unix)]
#[test]
fn mcp_generate_maximum_batch_fits_a_constrained_descriptor_limit() {
    const GENERATED_SPECS: usize = 1_000;
    const DESCRIPTOR_LIMIT: u64 = 1_200;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    write_config(root, "specs", &["src"]);
    for index in 0..GENERATED_SPECS {
        let module = format!("module{index:04}");
        let source = root.join("src").join(&module);
        fs::create_dir(&source).unwrap();
        fs::write(
            source.join("lib.rs"),
            format!("pub fn value_{index:04}() -> usize {{ {index} }}\n"),
        )
        .unwrap();
    }

    let responses = mcp_request_with_fd_limit(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "specsync_generate", "arguments": {} }
        })],
        DESCRIPTOR_LIMIT,
        true,
    );

    assert_eq!(responses.len(), 1);
    assert_ne!(
        responses[0]["result"]["isError"], true,
        "the maximum generated batch must fit beneath a {DESCRIPTOR_LIMIT}-descriptor limit: {}",
        responses[0]
    );
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("successful generation must return text content");
    let result: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(result["count"].as_u64(), Some(GENERATED_SPECS as u64));
    assert!(root.join("specs/module0000/module0000.spec.md").is_file());
    assert!(root.join("specs/module0999/module0999.spec.md").is_file());
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
