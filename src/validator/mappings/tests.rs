use crate::types::SpecSyncConfig;
use crate::validator::validate_spec;
use std::collections::{HashMap, HashSet};
use std::fs;

#[test]
fn directory_source_mapping_fails_loud_and_names_the_files_to_list() {
    // Regression (#472): a `files:` entry that resolves to a directory yields
    // zero exports, so the whole Public API comparison used to be skipped while
    // the entry still counted as an existing source file — a spec documenting
    // nothing passed `check --strict`. The directory must fail loud instead.
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src/provider");
    fs::create_dir_all(src_dir.join("nested")).unwrap();
    fs::write(src_dir.join("provider.ts"), "export class Provider {}\n").unwrap();
    fs::write(
        src_dir.join("nested/deep.ts"),
        "export function deep() {}\n",
    )
    .unwrap();

    let spec = tmp.path().join("provider.spec.md");
    fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: stable\nfiles:\n  - src/provider\n---\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("`src/provider` is a directory")),
        "expected a directory mapping error, got: {:?}",
        result.errors
    );
    assert!(
        result.fixes.iter().any(|fix| {
            fix.contains("src/provider/provider.ts") && fix.contains("src/provider/nested/deep.ts")
        }),
        "expected the fix to name the source files beneath the directory, got: {:?}",
        result.fixes
    );
}

#[test]
fn directory_source_mapping_does_not_disturb_sibling_file_mappings() {
    // A mixed list keeps validating its file entries: the directory entry is the
    // only failure, and the real file's undocumented export is still reported.
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src/provider");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("provider.ts"), "export class Provider {}\n").unwrap();

    let spec = tmp.path().join("provider.spec.md");
    fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: stable\nfiles:\n  - src/provider\n  - src/provider/provider.ts\n---\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
    assert_eq!(
        result
            .errors
            .iter()
            .filter(|error| error.contains("is a directory"))
            .count(),
        1,
        "only the directory entry may fail: {:?}",
        result.errors
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("Provider")),
        "the file entry must still be scanned for exports: {:?}",
        result.warnings
    );
}

#[test]
fn draft_directory_source_mapping_is_not_a_planned_mapping_notice() {
    // A draft may map a file that does not exist yet; an existing directory is a
    // wrong-shaped mapping in any status and must not pass as a planned mapping.
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src/provider")).unwrap();

    let spec = tmp.path().join("provider.spec.md");
    fs::write(
            &spec,
            "---\nmodule: provider\nversion: 1\nstatus: draft\nfiles:\n  - src/provider\n---\n\n## Purpose\nTest\n",
        )
        .unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("is a directory")),
        "expected a directory mapping error for a draft, got: {:?}",
        result.errors
    );
    assert!(
        result.notices.is_empty(),
        "a directory must not be recorded as a planned mapping: {:?}",
        result.notices
    );
}

#[test]
fn test_validate_spec_absolute_path_source_is_rejected() {
    // Security: a `files:` entry resolving OUTSIDE the project root (here an
    // absolute path into a different directory) must be rejected, and its
    // exported identifiers must never leak into the report.
    let root_tmp = tempfile::tempdir().unwrap();
    let outside_tmp = tempfile::tempdir().unwrap();
    let secret = outside_tmp.path().join("secret.ts");
    fs::write(&secret, "export const AWS_SECRET_ACCESS_KEY = \"leak\";\n").unwrap();

    let spec = root_tmp.path().join("s.spec.md");
    let body = format!(
        "---\nmodule: s\nversion: 1\nstatus: active\nfiles:\n  - {}\n---\n\n## Purpose\nx\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        secret.display()
    );
    fs::write(&spec, body).unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, root_tmp.path(), &tables, &schema_cols, &config);

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("outside the project root")),
        "expected an out-of-root rejection, got: {:?}",
        result.errors
    );
    assert!(
        !result
            .warnings
            .iter()
            .any(|w| w.contains("AWS_SECRET_ACCESS_KEY")),
        "out-of-root export leaked into warnings: {:?}",
        result.warnings
    );
}

#[test]
fn test_validate_spec_parent_escape_source_is_rejected() {
    // Security: `..` traversal that leaves the project root is rejected.
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        parent.path().join("outside.ts"),
        "export function leakMe() {}\n",
    )
    .unwrap();

    let spec = root.join("s.spec.md");
    fs::write(
            &spec,
            "---\nmodule: s\nversion: 1\nstatus: active\nfiles:\n  - ../outside.ts\n---\n\n## Purpose\nx\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
        )
        .unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, &root, &tables, &schema_cols, &config);

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("outside the project root")),
        "expected an out-of-root rejection for `..` escape, got: {:?}",
        result.errors
    );
    assert!(
        !result.warnings.iter().any(|w| w.contains("leakMe")),
        "escaped export leaked into warnings: {:?}",
        result.warnings
    );
}
