use super::super::super::{SourceSnapshot, validate_spec, validate_spec_content_with_sources};
use crate::types::SpecSyncConfig;
use std::collections::{HashMap, HashSet};
use std::fs;

/// #578: a spec body carrying live markers used to pass with
/// "✓ All required sections present" — the tool asserting a contract over
/// text that is not a document.
#[test]
fn a_spec_body_with_an_unresolved_conflict_fails_validation() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("calc.spec.md");
    fs::write(
        &spec,
        "---\nmodule: calc\nversion: 1\nstatus: active\nfiles: []\n---\n\n\
             ## Purpose\n\nMath.\n\n## Invariants\n\n\
             <<<<<<< HEAD\n1. Pure functions.\n=======\n1. Total on i32.\n>>>>>>> feature/other\n",
    )
    .unwrap();

    let result = validate_spec(
        &spec,
        tmp.path(),
        &HashSet::new(),
        &HashMap::new(),
        &SpecSyncConfig::default(),
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Unresolved merge conflict in spec body")
                && e.contains("HEAD")
                && e.contains("feature/other")),
        "{:?}",
        result.errors
    );
}

/// The companion false-positive guard: a spec that *documents* markers in a
/// fenced example is a legitimate document and must still validate.
#[test]
fn a_spec_documenting_markers_inside_a_fence_still_validates() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("merge.spec.md");
    fs::write(
        &spec,
        "---\nmodule: merge\nversion: 1\nstatus: active\nfiles: []\n---\n\n\
             ## Purpose\n\nDocuments conflict handling.\n\n## Invariants\n\n\
             ```\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n```\n",
    )
    .unwrap();

    let result = validate_spec(
        &spec,
        tmp.path(),
        &HashSet::new(),
        &HashMap::new(),
        &SpecSyncConfig::default(),
    );
    assert!(
        !result
            .errors
            .iter()
            .any(|e| e.contains("Unresolved merge conflict")),
        "{:?}",
        result.errors
    );
}

/// The end-to-end shape of the confirmed repro: source conflicted, spec
/// documenting both sides. Before the fix this reported
/// `3/3 exports documented` and passed.
#[test]
fn a_spec_over_conflicted_source_fails_instead_of_documenting_the_union() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/calc.rs"),
        "pub fn add() {}\n\
             <<<<<<< HEAD\npub fn sub() {}\n=======\npub fn mul() {}\n>>>>>>> feature/other\n",
    )
    .unwrap();
    let spec = tmp.path().join("calc.spec.md");
    fs::write(
        &spec,
        "---\nmodule: calc\nversion: 1\nstatus: active\nfiles:\n  - src/calc.rs\n---\n\n\
             ## Purpose\n\nMath.\n\n## Public API\n\n### Exported Functions\n\n\
             | Function | Description |\n|---|---|\n\
             | `add` | adds |\n| `sub` | subtracts |\n| `mul` | multiplies |\n",
    )
    .unwrap();

    let result = validate_spec(
        &spec,
        tmp.path(),
        &HashSet::new(),
        &HashMap::new(),
        &SpecSyncConfig::default(),
    );
    assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Unresolved merge conflict in source")
                    && e.contains("src/calc.rs")),
            "{:?}",
            result.errors
        );
    assert_eq!(
        result.export_summary, None,
        "no export summary may be reported over a tree that does not exist"
    );
}

/// The third path. `specsync issues` never calls `validate_spec` — it
/// pre-reads bytes through `snapshot_source_file` and validates the retained
/// snapshot. A fix that stopped at the path-based read would leave this
/// entry point with the original union bug.
#[test]
fn the_snapshot_validation_path_also_refuses_a_conflicted_source() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("calc.spec.md");
    let spec_content = "---\nmodule: calc\nversion: 1\nstatus: active\nfiles:\n  - src/calc.rs\n---\n\n\
             ## Purpose\n\nMath.\n\n## Public API\n\n### Exported Functions\n\n\
             | Function | Description |\n|---|---|\n\
             | `add` | adds |\n| `sub` | subtracts |\n| `mul` | multiplies |\n";

    let conflicted = "pub fn add() {}\n\
             <<<<<<< HEAD\npub fn sub() {}\n=======\npub fn mul() {}\n>>>>>>> feature/other\n";
    let mut sources = HashMap::new();
    sources.insert(
        "src/calc.rs".to_string(),
        SourceSnapshot::Present(conflicted.as_bytes().to_vec()),
    );

    let result = validate_spec_content_with_sources(
        &spec,
        spec_content,
        tmp.path(),
        &HashSet::new(),
        &HashMap::new(),
        &SpecSyncConfig::default(),
        &sources,
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Unresolved merge conflict in source")),
        "{:?}",
        result.errors
    );
    assert_eq!(result.export_summary, None);

    // Healthy control on the same entry point: a clean snapshot exporting
    // all three names still validates and still reports its summary.
    let clean = "pub fn add() {}\npub fn sub() {}\npub fn mul() {}\n";
    let mut clean_sources = HashMap::new();
    clean_sources.insert(
        "src/calc.rs".to_string(),
        SourceSnapshot::Present(clean.as_bytes().to_vec()),
    );
    let clean_result = validate_spec_content_with_sources(
        &spec,
        spec_content,
        tmp.path(),
        &HashSet::new(),
        &HashMap::new(),
        &SpecSyncConfig::default(),
        &clean_sources,
    );
    assert!(
        !clean_result
            .errors
            .iter()
            .any(|e| e.contains("Unresolved merge conflict")),
        "{:?}",
        clean_result.errors
    );
    assert_eq!(
        clean_result.export_summary,
        Some("3/3 exports documented".to_string())
    );
}
