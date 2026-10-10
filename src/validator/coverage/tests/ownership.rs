use crate::types::SpecSyncConfig;
use crate::validator::compute_coverage_checked;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn gradle_coverage_names_the_root_project_not_the_package_tld() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("payments");
    fs::create_dir_all(root.join("src/main/kotlin/com/example/tool")).unwrap();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/tool/Profile.kt"),
        "package com.example.tool\nclass Profile\n",
    )
    .unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src/main/kotlin".to_string()],
        source_extensions: vec!["kt".to_string()],
        ..SpecSyncConfig::default()
    };

    let report = compute_coverage_checked(&root, &[], &config).unwrap();
    assert_eq!(
        report.unspecced_files.len(),
        1,
        "{:?}",
        report.unspecced_files
    );
    assert!(
        report
            .unspecced_modules
            .iter()
            .any(|name| name == "payments"),
        "expected directory identity, got {:?}",
        report.unspecced_modules
    );
    assert!(
        !report.unspecced_modules.iter().any(|name| name == "com"),
        "package TLD must not be a module: {:?}",
        report.unspecced_modules
    );
    assert!(
        !report.unspecced_modules.iter().any(|name| name == "src"),
        "shared src/ leaf must not be a module: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn conventional_src_child_is_still_a_module() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src/auth")).unwrap();
    fs::write(
        tmp.path().join("src/auth/session.py"),
        "def login():\n    return True\n",
    )
    .unwrap();
    let config = SpecSyncConfig {
        source_dirs: vec!["src".to_string()],
        source_extensions: vec!["py".to_string()],
        ..SpecSyncConfig::default()
    };

    let report = compute_coverage_checked(tmp.path(), &[], &config).unwrap();
    assert!(
        report.unspecced_modules.iter().any(|name| name == "auth"),
        "expected subdirectory module auth, got {:?}",
        report.unspecced_modules
    );
    assert!(!report.unspecced_modules.iter().any(|name| name == "com"));
}

// ─── #529: a module name nothing owns is not a missing spec ──────────────

/// Write a spec at `specs/<module>/<module>.spec.md` mapping `files`.
fn write_mapping_spec(root: &Path, module: &str, files: &[&str]) -> PathBuf {
    let directory = root.join("specs").join(module);
    fs::create_dir_all(&directory).unwrap();
    let mapped: String = files.iter().map(|file| format!("  - {file}\n")).collect();
    let spec_path = directory.join(format!("{module}.spec.md"));
    fs::write(
        &spec_path,
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: active\nfiles:\n{mapped}---\n\n# {module}\n"
        ),
    )
    .unwrap();
    spec_path
}

fn multi_language_config() -> SpecSyncConfig {
    SpecSyncConfig {
        specs_dir: "specs".to_string(),
        source_dirs: vec!["src".to_string()],
        source_extensions: vec!["py".to_string(), "mjs".to_string()],
        ..SpecSyncConfig::default()
    }
}

#[test]
fn language_specific_specs_do_not_invent_a_parent_module() {
    // #529: `src/strutil.{py,mjs}` are mapped by `strutil_py` and
    // `strutil_js`. No `specs/strutil/` exists — and none should, because
    // nothing is missing. Reporting `strutil` as a module without a spec
    // beside `2/2 files covered` is an answer read off the absence of a
    // NAME, not off any measurement of the files.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
    fs::write(
        root.join("src/strutil.mjs"),
        "export function upper(s) {\n}\n",
    )
    .unwrap();
    let specs = vec![
        write_mapping_spec(root, "strutil_py", &["src/strutil.py"]),
        write_mapping_spec(root, "strutil_js", &["src/strutil.mjs"]),
    ];

    let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
    assert_eq!(report.specced_file_count, 2);
    assert!(report.unspecced_files.is_empty());
    assert!(
        !report
            .unspecced_modules
            .iter()
            .any(|name| name == "strutil"),
        "stem of files that are all mapped must not be a module without a spec: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_stem_with_one_unmapped_file_is_still_a_module_without_a_spec() {
    // VACUITY CONTROL for the test above. Suppressing every stem would pass
    // that assertion; only a stem whose files were all measured and all
    // found mapped may go quiet. Here `src/strutil.mjs` is mapped by
    // nothing, so `strutil` is a real gap and must survive.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
    fs::write(
        root.join("src/strutil.mjs"),
        "export function upper(s) {\n}\n",
    )
    .unwrap();
    let specs = vec![write_mapping_spec(root, "strutil_py", &["src/strutil.py"])];

    let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
    assert_eq!(report.unspecced_files, ["src/strutil.mjs"]);
    assert!(
        report
            .unspecced_modules
            .iter()
            .any(|name| name == "strutil"),
        "a stem with an unmapped file must stay reported: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_directory_module_mapped_by_language_specific_specs_is_not_unspecced() {
    // The sibling derivation: the same phantom, one directory up. Nothing
    // in the reported issue named this site, and it fails identically.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src/textkit")).unwrap();
    fs::write(
        root.join("src/textkit/case.py"),
        "def upper(s):\n    return s\n",
    )
    .unwrap();
    fs::write(
        root.join("src/textkit/case.mjs"),
        "export function upper(s) {\n}\n",
    )
    .unwrap();
    let specs = vec![
        write_mapping_spec(root, "textkit_py", &["src/textkit/case.py"]),
        write_mapping_spec(root, "textkit_js", &["src/textkit/case.mjs"]),
    ];

    let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
    assert_eq!(report.specced_file_count, 2);
    assert!(
        !report
            .unspecced_modules
            .iter()
            .any(|name| name == "textkit"),
        "directory whose files are all mapped must not be a module without a spec: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_directory_module_with_an_unmapped_file_is_still_unspecced() {
    // VACUITY CONTROL for the directory site.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src/textkit")).unwrap();
    fs::write(
        root.join("src/textkit/case.py"),
        "def upper(s):\n    return s\n",
    )
    .unwrap();
    fs::write(
        root.join("src/textkit/case.mjs"),
        "export function upper(s) {\n}\n",
    )
    .unwrap();
    let specs = vec![write_mapping_spec(
        root,
        "textkit_py",
        &["src/textkit/case.py"],
    )];

    let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
    assert_eq!(report.unspecced_files, ["src/textkit/case.mjs"]);
    assert!(
        report
            .unspecced_modules
            .iter()
            .any(|name| name == "textkit"),
        "a directory with an unmapped file must stay reported: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_directory_with_no_discovered_source_file_is_still_unspecced() {
    // VACUITY CONTROL for the campaign's own defect class. A directory
    // holding nothing the traversal measures owns zero files, which is the
    // ABSENCE OF INPUT, not a clean bill of health — it keeps its report.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src/assets")).unwrap();
    fs::write(root.join("src/assets/logo.svg"), "<svg/>\n").unwrap();
    fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
    let specs = vec![write_mapping_spec(root, "strutil_py", &["src/strutil.py"])];

    let report = compute_coverage_checked(root, &specs, &multi_language_config()).unwrap();
    assert!(
        report.unspecced_modules.iter().any(|name| name == "assets"),
        "a directory with nothing measured must stay reported: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_manifest_module_whose_files_are_all_mapped_is_not_unspecced() {
    // Same phantom from the manifest site: a Cargo package named `toolkit`
    // whose every source file is mapped by `toolkit_core`.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"toolkit\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn upper() {}\n").unwrap();
    let config = SpecSyncConfig {
        specs_dir: "specs".to_string(),
        source_dirs: vec!["src".to_string()],
        source_extensions: vec!["rs".to_string()],
        ..SpecSyncConfig::default()
    };
    let specs = vec![write_mapping_spec(root, "toolkit_core", &["src/lib.rs"])];

    let report = compute_coverage_checked(root, &specs, &config).unwrap();
    assert_eq!(report.specced_file_count, 1);
    assert!(
        !report
            .unspecced_modules
            .iter()
            .any(|name| name == "toolkit"),
        "manifest module whose files are all mapped must not be reported: {:?}",
        report.unspecced_modules
    );

    // VACUITY CONTROL: add a file the spec does not map and the manifest
    // module comes back.
    fs::write(root.join("src/extra.rs"), "pub fn lower() {}\n").unwrap();
    let report = compute_coverage_checked(root, &specs, &config).unwrap();
    assert_eq!(report.unspecced_files, ["src/extra.rs"]);
    assert!(
        report
            .unspecced_modules
            .iter()
            .any(|name| name == "toolkit"),
        "manifest module with an unmapped file must stay reported: {:?}",
        report.unspecced_modules
    );
}

#[test]
fn a_configured_module_whose_files_are_all_mapped_is_not_unspecced() {
    // The configured-module site makes the same claim on the same evidence.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/strutil.py"), "def upper(s):\n    return s\n").unwrap();
    fs::write(
        root.join("src/strutil.mjs"),
        "export function upper(s) {\n}\n",
    )
    .unwrap();
    let mut config = multi_language_config();
    config.modules.insert(
        "strutil".to_string(),
        crate::types::ModuleDefinition {
            files: vec!["src/strutil.py".to_string(), "src/strutil.mjs".to_string()],
            depends_on: Vec::new(),
            owns: Vec::new(),
        },
    );
    let specs = vec![
        write_mapping_spec(root, "strutil_py", &["src/strutil.py"]),
        write_mapping_spec(root, "strutil_js", &["src/strutil.mjs"]),
    ];

    let report = compute_coverage_checked(root, &specs, &config).unwrap();
    assert!(
        !report
            .unspecced_modules
            .iter()
            .any(|name| name == "strutil"),
        "configured module whose files are all mapped must not be reported: {:?}",
        report.unspecced_modules
    );

    // VACUITY CONTROL: a configured module that declares no files at all
    // has nothing measured, so it stays reported.
    let mut declared_nothing = multi_language_config();
    declared_nothing.modules.insert(
        "ghost".to_string(),
        crate::types::ModuleDefinition::default(),
    );
    let report = compute_coverage_checked(root, &specs, &declared_nothing).unwrap();
    assert!(
        report.unspecced_modules.iter().any(|name| name == "ghost"),
        "configured module declaring no files must stay reported: {:?}",
        report.unspecced_modules
    );
}
