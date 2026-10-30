//! Shared MCP test fixtures. Compiled only for tests.

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde_json::json;
use std::path::Path;
use tempfile::TempDir;
use walkdir::WalkDir;

pub(super) fn open_test_directory(path: &Path) -> Dir {
    Dir::open_ambient_dir(path, ambient_authority()).unwrap()
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

pub(super) fn quarantine_contains(parent: &Dir, expected: &[u8]) -> bool {
    parent
        .read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".specsync-mcp-quarantine-")
        })
        .any(|entry| {
            let Ok(directory) = entry.open_dir() else {
                return false;
            };
            if directory.read("entry").is_ok_and(|bytes| bytes == expected) {
                return true;
            }
            let Ok(nested) = directory.open_dir("entry") else {
                return false;
            };
            nested
                .read("replacement.txt")
                .is_ok_and(|bytes| bytes == expected)
        })
}
