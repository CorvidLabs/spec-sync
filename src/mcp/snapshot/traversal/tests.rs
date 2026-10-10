use super::*;
use crate::mcp::tests::{open_test_directory, setup_project};
use std::fs;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn snapshot_rejects_directory_replacement_after_enumeration_before_recursion() {
    let source_temp = setup_project();
    let destination_temp = TempDir::new().unwrap();
    let source_root = source_temp.path();
    let original = source_root.join("src");
    let retained = source_root.join("retained-src");
    let destination = open_test_directory(destination_temp.path());
    let source = open_test_directory(source_root);
    let mut budget = SnapshotBudget::default();
    let mut replaced = false;

    let error = copy_snapshot_directory_with_hook(
        &source,
        &destination,
        Path::new("."),
        &HashSet::new(),
        &[PathBuf::from("src"), PathBuf::from("specs")],
        &mut budget,
        |relative| {
            if relative == Path::new(".") && !replaced {
                fs::rename(&original, &retained).unwrap();
                fs::create_dir(&original).unwrap();
                fs::write(original.join("attacker.rs"), "pub fn attacker() {}\n").unwrap();
                replaced = true;
            }
            Ok(())
        },
    )
    .expect_err("a post-enumeration directory replacement must fail closed");

    assert!(
        error.contains("changed during snapshot traversal"),
        "{error}"
    );
    assert!(error.contains("src"), "{error}");
    assert!(!destination_temp.path().join("src/attacker.rs").exists());
}
