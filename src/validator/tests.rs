
#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_coverage_double_star_exclude_no_panic() {
        // Regression: a `**/**` exclude pattern used to panic in the glob matcher
        // (`&pattern[3..len-3]` reverses to `[3..2]` for the len-5 string). It must
        // instead match every path (empty middle) and exclude all source files.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.ts"), "export function a() {}").unwrap();

        let mut config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_patterns: vec!["**/**".to_string()],
            ..SpecSyncConfig::default()
        };

        // Must not panic; `**/**` excludes all source files.
        let report = compute_coverage(tmp.path(), &[], &config);
        assert_eq!(
            report.total_source_files, 0,
            "`**/**` should exclude every source file"
        );

        // A normal `**/dir/**` pattern still excludes only that directory.
        fs::create_dir_all(src.join("gen")).unwrap();
        fs::write(src.join("gen/z.ts"), "export function z() {}").unwrap();
        config.exclude_patterns = vec!["**/gen/**".to_string()];
        let report = compute_coverage(tmp.path(), &[], &config);
        assert_eq!(
            report.total_source_files, 1,
            "only src/gen should be excluded, leaving src/a.ts"
        );
    }

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
    fn retained_coverage_snapshot_rejects_post_discovery_symlink_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("member/src/main/kotlin")).unwrap();
        fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
        let outside_source = outside.join("src/main/kotlin/Secret.kt");
        let sentinel = b"const val SECRET = \"RETAINED_COVERAGE_SENTINEL\"\n";
        fs::write(
            root.join("member/src/main/kotlin/Local.kt"),
            "const val LOCAL = 1\n",
        )
        .unwrap();
        fs::write(&outside_source, sentinel).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["member/src/main/kotlin".to_string()],
            ..SpecSyncConfig::default()
        };

        let project = open_coverage_project_root(&root).unwrap();
        fs::rename(root.join("member"), root.join("original-member")).unwrap();
        symlink(&outside, root.join("member")).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let error =
            snapshot_coverage_sources(&project, &root, &config, &HashSet::new(), &mut budget)
                .unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains("RETAINED_COVERAGE_SENTINEL"));
        assert_eq!(fs::read(outside_source).unwrap(), sentinel);
    }

    #[test]
    fn retained_coverage_sources_reject_regular_directory_replacement_after_selection() {
        let project = tempfile::tempdir().unwrap();
        let original = project.path().join("original-src");
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(
            project.path().join("src/original.rs"),
            "pub fn original() {}\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let selected_sources = select_coverage_source_directories(
            &retained,
            &config,
            CoverageTraversalLimits::default().max_depth,
        )
        .unwrap();
        fs::rename(project.path().join("src"), &original).unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(
            project.path().join("src/replacement.rs"),
            "pub fn replacement() {}\n",
        )
        .unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let error = snapshot_selected_coverage_sources_with_hook(
            &retained,
            project.path(),
            &config,
            &HashSet::new(),
            &mut budget,
            selected_sources,
            &mut |_| {},
        )
        .unwrap_err();

        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_source_traversal_rejects_directory_replacement_after_enumeration() {
        let project = tempfile::tempdir().unwrap();
        let source_parent = project.path().join("src/module");
        let original = source_parent.join("nested");
        let detached = project.path().join("detached-source");
        let replacement = project.path().join("replacement-source");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::write(original.join("original.rs"), "pub fn original() {}\n").unwrap();
        fs::write(
            replacement.join("replacement.rs"),
            "pub fn replacement() {}\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error = snapshot_coverage_sources_with_hook(
            &retained,
            project.path(),
            &config,
            &HashSet::new(),
            &mut budget,
            |relative| {
                if replaced || relative != Path::new("src/module") {
                    return;
                }
                replaced = true;
                fs::rename(&original, &detached).unwrap();
                fs::rename(&replacement, &original).unwrap();
            },
        )
        .unwrap_err();

        assert!(replaced);
        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_spec_traversal_rejects_directory_replacement_after_enumeration() {
        let project = tempfile::tempdir().unwrap();
        let spec_parent = project.path().join("specs/module");
        let original = spec_parent.join("nested");
        let detached = project.path().join("detached-specs");
        let replacement = project.path().join("replacement-specs");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::write(
            original.join("original.spec.md"),
            "---\nmodule: original\nversion: 1\nstatus: stable\nfiles: []\n---\n",
        )
        .unwrap();
        fs::write(
            replacement.join("replacement.spec.md"),
            "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles: []\n---\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error =
            discover_coverage_spec_files_with_hook(&retained, "specs", &mut budget, |relative| {
                if replaced || relative != Path::new("specs/module") {
                    return;
                }
                replaced = true;
                fs::rename(&original, &detached).unwrap();
                fs::rename(&replacement, &original).unwrap();
            })
            .unwrap_err();

        assert!(replaced);
        assert!(
            error.contains("changed during retained traversal"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_coverage_spec_mapping_ignores_an_ambient_root_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        let relative_spec = Path::new("specs/auth/auth.spec.md");
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::create_dir_all(replacement.join("specs/auth")).unwrap();
        fs::write(
            root.join(relative_spec),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/retained.rs\n---\n",
        )
        .unwrap();
        fs::write(
            replacement.join(relative_spec),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
        )
        .unwrap();
        let project = open_coverage_project_root(&root).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&project, "specs", &mut inventory_budget).unwrap();
        fs::rename(&root, &original).unwrap();
        symlink(&replacement, &root).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());

        let mappings = collect_specced_files(&project, &inventory, &mut budget).unwrap();

        assert!(mappings.contains("src/retained.rs"));
        assert!(!mappings.contains("src/replacement.rs"));
    }

    #[test]
    fn retained_coverage_spec_mapping_rejects_a_path_outside_the_project() {
        let project = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();

        let error = select_coverage_spec_files(
            project.path(),
            &[outside.path().join("outside.spec.md")],
            &[],
        )
        .unwrap_err();

        assert!(
            error.contains("must remain beneath the project root"),
            "{error}"
        );
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

    #[test]
    fn retained_selected_specs_enforce_the_shared_entry_budget() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        let first = project.path().join("specs/first.spec.md");
        let second = project.path().join("specs/second.spec.md");
        let spec = "---\nmodule: fixture\nversion: 1\nstatus: stable\nfiles: []\n---\n";
        fs::write(&first, spec).unwrap();
        fs::write(&second, spec).unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
        let limits = CoverageTraversalLimits {
            max_entries: 1,
            ..CoverageTraversalLimits::default()
        };
        let mut within_budget = CoverageTraversalBudget::new(limits);
        collect_specced_files(
            &retained,
            std::slice::from_ref(&inventory[0]),
            &mut within_budget,
        )
        .unwrap();

        let mut over_budget = CoverageTraversalBudget::new(limits);
        let error = collect_specced_files(&retained, &inventory, &mut over_budget).unwrap_err();

        assert!(error.contains("1-entry limit"), "{error}");
    }

    #[test]
    fn retained_spec_enumeration_is_bounded_before_returning_paths() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        fs::write(project.path().join("specs/first.spec.md"), "").unwrap();
        fs::write(project.path().join("specs/second.spec.md"), "").unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits {
            max_entries: 1,
            ..CoverageTraversalLimits::default()
        });

        let error = discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap_err();

        assert!(error.contains("1-entry limit"), "{error}");
    }

    #[test]
    fn retained_spec_inventory_applies_caller_selection_before_ownership_parsing() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        let first = project.path().join("specs/first.spec.md");
        let second = project.path().join("specs/second.spec.md");
        fs::write(
            &first,
            "---\nmodule: first\nversion: 1\nstatus: stable\nfiles:\n  - src/first.rs\n---\n",
        )
        .unwrap();
        fs::write(
            &second,
            "---\nmodule: second\nversion: 1\nstatus: stable\nfiles:\n  - src/second.rs\n---\n",
        )
        .unwrap();
        fs::write(project.path().join("src/first.rs"), "pub fn first() {}\n").unwrap();
        fs::write(project.path().join("src/second.rs"), "pub fn second() {}\n").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let report =
            compute_coverage_checked(project.path(), std::slice::from_ref(&first), &config)
                .unwrap();

        assert_eq!(report.total_source_files, 2);
        assert_eq!(report.specced_file_count, 1);
        assert_eq!(report.unspecced_files, ["src/second.rs"]);
    }

    #[test]
    fn retained_spec_inventory_rejects_replacement_before_ownership_parse() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("specs")).unwrap();
        let selected = project.path().join("specs/selected.spec.md");
        let replacement = project.path().join("specs/replacement.spec.md");
        fs::write(
            &selected,
            "---\nmodule: selected\nversion: 1\nstatus: stable\nfiles:\n  - src/original.rs\n---\n",
        )
        .unwrap();
        fs::write(
            &replacement,
            "---\nmodule: replacement\nversion: 1\nstatus: stable\nfiles:\n  - src/replacement.rs\n---\n",
        )
        .unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut inventory_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let inventory =
            discover_coverage_spec_files(&retained, "specs", &mut inventory_budget).unwrap();
        let selected_inventory: Vec<CoverageSpecFile> = inventory
            .into_iter()
            .filter(|spec| spec.relative_path == Path::new("specs/selected.spec.md"))
            .collect();
        let mut read_budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let mut replaced = false;

        let error = collect_specced_files_with_hook(
            &retained,
            &selected_inventory,
            &mut read_budget,
            |_| {
                if replaced {
                    return;
                }
                replaced = true;
                fs::remove_file(&selected).unwrap();
                fs::rename(&replacement, &selected).unwrap();
            },
        )
        .unwrap_err();

        assert!(
            error.contains("changed after retained inventory"),
            "{error}"
        );
    }

    #[test]
    fn retained_coverage_file_read_rejects_preopen_regular_replacement() {
        let project = tempfile::tempdir().unwrap();
        let source = project.path().join("source.rs");
        let replacement = project.path().join("replacement.rs");
        fs::write(&source, "pub fn original() {}\n").unwrap();
        fs::write(&replacement, "pub fn replacement() {}\n").unwrap();
        let retained = open_coverage_project_root(project.path()).unwrap();

        let error = read_coverage_file_with_hook(
            &retained,
            std::ffi::OsStr::new("source.rs"),
            Path::new("source.rs"),
            MAX_COVERAGE_FILE_BYTES,
            MAX_COVERAGE_INPUT_BYTES,
            MAX_COVERAGE_INPUT_BYTES,
            || {
                fs::remove_file(&source).unwrap();
                fs::rename(&replacement, &source).unwrap();
            },
        )
        .unwrap_err();

        assert!(error.contains("changed during retained open"), "{error}");
    }

    fn snapshot_with_limits(
        root: &Path,
        config: &SpecSyncConfig,
        exclude_dirs: &HashSet<String>,
        limits: CoverageTraversalLimits,
    ) -> Result<CoverageSourceSnapshot, String> {
        let project = open_coverage_project_root(root).unwrap();
        let mut budget = CoverageTraversalBudget::new(limits);
        snapshot_coverage_sources(&project, root, config, exclude_dirs, &mut budget)
    }

    #[cfg(unix)]
    #[test]
    fn broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit() {
        const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_source_tree_succeeds_with_a_bounded_file_descriptor_limit --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "broad traversal failed with a 64-descriptor soft limit"
            );
            return;
        }

        let project = tempfile::tempdir().unwrap();
        let source = project.path().join("src");
        let specs = project.path().join("specs");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&specs).unwrap();
        for index in 0..200 {
            let module = source.join(format!("module-{index:03}"));
            fs::create_dir(&module).unwrap();
            fs::write(module.join("lib.rs"), "pub fn visible() {}\n").unwrap();
            let spec_module = specs.join(format!("module-{index:03}"));
            fs::create_dir(&spec_module).unwrap();
            fs::write(
                spec_module.join("module.spec.md"),
                "---\nmodule: broad\nversion: 1\nstatus: stable\nfiles: []\n---\n",
            )
            .unwrap();
        }
        let retained = open_coverage_project_root(project.path()).unwrap();
        let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
        let discovered_specs =
            discover_coverage_spec_files(&retained, "specs", &mut budget).unwrap();
        assert_eq!(discovered_specs.len(), 200);
        drop(retained);

        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let snapshot = snapshot_with_limits(
            project.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits::default(),
        )
        .unwrap();

        assert_eq!(snapshot.files.len(), 200);
    }

    #[cfg(unix)]
    #[test]
    fn broad_configured_source_roots_bound_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_VALIDATOR_BOUNDED_ROOT_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" broad_configured_source_roots_bound_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "coverage retained handles across distinct configured roots"
            );
            return;
        }

        let project = tempfile::tempdir().unwrap();
        let mut source_dirs = Vec::new();
        for index in 0..90 {
            let source_dir = format!("source-{index:03}");
            fs::create_dir(project.path().join(&source_dir)).unwrap();
            fs::write(
                project.path().join(&source_dir).join("lib.rs"),
                format!("pub fn visible_{index:03}() {{}}\n"),
            )
            .unwrap();
            source_dirs.push(source_dir);
        }
        let config = SpecSyncConfig {
            source_dirs,
            ..SpecSyncConfig::default()
        };

        let snapshot = snapshot_with_limits(
            project.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits::default(),
        )
        .expect("distinct configured source roots must stay descriptor-bounded");

        assert_eq!(snapshot.files.len(), 90);
    }

    #[cfg(unix)]
    #[test]
    fn excluded_special_names_are_skipped_before_metadata() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
        symlink(&outside, root.join("src/excluded-link")).unwrap();
        let fifo = root.join("src/excluded-fifo");
        let fifo_status = Command::new("mkfifo").arg(&fifo).status().unwrap();
        assert!(fifo_status.success(), "mkfifo fixture failed");
        let _socket = match UnixListener::bind(root.join("src/excluded-socket")) {
            Ok(socket) => Some(socket),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
            Err(error) => panic!("cannot create excluded socket fixture: {error}"),
        };
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_dirs: vec![
                "excluded-link".to_string(),
                "excluded-fifo".to_string(),
                "excluded-socket".to_string(),
            ],
            ..SpecSyncConfig::default()
        };
        let excludes = config.exclude_dirs.iter().cloned().collect();

        let report = compute_coverage_checked(&root, &[], &config).unwrap();

        assert_eq!(report.total_source_files, 1);
        assert_eq!(report.unspecced_files, ["src/kept.rs"]);
        let snapshot = snapshot_with_limits(
            &root,
            &config,
            &excludes,
            CoverageTraversalLimits::default(),
        )
        .unwrap();
        assert_eq!(snapshot.files.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn directly_configured_special_source_roots_are_inconclusive() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("linked")).unwrap();
        let fifo_status = Command::new("mkfifo")
            .arg(root.join("fifo"))
            .status()
            .unwrap();
        assert!(fifo_status.success(), "mkfifo fixture failed");
        let socket = match UnixListener::bind(root.join("socket")) {
            Ok(socket) => Some(socket),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => None,
            Err(error) => panic!("cannot create configured socket fixture: {error}"),
        };
        let mut cases = vec![
            ("linked", "symlink or reparse point"),
            ("fifo", "regular directory"),
        ];
        if socket.is_some() {
            cases.push(("socket", "regular directory"));
        }

        for (configured, expected) in cases {
            let config = SpecSyncConfig {
                source_dirs: vec![configured.to_string()],
                exclude_dirs: vec![configured.to_string()],
                ..SpecSyncConfig::default()
            };
            let error = compute_coverage_checked(&root, &[], &config).unwrap_err();
            assert!(error.contains(expected), "{configured}: {error}");
        }
    }

    #[cfg(windows)]
    fn create_coverage_test_junction(junction: &Path, target: &Path) -> Result<(), String> {
        let unicode_path = |path: &Path| -> Result<String, String> {
            path.to_str()
                .map(str::to_string)
                .ok_or_else(|| "junction fixture paths must be valid Unicode".to_string())
        };
        let junction = unicode_path(junction)?;
        let target = unicode_path(target)?;
        let script = "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction \
                      -Path $env:SPECSYNC_TEST_JUNCTION \
                      -Target $env:SPECSYNC_TEST_TARGET | Out-Null";
        let mut unavailable = Vec::new();
        for executable in ["powershell.exe", "pwsh.exe"] {
            let output = match std::process::Command::new(executable)
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    script,
                ])
                .env("SPECSYNC_TEST_JUNCTION", &junction)
                .env("SPECSYNC_TEST_TARGET", &target)
                .output()
            {
                Ok(output) => output,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    unavailable.push(executable);
                    continue;
                }
                Err(error) => {
                    return Err(format!(
                        "failed to launch {executable} junction fixture: {error}"
                    ));
                }
            };
            if output.status.success() {
                return Ok(());
            }
            return Err(format!(
                "{executable} junction fixture exited with {:?}; stdout: {}; stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Err(format!(
            "failed to launch a PowerShell junction fixture; unavailable executables: {}",
            unavailable.join(", ")
        ))
    }

    #[cfg(windows)]
    #[test]
    fn excluded_junction_is_skipped_but_configured_junction_is_inconclusive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/kept.rs"), "pub fn kept() {}\n").unwrap();
        let target_proof = b"outside junction target\n";
        fs::write(outside.join("junction-target-proof.txt"), target_proof).unwrap();
        let excluded_junction = root.join("src/excluded-junction");
        let configured_junction = root.join("configured-junction");
        create_coverage_test_junction(&excluded_junction, &outside)
            .unwrap_or_else(|error| panic!("failed to create excluded junction: {error}"));
        create_coverage_test_junction(&configured_junction, &outside)
            .unwrap_or_else(|error| panic!("failed to create configured junction: {error}"));
        for junction in [&excluded_junction, &configured_junction] {
            assert_eq!(
                fs::read(junction.join("junction-target-proof.txt")).unwrap(),
                target_proof,
                "{} does not resolve to the intended outside target",
                junction.display()
            );
        }

        let excluded = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            exclude_dirs: vec!["excluded-junction".to_string()],
            ..SpecSyncConfig::default()
        };
        let report = compute_coverage_checked(&root, &[], &excluded).unwrap();
        assert_eq!(report.total_source_files, 1);

        let configured = SpecSyncConfig {
            source_dirs: vec!["configured-junction".to_string()],
            exclude_dirs: vec!["configured-junction".to_string()],
            ..SpecSyncConfig::default()
        };
        let error = compute_coverage_checked(&root, &[], &configured).unwrap_err();
        assert!(error.contains("symlink or reparse point"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn unix_backslash_and_directory_separator_paths_do_not_collide() {
        assert_eq!(
            normalize_source_mapping("src/collision\\name.rs"),
            Some("src/collision\\name.rs".to_string())
        );
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/collision")).unwrap();
        fs::write(
            root.join("src").join("collision\\name.rs"),
            "pub fn literal_backslash() {}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/collision/name.rs"),
            "pub fn directory_separator() {}\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let report = compute_coverage_checked(root, &[], &config).unwrap();

        assert_eq!(report.total_source_files, 2);
        assert!(
            report
                .unspecced_files
                .contains(&"src/collision\\name.rs".to_string())
        );
        assert!(
            report
                .unspecced_files
                .contains(&"src/collision/name.rs".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_source_name_is_inconclusive_instead_of_colliding() {
        use std::os::unix::ffi::OsStringExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        let invalid = OsString::from_vec(b"collision-\xff.rs".to_vec());
        let invalid_path = PathBuf::from("src").join(&invalid);
        assert!(coverage_relative_path_text(&invalid_path).is_err());
        if fs::write(root.join("src").join(invalid), "pub fn hidden() {}\n").is_err() {
            return;
        }
        fs::write(
            root.join("src/collision-\u{fffd}.rs"),
            "pub fn visible() {}\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let error = compute_coverage_checked(root, &[], &config).unwrap_err();

        assert!(error.contains("not valid UTF-8"), "{error}");
    }

    #[test]
    fn invalid_utf8_supported_source_content_is_inconclusive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/bad.rs"), b"pub fn visible() {}\n\xff").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };

        let error = compute_coverage_checked(root, &[], &config).unwrap_err();

        assert!(error.replace('\\', "/").contains("src/bad.rs"), "{error}");
        assert!(error.contains("not valid UTF-8"), "{error}");
    }

    #[test]
    fn coverage_traversal_enforces_per_file_and_cumulative_byte_limits() {
        assert_eq!(MAX_COVERAGE_FILE_BYTES, 8 * 1024 * 1024);
        assert_eq!(MAX_COVERAGE_INPUT_BYTES, 64 * 1024 * 1024);
        let per_file = tempfile::tempdir().unwrap();
        fs::create_dir_all(per_file.path().join("src")).unwrap();
        fs::write(per_file.path().join("src/large.rs"), b"123456789").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let per_file_error = snapshot_with_limits(
            per_file.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 64,
                max_entries: 100,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(per_file_error.contains("8 bytes per-file limit"));

        let cumulative = tempfile::tempdir().unwrap();
        fs::create_dir_all(cumulative.path().join("src")).unwrap();
        fs::write(cumulative.path().join("src/a.rs"), b"123456").unwrap();
        fs::write(cumulative.path().join("src/b.rs"), b"123456").unwrap();
        let cumulative_error = snapshot_with_limits(
            cumulative.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 10,
                max_entries: 100,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(cumulative_error.contains("10 bytes cumulative limit"));
    }

    #[test]
    fn coverage_traversal_enforces_entry_and_depth_limits_iteratively() {
        assert_eq!(MAX_COVERAGE_ENTRIES, 100_000);
        assert_eq!(MAX_COVERAGE_DEPTH, 256);
        let entries = tempfile::tempdir().unwrap();
        fs::create_dir_all(entries.path().join("src")).unwrap();
        fs::write(entries.path().join("src/a.rs"), "").unwrap();
        fs::write(entries.path().join("src/b.rs"), "").unwrap();
        let config = SpecSyncConfig {
            source_dirs: vec!["src".to_string()],
            ..SpecSyncConfig::default()
        };
        let entry_error = snapshot_with_limits(
            entries.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 16,
                max_entries: 1,
                max_depth: 16,
            },
        )
        .unwrap_err();
        assert!(entry_error.contains("1-entry limit"));

        let depth = tempfile::tempdir().unwrap();
        fs::create_dir_all(depth.path().join("src/a/b/c")).unwrap();
        fs::write(depth.path().join("src/a/b/c/deep.rs"), "").unwrap();
        let depth_error = snapshot_with_limits(
            depth.path(),
            &config,
            &HashSet::new(),
            CoverageTraversalLimits {
                max_file_bytes: 8,
                max_input_bytes: 16,
                max_entries: 100,
                max_depth: 3,
            },
        )
        .unwrap_err();
        assert!(depth_error.contains("3-component depth limit"));
    }

    #[test]
    fn configured_static_html_has_non_vacuous_coverage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("landing")).unwrap();
        fs::create_dir_all(root.join("specs/landing")).unwrap();
        fs::write(root.join("landing/index.html"), "<main>Welcome</main>\n").unwrap();
        fs::write(root.join("landing/logo.png"), [0_u8, 1, 2]).unwrap();
        let spec_path = root.join("specs/landing/landing.spec.md");
        fs::write(
            &spec_path,
            "---\nmodule: landing\nversion: 1\nstatus: stable\nfiles:\n  - landing/index.html\n---\n\n# Landing\n",
        )
        .unwrap();
        let config = SpecSyncConfig {
            specs_dir: "specs".into(),
            source_dirs: vec!["landing".into()],
            ..SpecSyncConfig::default()
        };

        let mapped = compute_coverage(root, std::slice::from_ref(&spec_path), &config);
        assert_eq!(mapped.total_source_files, 1);
        assert_eq!(mapped.specced_file_count, 1);
        assert_eq!(mapped.file_coverage_percent(), Some(100));
        assert!(mapped.unspecced_files.is_empty());

        let unmapped = compute_coverage(root, &[], &config);
        assert_eq!(unmapped.total_source_files, 1);
        assert_eq!(unmapped.specced_file_count, 0);
        assert_eq!(unmapped.file_coverage_percent(), Some(0));
        assert_eq!(unmapped.unspecced_files, ["landing/index.html"]);
    }

    #[test]
    fn companion_scaffold_markers_are_precise_and_ignore_fenced_examples() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let directory = root.join("specs/auth");
        fs::create_dir_all(&directory).unwrap();
        let spec_path = directory.join("auth.spec.md");
        fs::write(&spec_path, "---\nmodule: auth\n---\n").unwrap();
        fs::write(
            directory.join("context.md"),
            "# Context\n\n<!-- Describe the context and motivation for this module. -->\n",
        )
        .unwrap();
        fs::write(
            directory.join("requirements.md"),
            "# Requirements\n\n```markdown\n- <!-- List measurable acceptance criteria. -->\n```\n\nThis prose discusses future acceptance criteria without using the generated marker.\n",
        )
        .unwrap();
        fs::write(
            directory.join("testing.md"),
            "# Testing\n\nList the automated tests and fixtures that protect this module.\n",
        )
        .unwrap();
        fs::write(
            directory.join("design.md"),
            "# Design\n\n## Layout\n\n- Document layout structure, responsive breakpoints, and positioning rules.\n\n## Components\n\n- Document component tree, inputs, outputs, and slots.\n\n## Tokens\n\n- Document color, spacing, typography, and state token overrides.\n\n## Assets\n\n- List icons, images, illustrations, and asset ownership.\n",
        )
        .unwrap();
        let mut result = ValidationResult::new("specs/auth/auth.spec.md".into());

        validate_companion_scaffold_markers(&spec_path, root, &mut result);

        assert_eq!(result.warnings.len(), 6, "{:?}", result.warnings);
        assert!(result.warnings.iter().any(|warning| {
            warning.contains("specs/auth/context.md:3") && warning.contains("concrete motivation")
        }));
        assert!(result.warnings.iter().any(|warning| {
            warning.contains("specs/auth/testing.md:3") && warning.contains("automated tests")
        }));
        for line in [5, 9, 13, 17] {
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains(&format!("specs/auth/design.md:{line}"))),
                "missing design marker warning at line {line}: {:?}",
                result.warnings
            );
        }
        assert!(
            !result
                .warnings
                .iter()
                .any(|warning| warning.contains("requirements.md"))
        );
    }

    #[test]
    fn content_validation_skips_companions_while_path_validation_preserves_them() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let directory = root.join("specs/auth");
        fs::create_dir_all(&directory).unwrap();
        let spec_path = directory.join("auth.spec.md");
        let content =
            "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n";
        fs::write(&spec_path, content).unwrap();
        fs::write(
            directory.join("context.md"),
            "<!-- Describe the context and motivation for this module. -->\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let schema_tables = HashSet::new();
        let schema_columns = HashMap::new();
        let config = SpecSyncConfig::default();

        let path_result = validate_spec(&spec_path, root, &schema_tables, &schema_columns, &config);
        let content_result = validate_spec_content(
            &spec_path,
            content,
            root,
            &schema_tables,
            &schema_columns,
            &config,
        );

        assert!(
            path_result
                .warnings
                .iter()
                .any(|warning| warning.contains("Unfilled companion scaffold marker"))
        );
        assert!(
            content_result
                .warnings
                .iter()
                .all(|warning| !warning.contains("Unfilled companion scaffold marker"))
        );
    }

    #[test]
    fn supplied_content_validation_normalizes_crlf_identically() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        let path = root.join("specs/auth/auth.spec.md");
        let lf = "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n";
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
                fix.contains("src/provider/provider.ts")
                    && fix.contains("src/provider/nested/deep.ts")
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

    #[test]
    fn test_validate_spec_schema_columns() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("msg.ts"), "export function send() {}").unwrap();

        let spec = tmp.path().join("msg.spec.md");
        fs::write(
            &spec,
            r#"---
module: msg
version: 1
status: active
files:
  - src/msg.ts
db_tables:
  - messages
---

## Purpose
Messaging

## Requirements

### Schema: messages

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `content` | TEXT | NOT NULL |
| `ghost_col` | TEXT | NOT NULL |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `send` | msg: string | void | Sends |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
        )
        .unwrap();

        let mut table_names = HashSet::new();
        table_names.insert("messages".to_string());

        let mut schema_cols = HashMap::new();
        schema_cols.insert(
            "messages".to_string(),
            SchemaTable {
                columns: vec![
                    crate::schema::SchemaColumn {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: true,
                    },
                    crate::schema::SchemaColumn {
                        name: "content".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: false,
                    },
                    crate::schema::SchemaColumn {
                        name: "created_at".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        has_default: true,
                        is_primary_key: false,
                    },
                ],
            },
        );

        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

        // ghost_col is in spec but not in schema → ERROR
        assert!(result.errors.iter().any(|e| e.contains("ghost_col")));
        // created_at is in schema but not in spec → WARNING
        assert!(result.warnings.iter().any(|w| w.contains("created_at")));
    }

    #[test]
    fn test_validate_spec_schema_type_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("t.ts"), "export function f() {}").unwrap();

        let spec = tmp.path().join("t.spec.md");
        fs::write(
            &spec,
            r#"---
module: t
version: 1
status: active
files:
  - src/t.ts
db_tables:
  - items
---

## Purpose
Test

## Requirements

### Schema: items

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `price` | TEXT | |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `f` | | void | Does stuff |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
        )
        .unwrap();

        let mut table_names = HashSet::new();
        table_names.insert("items".to_string());

        let mut schema_cols = HashMap::new();
        schema_cols.insert(
            "items".to_string(),
            SchemaTable {
                columns: vec![
                    crate::schema::SchemaColumn {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        has_default: false,
                        is_primary_key: true,
                    },
                    crate::schema::SchemaColumn {
                        name: "price".to_string(),
                        col_type: "REAL".to_string(),
                        nullable: true,
                        has_default: false,
                        is_primary_key: false,
                    },
                ],
            },
        );

        let config = SpecSyncConfig::default();
        let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

        // price type mismatch: spec says TEXT, schema says REAL → WARNING
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("type mismatch") && w.contains("price"))
        );
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

    /// #684: a project whose schema lives in application code cannot configure `schema_dir` —
    /// there are no `.sql` migrations to point it at, and `schema_dir` is not a spec frontmatter
    /// field. Carrying "validation skipped" as a WARNING escalated it to an error under
    /// `--strict`, so the only ways to clear it were deleting the `db_tables` documentation or
    /// abandoning `strict`. Gating on advice the reader cannot take is not drift detection.
    ///
    /// DISCRIMINATOR. Before the fix this message was pushed to `warnings`.
    #[test]
    fn db_tables_without_schema_dir_is_a_notice_not_a_warning() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("specs/tracker")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/tracker.rs"), "pub fn track() {}\n").unwrap();
        let spec_path = root.join("specs/tracker/tracker.spec.md");
        let content = "---\nmodule: tracker\nversion: 1\nstatus: draft\nfiles:\n  - src/tracker.rs\ndb_tables:\n  - events\n---\n";
        fs::write(&spec_path, content).unwrap();

        let config = SpecSyncConfig::default();
        assert!(config.schema_dir.is_none(), "premise: no schema_dir");
        let result = validate_spec_content(
            &spec_path,
            content,
            root,
            &HashSet::new(),
            &HashMap::new(),
            &config,
        );

        let skipped = |v: &Vec<String>| v.iter().any(|m| m.contains("DB table validation skipped"));
        assert!(
            skipped(&result.notices),
            "the disclosure must survive as a notice: {:?}",
            result.notices
        );
        assert!(
            !skipped(&result.warnings),
            "it must not be a warning — warnings escalate under --strict: {:?}",
            result.warnings
        );
    }

    /// VACUITY CONTROL. Behaviour only: when `schema_dir` IS configured, a declared table the
    /// schema does not contain must still be an ERROR. A fix that simply stopped checking
    /// `db_tables` would pass the discriminator above and fail this.
    #[test]
    fn a_missing_db_table_is_still_an_error_when_schema_dir_is_configured() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("specs/tracker")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/tracker.rs"), "pub fn track() {}\n").unwrap();
        let spec_path = root.join("specs/tracker/tracker.spec.md");
        let content = "---\nmodule: tracker\nversion: 1\nstatus: draft\nfiles:\n  - src/tracker.rs\ndb_tables:\n  - events\n---\n";
        fs::write(&spec_path, content).unwrap();

        // The schema must be READABLE for "missing" to be a legitimate verdict — an unreadable
        // schema correctly degrades to "unknown" (#672), so a fixture without a real schema
        // directory would prove nothing here.
        fs::create_dir_all(root.join("schema")).unwrap();
        fs::write(root.join("schema/001.sql"), "CREATE TABLE other (id TEXT);").unwrap();
        let mut config = SpecSyncConfig::default();
        config.schema_dir = Some("schema".to_string());
        let mut tables = HashSet::new();
        tables.insert("other".to_string());

        let result =
            validate_spec_content(&spec_path, content, root, &tables, &HashMap::new(), &config);
        assert!(
            result
                .errors
                .iter()
                .any(|m| m.contains("DB table not found in schema: events")),
            "a genuinely absent table must remain an error: {:?}",
            result.errors
        );
    }
}

// ─── Coverage ────────────────────────────────────────────────────────────

fn collect_specced_files(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
) -> Result<HashSet<String>, String> {
    collect_specced_files_with_hook(project, spec_files, budget, |_| {})
}

fn collect_specced_files_with_hook<BeforeRead>(
    project: &Dir,
    spec_files: &[CoverageSpecFile],
    budget: &mut CoverageTraversalBudget,
    mut before_read: BeforeRead,
) -> Result<HashSet<String>, String>
where
    BeforeRead: FnMut(&Path),
{
    let mut specced = HashSet::new();
    let mut observed_specs = HashSet::new();
    for spec_file in spec_files {
        let relative = &spec_file.relative_path;
        if !observed_specs.insert(relative.clone()) {
            continue;
        }
        budget.charge_entries(1)?;
        ensure_coverage_depth(relative, budget.limits.max_depth)?;
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let name = relative.file_name().ok_or_else(|| {
            format!(
                "Coverage spec path {} has no terminal filename",
                relative.display()
            )
        })?;
        let Some(retained_directory) = open_retained_coverage_directory(project, parent)? else {
            return Err(format!(
                "Coverage spec file {} is missing from the retained project",
                relative.display()
            ));
        };
        let directory = &retained_directory.directory;
        verify_retained_coverage_directory(project, &retained_directory)?;
        before_read(relative);
        let bytes = read_coverage_file_with_expected_identity_and_hook(
            directory,
            name,
            relative,
            budget.limits.max_file_bytes,
            budget.remaining_bytes(),
            budget.limits.max_input_bytes,
            Some(spec_file.identity),
            || {},
        )?;
        verify_retained_coverage_directory(project, &retained_directory)?;
        budget.charge_input_file(relative, bytes.len() as u64)?;
        let content = std::str::from_utf8(&bytes).map_err(|_| {
            format!(
                "Coverage spec file {} is not valid UTF-8",
                relative.display()
            )
        })?;
        let normalized = content.replace("\r\n", "\n");
        if let Some(parsed) = parse_frontmatter(&normalized) {
            if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived) {
                continue;
            }
            for file in &parsed.frontmatter.files {
                if planned_source_path_is_safe(file)
                    && let Some(normalized) = normalize_source_mapping(file)
                {
                    specced.insert(normalized);
                }
            }
        }
    }
    Ok(specced)
}

fn select_coverage_spec_files(
    root: &Path,
    spec_files: &[PathBuf],
    retained_inventory: &[CoverageSpecFile],
) -> Result<Vec<CoverageSpecFile>, String> {
    let selected_spec_files: HashSet<PathBuf> = spec_files
        .iter()
        .map(|spec_file| coverage_relative_spec_path(root, spec_file))
        .collect::<Result<_, _>>()?;
    let retained_spec_files: Vec<CoverageSpecFile> = retained_inventory
        .iter()
        .filter(|spec| selected_spec_files.contains(&spec.relative_path))
        .cloned()
        .collect();
    if retained_spec_files.len() != selected_spec_files.len() {
        let retained_spec_set: HashSet<&PathBuf> = retained_spec_files
            .iter()
            .map(|spec| &spec.relative_path)
            .collect();
        let missing = selected_spec_files
            .iter()
            .find(|selected| !retained_spec_set.contains(selected))
            .expect("selected and retained spec counts differ");
        return Err(format!(
            "Coverage selected spec {} is missing from the retained spec inventory",
            missing.display()
        ));
    }
    Ok(retained_spec_files)
}

fn coverage_relative_spec_path(root: &Path, spec_file: &Path) -> Result<PathBuf, String> {
    let candidate = match spec_file.strip_prefix(root) {
        Ok(relative) => relative,
        Err(_) if spec_file.is_relative() => spec_file,
        Err(_) => {
            return Err(format!(
                "Coverage spec path {} must remain beneath the project root",
                spec_file.display()
            ));
        }
    };
    let mut relative = PathBuf::new();
    for component in candidate.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => relative.push(name),
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(format!(
                    "Coverage spec path {} must remain beneath the project root",
                    spec_file.display()
                ));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err("Coverage spec path cannot be the project root".to_string());
    }
    Ok(relative)
}

/// Get spec module directories that actually contain a .spec.md file.
/// Empty directories (e.g. from a failed prior generation) are ignored.
fn get_spec_module_dirs(
    project: &Dir,
    configured: &str,
    budget: &mut CoverageTraversalBudget,
) -> Result<Vec<String>, String> {
    let relative = if configured == "." {
        PathBuf::new()
    } else {
        PathBuf::from(normalize_source_mapping(configured).ok_or_else(|| {
            format!("Coverage specs directory must remain beneath the project root: {configured}")
        })?)
    };
    ensure_coverage_depth(&relative, budget.limits.max_depth)?;
    let mut modules = Vec::new();
    let Some(retained_directory) = open_retained_coverage_directory(project, &relative)? else {
        return Ok(modules);
    };
    let directory = &retained_directory.directory;
    let names = read_coverage_entry_names(directory, &relative, budget)?;
    for name in names {
        let name_text = name.to_str().ok_or_else(|| {
            format!(
                "Coverage spec-module path beneath {} is not valid UTF-8",
                display_coverage_path(&relative)
            )
        })?;
        let child = relative.join(&name);
        ensure_coverage_depth(&child, budget.limits.max_depth)?;
        let metadata = directory.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect retained coverage spec-module path {}: {error}",
                child.display()
            )
        })?;
        if coverage_metadata_is_link(&metadata) {
            return Err(format!(
                "Coverage spec-module path {} must not traverse a symlink or reparse point",
                child.display()
            ));
        }
        if !metadata.is_dir() {
            continue;
        }
        let child_directory = open_coverage_child_directory(directory, &name, &child)?;
        let spec_names = read_coverage_entry_names(&child_directory, &child, budget)?;
        let mut has_spec = false;
        for spec_name in spec_names {
            let spec_name_text = spec_name.to_str().ok_or_else(|| {
                format!(
                    "Coverage spec path beneath {} is not valid UTF-8",
                    child.display()
                )
            })?;
            if !spec_name_text.ends_with(".spec.md") {
                continue;
            }
            let spec_path = child.join(&spec_name);
            ensure_coverage_depth(&spec_path, budget.limits.max_depth)?;
            let spec_metadata = child_directory
                .symlink_metadata(&spec_name)
                .map_err(|error| {
                    format!(
                        "Cannot inspect retained coverage spec path {}: {error}",
                        spec_path.display()
                    )
                })?;
            if coverage_metadata_is_link(&spec_metadata) {
                return Err(format!(
                    "Coverage spec path {} must not traverse a symlink or reparse point",
                    spec_path.display()
                ));
            }
            if spec_metadata.is_file() {
                has_spec = true;
                continue;
            }
            if !spec_metadata.is_dir() {
                return Err(format!(
                    "Coverage spec path {} must be a regular file",
                    spec_path.display()
                ));
            }
        }
        verify_coverage_child_directory(directory, &name, &child, &child_directory)?;
        if has_spec {
            modules.push(name_text.to_string());
        }
    }
    verify_retained_coverage_directory(project, &retained_directory)?;
    modules.sort();
    Ok(modules)
}

fn coverage_source_modules(
    snapshot: &CoverageSourceSnapshot,
    configured: &str,
) -> Result<Vec<String>, String> {
    snapshot
        .immediate_modules
        .get(configured)
        .into_iter()
        .flatten()
        .map(|name| {
            name.to_str()
                .map(str::to_string)
                .ok_or_else(|| "Coverage source module name is not valid UTF-8".to_string())
        })
        .collect()
}

/// A configured source directory as a `/`-joined project-relative prefix, with
/// `.` becoming the empty root prefix.
fn normalized_coverage_source_dir(configured: &str) -> Result<String, String> {
    if configured == "." {
        return Ok(String::new());
    }
    normalize_source_mapping(configured).ok_or_else(|| {
        format!("Coverage source directory must remain beneath the project root: {configured}")
    })
}

/// Join a source-directory prefix with a child directory name.
fn coverage_module_directory(source_dir: &str, child: &str) -> String {
    if source_dir.is_empty() {
        child.to_string()
    } else {
        format!("{source_dir}/{child}")
    }
}

/// How many discovered source files a candidate module owns, and how many of
/// those no spec maps.
#[derive(Clone, Copy, Debug, Default)]
struct ModuleFileOwnership {
    owned: usize,
    unmapped: usize,
}

impl ModuleFileOwnership {
    fn observe(&mut self, mapped: bool) {
        self.owned += 1;
        if !mapped {
            self.unmapped += 1;
        }
    }

    fn absorb(&mut self, other: Self) {
        self.owned += other.owned;
        self.unmapped += other.unmapped;
    }

    /// Whether this module still has a coverage gap worth reporting.
    ///
    /// Owning NO discovered source file is not evidence of coverage: nothing
    /// was measured, so the module keeps its report rather than being declared
    /// covered by default. Only a module whose files were all looked at, and
    /// all found mapped, is silent.
    fn is_uncovered(self) -> bool {
        self.owned == 0 || self.unmapped > 0
    }
}

/// Which discovered source files each candidate module name owns.
///
/// Built in one pass over the measured file list so a candidate can be answered
/// without rescanning it, and keyed only by the directories and stems a
/// candidate can actually name — an unbounded ancestor index would grow with
/// tree depth for names nothing ever asks about.
#[derive(Debug, Default)]
struct CoverageModuleOwnership {
    directories: HashMap<String, ModuleFileOwnership>,
    flat_stems: HashMap<String, HashMap<String, ModuleFileOwnership>>,
}

impl CoverageModuleOwnership {
    fn index(
        source_files: &[String],
        specced_files: &HashSet<String>,
        candidate_directories: &HashSet<String>,
        source_roots: &HashSet<String>,
    ) -> Self {
        let mut ownership = Self {
            directories: candidate_directories
                .iter()
                .map(|directory| (directory.clone(), ModuleFileOwnership::default()))
                .collect(),
            flat_stems: HashMap::new(),
        };
        for file in source_files {
            let mapped = specced_files.contains(file.as_str());
            let (parent, name) = match file.rsplit_once('/') {
                Some((parent, name)) => (parent, name),
                None => ("", file.as_str()),
            };
            if source_roots.contains(parent) {
                // Keyed with the same derivation the flat-module block uses, so
                // a stem always finds its own files.
                let stem = Path::new(name)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(name);
                ownership
                    .flat_stems
                    .entry(parent.to_string())
                    .or_default()
                    .entry(stem.to_string())
                    .or_default()
                    .observe(mapped);
            }
            let mut prefix = String::new();
            if let Some(entry) = ownership.directories.get_mut(&prefix) {
                entry.observe(mapped);
            }
            for component in parent.split('/').filter(|component| !component.is_empty()) {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if let Some(entry) = ownership.directories.get_mut(&prefix) {
                    entry.observe(mapped);
                }
            }
        }
        ownership
    }

    fn directory(&self, directory: &str) -> ModuleFileOwnership {
        self.directories.get(directory).copied().unwrap_or_default()
    }

    /// Ownership across every source path a manifest declares for a module.
    /// Paths that do not normalize contribute nothing, so a module whose
    /// declared sources cannot be located keeps its report.
    fn declared_directories(&self, source_paths: &[String]) -> ModuleFileOwnership {
        let mut total = ModuleFileOwnership::default();
        for path in source_paths {
            let Some(normalized) = normalize_source_mapping(path) else {
                continue;
            };
            total.absorb(self.directory(&normalized));
        }
        total
    }

    fn flat_stem(&self, source_dir: &str, stem: &str) -> ModuleFileOwnership {
        self.flat_stems
            .get(source_dir)
            .and_then(|stems| stems.get(stem))
            .copied()
            .unwrap_or_default()
    }
}

/// Ownership over the explicit file list a configured module declares. A file
/// the traversal never discovered is not counted as owned, so a declaration
/// pointing at nothing keeps its report.
fn declared_files_ownership(
    files: &[String],
    discovered_files: &HashSet<&str>,
    specced_files: &HashSet<String>,
) -> ModuleFileOwnership {
    let mut ownership = ModuleFileOwnership::default();
    for file in files {
        let Some(normalized) = normalize_source_mapping(file) else {
            continue;
        };
        if !discovered_files.contains(normalized.as_str()) {
            continue;
        }
        ownership.observe(specced_files.contains(&normalized));
    }
    ownership
}

/// Compute file and module coverage.
#[allow(dead_code)]
pub fn compute_coverage(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> CoverageReport {
    compute_coverage_checked(root, spec_files, config).unwrap_or_else(|error| CoverageReport {
        total_source_files: 0,
        specced_file_count: 0,
        unspecced_files: Vec::new(),
        unspecced_modules: vec![format!("Coverage inconclusive: {error}")],
        total_loc: 0,
        specced_loc: 0,
        unspecced_file_loc: Vec::new(),
        missing_files: Vec::new(),
        skipped_links: Vec::new(),
        manifest_notices: Vec::new(),
    })
}

/// Retained manifest discovery for coverage, and what a failure of it means.
///
/// Discovery exists to INFER a source list the project did not state, and to
/// name the modules a manifest declares. Which of those a given project is
/// actually relying on decides whether a manifest that cannot be parsed is a
/// verdict or a note:
///
/// - **`source_dirs` omitted** — the list coverage would measure came out of
///   this discovery. There is nothing trustworthy to measure over, so the error
///   propagates and the command is inconclusive, exactly as before.
/// - **`source_dirs` stated** — the user already said where the sources are.
///   A failure to infer what they stated cannot overrule them: it costs only
///   manifest-declared module names, so it degrades to a notice and coverage
///   proceeds over the declared list.
///
/// The second case used to abort too. A Gradle project with an ordinary in-repo
/// `includeBuild("vendor/…")` could not run `check --strict` or `coverage` at
/// all — on any 6.0 candidate — even with `source_dirs` explicitly configured,
/// because an input the parser could not read was treated as a verdict about
/// the project (#723). Fixing only the parser would have left the class intact:
/// any manifest, in any ecosystem, that a future parser cannot read would again
/// override an explicit declaration.
///
/// The notice is not optional. Manifest modules seed module attribution, so a
/// degraded run reports fewer modules without specs than the tree has — a
/// report that improved because part of the measurement stopped. It travels
/// with the figures rather than to stderr, because stderr is precisely what a
/// CI job capturing stdout does not read (#570).
fn retained_coverage_manifest(
    root: &Path,
    project: &Dir,
    config: &SpecSyncConfig,
) -> Result<(crate::manifest::ManifestDiscovery, Vec<String>), String> {
    match crate::manifest::discover_from_manifests_checked_with_root(root, project) {
        Ok(discovery) => Ok((discovery, Vec::new())),
        Err(error) if config.source_dirs_set => Ok((
            crate::manifest::ManifestDiscovery::default(),
            vec![format!(
                "Manifest discovery was skipped ({error}); coverage used the configured source_dirs, \
                 so modules declared only by that manifest are not reported"
            )],
        )),
        Err(error) => Err(error),
    }
}

/// Compute file and module coverage while surfacing malformed manifest inputs.
pub fn compute_coverage_checked(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> Result<CoverageReport, String> {
    let project = open_coverage_project_root(root)?;
    coverage_snapshot_test_barrier(CoverageSnapshotCheckpoint::RootRetained)?;
    let mut budget = CoverageTraversalBudget::new(CoverageTraversalLimits::default());
    if spec_files.len() > budget.limits.max_entries {
        return Err(format!(
            "Coverage selected-spec input exceeds the {}-entry limit",
            budget.limits.max_entries
        ));
    }
    let (manifest, manifest_notices) = retained_coverage_manifest(root, &project, config)?;
    let selected_sources =
        select_coverage_source_directories(&project, config, budget.limits.max_depth)?;
    coverage_snapshot_test_barrier(CoverageSnapshotCheckpoint::ManifestDiscovered)?;
    let retained_spec_inventory =
        discover_coverage_spec_files(&project, &config.specs_dir, &mut budget)?;
    let retained_spec_files =
        select_coverage_spec_files(root, spec_files, &retained_spec_inventory)?;
    let specced_files = collect_specced_files(&project, &retained_spec_files, &mut budget)?;
    let exclude_dirs: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let source_snapshot = snapshot_selected_coverage_sources_with_hook(
        &project,
        root,
        config,
        &exclude_dirs,
        &mut budget,
        selected_sources,
        &mut |_| {},
    )?;
    let spec_modules: HashSet<String> =
        get_spec_module_dirs(&project, &config.specs_dir, &mut budget)?
            .into_iter()
            .collect();
    verify_coverage_project_root(root, &project)?;

    let mut all_source_files: Vec<String> = Vec::new();
    let mut file_loc: HashMap<String, usize> = HashMap::new();
    for file in &source_snapshot.files {
        let rel_str = coverage_relative_path_text(&file.relative_path)?;
        let excluded = config.exclude_patterns.iter().any(|pattern| {
            if pattern.starts_with("**/") && pattern.ends_with("/**") {
                let dir_part = pattern
                    .strip_prefix("**/")
                    .and_then(|rest| rest.strip_suffix("/**"))
                    .unwrap_or("");
                rel_str.contains(dir_part)
            } else if let Some(suffix) = pattern.strip_prefix("**/") {
                if let Some(ext) = suffix.strip_prefix('*') {
                    rel_str.ends_with(ext)
                } else {
                    rel_str.ends_with(&format!("/{suffix}")) || rel_str == suffix
                }
            } else {
                rel_str.contains(pattern.as_str())
            }
        });
        if excluded {
            continue;
        }
        all_source_files.push(rel_str.clone());
        file_loc.insert(rel_str.clone(), file.loc);
    }
    all_source_files.sort();
    all_source_files.dedup();

    let total_loc: usize = file_loc.values().sum();
    let specced_loc: usize = all_source_files
        .iter()
        .filter(|f| specced_files.contains(*f))
        .map(|f| file_loc.get(f.as_str()).copied().unwrap_or(0))
        .sum();

    let unspecced_files: Vec<String> = all_source_files
        .iter()
        .filter(|f| !specced_files.contains(*f))
        .cloned()
        .collect();

    let mut unspecced_file_loc: Vec<(String, usize)> = unspecced_files
        .iter()
        .map(|f| (f.clone(), file_loc.get(f.as_str()).copied().unwrap_or(0)))
        .collect();
    unspecced_file_loc.sort_by_key(|b| std::cmp::Reverse(b.1));

    let mut unspecced_modules = Vec::new();
    let mut seen_modules: HashSet<String> = HashSet::new();

    // "This module has no spec" used to mean nothing more than "no spec
    // DIRECTORY carries this name". When language-specific specs own the files
    // — `strutil_py`, `strutil_js`, … mapping `src/strutil.{py,mjs,…}` — no
    // `specs/strutil/` is ever created, so the missing NAME was read as a
    // missing SPEC and coverage invented an uncovered module `strutil` beside
    // `5/5 files covered` (#529). Every candidate below now has to show an
    // actual gap in its own files: a module whose discovered files were all
    // looked at and all found mapped is covered, whatever the spec is called.
    // Owning no discovered file is NOT such a showing — it is the absence of
    // input, and stays reported.
    let discovered_files: HashSet<&str> = all_source_files
        .iter()
        .map(std::string::String::as_str)
        .collect();
    let source_roots: HashSet<String> = config
        .source_dirs
        .iter()
        .map(|src_dir| normalized_coverage_source_dir(src_dir))
        .collect::<Result<_, _>>()?;
    let mut candidate_directories: HashSet<String> = manifest
        .modules
        .values()
        .flat_map(|module| module.source_paths.iter())
        .filter_map(|path| normalize_source_mapping(path))
        .collect();
    for src_dir in &config.source_dirs {
        let normalized = normalized_coverage_source_dir(src_dir)?;
        for module in coverage_source_modules(&source_snapshot, src_dir)? {
            candidate_directories.insert(coverage_module_directory(&normalized, &module));
        }
    }
    let ownership = CoverageModuleOwnership::index(
        &all_source_files,
        &specced_files,
        &candidate_directories,
        &source_roots,
    );

    // User-defined modules from specsync.json take priority
    if !config.modules.is_empty() {
        for (name, definition) in &config.modules {
            if spec_modules.contains(name) {
                continue;
            }
            let files =
                declared_files_ownership(&definition.files, &discovered_files, &specced_files);
            if files.is_uncovered() && seen_modules.insert(name.clone()) {
                unspecced_modules.push(name.clone());
            }
        }
    }

    // Then: detect modules from manifest files (Package.swift, Cargo.toml, etc.)
    for (name, module) in &manifest.modules {
        if spec_modules.contains(name) {
            continue;
        }
        if ownership
            .declared_directories(&module.source_paths)
            .is_uncovered()
            && seen_modules.insert(name.clone())
        {
            unspecced_modules.push(name.clone());
        }
    }

    // Detect subdirectory-based modules. JVM language roots (`src/main/kotlin`,
    // …) hold package segments, not modules — naming from those children is
    // how a conventional Gradle tree became module `com` (#473).
    for src_dir in &config.source_dirs {
        if crate::manifest::is_jvm_package_source_root(src_dir) {
            continue;
        }
        let normalized = normalized_coverage_source_dir(src_dir)?;
        for module in coverage_source_modules(&source_snapshot, src_dir)? {
            if spec_modules.contains(&module) {
                continue;
            }
            let directory = coverage_module_directory(&normalized, &module);
            if ownership.directory(&directory).is_uncovered() && seen_modules.insert(module.clone())
            {
                unspecced_modules.push(module);
            }
        }
    }

    // Detect flat source files as modules (e.g. src/config.rs → module "config")
    let skip_stems: HashSet<&str> = ["main", "lib", "mod", "index", "__init__", "app"]
        .into_iter()
        .collect();
    for src_dir in &config.source_dirs {
        let normalized = normalized_coverage_source_dir(src_dir)?;
        let normalized_path = PathBuf::from(&normalized);
        for file in source_snapshot.files.iter().filter(|file| {
            file.relative_path.parent().unwrap_or_else(|| Path::new("")) == normalized_path
        }) {
            let Some(stem) = file
                .relative_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            if skip_stems.contains(stem.as_str()) {
                continue;
            }
            if spec_modules.contains(&stem) {
                continue;
            }
            if ownership.flat_stem(&normalized, &stem).is_uncovered()
                && seen_modules.insert(stem.clone())
            {
                unspecced_modules.push(stem);
            }
        }
    }

    // Files a non-draft spec references in `files:` that do not exist on disk.
    // Draft planned mappings (status: draft, file not created yet) are intentional
    // and already surface as notices — they must not defeat `--require-coverage`.
    // Counting other missing references toward the denominator stops
    // `--require-coverage 100` from passing vacuously over broken references.
    let mut missing_files: Vec<String> = Vec::new();
    for spec_file in &retained_spec_files {
        let Ok(content) = std::fs::read_to_string(root.join(&spec_file.relative_path)) else {
            continue;
        };
        let normalized = content.replace("\r\n", "\n");
        let Some(parsed) = parse_frontmatter(&normalized) else {
            continue;
        };
        if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Draft)
            && !config.require_draft_files
        {
            continue;
        }
        if parsed.frontmatter.parsed_status() == Some(crate::types::SpecStatus::Archived) {
            continue;
        }
        for file in &parsed.frontmatter.files {
            if let Some(normalized) = normalize_source_mapping(file)
                && !root.join(&normalized).exists()
            {
                missing_files.push(normalized);
            }
        }
    }
    missing_files.sort();
    missing_files.dedup();

    let specced_count = all_source_files.len() - unspecced_files.len();

    // No percentage is computed here. The denominator can be zero — an empty or
    // misconfigured `source_dirs`, an over-broad `exclude_patterns` — and any
    // value stored for that case is a fabrication. `CoverageReport` derives the
    // percentages on demand and returns `None` when there was nothing to
    // measure (#582); the previous `100` fallbacks are gone.
    Ok(CoverageReport {
        total_source_files: all_source_files.len(),
        specced_file_count: specced_count,
        unspecced_files,
        unspecced_modules,
        total_loc,
        specced_loc,
        unspecced_file_loc,
        missing_files,
        skipped_links: budget.skipped_links.iter().cloned().collect(),
        manifest_notices,
    })
}

pub(crate) fn validate_local_dependency(
    dep: &str,
    root: &Path,
    specs_dir: &str,
) -> Result<PathBuf, String> {
    if is_cross_project_ref(dep) {
        // Cross-project refs (e.g. "owner/repo@module") are validated
        // by `specsync resolve`, not during local checks.
        return Ok(root.join(specs_dir));
    }
    let trimmed = dep.trim();
    if trimmed.is_empty() {
        return Err("Dependency spec entry is empty".to_string());
    }
    let as_path = Path::new(trimmed);
    if as_path.is_absolute()
        || as_path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
        || trimmed.contains('\\')
    {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (absolute paths and `..` traversal are not allowed)"
        ));
    }
    // Bare module names (no path separators or extension) resolve
    // against the specs directory, not the project root.
    let full_path = if !trimmed.contains('/') && !trimmed.contains('.') {
        root.join(specs_dir).join(trimmed)
    } else {
        root.join(trimmed)
    };
    if !source_within_root(root, trimmed) {
        return Err(format!(
            "Dependency spec path escapes the project root: {trimmed} (symlink resolution leaves the project)"
        ));
    }
    if !full_path.exists() {
        return Err(format!("Dependency spec not found: {trimmed}"));
    }
    Ok(full_path)
}
