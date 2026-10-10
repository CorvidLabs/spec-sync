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

#[test]
fn cargo_preflight_charges_duplicate_members_without_replaying_manifests() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("crates/member")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0;
    let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

    validate_cargo_workspace_manifest(
        tmp.path(),
        tmp.path(),
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )
    .expect("duplicate members must fit exactly at the declaration limit");

    assert_eq!(traversal_entries_seen, MAX_CONFINEMENT_ENTRIES);
    assert_eq!(manifests_seen, 2);

    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
    )
    .unwrap();
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0;
    let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
    let error = validate_cargo_workspace_manifest(
        tmp.path(),
        tmp.path(),
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )
    .expect_err("the limit-plus-one duplicate declaration must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
    assert_eq!(manifests_seen, 2);
}

#[test]
fn node_preflight_deduplicates_patterns_bases_and_workspace_paths() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("packages/member")).unwrap();
    fs::write(
        tmp.path().join("packages/member/package.json"),
        r#"{"name":"member"}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    let mut exact_entries_seen = MAX_CONFINEMENT_ENTRIES - 4;

    validate_package_workspaces(tmp.path(), &mut exact_entries_seen)
        .expect("three declarations plus one unique workspace must fit exactly");

    assert_eq!(exact_entries_seen, MAX_CONFINEMENT_ENTRIES);

    let mut overflow_entries_seen = MAX_CONFINEMENT_ENTRIES - 3;
    let error = validate_package_workspaces(tmp.path(), &mut overflow_entries_seen)
        .expect_err("the limit-plus-one Node declaration must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
}

#[test]
fn node_preflight_fails_closed_and_charges_entries_before_type_validation() {
    for (content, expected) in [
        ("{", "Cannot parse MCP package.json as JSON"),
        ("false", "root must be a JSON object"),
        (
            r#"{"workspaces":"packages/*"}"#,
            "`workspaces` must be an array or an object",
        ),
        (
            r#"{"workspaces":{"packages":{}}}"#,
            "`workspaces.packages` must be an array",
        ),
        (
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), content).unwrap();
        let mut entries_seen = 0;
        let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
            .expect_err("invalid package.json must make MCP preflight inconclusive");
        assert!(error.contains(expected), "{content}: {error}");
    }

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":[false,"packages/*"]}"#,
    )
    .unwrap();
    let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
    let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
        .expect_err("wrong-typed workspace entries must fail after declaration charging");

    assert!(
        error.contains("workspace entries must be strings"),
        "{error}"
    );
    assert_eq!(entries_seen, MAX_CONFINEMENT_ENTRIES);
}
