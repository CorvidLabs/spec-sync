use super::{find_spec_files, is_cross_project_ref, parse_cross_project_ref, validate_spec};
use crate::types::SpecSyncConfig;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[test]
fn test_is_cross_project_ref() {
    assert!(is_cross_project_ref("corvid-labs/algochat@auth"));
    assert!(is_cross_project_ref("owner/repo@module"));
    assert!(!is_cross_project_ref("specs/auth/auth.spec.md"));
    assert!(!is_cross_project_ref("auth"));
    assert!(!is_cross_project_ref("owner/repo")); // no @
    assert!(!is_cross_project_ref("@module")); // no /
}

#[test]
fn test_parse_cross_project_ref() {
    let (repo, module) = parse_cross_project_ref("corvid-labs/algochat@auth").unwrap();
    assert_eq!(repo, "corvid-labs/algochat");
    assert_eq!(module, "auth");

    assert!(parse_cross_project_ref("not-a-ref").is_none());
    assert!(parse_cross_project_ref("/@").is_none()); // empty parts
}

#[test]
fn test_find_spec_files_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let files = find_spec_files(tmp.path());
    assert!(files.is_empty());
}

#[test]
fn test_find_spec_files_nonexistent() {
    let files = find_spec_files(Path::new("/nonexistent/path"));
    assert!(files.is_empty());
}

#[test]
fn test_find_spec_files_with_specs() {
    let tmp = tempfile::tempdir().unwrap();
    let spec_dir = tmp.path().join("auth");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(spec_dir.join("auth.spec.md"), "---\nmodule: auth\n---\n").unwrap();
    fs::write(spec_dir.join("not-a-spec.md"), "other").unwrap();

    let files = find_spec_files(tmp.path());
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("auth.spec.md"));
}

#[test]
fn test_validate_spec_bare_dep_name_resolves_via_specs_dir() {
    let tmp = tempfile::tempdir().unwrap();

    // Create the dependency module directory
    let dep_dir = tmp.path().join("specs").join("run");
    fs::create_dir_all(&dep_dir).unwrap();
    fs::write(dep_dir.join("run.spec.md"), "# fixture content").unwrap();

    let spec = tmp.path().join("deps.spec.md");
    fs::write(
            &spec,
            "---\nmodule: deps\nversion: 1\nstatus: active\nfiles: []\ndepends_on:\n  - run\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default(); // specs_dir = "specs"
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);

    // Bare module name "run" should resolve to specs/run — no dep error
    assert!(
        !result
            .errors
            .iter()
            .any(|e| e.contains("Dependency spec not found")),
        "bare dep name should resolve via specs dir: {:?}",
        result.errors
    );
}
