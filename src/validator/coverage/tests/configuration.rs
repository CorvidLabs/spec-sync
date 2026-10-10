use super::super::super::{compute_coverage, compute_coverage_checked};
use super::super::access::open_coverage_project_root;
use super::super::{CoverageTraversalBudget, CoverageTraversalLimits};
use super::*;
use crate::types::SpecSyncConfig;
use std::fs;
#[cfg(unix)]
use std::path::Path;

#[test]
fn malformed_gradle_settings_make_coverage_inconclusive() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        "include(\":member\"\n",
    )
    .unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        ..SpecSyncConfig::default()
    };

    let error = compute_coverage_checked(tmp.path(), &[], &config).unwrap_err();
    assert!(error.contains("Cannot parse Gradle settings manifest"));
    let report = compute_coverage(tmp.path(), &[], &config);
    // Nothing was discovered, so there is no percentage to report — not 0
    // and emphatically not 100.
    assert_eq!(report.file_coverage_percent(), None);
    assert!(report.unspecced_modules[0].contains("inconclusive"));
}

#[cfg(unix)]
#[test]
fn retained_config_uses_configured_source_dirs_after_root_replacement() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    let original = tmp.path().join("original-project");
    let replacement = tmp.path().join("replacement-project");
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(replacement.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"retained-specs\"\nsource_dirs = [\"retained-source\"]\n",
    )
    .unwrap();
    fs::write(
        replacement.join(".specsync/config.toml"),
        "specs_dir = \"replacement-specs\"\nsource_dirs = [\"replacement-source\"]\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(&root).unwrap();
    fs::rename(&root, &original).unwrap();
    symlink(&replacement, &root).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let config = retained_config(&retained, &root, &mut budget).unwrap();

    assert_eq!(config.specs_dir, "retained-specs");
    assert_eq!(config.source_dirs, ["retained-source"]);
}

#[test]
fn retained_explicit_source_dirs_skip_unrelated_manifest_autodetection() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join(".specsync")).unwrap();
    fs::write(
        project.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"configured-source\"]\n",
    )
    .unwrap();
    fs::write(
        project.path().join("settings.gradle"),
        "include(\":member\"\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let config = retained_config(&retained, project.path(), &mut budget).unwrap();

    assert_eq!(config.source_dirs, ["configured-source"]);
}

#[test]
fn retained_omitted_source_dirs_scan_after_a_malformed_manifest() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("lib")).unwrap();
    fs::write(project.path().join("lib/source.rs"), "pub fn value() {}\n").unwrap();
    fs::write(
        project.path().join("settings.gradle"),
        "include(\":member\"\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let config = retained_config(&retained, project.path(), &mut budget).unwrap();

    assert_eq!(config.source_dirs, ["lib"]);
    let checked_error = compute_coverage_checked(project.path(), &[], &config).unwrap_err();
    assert!(
        checked_error.contains("Cannot parse Gradle settings manifest"),
        "{checked_error}"
    );
}

/// THE REGRESSION (#723). A configured `source_dirs` is a statement about
/// where the sources are. Manifest discovery exists to INFER that when it
/// was not stated, so a failure to infer it cannot overrule the statement
/// and abort the command — yet `compute_coverage_checked` propagated the
/// error unconditionally, and `check`/`coverage` both exit on it. A Gradle
/// project whose settings this parser could not read could therefore not be
/// measured at all, on any 6.0 candidate, `source_dirs` set or not.
///
/// Both halves are asserted together because the fix is a PRECEDENCE, not a
/// softening. The same unreadable manifest must still be fatal when the
/// project did not state `source_dirs` — there the list coverage would
/// measure is itself discovery output, and degrading would report a
/// percentage over a guess. Asserting only the first half would pass
/// equally well against a change that just stopped failing.
///
/// The manifest here is malformed rather than merely unsupported, so this
/// test is independent of which Gradle forms the parser happens to accept:
/// it is about what an unreadable input is allowed to VETO.
#[test]
fn configured_source_dirs_survive_a_manifest_that_cannot_be_parsed() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("app/src/main/java/com/podo")).unwrap();
    fs::create_dir_all(project.path().join(".specsync")).unwrap();
    fs::write(
        project
            .path()
            .join("app/src/main/java/com/podo/Widget.java"),
        "package com.podo;\npublic class Widget {}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("settings.gradle"),
        "include(\":member\"\n",
    )
    .unwrap();
    fs::write(
        project.path().join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"app/src/main/java\"]\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

    let config = retained_config(&retained, project.path(), &mut budget).unwrap();
    assert!(
        config.source_dirs_set,
        "a stated source_dirs must be recorded as stated"
    );
    assert_eq!(config.source_dirs, ["app/src/main/java"]);

    let report = compute_coverage_checked(project.path(), &[], &config)
        .unwrap_or_else(|error| panic!("configured source_dirs were vetoed by: {error}"));

    // Coverage ran over the declared list and reports real numbers, rather
    // than the `total_source_files: 0` of an inconclusive report.
    assert_eq!(report.total_source_files, 1);
    assert_eq!(report.specced_file_count, 0);
    assert_eq!(
        report.unspecced_files,
        ["app/src/main/java/com/podo/Widget.java"]
    );

    // Degraded, not silent. The manifest also declares modules, so this run
    // names fewer modules without specs than the tree could hold — a report
    // improved by a measurement that stopped, which travels with the
    // figures instead of to stderr (#570).
    assert_eq!(
        report.manifest_notices.len(),
        1,
        "expected one degradation notice, got {:?}",
        report.manifest_notices
    );
    assert!(
        report.manifest_notices[0].contains("Cannot parse Gradle settings manifest"),
        "the notice must name what could not be read: {:?}",
        report.manifest_notices
    );

    // Honest label: this half is the CONTROL, and it is what fails if the
    // precedence were implemented as "manifest errors are never fatal".
    // Same tree, same unreadable manifest, `source_dirs` no longer stated.
    let inferred = SpecSyncConfig {
        source_dirs_set: false,
        ..config
    };
    let error = compute_coverage_checked(project.path(), &[], &inferred).unwrap_err();
    assert!(
        error.contains("Cannot parse Gradle settings manifest"),
        "an unstated source list must still fail closed: {error}"
    );
}

#[cfg(unix)]
#[test]
fn retained_config_rejects_a_detached_parent_before_read() {
    let project = tempfile::tempdir().unwrap();
    let original_parent = project.path().join(".specsync-original");
    fs::create_dir_all(project.path().join(".specsync")).unwrap();
    fs::write(
        project.path().join(".specsync/config.toml"),
        "source_dirs = [\"original-source\"]\n",
    )
    .unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    let mut replaced = false;

    let error = retained_config_with_hook(&retained, project.path(), &mut budget, |relative| {
        if replaced || relative != Path::new(".specsync/config.toml") {
            return;
        }
        replaced = true;
        fs::rename(project.path().join(".specsync"), &original_parent).unwrap();
        fs::create_dir_all(project.path().join(".specsync")).unwrap();
        fs::write(
            project.path().join(".specsync/config.toml"),
            "source_dirs = [\"replacement-source\"]\n",
        )
        .unwrap();
    })
    .unwrap_err();

    assert!(
        error.contains("changed during retained traversal"),
        "{error}"
    );
}
