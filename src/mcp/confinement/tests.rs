use super::*;
use tempfile::TempDir;

#[test]
fn test_repeated_tree_scans_share_one_confinement_budget() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    let exclusions = HashSet::new();
    let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

    validate_configured_path_with_budget(
        tmp.path(),
        "src",
        "source_dirs entry",
        Some(&exclusions),
        &mut entries_seen,
    )
    .unwrap();
    let error = validate_configured_path_with_budget(
        tmp.path(),
        "src",
        "source_dirs entry",
        Some(&exclusions),
        &mut entries_seen,
    )
    .unwrap_err();

    assert!(error.contains("confinement scan exceeds"));
}
