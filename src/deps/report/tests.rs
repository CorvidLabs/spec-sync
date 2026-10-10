//! Analysis disclosures and terminal report regressions.

use super::*;
use crate::deps::test_support::{create_source, create_spec};
use crate::deps::validate_deps;
use tempfile::TempDir;

#[test]
fn test_unanalyzed_language_is_disclosed_not_counted_as_clean() {
    // A language with no import extractor contributes no edges; the report
    // must say so instead of letting the silence read as "no problems".
    let tmp = TempDir::new().unwrap();
    create_source(tmp.path(), "src/core/core.go", "package core\n");
    create_source(
        tmp.path(),
        "src/feature/feature.go",
        "package feature\n\nimport \"example.com/src/core\"\n",
    );
    create_spec(tmp.path(), "core", &[], &["src/core/core.go"]);
    create_spec(tmp.path(), "feature", &[], &["src/feature/feature.go"]);

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(report.unanalyzed_languages, vec![("Go".to_string(), 2)]);
    assert_eq!(
        valid_declarations_line(&report),
        "All dependency declarations are valid for the languages analysed."
    );
    let note = unanalyzed_languages_note(&report).expect("Go must be disclosed");
    assert!(note.contains("Go (2 file(s))"), "got: {note}");
    // Disclosure is not a failure: the exit code is untouched.
    assert!(report.errors.is_empty() && report.warnings.is_empty());
}

#[test]
fn test_both_disclosures_qualify_the_verdict_together() {
    // Unread languages and unattributable imports are separate gaps, and a
    // tree with both must own up to both rather than pick one sentence.
    let tmp = TempDir::new().unwrap();
    create_source(tmp.path(), "src/core/core.go", "package core\n");
    create_source(
        tmp.path(),
        "src/kt/Feature.kt",
        "package com.example.feature\n\nimport com.example.missing.Gone\n",
    );
    create_spec(tmp.path(), "core", &[], &["src/core/core.go"]);
    create_spec(tmp.path(), "feature", &[], &["src/kt/Feature.kt"]);

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(report.unanalyzed_languages, vec![("Go".to_string(), 1)]);
    assert_eq!(
        report.unresolved_imports,
        vec![("feature".to_string(), "com.example.missing".to_string())]
    );
    assert_eq!(
        valid_declarations_line(&report),
        "All dependency declarations are valid for the languages analysed \
             and the imports that resolved."
    );
    assert!(unanalyzed_languages_note(&report).is_some());
    assert!(unresolved_imports_note(&report).is_some());
    assert!(report.errors.is_empty() && report.warnings.is_empty());
}

#[test]
fn test_analyzable_tree_reports_no_unanalyzed_languages() {
    let tmp = TempDir::new().unwrap();
    create_source(tmp.path(), "src/api.py", "from .auth import login\n");
    create_spec(tmp.path(), "api", &[], &["src/api.py"]);
    create_spec(tmp.path(), "auth", &[], &[]);

    let report = validate_deps(tmp.path(), "specs");
    assert!(report.unanalyzed_languages.is_empty());
    assert_eq!(
        valid_declarations_line(&report),
        "All dependency declarations are valid."
    );
    assert!(unanalyzed_languages_note(&report).is_none());
}

#[test]
fn test_language_without_an_import_concept_is_not_disclosed() {
    // A YAML file has no imports to miss, and a shell script names a path
    // rather than a module. Listing them as "not analysed" would bury the
    // real disclosure under noise every pure-Rust project sees.
    let tmp = TempDir::new().unwrap();
    create_source(tmp.path(), "src/lib.rs", "pub fn run() {}\n");
    create_source(tmp.path(), "src/ci.yml", "jobs: {}\n");
    create_source(tmp.path(), "src/tool.sh", "#!/usr/bin/env bash\ntrue\n");
    create_spec(
        tmp.path(),
        "tooling",
        &[],
        &["src/lib.rs", "src/ci.yml", "src/tool.sh"],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report.unanalyzed_languages.is_empty(),
        "neither YAML nor shell has an import to miss: {:?}",
        report.unanalyzed_languages
    );
    assert!(unanalyzed_languages_note(&report).is_none());
    assert_eq!(
        valid_declarations_line(&report),
        "All dependency declarations are valid.",
        "nothing was skipped, so the verdict must not be hedged"
    );
}

#[test]
fn test_format_report_clean() {
    let report = DepsReport {
        module_count: 3,
        edge_count: 2,
        ..DepsReport::default()
    };
    let out = format_report(&report);
    assert!(out.contains("Modules: 3"));
    assert!(out.contains("Edges: 2"));
    assert!(out.contains("valid"));
}

#[test]
fn test_format_report_with_errors() {
    let report = DepsReport {
        module_count: 2,
        edge_count: 1,
        errors: vec!["missing dep".to_string()],
        warnings: vec!["undeclared import".to_string()],
        ..DepsReport::default()
    };
    let out = format_report(&report);
    assert!(out.contains("Errors (1)"));
    assert!(out.contains("Warnings (1)"));
}
