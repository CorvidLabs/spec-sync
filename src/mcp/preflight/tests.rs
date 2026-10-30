use super::*;
use tempfile::TempDir;

#[test]
fn test_cargo_manifest_preflight_enforces_manifest_bound() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = MAX_MANIFEST_PREFLIGHTS;
    let mut traversal_entries_seen = 0;

    let error = validate_cargo_workspace_manifest(
        tmp.path(),
        tmp.path(),
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )
    .unwrap_err();
    assert!(error.contains("preflight exceeds"));
}
