use super::*;

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
