use super::*;
use std::fs;
use tempfile::TempDir;
use walkdir::WalkDir;

pub(super) fn open_test_directory(path: &Path) -> Dir {
    Dir::open_ambient_dir(path, ambient_authority()).unwrap()
}

pub(super) fn setup_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "requiredSections": ["Purpose", "Public API"]
    });
    std::fs::write(
        tmp.path().join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    tmp
}

pub(super) fn setup_project_with_spec(spec_name: &str, spec_content: &str) -> TempDir {
    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join(spec_name);
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join(format!("{spec_name}.spec.md")), spec_content).unwrap();
    tmp
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

pub(super) fn assert_no_mcp_transaction_debris(root: &Path) {
    assert!(
        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .all(|entry| {
                let name = entry.file_name().to_string_lossy();
                !name.starts_with(".specsync-mcp-stage-")
                    && !name.starts_with(".specsync-mcp-quarantine-")
            }),
        "successful MCP publication must remove private staging and quarantine entries"
    );
}
