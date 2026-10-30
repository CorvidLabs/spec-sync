use super::super::preflight::{validate_cargo_workspace_manifest, validate_package_workspaces};
use super::super::publication::remove_identity_bound_file_with_hook;
use super::super::snapshot::read_snapshot_configuration_with_hook;
use super::super::test_support::{open_test_directory, quarantine_contains};
use super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn server_root_capability_rejects_a_root_replaced_before_canonicalization() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let moved = tmp.path().join("moved-server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let error = open_server_root_capability(&root, || {
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();
        Ok(())
    })
    .expect_err("a swapped root must be rejected");

    assert!(error.contains("root") || error.contains("capability"));
}

#[test]
fn windows_read_root_suffix_normalizes_drive_extended_unc_and_case() {
    assert_eq!(
        windows_relative_suffix_text(r"C:\Project", r"c:\PROJECT\Specs\Auth"),
        Some(PathBuf::from("Specs").join("Auth"))
    );
    assert_eq!(
        windows_relative_suffix_text(r"\\?\C:\Project", r"C:\Project\src"),
        Some(PathBuf::from("src"))
    );
    assert_eq!(
        windows_relative_suffix_text(
            r"\\server\share\Project",
            r"\\?\UNC\SERVER\SHARE\project\specs"
        ),
        Some(PathBuf::from("specs"))
    );
    assert_eq!(
        windows_relative_suffix_text(r"C:\Project", r"C:\ProjectElsewhere\src"),
        None
    );
    assert_eq!(
        windows_relative_suffix_text(r"\\server\share\Project", r"\\server\other\Project"),
        None
    );
    assert_eq!(
        windows_relative_suffix_text(r"C:\Projekt\Ärger", r"c:\PROJEKT\ärger\Specs"),
        Some(PathBuf::from("Specs"))
    );
}

#[test]
fn read_root_suffix_accepts_the_identity_bound_startup_alias_only_as_a_prefix() {
    #[cfg(windows)]
    let canonical_root = Path::new(r"C:\Users\runneradmin\AppData\Local\Temp\server");
    #[cfg(windows)]
    let requested_root = Path::new(r"C:\Users\RUNNER~1\AppData\Local\Temp\server");
    #[cfg(windows)]
    let child = r"C:\Users\RUNNER~1\AppData\Local\Temp\server\child";
    #[cfg(windows)]
    let sibling = r"C:\Users\RUNNER~1\AppData\Local\Temp\server-sibling\child";

    #[cfg(not(windows))]
    let canonical_root = Path::new("/resolved/temp/server");
    #[cfg(not(windows))]
    let requested_root = Path::new("/startup-alias/temp/server");
    #[cfg(not(windows))]
    let child = "/startup-alias/temp/server/child";
    #[cfg(not(windows))]
    let sibling = "/startup-alias/temp/server-sibling/child";

    assert_eq!(
        resolve_read_root(canonical_root, requested_root, Some(child)).unwrap(),
        PathBuf::from("child")
    );
    assert_eq!(
        resolve_read_root(canonical_root, requested_root, Some(sibling)).unwrap_err(),
        "Read root override escapes the configured server root"
    );
}

#[test]
fn read_root_rejects_git_metadata_components_case_insensitively() {
    #[cfg(not(windows))]
    let canonical_root = Path::new("/resolved/temp/server");
    #[cfg(not(windows))]
    let requested_root = Path::new("/startup-alias/temp/server");
    #[cfg(not(windows))]
    let absolute_git_root = "/startup-alias/temp/server/.gIt";

    #[cfg(windows)]
    let canonical_root = Path::new(r"C:\resolved\temp\server");
    #[cfg(windows)]
    let requested_root = Path::new(r"C:\startup-alias\temp\server");
    #[cfg(windows)]
    let absolute_git_root = r"C:\startup-alias\temp\server\.gIt";

    for candidate in [".git", ".GIT", "child/.GiT", absolute_git_root] {
        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some(candidate)).unwrap_err(),
            "Read root override must not select Git metadata"
        );
    }
    assert_eq!(
        resolve_read_root(canonical_root, requested_root, Some("child")).unwrap(),
        PathBuf::from("child")
    );
}

#[cfg(windows)]
#[test]
fn windows_read_root_accepts_runtime_absolute_variants() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("Child")).unwrap();
    let canonical = tmp.path().canonicalize().unwrap();
    let child = canonical.join("Child");
    let ordinary = child.to_string_lossy().replace(r"\\?\", "");
    let case_varied = ordinary.to_ascii_uppercase();

    assert_eq!(
        resolve_read_root(&canonical, &canonical, Some(&ordinary)).unwrap(),
        PathBuf::from("Child")
    );
    assert_eq!(
        resolve_read_root(&canonical, &canonical, Some(&case_varied)).unwrap(),
        PathBuf::from("CHILD")
    );

    let unicode_root = canonical.join("Ärger");
    fs::create_dir_all(unicode_root.join("Child")).unwrap();
    let unicode_candidate = unicode_root
        .join("Child")
        .to_string_lossy()
        .replace("Ärger", "ärger");
    assert_eq!(
        resolve_read_root(&unicode_root, &unicode_root, Some(&unicode_candidate)).unwrap(),
        PathBuf::from("Child")
    );
}

#[test]
fn selected_config_read_revalidates_each_parent_identity() {
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_SHARING_VIOLATION: i32 = 32;

    for replaced_parent in ["selected", "selected/nested"] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("selected/nested")).unwrap();
        fs::write(
            root.join("selected/nested/specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        let replacement = root.join("replacement");
        if replaced_parent == "selected" {
            fs::create_dir_all(replacement.join("nested")).unwrap();
            fs::write(
                replacement.join("nested/specsync.json"),
                r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
            )
            .unwrap();
        } else {
            fs::create_dir(&replacement).unwrap();
            fs::write(
                replacement.join("specsync.json"),
                r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
            )
            .unwrap();
        }
        let selected_parent = root.join(replaced_parent);
        let original_identity = directory_identity(&open_test_directory(&selected_parent)).unwrap();
        let source = open_test_directory(root);
        let replacement_denied = std::cell::Cell::new(false);

        let result = read_snapshot_configuration_with_hook(
            &source,
            Path::new("selected/nested/specsync.json"),
            || {
                let original_parent = if replaced_parent == "selected" {
                    root.join("original-selected")
                } else {
                    root.join("selected/original-nested")
                };
                match fs::rename(&selected_parent, &original_parent) {
                    Ok(()) => fs::rename(&replacement, &selected_parent).unwrap(),
                    Err(error)
                        if cfg!(windows)
                            && matches!(
                                error.raw_os_error(),
                                Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)
                            ) =>
                    {
                        replacement_denied.set(true);
                    }
                    Err(error) => panic!(
                        "cannot rename retained selected-config parent {replaced_parent}: {error}"
                    ),
                }
            },
        );

        if replacement_denied.get() {
            let bytes = result
                .expect("Windows sharing protection must preserve the retained parent identity")
                .expect("the retained configuration must remain present");
            assert_eq!(
                bytes,
                r#"{"specsDir":"specs","sourceDirs":["src"]}"#.as_bytes(),
                "{replaced_parent}: retained capability read replacement bytes"
            );
            let original_parent = if replaced_parent == "selected" {
                root.join("original-selected")
            } else {
                root.join("selected/original-nested")
            };
            assert!(selected_parent.is_dir());
            assert!(!original_parent.exists());
            assert!(replacement.is_dir());
            assert_eq!(
                directory_identity(&open_test_directory(&selected_parent)).unwrap(),
                original_identity,
                "{replaced_parent}: refused replacement changed the original identity"
            );
            drop(source);
            fs::rename(&selected_parent, &original_parent).expect(
                "the same rename must succeed after selected-config capabilities are released",
            );
            fs::rename(&original_parent, &selected_parent)
                .expect("the original selected-config parent must restore");
            assert_eq!(
                directory_identity(&open_test_directory(&selected_parent)).unwrap(),
                original_identity,
                "{replaced_parent}: negative-control rename changed the original identity"
            );
            assert!(replacement.is_dir());
            continue;
        }

        let error =
            result.expect_err("every selected-config parent replacement must fail after the read");
        assert!(
            error.contains("changed while the configuration was being read"),
            "{replaced_parent}: {error}"
        );
    }
}

#[test]
fn file_rollback_quarantines_a_replacement_swapped_after_verification() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
    let parent = open_test_directory(tmp.path());
    let expected = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

    let error = remove_identity_bound_file_with_hook(
        &parent,
        Path::new("generated.spec.md"),
        Path::new("generated.spec.md"),
        expected,
        |parent, name| {
            parent.remove_file(name).unwrap();
            parent.write(name, b"replacement\n").unwrap();
        },
    )
    .unwrap_err();

    assert!(error.contains("quarantined replacement was preserved"));
    assert!(!tmp.path().join("generated.spec.md").exists());
    assert!(quarantine_contains(&parent, b"replacement\n"));
}

#[test]
fn file_identity_rejects_changed_bytes_on_the_same_filesystem_entry() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
    let parent = open_test_directory(tmp.path());
    let original = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

    fs::write(tmp.path().join("generated.spec.md"), "replacement\n").unwrap();
    let replacement = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

    assert_ne!(original, replacement);
}

#[test]
fn file_identity_digest_fails_closed_at_the_generated_output_bound() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("oversized"), b"123456789").unwrap();
    let parent = open_test_directory(tmp.path());
    let file = parent.open("oversized").unwrap();

    let error = file_content_digest_with_limit(&file, 8).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("8-byte limit"));
}

#[test]
fn test_repeated_tree_scans_share_one_confinement_budget() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    let exclusions = HashSet::new();
    let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

    validate_configured_path_with_budget(
        tmp.path(),
        "src",
        "source_dirs entry",
        Some(&exclusions),
        &mut entries_seen,
    )
    .unwrap();
    let error = validate_configured_path_with_budget(
        tmp.path(),
        "src",
        "source_dirs entry",
        Some(&exclusions),
        &mut entries_seen,
    )
    .unwrap_err();

    assert!(error.contains("confinement scan exceeds"));
}

#[test]
fn cargo_preflight_charges_duplicate_members_without_replaying_manifests() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("crates/member")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0;
    let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

    validate_cargo_workspace_manifest(
        tmp.path(),
        tmp.path(),
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )
    .expect("duplicate members must fit exactly at the declaration limit");

    assert_eq!(traversal_entries_seen, MAX_CONFINEMENT_ENTRIES);
    assert_eq!(manifests_seen, 2);

    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
    )
    .unwrap();
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0;
    let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
    let error = validate_cargo_workspace_manifest(
        tmp.path(),
        tmp.path(),
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )
    .expect_err("the limit-plus-one duplicate declaration must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
    assert_eq!(manifests_seen, 2);
}

#[test]
fn node_preflight_deduplicates_patterns_bases_and_workspace_paths() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("packages/member")).unwrap();
    fs::write(
        tmp.path().join("packages/member/package.json"),
        r#"{"name":"member"}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    let mut exact_entries_seen = MAX_CONFINEMENT_ENTRIES - 4;

    validate_package_workspaces(tmp.path(), &mut exact_entries_seen)
        .expect("three declarations plus one unique workspace must fit exactly");

    assert_eq!(exact_entries_seen, MAX_CONFINEMENT_ENTRIES);

    let mut overflow_entries_seen = MAX_CONFINEMENT_ENTRIES - 3;
    let error = validate_package_workspaces(tmp.path(), &mut overflow_entries_seen)
        .expect_err("the limit-plus-one Node declaration must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
}

#[test]
fn node_preflight_fails_closed_and_charges_entries_before_type_validation() {
    for (content, expected) in [
        ("{", "Cannot parse MCP package.json as JSON"),
        ("false", "root must be a JSON object"),
        (
            r#"{"workspaces":"packages/*"}"#,
            "`workspaces` must be an array or an object",
        ),
        (
            r#"{"workspaces":{"packages":{}}}"#,
            "`workspaces.packages` must be an array",
        ),
        (
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), content).unwrap();
        let mut entries_seen = 0;
        let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
            .expect_err("invalid package.json must make MCP preflight inconclusive");
        assert!(error.contains(expected), "{content}: {error}");
    }

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":[false,"packages/*"]}"#,
    )
    .unwrap();
    let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
    let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
        .expect_err("wrong-typed workspace entries must fail after declaration charging");

    assert!(
        error.contains("workspace entries must be strings"),
        "{error}"
    );
    assert_eq!(entries_seen, MAX_CONFINEMENT_ENTRIES);
}
