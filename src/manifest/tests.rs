
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::fs;
    use tempfile::tempdir;

    fn windows_directory_replacement_was_refused(error: &io::Error) -> bool {
        const ERROR_ACCESS_DENIED: i32 = 5;
        const ERROR_SHARING_VIOLATION: i32 = 32;

        cfg!(windows)
            && matches!(
                error.raw_os_error(),
                Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)
            )
    }

    fn directory_identity_for_race_test(path: &Path) -> GradleFilesystemIdentity {
        let directory = Dir::open_ambient_dir(path, ambient_authority()).unwrap_or_else(|error| {
            panic!(
                "cannot open race-test directory {} for identity inspection: {error}",
                path.display()
            )
        });
        let metadata = directory.dir_metadata().unwrap_or_else(|error| {
            panic!(
                "cannot inspect race-test directory {}: {error}",
                path.display()
            )
        });
        gradle_filesystem_identity(&metadata).unwrap_or_else(|error| {
            panic!(
                "cannot identify race-test directory {}: {error}",
                path.display()
            )
        })
    }

    fn replace_directory_for_race_test(
        current: &Path,
        detached: &Path,
        replacement: &Path,
        replacement_refused: &Cell<bool>,
    ) {
        match fs::rename(current, detached) {
            Ok(()) => {
                fs::rename(replacement, current).unwrap_or_else(|error| {
                    panic!(
                        "cannot install replacement directory {} at {}: {error}",
                        replacement.display(),
                        current.display()
                    )
                });
            }
            Err(error) if windows_directory_replacement_was_refused(&error) => {
                replacement_refused.set(true);
            }
            Err(error) => panic!(
                "cannot detach retained directory {} to {}: {error}",
                current.display(),
                detached.display()
            ),
        }
    }

    fn assert_refused_replacement_and_released_handle(
        current: &Path,
        detached: &Path,
        replacement: &Path,
        expected_identity: GradleFilesystemIdentity,
    ) {
        assert!(
            current.is_dir(),
            "the original directory must remain present"
        );
        assert!(
            !detached.exists(),
            "a refused replacement must not detach the original directory"
        );
        assert!(
            replacement.is_dir(),
            "a refused replacement must leave the replacement untouched"
        );
        assert_eq!(
            directory_identity_for_race_test(current),
            expected_identity,
            "a refused replacement must preserve the original directory identity"
        );

        fs::rename(current, detached)
            .expect("the same rename must succeed after retained capabilities are released");
        fs::rename(detached, current)
            .expect("the original directory must restore after the negative-control rename");
        assert_eq!(
            directory_identity_for_race_test(current),
            expected_identity,
            "the negative-control rename must restore the original directory identity"
        );
        assert!(
            replacement.is_dir(),
            "the negative-control rename must not consume the replacement"
        );
    }

    struct CountingManifestAccess {
        texts: HashMap<PathBuf, String>,
        directories: HashSet<PathBuf>,
        children: HashMap<PathBuf, Vec<String>>,
        text_reads: HashMap<PathBuf, usize>,
        child_reads: HashMap<PathBuf, usize>,
        entries: usize,
        entry_limit: usize,
    }

    impl CountingManifestAccess {
        fn new(entry_limit: usize) -> Self {
            Self {
                texts: HashMap::new(),
                directories: HashSet::new(),
                children: HashMap::new(),
                text_reads: HashMap::new(),
                child_reads: HashMap::new(),
                entries: 0,
                entry_limit,
            }
        }
    }

    impl ManifestAccess for CountingManifestAccess {
        fn read_text(&mut self, relative: &Path, _label: &str) -> Result<Option<String>, String> {
            *self.text_reads.entry(relative.to_path_buf()).or_default() += 1;
            Ok(self.texts.get(relative).cloned())
        }

        fn directory_exists(&mut self, relative: &Path, _label: &str) -> Result<bool, String> {
            Ok(self.directories.contains(relative))
        }

        fn child_directories(
            &mut self,
            relative: &Path,
            _label: &str,
        ) -> Result<Vec<String>, String> {
            *self.child_reads.entry(relative.to_path_buf()).or_default() += 1;
            let children = self.children.get(relative).cloned().unwrap_or_default();
            self.charge_entries(children.len(), "test child directory")?;
            Ok(children)
        }

        fn charge_entries(&mut self, count: usize, _label: &str) -> Result<(), String> {
            if self.entries.saturating_add(count) > self.entry_limit {
                return Err(format!(
                    "Retained manifest discovery exceeds the {}-entry limit",
                    self.entry_limit
                ));
            }
            self.entries = self.entries.saturating_add(count);
            Ok(())
        }
    }

    #[test]
    fn test_parse_cargo_toml_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "").unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "my-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
regex = "1.0"
"#,
        )
        .unwrap();

        let result = parse_cargo_toml(tmp.path()).unwrap();
        assert!(result.modules.contains_key("my-crate"));
        let module = &result.modules["my-crate"];
        assert_eq!(module.source_paths, vec!["src"]);
        assert!(module.dependencies.contains(&"serde".to_string()));
        assert!(module.dependencies.contains(&"regex".to_string()));
    }

    #[test]
    fn cargo_library_target_paths_remain_exact_and_confined() {
        let tmp = tempdir().unwrap();
        for path in ["src/custom.rs", "lib/entry.rs", "entry.rs"] {
            fs::write(tmp.path().join("Cargo.toml"), format!("[package]\nname=\"demo\"\nversion=\"0.1.0\"\n[lib]\nname=\"alternate\"\npath=\"{path}\"\n")).unwrap();
            let discovery = discover_from_manifests_checked(tmp.path()).unwrap();
            assert_eq!(discovery.modules["demo"].source_paths, ["src", path]);
            let parent = Path::new(path).parent().unwrap().to_str().unwrap();
            let scan = if parent.is_empty() { path } else { parent };
            assert!(discovery.source_dirs.contains(&scan.to_string()));
            assert_eq!(discovery.exact_cargo_source_files.contains(path), parent.is_empty());
            assert!(!discovery.modules.contains_key("alternate"));
        }
        fs::write(tmp.path().join("Cargo.toml"), "[package]\nname=\"demo\"\nversion=\"0.1.0\"\n[lib]\npath=\"../outside.rs\"\n").unwrap();
        assert!(discover_from_manifests_checked(tmp.path()).unwrap_err().contains("escapes the project root"));
    }

    #[test]
    fn cargo_workspace_custom_library_scan_roots_survive_nested_members() {
        for external_to_nested_workspace in [false, true] {
            let tmp = tempdir().unwrap();
            fs::create_dir_all(tmp.path().join("workspace/crates/demo")).unwrap();
            fs::create_dir_all(tmp.path().join("workspace/crates/other")).unwrap();
            fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers=[\"workspace\"]\n").unwrap();
            fs::write(tmp.path().join("workspace/Cargo.toml"), "[workspace]\nmembers=[\"crates/demo\",\"crates/other\"]\n").unwrap();
            let traversal = if external_to_nested_workspace { "../../../" } else { "../../" };
            for name in ["demo", "other"] {
                fs::write(tmp.path().join(format!("workspace/crates/{name}/Cargo.toml")), format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\n[lib]\npath=\"{traversal}shared/{name}.rs\"\n")).unwrap();
            }
            let discovery = discover_from_manifests_checked(tmp.path()).unwrap();
            let shared = if external_to_nested_workspace { "shared" } else { "workspace/shared" };
            for name in ["demo", "other"] {
                assert_eq!(discovery.modules[name].source_paths, [format!("workspace/crates/{name}/src"), format!("{shared}/{name}.rs")]);
            }
            // Retention inside the enclosing workspace is not a discriminator.
            // Escaping only the nested workspace requires an extra shared root.
            let expected = if external_to_nested_workspace { vec!["shared", "workspace"] } else { vec!["workspace"] };
            assert_eq!(discovery.source_dirs, expected);
        }
    }

    #[test]
    fn cargo_root_package_targets_inside_members_keep_whole_member_scan_roots() {
        for reversed_members in [false, true] {
            let tmp = tempdir().unwrap();
            for name in ["beta", "gamma"] {
                let member = tmp.path().join("crates").join(name);
                fs::create_dir_all(member.join("src")).unwrap();
                fs::write(member.join("Cargo.toml"), format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\n")).unwrap();
            }
            let members = if reversed_members { "[\"crates/gamma\",\"crates/beta\"]" } else { "[\"crates/beta\",\"crates/gamma\"]" };
            fs::write(tmp.path().join("Cargo.toml"), format!("[package]\nname=\"root\"\nversion=\"0.1.0\"\n[lib]\npath=\"crates/beta/src/extra.rs\"\n[[bin]]\nname=\"tool\"\npath=\"crates/gamma/src/tool.rs\"\n[workspace]\nmembers={members}\n")).unwrap();
            let discovery = discover_from_manifests_checked(tmp.path()).unwrap();
            assert_eq!(discovery.modules["root"].source_paths, ["src", "crates/beta/src/extra.rs"]);
            assert_eq!(discovery.modules["tool"].source_paths, ["crates/gamma/src/tool.rs"]);
            let mut selections = discovery.source_dirs;
            selections.sort();
            assert_eq!(selections, ["crates/beta", "crates/gamma", "src"]);
        }
    }

    #[test]
    fn cargo_binary_target_paths_remain_exact_and_merge_same_name_package_roots() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\nname=\"server\"\nversion=\"0.1.0\"\n[[bin]]\nname=\"server\"\npath=\"src/foo.rs\"\n[[bin]]\nname=\"other\"\npath=\"tools/entry.rs\"\n").unwrap();
        let discovery = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(discovery.modules["server"].source_paths, ["src", "src/foo.rs"]);
        assert_eq!(discovery.modules["other"].source_paths, ["tools/entry.rs"]);
        assert!(discovery.source_dirs.contains(&"src".into()));
        assert!(discovery.source_dirs.contains(&"tools".into()));

        fs::create_dir_all(tmp.path().join("crates/demo")).unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers=[\"crates/demo\"]\n").unwrap();
        fs::write(tmp.path().join("crates/demo/Cargo.toml"), "[package]\nname=\"demo\"\nversion=\"0.1.0\"\n[[bin]]\nname=\"server\"\npath=\"bin/entry.rs\"\n").unwrap();
        let discovery = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(discovery.modules["server"].source_paths, ["crates/demo/bin/entry.rs"]);
        assert_eq!(discovery.modules["demo"].source_paths, ["crates/demo/src"]);
    }

    #[test]
    fn cargo_implicit_binary_paths_include_flat_and_nested_defaults() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("crates/demo/src/bin")).unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers=[\"crates/demo\"]\n").unwrap();
        fs::write(tmp.path().join("crates/demo/Cargo.toml"), "[package]\nname=\"demo\"\nversion=\"0.1.0\"\n[[bin]]\nname=\"server\"\n").unwrap();
        let discovery = parse_cargo_toml(tmp.path()).unwrap();
        assert_eq!(discovery.modules["server"].source_paths, ["crates/demo/src/bin/server.rs", "crates/demo/src/bin/server/main.rs"]);
        assert!(discovery.source_dirs.contains(&"crates/demo".into()));
        let member = parse_cargo_toml(&tmp.path().join("crates/demo")).unwrap();
        assert!(member.source_dirs.contains(&"src/bin".into()));
    }

    #[test]
    fn cargo_workspace_traversal_charges_declarations_and_memoizes_completed_members() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            "[workspace]\nmembers = [\"level-one\", \"level-one\"]\n".to_string(),
        );
        access.texts.insert(
            PathBuf::from("level-one/Cargo.toml"),
            "[package]\nname = \"level-one\"\n[workspace]\nmembers = [\"level-two\", \"level-two\"]\n"
                .to_string(),
        );
        access.texts.insert(
            PathBuf::from("level-one/level-two/Cargo.toml"),
            "[package]\nname = \"level-two\"\n".to_string(),
        );

        let mut active = HashSet::new();
        let mut completed = HashSet::new();
        let discovery =
            parse_cargo_toml_with_access(&mut access, Path::new(""), &mut active, &mut completed)
                .unwrap()
                .unwrap();

        assert!(discovery.modules.contains_key("level-one"));
        assert!(discovery.modules.contains_key("level-two"));
        assert_eq!(access.entries, 4);
        assert_eq!(access.text_reads[Path::new("Cargo.toml")], 1);
        assert_eq!(access.text_reads[Path::new("level-one/Cargo.toml")], 1);
        assert_eq!(
            access.text_reads[Path::new("level-one/level-two/Cargo.toml")],
            1
        );
        assert_eq!(completed.len(), 3);
        assert!(active.is_empty());
    }

    #[test]
    fn cargo_workspace_operational_discovery_supports_multiline_members() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            r#"
[workspace]
members = [
    "crates/core",
    # Inline comments and trailing commas are valid Cargo TOML.
    "crates/cli",
]
"#
            .to_string(),
        );
        access.texts.insert(
            PathBuf::from("crates/core/Cargo.toml"),
            "[package]\nname = \"core\"\nversion = \"0.1.0\"\n".to_string(),
        );
        access.texts.insert(
            PathBuf::from("crates/cli/Cargo.toml"),
            "[package]\nname = \"cli\"\nversion = \"0.1.0\"\n".to_string(),
        );

        let discovery = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap()
        .unwrap();

        assert!(discovery.modules.contains_key("core"));
        assert!(discovery.modules.contains_key("cli"));
        assert_eq!(access.entries, 2);
        assert_eq!(access.text_reads[Path::new("crates/core/Cargo.toml")], 1);
        assert_eq!(access.text_reads[Path::new("crates/cli/Cargo.toml")], 1);
    }

    #[test]
    fn cargo_workspace_operational_discovery_rejects_malformed_and_wrong_typed_toml() {
        for (content, expected) in [
            ("[workspace\nmembers = []\n", "as TOML"),
            ("workspace = []\n", "`workspace` as a TOML table"),
            (
                "[workspace]\nmembers = \"crates/core\"\n",
                "`workspace.members` as a string array",
            ),
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("Cargo.toml"), content.to_string());

            let error = parse_cargo_toml_with_access(
                &mut access,
                Path::new(""),
                &mut HashSet::new(),
                &mut HashSet::new(),
            )
            .unwrap_err();

            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn cargo_workspace_charges_every_member_before_rejecting_a_non_string() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/core\", 7]\n".to_string(),
        );

        let error = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap_err();

        assert!(error.contains("non-string workspace member"), "{error}");
        assert_eq!(access.entries, 2);
    }

    #[test]
    fn retained_entry_budget_accepts_the_limit_and_rejects_limit_plus_one() {
        let project = tempdir().unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);

        access
            .charge_entries(MAX_RETAINED_MANIFEST_ENTRIES, "workspace declarations")
            .unwrap();
        assert_eq!(access.entries, MAX_RETAINED_MANIFEST_ENTRIES);

        let error = access
            .charge_entries(1, "workspace declaration")
            .unwrap_err();
        assert!(
            error.contains(&format!("{MAX_RETAINED_MANIFEST_ENTRIES}-entry limit")),
            "{error}"
        );
        assert_eq!(access.entries, MAX_RETAINED_MANIFEST_ENTRIES);
    }

    #[test]
    fn test_parse_package_swift_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("Sources/MyLib")).unwrap();
        fs::write(
            tmp.path().join("Package.swift"),
            r#"
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    targets: [
        .target(name: "MyLib", dependencies: ["Logging"]),
        .target(name: "MyApp", dependencies: [.target(name: "MyLib")], path: "Sources/App"),
        .testTarget(name: "MyLibTests", dependencies: ["MyLib"]),
    ]
)
"#,
        )
        .unwrap();

        let result = parse_package_swift(tmp.path()).unwrap();
        assert!(result.modules.contains_key("MyLib"));
        assert!(result.modules.contains_key("MyApp"));
        // testTarget should NOT be in modules
        assert!(!result.modules.contains_key("MyLibTests"));

        let mylib = &result.modules["MyLib"];
        assert_eq!(mylib.source_paths, vec!["Sources/MyLib"]);
        assert!(mylib.dependencies.contains(&"Logging".to_string()));

        let myapp = &result.modules["MyApp"];
        assert_eq!(myapp.source_paths, vec!["Sources/App"]);
    }

    #[test]
    fn test_parse_package_json_workspaces() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("packages/core/src")).unwrap();
        fs::create_dir_all(tmp.path().join("packages/web/src")).unwrap();
        fs::write(
            tmp.path().join("packages/core/package.json"),
            r#"{"name": "@app/core"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("packages/web/package.json"),
            r#"{"name": "@app/web"}"#,
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"name": "my-app", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let result = parse_package_json(tmp.path()).unwrap();
        assert!(result.modules.contains_key("core"));
        assert!(result.modules.contains_key("web"));
        assert!(
            result
                .source_dirs
                .contains(&"packages/core/src".to_string())
        );
    }

    #[test]
    fn node_workspace_traversal_charges_patterns_and_deduplicates_cached_expansion() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{"name":"root","workspaces":["packages/*","packages/**"]}"#.to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            r#"{"name":"member"}"#.to_string(),
        );
        access
            .directories
            .insert(PathBuf::from("packages/member/src"));

        let discovery = parse_package_json_with_access(&mut access)
            .unwrap()
            .unwrap();

        assert!(discovery.modules.contains_key("member"));
        assert_eq!(access.entries, 3);
        assert_eq!(access.child_reads[Path::new("packages")], 1);
        assert_eq!(
            access.text_reads[Path::new("packages/member/package.json")],
            1
        );
    }

    #[test]
    fn node_workspace_object_form_preserves_yarn_and_pnpm_package_metadata() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{
                "name": "root",
                "packageManager": "pnpm@9.0.0",
                "workspaces": {
                    "packages": ["packages/*"],
                    "nohoist": ["**/native"]
                }
            }"#
            .to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            r#"{"name":"member"}"#.to_string(),
        );

        let discovery = parse_package_json_with_access(&mut access)
            .unwrap()
            .unwrap();

        assert!(discovery.modules.contains_key("member"));
        assert_eq!(access.entries, 2);
    }

    #[test]
    fn node_package_manifests_fail_closed_for_malformed_and_wrong_typed_inputs() {
        for (content, expected) in [
            ("{\"name\":", "as JSON"),
            ("[]", "must contain a JSON object"),
            (
                r#"{"workspaces":"packages/*"}"#,
                "`workspaces` as a string array",
            ),
            (
                r#"{"workspaces":{"packages":"packages/*"}}"#,
                "`workspaces.packages` as a string array",
            ),
            (
                r#"{"workspaces":{"nohoist":[]}}"#,
                "`workspaces.packages` as a string array",
            ),
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("package.json"), content.to_string());

            let error = parse_package_json_with_access(&mut access).unwrap_err();

            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn node_workspace_charges_every_entry_before_rejecting_a_non_string() {
        for content in [
            r#"{"workspaces":["packages/*",7]}"#,
            r#"{"workspaces":{"packages":["packages/*",7]}}"#,
        ] {
            let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
            access
                .texts
                .insert(PathBuf::from("package.json"), content.to_string());

            let error = parse_package_json_with_access(&mut access).unwrap_err();

            assert!(error.contains("non-string workspace entry"), "{error}");
            assert_eq!(access.entries, 2);
        }
    }

    #[test]
    fn nested_node_package_manifest_is_strict_json_object_input() {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access.texts.insert(
            PathBuf::from("package.json"),
            r#"{"workspaces":["packages/*"]}"#.to_string(),
        );
        access
            .children
            .insert(PathBuf::from("packages"), vec!["member".to_string()]);
        access.texts.insert(
            PathBuf::from("packages/member/package.json"),
            "[]".to_string(),
        );

        let error = parse_package_json_with_access(&mut access).unwrap_err();

        let manifest_path = Path::new("packages").join("member").join("package.json");
        assert!(
            error.contains(&display_manifest_path(&manifest_path)),
            "{error}"
        );
        assert!(error.contains("must contain a JSON object"), "{error}");
    }

    #[test]
    fn gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides() {
        let modules = parse_gradle_settings(
            r#"
include ':groovy:member',
        ':second'
include(
    ":kotlin:member",
    ':vendor:member'
);
project(':vendor:member').projectDir = file('vendor/custom-member');
project(":second").projectDir = new File(rootDir, "modules/second")
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "groovy/member".to_string(),
                    path: "groovy/member".to_string(),
                },
                GradleSettingsModule {
                    name: "kotlin/member".to_string(),
                    path: "kotlin/member".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
                GradleSettingsModule {
                    name: "vendor/member".to_string(),
                    path: "vendor/custom-member".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":safe", dynamicModule)"#,
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Unsupported or dynamic Gradle expression"));
        let compatibility_result = discover_from_manifests(tmp.path());
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_settings_reject_unsupported_include_prefixed_workspace_mutators() {
        // `includeBuild` is no longer in this list: it is judged by its
        // argument, and its own tests below cover both verdicts. `includeFlat`
        // resolves against the PARENT of the root, so its argument is outside
        // the project by construction, and `includeWorkspace` is not a form
        // this parser models — neither becomes supportable by reading its path.
        for settings in [
            r#"includeFlat("../outside")"#,
            r#"includeWorkspace("../outside")"#,
            r#"settings.includeFlat "../outside""#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported Gradle workspace mutator"),
                "unexpected include-prefixed mutator error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
val includeFlat = "documentation"
println(includeFlat)
include(":member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    /// THE REGRESSION (#723). An in-repo composite build is ordinary Gradle,
    /// and it was refused on the token prefix alone — the argument was never
    /// read. Every fixture the old guard was written against used
    /// `"../outside"`, so no test could fail for this reason: the guard was
    /// built for escapes and caught every composite build, and the two cases
    /// were indistinguishable. This is the case that had no coverage at all.
    ///
    /// The included build is accepted and then ignored: it names a separate
    /// build, so the root build's `include(...)` list is what modules still
    /// come from, and `:app` must survive beside it untouched.
    #[test]
    fn gradle_settings_accept_an_in_repo_include_build() {
        for settings in [
            "includeBuild(\"vendor/podo-shared\")\ninclude(\":app\")\n",
            "includeBuild 'vendor/podo-shared'\ninclude(':app')\n",
            "settings.includeBuild(\"vendor/podo-shared\")\ninclude(\":app\")\n",
            "includeBuild(\"./vendor/../vendor/podo-shared\")\ninclude(\":app\")\n",
            "includeBuild(\n    \"vendor/podo-shared\"\n)\ninclude(\":app\")\n",
            // Conditional forms, on one line and across three. They must agree:
            // a composite build contributes no module whether or not its branch
            // runs, so its position cannot change what is discovered. These two
            // disagreed at first — only the one-line form put the closing brace
            // on the declaration's own line.
            "if (loadShared) { includeBuild(\"vendor/podo-shared\") }\ninclude(\":app\")\n",
            "if (loadShared) {\n    includeBuild(\"vendor/podo-shared\")\n}\ninclude(\":app\")\n",
            "if (loadShared) { includeBuild 'vendor/podo-shared' }\ninclude(':app')\n",
        ] {
            let modules = parse_gradle_settings(settings)
                .unwrap_or_else(|error| panic!("in-repo includeBuild refused: {error}"));
            assert_eq!(
                modules,
                vec![GradleSettingsModule {
                    name: "app".to_string(),
                    path: "app".to_string(),
                }],
                "unexpected modules for {settings:?}"
            );
        }
    }

    /// THE REGRESSION (#725). `includeBuild(path) { dependencySubstitution { … } }`
    /// is the NORMAL way to write a composite build — it is the whole reason to
    /// include one — and #723 left it refused while accepting the bare
    /// `includeBuild(path)` minority form. The parser was therefore accepting
    /// the uncommon spelling and refusing the common one.
    ///
    /// The block carries substitution rules, not project declarations, so it
    /// contributes no module and no source directory. Once the path argument has
    /// been read and confined, a balanced block can be skipped whole, and `:app`
    /// must still survive beside it untouched — the same claim the bare form
    /// makes.
    ///
    /// The two brace cases are the reason the skip needs a real scan rather than
    /// a search for `}`: a brace inside a string literal must not move the
    /// depth, and a brace inside a comment must already be gone.
    #[test]
    fn gradle_settings_accept_an_include_build_with_a_configuration_block() {
        for settings in [
            // The reported form, verbatim (#725).
            "includeBuild(\"vendor/podo-shared\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo\")).using(project(\":\"))\n    }\n}\n\ninclude(\":app\")\n",
            // The same declaration on one line.
            "includeBuild(\"vendor/podo-shared\") { dependencySubstitution { substitute(module(\"com.example:podo\")).using(project(\":\")) } }\ninclude(\":app\")\n",
            "includeBuild('vendor/podo-shared') { dependencySubstitution { } }\ninclude(':app')\n",
            "settings.includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} }\ninclude(\":app\")\n",
            "includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} };\ninclude(\":app\")\n",
            "includeBuild(\"./vendor/../vendor/podo-shared\") { dependencySubstitution {} }\ninclude(\":app\")\n",
            // A brace inside a STRING is not a brace. Counted naively this block
            // never closes and the file is refused.
            "includeBuild(\"vendor/podo-shared\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo{\")).using(project(\":\"))\n    }\n}\ninclude(\":app\")\n",
            // A brace inside a COMMENT is not a brace either. Counted naively
            // this block closes early and the rest of the comment is trailing
            // junk.
            "includeBuild(\"vendor/podo-shared\") {\n    // the closing brace } belongs to prose\n    dependencySubstitution {}\n}\ninclude(\":app\")\n",
            "includeBuild(\"vendor/podo-shared\") {\n    /* } */ dependencySubstitution {}\n}\ninclude(\":app\")\n",
            // Position is not judged for the block form either.
            "if (loadShared) { includeBuild(\"vendor/podo-shared\") { dependencySubstitution {} } }\ninclude(\":app\")\n",
            "if (loadShared) {\n    includeBuild(\"vendor/podo-shared\") {\n        dependencySubstitution {}\n    }\n}\ninclude(\":app\")\n",
        ] {
            let modules = parse_gradle_settings(settings).unwrap_or_else(|error| {
                panic!("includeBuild with a configuration block refused: {error}; settings={settings:?}")
            });
            assert_eq!(
                modules,
                vec![GradleSettingsModule {
                    name: "app".to_string(),
                    path: "app".to_string(),
                }],
                "unexpected modules for {settings:?}"
            );
        }
    }

    /// Skipping the block must not hide what is written INSIDE it. The block is
    /// stepped over for the purpose of finding where the declaration ends; the
    /// text stays in the file, and every other guard still walks it.
    #[test]
    fn gradle_settings_still_judge_directives_inside_an_include_build_block() {
        for (settings, expected) in [
            (
                "includeBuild(\"vendor/shared\") {\n    include(\":smuggled\")\n}\n",
                "Unsupported block-scoped Gradle include",
            ),
            (
                "includeBuild(\"vendor/shared\") {\n    includeFlat(\"../outside\")\n}\n",
                "Unsupported Gradle workspace mutator",
            ),
            (
                "include(\":member\")\nincludeBuild(\"vendor/shared\") {\n    project(\":member\").projectDir = file(\"../outside\")\n}\n",
                "Unsupported block-scoped Gradle projectDir mutation",
            ),
            (
                "include(\":member\")\nincludeBuild(\"vendor/shared\") { project(\":member\").setProperty(\"projectDir\", outside) }\n",
                "Unsupported dynamic Gradle project mutation",
            ),
        ] {
            let error = match parse_gradle_settings(settings) {
                Ok(modules) => panic!("{settings} was accepted as {modules:?}"),
                Err(error) => error,
            };
            assert!(
                error.contains(expected),
                "unexpected error for a directive inside an includeBuild block: {error}; settings={settings:?}"
            );
        }
    }

    /// Honest label: the refusals asserted first are the CONTROL, and they
    /// pass on the unfixed parser too — that is the point. Reading the argument
    /// must not turn `includeBuild` into an accept-anything token, so what
    /// breaks this loop is a relaxation that stopped confining the path or
    /// stopped requiring one complete literal: a `../` escape leaves the
    /// repository, and an interpolated, dynamic, multi-argument, or
    /// trailing-expression form names something this parser cannot resolve from
    /// the settings text at all.
    ///
    /// The diagnostics asserted second are what is genuinely new. The old
    /// refusal said `Unsupported Gradle workspace mutator includeBuild` for
    /// every one of these — it was reached without ever reading the argument,
    /// so it could not say which argument was wrong, and it said exactly the
    /// same thing about the valid in-repo build above. Naming the path is how a
    /// reader can tell an escape from a form the parser does not model.
    ///
    /// The block-carrying fixtures added for #725 sit under the same honest
    /// label. Every one of them is refused on the unfixed parser too — but only
    /// because the block itself was refused, which said nothing about the
    /// argument and would have said the same thing about a perfectly safe path.
    /// Now the block is skipped, so the refusal has to come from the argument or
    /// not at all: these assert that skipping it did not make the block a way to
    /// carry an escape, an interpolation, an extra argument, or an unknowable
    /// extent past the checks in front of it.
    #[test]
    fn gradle_settings_still_refuse_an_escaping_or_dynamic_include_build() {
        for (settings, expected) in [
            (
                r#"includeBuild("../outside")"#,
                "must remain beneath the project root",
            ),
            (
                r#"includeBuild("/etc/outside")"#,
                "must remain beneath the project root",
            ),
            (
                r#"includeBuild("vendor/../../outside")"#,
                "must remain beneath the project root",
            ),
            (
                r#"settings.includeBuild "../outside""#,
                "must remain beneath the project root",
            ),
            (
                r#"includeBuild("${vendorRoot}/shared")"#,
                "Unsupported or dynamic Gradle expression",
            ),
            (
                r#"includeBuild(vendorRoot)"#,
                "Unsupported or dynamic Gradle expression",
            ),
            (
                r#"includeBuild("vendor/a", "vendor/b")"#,
                "must name one literal path",
            ),
            (
                r#"includeBuild("vendor/shared") = shared"#,
                "Unsupported trailing Gradle includeBuild declaration expression",
            ),
            // The block is skipped, never the ARGUMENT check in front of it
            // (#725). Each of these is the accepted form of the test above with
            // a configuration block appended, and each must reach the same
            // verdict it reaches without one.
            (
                "includeBuild(\"../outside\") { dependencySubstitution {} }",
                "must remain beneath the project root",
            ),
            (
                "includeBuild(\"../outside\") {\n    dependencySubstitution {\n        substitute(module(\"com.example:podo\")).using(project(\":\"))\n    }\n}\n",
                "must remain beneath the project root",
            ),
            (
                r#"includeBuild("${vendorRoot}/shared") { dependencySubstitution {} }"#,
                "Unsupported or dynamic Gradle expression",
            ),
            (
                r#"includeBuild("vendor/a", "vendor/b") { dependencySubstitution {} }"#,
                "must name one literal path",
            ),
            // An unbalanced block cannot be skipped, because its extent is
            // exactly what is unknown.
            (
                "includeBuild(\"vendor/shared\") {\n    dependencySubstitution {\n",
                "configuration block has unbalanced braces",
            ),
            // A brace inside a string literal does not close the block. Without
            // a quote-aware scan the `"}"` below would balance it and this would
            // be accepted with the real closing brace left over.
            (
                r#"includeBuild("vendor/shared") { dependencySubstitution { substitute("}") } "#,
                "configuration block has unbalanced braces",
            ),
            (r#"includeBuild(".")"#, "must identify a project path"),
            // Position is not judged, but the ARGUMENT still is — wherever the
            // declaration sits.
            (
                r#"if (loadShared) { includeBuild("../outside") }"#,
                "must remain beneath the project root",
            ),
        ] {
            // The control: refused at all. True before this change and after.
            let error = match parse_gradle_settings(settings) {
                Ok(modules) => panic!("{settings} was accepted as {modules:?}"),
                Err(error) => error,
            };
            // What is new: the refusal names the argument, not the token.
            assert!(
                error.contains(expected),
                "unexpected includeBuild error for {settings}: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_only_workspace_discovers_included_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("member/src/main/kotlin")).unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();

        let result = discover_from_manifests_checked(tmp.path()).unwrap();

        assert_eq!(
            result.modules["member"].source_paths,
            vec!["member/src/main/kotlin"]
        );
    }

    #[test]
    fn gradle_settings_only_workspace_fails_closed_when_malformed() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("settings.gradle"), "include(\":member\"\n").unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

        assert!(error.contains("Cannot parse Gradle settings manifest"));
        assert!(discover_from_manifests(tmp.path()).modules.is_empty());
    }

    #[test]
    fn gradle_manifest_discovery_rejects_non_regular_oversized_and_non_utf8_manifests() {
        for name in [
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
        ] {
            let directory_manifest = tempdir().unwrap();
            fs::create_dir(directory_manifest.path().join(name)).unwrap();
            let error = discover_from_manifests_checked(directory_manifest.path()).unwrap_err();
            assert!(
                error.contains(name) && error.contains("must be a regular file"),
                "unexpected {name} preflight error: {error}"
            );
        }

        let oversized_manifest = tempdir().unwrap();
        fs::write(
            oversized_manifest.path().join("settings.gradle.kts"),
            vec![b'a'; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
        )
        .unwrap();
        let error = discover_from_manifests_checked(oversized_manifest.path()).unwrap_err();
        assert!(error.contains("exceeds the"));

        let non_utf8_manifest = tempdir().unwrap();
        fs::write(
            non_utf8_manifest.path().join("settings.gradle.kts"),
            [0xff, 0xfe],
        )
        .unwrap();
        let error = discover_from_manifests_checked(non_utf8_manifest.path()).unwrap_err();
        assert!(error.contains("not valid UTF-8"));
    }

    #[test]
    fn gradle_manifest_discovery_preflights_shadowed_groovy_variants() {
        for shadowed in ["build.gradle", "settings.gradle"] {
            let tmp = tempdir().unwrap();
            fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
            fs::write(
                tmp.path().join("settings.gradle.kts"),
                "include(\":member\")\n",
            )
            .unwrap();
            fs::create_dir(tmp.path().join(shadowed)).unwrap();

            let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

            assert!(
                error.contains(shadowed) && error.contains("must be a regular file"),
                "unexpected shadowed Gradle manifest error: {error}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_reads_reject_regular_file_replacement_at_every_checkpoint() {
        for target in [
            GradleManifestReadCheckpoint::PreOpen,
            GradleManifestReadCheckpoint::Opened,
            GradleManifestReadCheckpoint::AfterOpen,
            GradleManifestReadCheckpoint::AfterRead,
        ] {
            let tmp = tempdir().unwrap();
            let manifest = tmp.path().join("settings.gradle.kts");
            let replacement = tmp.path().join("replacement.gradle.kts");
            fs::write(&manifest, "include(\":original\")\n").unwrap();
            fs::write(&replacement, "include(\":replacement\")\n").unwrap();
            let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();
            let mut replaced = false;

            let error = gradle_confined_manifest_text_with_checkpoint(
                &root,
                "settings.gradle.kts",
                "settings",
                |checkpoint| {
                    if checkpoint == target && !replaced {
                        fs::rename(&replacement, &manifest).unwrap();
                        replaced = true;
                    }
                },
            )
            .unwrap_err();

            assert!(replaced);
            assert!(
                error.contains("changed during confined open")
                    || error.contains("changed during confined read"),
                "unexpected identity replacement error at {target:?}: {error}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn gradle_manifest_reads_reject_preopen_regular_file_replacement_on_windows() {
        let tmp = tempdir().unwrap();
        let manifest = tmp.path().join("settings.gradle.kts");
        let replacement = tmp.path().join("replacement.gradle.kts");
        fs::write(&manifest, "include(\":original\")\n").unwrap();
        fs::write(&replacement, "include(\":replacement\")\n").unwrap();
        let root = Dir::open_ambient_dir(tmp.path(), ambient_authority()).unwrap();

        let error = gradle_confined_manifest_text_with_checkpoint(
            &root,
            "settings.gradle.kts",
            "settings",
            |checkpoint| {
                if checkpoint == GradleManifestReadCheckpoint::PreOpen {
                    fs::remove_file(&manifest).unwrap();
                    fs::rename(&replacement, &manifest).unwrap();
                }
            },
        )
        .unwrap_err();

        assert!(error.contains("changed during confined open"));
    }

    #[test]
    fn retained_root_manifest_discovery_accepts_matching_capability_and_rejects_mismatch() {
        let project = tempdir().unwrap();
        fs::create_dir_all(project.path().join("member/src/main/kotlin")).unwrap();
        fs::write(
            project.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap();
        assert_eq!(
            discovery.modules["member"].source_paths,
            vec!["member/src/main/kotlin"]
        );

        let other = tempdir().unwrap();
        let error = discover_from_manifests_checked_with_root(other.path(), &retained).unwrap_err();
        assert!(error.contains("does not match the retained project root"));
    }

    #[cfg(unix)]
    #[test]
    fn retained_non_gradle_manifest_access_ignores_an_ambient_root_replacement() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let root = tmp.path().join("project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(replacement.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"retained-project\"\n",
        )
        .unwrap();
        fs::write(
            replacement.join("Cargo.toml"),
            "[package]\nname = \"replacement-project\"\n",
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(&root, ambient_authority()).unwrap();
        fs::rename(&root, &original).unwrap();
        symlink(&replacement, &root).unwrap();

        let mut access = RetainedManifestAccess::new(&retained);
        let discovery = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap()
        .unwrap();

        assert!(discovery.modules.contains_key("retained-project"));
        assert!(!discovery.modules.contains_key("replacement-project"));
    }

    #[cfg(unix)]
    #[test]
    fn retained_non_gradle_manifest_access_rejects_a_symlink_without_disclosing_referent() {
        use std::os::unix::fs::symlink;

        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let sentinel = "RETAINED_MANIFEST_SENTINEL";
        fs::write(
            outside.path().join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        symlink(
            outside.path().join("Cargo.toml"),
            project.path().join("Cargo.toml"),
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let error =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn retained_duplicate_cargo_members_reject_a_linked_workspace_without_disclosure() {
        use std::os::unix::fs::symlink;

        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let sentinel = "RETAINED_WORKSPACE_SENTINEL";
        fs::write(
            project.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\", \"member\"]\n",
        )
        .unwrap();
        fs::write(
            outside.path().join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        symlink(outside.path(), project.path().join("member")).unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let error =
            discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

        assert!(error.contains("symlink or reparse point"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    #[test]
    fn retained_non_gradle_manifest_access_rejects_non_regular_and_oversized_inputs() {
        let non_regular = tempdir().unwrap();
        fs::create_dir(non_regular.path().join("Cargo.toml")).unwrap();
        let retained = Dir::open_ambient_dir(non_regular.path(), ambient_authority()).unwrap();
        let error =
            discover_from_manifests_checked_with_root(non_regular.path(), &retained).unwrap_err();
        assert!(error.contains("must be a regular file"), "{error}");

        let oversized = tempdir().unwrap();
        let manifest = fs::File::create(oversized.path().join("Cargo.toml")).unwrap();
        manifest
            .set_len(MAX_RETAINED_MANIFEST_BYTES.saturating_add(1))
            .unwrap();
        let retained = Dir::open_ambient_dir(oversized.path(), ambient_authority()).unwrap();
        let error =
            discover_from_manifests_checked_with_root(oversized.path(), &retained).unwrap_err();
        assert!(error.contains("exceeds the"), "{error}");
        assert!(error.contains("byte limit"), "{error}");
    }

    #[test]
    fn retained_nested_manifest_read_rejects_a_replaced_parent_directory() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("workspace");
        let original = project.path().join("original-workspace");
        let replacement = project.path().join("replacement-workspace");
        fs::create_dir(&workspace).unwrap();
        fs::create_dir(&replacement).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"retained\"\n",
        )
        .unwrap();
        let original_content = "[package]\nname = \"retained\"\n";
        let sentinel = "REPLACEMENT_MANIFEST_SENTINEL";
        fs::write(
            replacement.join("Cargo.toml"),
            format!("[package]\nname = \"{sentinel}\"\n"),
        )
        .unwrap();
        let original_identity = directory_identity_for_race_test(&workspace);
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let replacement_refused = Cell::new(false);

        let result = read_retained_manifest_text_with_hook(
            &retained,
            Path::new("workspace/Cargo.toml"),
            "Cargo manifest",
            MAX_RETAINED_MANIFEST_BYTES,
            MAX_RETAINED_MANIFEST_INPUT_BYTES,
            || {
                replace_directory_for_race_test(
                    &workspace,
                    &original,
                    &replacement,
                    &replacement_refused,
                );
            },
        );

        if replacement_refused.get() {
            let content = result
                .unwrap()
                .expect("the original retained manifest must remain readable");
            assert_eq!(content, original_content);
            drop(retained);
            assert_refused_replacement_and_released_handle(
                &workspace,
                &original,
                &replacement,
                original_identity,
            );
            return;
        }

        let error = result.unwrap_err();
        assert!(error.contains("directory workspace changed"), "{error}");
        assert!(!error.contains(sentinel), "{error}");
    }

    fn assert_retained_child_enumeration_rejects_nested_directory_replacement(
        relative: &Path,
        label: &str,
    ) {
        let project = tempdir().unwrap();
        let workspace = project.path().join(relative);
        let detached = project.path().join("detached-workspace");
        let replacement = project.path().join("replacement-workspace");
        fs::create_dir_all(workspace.join("original-member")).unwrap();
        fs::create_dir_all(replacement.join("replacement-member")).unwrap();
        let original_identity = directory_identity_for_race_test(&workspace);
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);
        let replacement_refused = Cell::new(false);

        let result = access.child_directories_with_hook(relative, label, || {
            replace_directory_for_race_test(
                &workspace,
                &detached,
                &replacement,
                &replacement_refused,
            );
        });

        if replacement_refused.get() {
            assert_eq!(result.unwrap(), vec!["original-member"]);
            drop(access);
            drop(retained);
            assert_refused_replacement_and_released_handle(
                &workspace,
                &detached,
                &replacement,
                original_identity,
            );
            return;
        }

        let error = result.unwrap_err();
        assert!(
            error.contains(&format!(
                "Retained {label} directory {} changed during confined read",
                relative.display()
            )),
            "{error}"
        );
        assert!(!access.children.contains_key(relative));
    }

    #[test]
    fn retained_cargo_child_enumeration_rejects_nested_regular_directory_replacement() {
        assert_retained_child_enumeration_rejects_nested_directory_replacement(
            Path::new("nested/crates"),
            "Cargo workspace",
        );
    }

    #[test]
    fn retained_node_child_enumeration_rejects_nested_regular_directory_replacement() {
        assert_retained_child_enumeration_rejects_nested_directory_replacement(
            Path::new("nested/packages"),
            "Node workspace",
        );
    }

    #[test]
    fn retained_node_workspace_rejects_replacement_after_enumeration_before_child_consumption() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("packages");
        let detached = project.path().join("detached-packages");
        let replacement = project.path().join("replacement-packages");
        fs::create_dir_all(workspace.join("member/src")).unwrap();
        fs::create_dir_all(replacement.join("member/src")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(
            workspace.join("member/package.json"),
            r#"{"name":"original"}"#,
        )
        .unwrap();
        fs::write(
            replacement.join("member/package.json"),
            r#"{"name":"replacement"}"#,
        )
        .unwrap();
        let original_identity = directory_identity_for_race_test(&workspace);
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);
        let replacement_refused = Cell::new(false);

        let result = parse_package_json_with_access_and_hook(&mut access, |relative| {
            assert_eq!(relative, Path::new("packages"));
            replace_directory_for_race_test(
                &workspace,
                &detached,
                &replacement,
                &replacement_refused,
            );
        });

        if replacement_refused.get() {
            let discovery = result
                .unwrap()
                .expect("the original retained Node workspace must remain discoverable");
            assert!(discovery.modules.contains_key("member"));
            assert_eq!(
                access
                    .texts
                    .get(Path::new("packages/member/package.json"))
                    .and_then(Option::as_deref),
                Some(r#"{"name":"original"}"#)
            );
            drop(access);
            drop(retained);
            assert_refused_replacement_and_released_handle(
                &workspace,
                &detached,
                &replacement,
                original_identity,
            );
            return;
        }

        let error = result.unwrap_err();
        assert!(
            error.contains("Retained Node workspace directory packages"),
            "{error}"
        );
    }

    #[test]
    fn retained_node_workspace_swap_read_restore_uses_enumerated_child_capability() {
        let project = tempdir().unwrap();
        let workspace = project.path().join("packages");
        let detached = project.path().join("detached-packages");
        let replacement = project.path().join("replacement-packages");
        fs::create_dir_all(workspace.join("member/src")).unwrap();
        fs::create_dir_all(replacement.join("member")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(
            workspace.join("member/package.json"),
            r#"{"name":"original"}"#,
        )
        .unwrap();
        fs::write(
            replacement.join("member/package.json"),
            r#"{"name":"replacement","workspaces":"SWAP_READ_RESTORE_SENTINEL"}"#,
        )
        .unwrap();
        let original_identity = directory_identity_for_race_test(&workspace);
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();
        let mut access = RetainedManifestAccess::new(&retained);
        let replacement_refused = Cell::new(false);
        let restored = Cell::new(false);

        let discovery = parse_package_json_with_access_and_hooks(
            &mut access,
            |relative| {
                assert_eq!(relative, Path::new("packages"));
                replace_directory_for_race_test(
                    &workspace,
                    &detached,
                    &replacement,
                    &replacement_refused,
                );
            },
            |relative, child| {
                assert_eq!(relative, Path::new("packages"));
                assert_eq!(child, "member");
                if !replacement_refused.get() {
                    fs::rename(&workspace, &replacement).unwrap();
                    fs::rename(&detached, &workspace).unwrap();
                    restored.set(true);
                }
            },
        )
        .unwrap()
        .unwrap();

        assert_eq!(restored.get(), !replacement_refused.get());
        assert!(discovery.modules.contains_key("member"));
        assert!(
            discovery
                .source_dirs
                .contains(&"packages/member/src".to_string())
        );
        assert_eq!(
            access
                .texts
                .get(Path::new("packages/member/package.json"))
                .and_then(Option::as_deref),
            Some(r#"{"name":"original"}"#)
        );
        if replacement_refused.get() {
            drop(access);
            drop(retained);
            assert_refused_replacement_and_released_handle(
                &workspace,
                &detached,
                &replacement,
                original_identity,
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn retained_node_workspace_enumeration_bounds_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_MANIFEST_BOUNDED_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" retained_node_workspace_enumeration_bounds_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "Node workspace discovery failed with a 64-descriptor soft limit"
            );
            return;
        }

        let project = tempdir().unwrap();
        fs::create_dir_all(project.path().join("packages")).unwrap();
        fs::write(
            project.path().join("package.json"),
            r#"{"name":"root","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        for index in 0..200 {
            let workspace = project.path().join(format!("packages/member-{index:03}"));
            fs::create_dir_all(workspace.join("src")).unwrap();
            fs::write(
                workspace.join("package.json"),
                format!(r#"{{"name":"member-{index:03}"}}"#),
            )
            .unwrap();
        }
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery = discover_from_manifests_checked_with_root(project.path(), &retained)
            .expect("broad retained Node workspace discovery must stay descriptor-bounded");

        assert_eq!(discovery.modules.len(), 200);
    }

    #[cfg(unix)]
    #[test]
    fn retained_node_workspace_bases_bound_open_directory_handles() {
        const CHILD_ENV: &str = "SPECSYNC_MANIFEST_BOUNDED_BASE_HANDLES_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            let executable = std::env::current_exe().unwrap();
            let status = std::process::Command::new("sh")
                .args([
                    "-c",
                    "ulimit -n 64; exec \"$1\" retained_node_workspace_bases_bound_open_directory_handles --nocapture",
                    "sh",
                ])
                .arg(executable)
                .env(CHILD_ENV, "1")
                .status()
                .unwrap();
            assert!(
                status.success(),
                "Node workspace discovery retained handles across distinct bases"
            );
            return;
        }

        let project = tempdir().unwrap();
        let mut patterns = Vec::new();
        for index in 0..90 {
            let base = format!("base-{index:03}");
            let member = format!("member-{index:03}");
            patterns.push(format!(r#""{base}/*""#));
            let workspace = project.path().join(&base).join(&member);
            fs::create_dir_all(workspace.join("src")).unwrap();
            fs::write(
                workspace.join("package.json"),
                format!(r#"{{"name":"{member}"}}"#),
            )
            .unwrap();
        }
        fs::write(
            project.path().join("package.json"),
            format!(r#"{{"name":"root","workspaces":[{}]}}"#, patterns.join(",")),
        )
        .unwrap();
        let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

        let discovery = discover_from_manifests_checked_with_root(project.path(), &retained)
            .expect("distinct retained Node workspace bases must stay descriptor-bounded");

        assert_eq!(discovery.modules.len(), 90);
    }

    #[test]
    fn gradle_settings_reject_unsupported_project_dir_bases_and_suffixes() {
        let outside_base = parse_gradle_settings(
            r#"
include(":outside")
project(":outside").projectDir = new File(rootDir.parentFile, "outside")
"#,
        )
        .unwrap_err();
        assert!(outside_base.contains("base must be exactly rootDir"));

        let project_root = parse_gradle_settings(
            "include(\":root\")\nproject(\":root\").projectDir = file(\".\")\n",
        )
        .unwrap();
        assert_eq!(project_root[0].path, ".");

        for assignment in [
            r#"file("modules/safe").parentFile"#,
            r#"new File(rootDir, "modules/safe") + "/outside""#,
        ] {
            let settings =
                format!("include(\":safe\")\nproject(\":safe\").projectDir = {assignment}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(error.contains("Unsupported trailing Gradle projectDir assignment expression"));
        }
    }

    #[test]
    fn gradle_settings_reject_project_root_escapes() {
        for settings in [
            r#"
include(":outside")
project(":outside").projectDir = file("../outside")
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("../outside"))
"#,
            r#"include(":..:outside")"#,
            r#"include("C:/outside")"#,
            r#"include("C:outside")"#,
            r#"include(":C:\\outside")"#,
            r#"
include(":outside")
project(":outside").projectDir = file("safe/../../outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("C:\\outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("\\\\server\\share")
"#,
            r#"
include(":safe")
project("C:/outside").projectDir = file("modules/safe")
"#,
            r#"
include(":safe")
project("C:outside").projectDir = file("modules/safe")
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("\u002e\u002e/outside"))
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("\056\056/outside"))
"#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("must remain beneath the project root"),
                "unexpected Gradle confinement error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_preserve_rooted_nested_names_that_resemble_drive_relative_paths() {
        let modules = parse_gradle_settings(
            r#"
include(":C:member")
project(":C:member").projectDir = file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "C/member".to_string(),
                path: "modules/member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_support_literal_set_project_dir_forms() {
        let modules = parse_gradle_settings(
            r#"
include(":first", ":second")
project(":first").setProjectDir(file("modules/first"))
project(":second").setProjectDir(new File(rootDir, "modules/second"))
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "first".to_string(),
                    path: "modules/first".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_settings_reject_dynamic_or_ambiguous_set_project_dir_forms() {
        for setter in [
            r#"project(":member").setProjectDir(projectDirProvider.get())"#,
            r#"project(":member").setProjectDir(file("modules/member"), file("other"))"#,
            r#"project(":member").setProjectDir(file("modules/member")).parentFile"#,
            r#"project(":member").setProjectDir(file("$outside"))"#,
            r#"project(":member").setProjectDir(file("${outside}"))"#,
            r#"project(":member").projectDir = file("$outside")"#,
            r#"project(":member").projectDir = file("\u0024outside")"#,
            r#"project(":member").projectDir = file("\044outside")"#,
            r#"project(":member").projectDir = newFile(rootDir, "../outside")"#,
            r#"project(":member").setProjectDir(newFile(rootDir, "../outside"))"#,
        ] {
            let settings = format!("include(\":member\")\n{setter}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(
                error.contains("Unsupported Gradle projectDir assignment")
                    || error.contains("Unsupported or dynamic Gradle expression")
                    || error.contains("must contain one path")
                    || error.contains("Unsupported trailing Gradle projectDir assignment"),
                "unexpected setProjectDir parser error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_reject_dynamic_and_unrecognized_project_mutations() {
        for mutation in [
            r#"project(":member").setProperty("projectDir", file("../outside"))"#,
            r#"val changed = project(":member").setProperty("projectDir", file("../outside"))"#,
            r#"project(":member")["projectDir"] = file("../outside")"#,
            r#"project(":member").properties["projectDir"] = file("../outside")"#,
            "project(\":member\")[\n    \"projectDir\"\n]\n    = file(\"../outside\")",
            r#"project(":member").configure { projectDir = file("../outside") }"#,
            r#"project(":member").customDirectory = file("../outside")"#,
            r#"if (enabled) project(":member").customDirectory = file("../outside")"#,
        ] {
            let settings = format!("include(\":member\")\n{mutation}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(
                error.contains("Unsupported")
                    && (error.contains("project mutation")
                        || error.contains("projectDir mutation")),
                "unexpected executable project mutation error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
include(":member")
project(":member").projectDir
project(":member")["projectDir"] == file("modules/member")
project(":member").properties["projectDir"]
val unchanged = project(":member").projectDir == file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_directive_control_flow_validation_is_local() {
        let modules = parse_gradle_settings(
            r#"
if (enabled) println("unrelated")
for (entry in entries) {
    println(entry)
}
include(":member")
project(":member").projectDir = file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "modules/member".to_string(),
            }]
        );

        for settings in [
            r#"if (enabled) include(":member")"#,
            "if (\n    enabled\n)\ninclude(\":member\")",
            "if (enabled) {\ninclude(\":member\")\n}",
            "if (\n    enabled\n)\nproject(\":member\").projectDir = file(\"modules/member\")",
            "if (enabled) {\nproject(\":member\").setProjectDir(file(\"modules/member\"))\n}",
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported")
                    && (error.contains("conditional") || error.contains("block-scoped")),
                "unexpected directive-local control-flow error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_reject_indirect_or_conditional_mutations_but_allow_reads() {
        for settings in [
            r#"
include(":member")
val member = project(":member")
member.setProjectDir(file("../outside"))
"#,
            r#"
include(":member")
val member = project(":member")
member.projectDir = file("../outside")
"#,
            r#"if (enabled) include(":member")"#,
            "if (enabled)\ninclude(\":member\")",
            "if (enabled) {\ninclude(\":member\")\n}",
            r#"settings.include(":member")"#,
            r#"
include(":member")
if (enabled) project(":member").projectDir = file("modules/member")
"#,
            r#"
include(":member")
project(":member").projectDir += file("modules/member")
"#,
            r#"
include(":member")
project(":member") {
    projectDir = file("../outside")
}
"#,
            r#"
include(":member")
project(":member") {
    setProjectDir(file("../outside"))
}
"#,
            r#"
include(":member")
project(":member") . setProjectDir(file("../outside"))
"#,
            r#"
include(":member")
if (enabled) {
    project(":member").projectDir = file("modules/member")
}
"#,
            r#"include(""":member""")"#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("Unsupported") || error.contains("must contain a literal module"),
                "unexpected indirect Gradle mutation error: {error}"
            );
        }

        let modules = parse_gradle_settings(
            r#"
include(":member")
println(project(":member").projectDir)
val unchanged = project(":member").projectDir == file("modules/member")
"#,
        )
        .unwrap();
        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "member".to_string(),
                path: "member".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_ignore_multiline_literals_and_nested_comments() {
        let modules = parse_gradle_settings(
            r#"
val kotlinDocumentation = """
include(":kotlin-phantom")
project(":real").projectDir = file("../outside")
"""
def groovyDocumentation = '''
include(":groovy-phantom")
project(":real").setProjectDir(file("../outside"))
'''
/*
  include(":outer-comment-phantom")
  /* include(":nested-comment-phantom") */
  project(":real").projectDir = file("../outside")
*/
val regularDocumentation = ".projectDir include(\":quoted-phantom\")"
include(":real")
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![GradleSettingsModule {
                name: "real".to_string(),
                path: "real".to_string(),
            }]
        );
    }

    #[test]
    fn gradle_settings_reject_interpolated_includes_without_partial_modules() {
        for include in [r#"include(":$module")"#, r#"include(":${module}")"#] {
            let error = parse_gradle_settings(include).unwrap_err();
            assert!(
                error.contains("Unsupported or dynamic Gradle expression"),
                "unexpected interpolated include error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_preserve_literal_dollars() {
        let modules = parse_gradle_settings(
            r#"
include(':literal$module', ":escaped\$module")
project(':literal$module').setProjectDir(file('modules/$literal'))
project(":escaped\$module").setProjectDir(file("modules/escaped\$literal"))
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "escaped$module".to_string(),
                    path: "modules/escaped$literal".to_string(),
                },
                GradleSettingsModule {
                    name: "literal$module".to_string(),
                    path: "modules/$literal".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_manifest_discovery_rejects_project_root_escape_without_partial_modules() {
        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside/src/main/kotlin");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(error.contains("must remain beneath the project root"));
        let compatibility_result = discover_from_manifests(&project);
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_discovery_rejects_symlinked_module_directories() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
        symlink(&outside, project.join("linked")).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":linked\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(
            error.contains("must not traverse a symlink or reparse point"),
            "unexpected Gradle symlink rejection: {error}"
        );
    }

    #[test]
    fn gradle_single_project_uses_directory_name_not_package_tld() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("payments");
        fs::create_dir_all(root.join("src/main/kotlin/com/example/tool")).unwrap();
        fs::write(
            root.join("build.gradle.kts"),
            "plugins { kotlin(\"jvm\") }\n",
        )
        .unwrap();
        fs::write(
            root.join("src/main/kotlin/com/example/tool/Profile.kt"),
            "package com.example.tool\nclass Profile\n",
        )
        .unwrap();

        let result = parse_gradle(&root).unwrap();
        assert!(
            result.modules.contains_key("payments"),
            "single-project Gradle with no rootProject.name must use the directory name: {:?}",
            result.modules.keys().collect::<Vec<_>>()
        );
        assert!(
            !result.modules.contains_key("com"),
            "package TLD must not become a module: {:?}",
            result.modules.keys().collect::<Vec<_>>()
        );
        assert!(
            !result.modules.contains_key("src"),
            "shared src/ leaf must not become a module: {:?}",
            result.modules.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            result.modules["payments"].source_paths,
            vec!["src/main/kotlin"]
        );
    }

    #[test]
    fn gradle_single_project_uses_literal_root_project_name() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("checkout-dir");
        fs::create_dir_all(root.join("src/main/kotlin/com/example")).unwrap();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            "rootProject.name = \"checkout\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main/kotlin/com/example/A.kt"), "class A\n").unwrap();

        let result = parse_gradle(&root).unwrap();
        assert!(result.modules.contains_key("checkout"));
        assert!(!result.modules.contains_key("checkout-dir"));
        assert!(!result.modules.contains_key("com"));
        assert_eq!(result.modules.len(), 1);
    }

    #[test]
    fn gradle_multi_project_keeps_include_names_and_does_not_add_root() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("workspace");
        fs::create_dir_all(root.join("packages/profile/src/main/kotlin/com/example")).unwrap();
        fs::create_dir_all(root.join("packages/provider/src/main/kotlin/com/example")).unwrap();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            "rootProject.name = \"workspace\"\ninclude(\":profile\", \":provider\")\nproject(\":profile\").projectDir = file(\"packages/profile\")\nproject(\":provider\").projectDir = file(\"packages/provider\")\n",
        )
        .unwrap();
        fs::write(
            root.join("packages/profile/src/main/kotlin/com/example/Profile.kt"),
            "class Profile\n",
        )
        .unwrap();
        fs::write(
            root.join("packages/provider/src/main/kotlin/com/example/Provider.kt"),
            "class Provider\n",
        )
        .unwrap();

        let result = parse_gradle(&root).unwrap();
        assert!(result.modules.contains_key("profile"));
        assert!(result.modules.contains_key("provider"));
        assert!(!result.modules.contains_key("workspace"));
        assert!(!result.modules.contains_key("com"));
        assert!(!result.modules.contains_key("src"));
        assert_eq!(result.modules.len(), 2);
    }

    #[test]
    fn gradle_manifest_discovery_uses_the_effective_project_directory() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/custom/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle"),
            "include ':member'\nproject(':member').projectDir = file('vendor/custom')\n",
        )
        .unwrap();

        let result = parse_gradle(tmp.path()).unwrap();
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["vendor/custom/src/main/kotlin"]
        );
        assert!(
            result
                .source_dirs
                .contains(&"vendor/custom/src/main/kotlin".to_string())
        );
    }

    #[test]
    fn gradle_settings_ignore_comments_and_decode_escaped_values() {
        let commented = parse_gradle_settings("include ':member' // \"comment\n").unwrap();
        assert_eq!(commented[0].name, "member");

        let modules = parse_gradle_settings(
            r#"
include(
    ":member", // ignored quote: "
    ':quoted\'member',
    ":slash//member" /* ignored quote: ' */
)
project(":member").projectDir = file("modules\\member")
"#,
        )
        .unwrap();

        assert_eq!(modules[0].name, "member");
        assert_eq!(modules[0].path, "modules/member");
        assert_eq!(modules[1].name, "quoted'member");
        assert_eq!(modules[2].name, "slash//member");
    }

    #[test]
    fn gradle_manifest_discovery_fails_closed_for_malformed_settings() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\"\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Cannot parse Gradle settings manifest"));
        let result = discover_from_manifests(tmp.path());
        assert!(result.modules.is_empty());
        assert!(result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_manifest_discovery_accepts_comments_and_escaped_paths() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("modules/member/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
        )
        .unwrap();

        let result = discover_from_manifests(tmp.path());
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["modules/member/src/main/kotlin"]
        );
    }

    #[test]
    fn test_parse_go_mod() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("cmd")).unwrap();
        fs::create_dir_all(tmp.path().join("internal")).unwrap();
        fs::write(
            tmp.path().join("go.mod"),
            "module github.com/user/myproject\n\ngo 1.21\n",
        )
        .unwrap();

        let result = parse_go_mod(tmp.path()).unwrap();
        assert!(result.modules.contains_key("myproject"));
        assert!(result.source_dirs.contains(&"cmd".to_string()));
        assert!(result.source_dirs.contains(&"internal".to_string()));
    }

    #[test]
    fn test_extract_balanced_parens() {
        assert_eq!(
            extract_balanced_parens("name: \"Foo\", path: \"bar\")"),
            Some("name: \"Foo\", path: \"bar\"".to_string())
        );
        assert_eq!(
            extract_balanced_parens("a(b), c)"),
            Some("a(b), c".to_string())
        );
        assert_eq!(extract_balanced_parens("no close paren"), None);
    }
}
