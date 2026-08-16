// Integration test suite for specsync
#[path = "integration/helpers.rs"]
pub mod helpers;

#[path = "integration/check.rs"]
mod check;

#[path = "integration/fix.rs"]
mod fix;

#[path = "integration/commands.rs"]
mod commands;

#[path = "integration/languages.rs"]
mod languages;

#[path = "integration/mcp.rs"]
mod mcp;

#[path = "integration/config.rs"]
mod config;

#[path = "integration/change.rs"]
mod change;

#[path = "integration/comment.rs"]
mod comment;

#[path = "integration/coverage_unmeasured.rs"]
mod coverage_unmeasured;

#[path = "integration/staleness_unmeasurable.rs"]
mod staleness_unmeasurable;

#[path = "integration/finding_identity_parity.rs"]
mod finding_identity_parity;

#[path = "integration/regression_w1.rs"]
mod regression_w1;

/// Guard against silently-orphaned integration test files (issue #585).
///
/// `tests/integration/regression_w1.rs` sat on disk for months with 20 tests and
/// no `#[path]` entry here. It never compiled, so it never failed, so the suite
/// read as green. An unregistered file is indistinguishable from a passing one.
///
/// This asserts every `.rs` file in `tests/integration/` is declared in this
/// file. It lives inline in the harness root rather than in its own module so
/// that it cannot itself become the orphan it is meant to detect.
#[test]
fn every_integration_test_file_is_registered() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let harness_root = manifest_dir.join("tests/integration.rs");
    let module_dir = manifest_dir.join("tests/integration");

    let source = std::fs::read_to_string(&harness_root)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", harness_root.display()));

    // Collect the file names named by `#[path = "integration/<name>.rs"]`.
    let mut registered: Vec<String> = source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("#[path = \"integration/")?;
            let name = rest.strip_suffix(".rs\"]")?;
            Some(format!("{name}.rs"))
        })
        .collect();
    registered.sort();

    let mut on_disk: Vec<String> = std::fs::read_dir(&module_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", module_dir.display()))
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            name.ends_with(".rs").then_some(name)
        })
        .collect();
    on_disk.sort();

    let orphaned: Vec<&String> = on_disk.iter().filter(|f| !registered.contains(f)).collect();
    assert!(
        orphaned.is_empty(),
        "{} test file(s) in tests/integration/ are not registered in tests/integration.rs \
         and therefore never compile and never run: {orphaned:?}. \
         Add `#[path = \"integration/<name>.rs\"] mod <name>;` for each.",
        orphaned.len()
    );

    let dangling: Vec<&String> = registered.iter().filter(|f| !on_disk.contains(f)).collect();
    assert!(
        dangling.is_empty(),
        "tests/integration.rs registers file(s) that do not exist: {dangling:?}"
    );
}
