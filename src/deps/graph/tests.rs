//! Graph construction, validation, cycles and ordering regressions.

use super::*;
use crate::deps::test_support::create_spec;
use tempfile::TempDir;

#[test]
fn test_extract_module_from_dep_path() {
    assert_eq!(
        extract_module_from_dep_path("specs/types/types.spec.md"),
        Some("types".to_string())
    );
    assert_eq!(
        extract_module_from_dep_path("specs/parser/parser.spec.md"),
        Some("parser".to_string())
    );
    assert_eq!(
        extract_module_from_dep_path("types"),
        Some("types".to_string())
    );
    assert_eq!(extract_module_from_dep_path("foo/bar.txt"), None);
}

#[test]
fn test_build_dep_graph_empty() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    let graph = build_dep_graph(tmp.path(), "specs");
    assert!(graph.is_empty());
}

#[test]
fn test_build_dep_graph_basic() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "auth", &[], &[]);
    create_spec(tmp.path(), "api", &["specs/auth/auth.spec.md"], &[]);

    let graph = build_dep_graph(tmp.path(), "specs");
    assert_eq!(graph.len(), 2);
    assert!(graph.contains_key("auth"));
    assert!(graph.contains_key("api"));
    assert_eq!(graph["api"].declared_deps, vec!["auth".to_string()]);
    assert!(graph["auth"].declared_deps.is_empty());
}

#[test]
fn test_validate_no_errors() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "types", &[], &[]);
    create_spec(tmp.path(), "parser", &["specs/types/types.spec.md"], &[]);
    create_spec(
        tmp.path(),
        "validator",
        &["specs/types/types.spec.md", "specs/parser/parser.spec.md"],
        &[],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(report.module_count, 3);
    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
    assert!(report.cycles.is_empty());
    assert!(report.missing_deps.is_empty());
}

#[test]
fn test_validate_missing_dep() {
    let tmp = TempDir::new().unwrap();
    create_spec(
        tmp.path(),
        "api",
        &["specs/nonexistent/nonexistent.spec.md"],
        &[],
    );

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(report.missing_deps.len(), 1);
    assert_eq!(report.missing_deps[0].0, "api");
    assert_eq!(report.missing_deps[0].1, "nonexistent");
    assert!(!report.errors.is_empty());
}

#[test]
fn test_detect_circular_deps() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
    create_spec(tmp.path(), "b", &["specs/a/a.spec.md"], &[]);

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        !report.cycles.is_empty(),
        "Expected circular dependency, got none"
    );
    assert!(!report.errors.is_empty());
    assert!(
        report.errors.iter().any(|e| e.contains("Circular")),
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn test_detect_three_node_cycle() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
    create_spec(tmp.path(), "b", &["specs/c/c.spec.md"], &[]);
    create_spec(tmp.path(), "c", &["specs/a/a.spec.md"], &[]);

    let report = validate_deps(tmp.path(), "specs");
    assert!(!report.cycles.is_empty());
}

#[test]
fn test_flow_style_depends_on_counts_edges() {
    // `depends_on: [b]` (the flow style `specsync new` scaffolds) must
    // produce a real edge, not silently parse as zero dependencies.
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "b", &[], &[]);
    let spec_dir = tmp.path().join("specs").join("a");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
            spec_dir.join("a.spec.md"),
            "---\nmodule: a\nversion: 1\nstatus: active\nfiles: []\ndepends_on: [b]\n---\n\n# A\n\n## Purpose\nTest\n",
        )
        .unwrap();

    let report = validate_deps(tmp.path(), "specs");
    assert_eq!(report.edge_count, 1, "report: {report:?}");
    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
}

#[test]
fn test_deps_rejects_escapes_and_missing_with_same_verdict_as_check() {
    let tmp = TempDir::new().unwrap();
    let spec_dir = tmp.path().join("specs").join("a");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
            spec_dir.join("a.spec.md"),
            "---\nmodule: a\nversion: 1\nstatus: active\nfiles: []\ndepends_on: [/etc/passwd, nosuchmod]\n---\n\n# A\n\n## Purpose\nTest\n",
        )
        .unwrap();

    let report = validate_deps(tmp.path(), "specs");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("escapes the project root") && e.contains("/etc/passwd")),
        "errors: {:?}",
        report.errors
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("Dependency spec not found: nosuchmod")),
        "errors: {:?}",
        report.errors
    );
}

#[test]
fn test_cross_project_refs_skipped() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "api", &["corvid-labs/algochat@auth"], &[]);

    let report = validate_deps(tmp.path(), "specs");
    // Cross-project refs should not be treated as missing
    assert!(report.missing_deps.is_empty());
    assert!(report.errors.is_empty());
}

#[test]
fn test_topological_sort_valid() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "types", &[], &[]);
    create_spec(tmp.path(), "parser", &["specs/types/types.spec.md"], &[]);
    create_spec(
        tmp.path(),
        "validator",
        &["specs/types/types.spec.md", "specs/parser/parser.spec.md"],
        &[],
    );

    let graph = build_dep_graph(tmp.path(), "specs");
    let order = topological_sort(&graph);
    assert!(order.is_some(), "Expected valid topological sort");
    let order = order.unwrap();

    // types must come before parser and validator
    let types_pos = order.iter().position(|m| m == "types").unwrap();
    let parser_pos = order.iter().position(|m| m == "parser").unwrap();
    let validator_pos = order.iter().position(|m| m == "validator").unwrap();
    assert!(types_pos < parser_pos);
    assert!(types_pos < validator_pos);
    assert!(parser_pos < validator_pos);
}

#[test]
fn test_topological_sort_cycle() {
    let tmp = TempDir::new().unwrap();
    create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
    create_spec(tmp.path(), "b", &["specs/a/a.spec.md"], &[]);

    let graph = build_dep_graph(tmp.path(), "specs");
    let order = topological_sort(&graph);
    assert!(order.is_none(), "Expected None for cyclic graph");
}
