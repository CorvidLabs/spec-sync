use super::super::super::{validate_spec, validate_spec_content};
use crate::types::SpecSyncConfig;
use std::collections::{HashMap, HashSet};
use std::fs;

#[test]
fn supplied_content_validation_normalizes_crlf_identically() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
    let path = root.join("specs/auth/auth.spec.md");
    let lf =
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n";
    let crlf = lf.replace('\n', "\r\n");
    let tables = HashSet::new();
    let columns = HashMap::new();
    let config = SpecSyncConfig::default();

    let lf_result = validate_spec_content(&path, lf, root, &tables, &columns, &config);
    let crlf_result = validate_spec_content(&path, &crlf, root, &tables, &columns, &config);

    assert_eq!(crlf_result.errors, lf_result.errors);
    assert_eq!(crlf_result.warnings, lf_result.warnings);
    assert_eq!(crlf_result.notices, lf_result.notices);
}

#[test]
fn supplied_content_size_policy_uses_supplied_bytes_not_path_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
    let path = root.join("specs/auth/auth.spec.md");
    let content = format!(
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n{}\n",
        "x".repeat(3 * 1024)
    );
    let mut config = SpecSyncConfig::default();
    config.rules.max_spec_size_kb = Some(1);

    let result = validate_spec_content(
        &path,
        &content,
        root,
        &HashSet::new(),
        &HashMap::new(),
        &config,
    );

    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("exceeds limit of 1 KB")),
        "{:?}",
        result.warnings
    );
}

#[test]
fn test_validate_spec_missing_frontmatter() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("bad.spec.md");
    fs::write(&spec, "# No frontmatter\n\nJust text.").unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("frontmatter"));
}

#[test]
fn test_validate_spec_missing_required_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("partial.spec.md");
    fs::write(&spec, "---\nmodule: test\n---\n\n## Purpose\nTest\n").unwrap();

    let tables = HashSet::new();
    let schema_cols = HashMap::new();
    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &tables, &schema_cols, &config);
    // Should have errors for missing version, status, files
    assert!(result.errors.iter().any(|e| e.contains("version")));
    assert!(result.errors.iter().any(|e| e.contains("status")));
    assert!(result.errors.iter().any(|e| e.contains("files")));
}

#[test]
fn test_validate_spec_missing_source_file() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("missing.spec.md");
    fs::write(
            &spec,
            "---\nmodule: test\nversion: 1\nstatus: active\nfiles:\n  - src/nonexistent.ts\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
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
            .any(|e| e.contains("Source file not found"))
    );
}

#[test]
fn test_validate_spec_non_utf8_source_fails_loud() {
    // Regression: a source file that exists but is not valid UTF-8 must
    // produce an ERROR, not silently contribute zero exports (which would
    // let a spec documenting nothing pass — a silent false-PASS).
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    // Valid JS export plus a stray Latin-1 byte (0xE9) → invalid UTF-8.
    fs::write(
        src_dir.join("bad.ts"),
        b"export function chargeCard() {}\n// \xE9",
    )
    .unwrap();

    let spec = tmp.path().join("bad.spec.md");
    fs::write(
            &spec,
            "---\nmodule: bad\nversion: 1\nstatus: active\nfiles:\n  - src/bad.ts\n---\n\n## Purpose\nTest\n## Requirements\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n",
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
            .any(|e| e.contains("could not be read as UTF-8")),
        "expected a UTF-8 read error, got: {:?}",
        result.errors
    );
}
