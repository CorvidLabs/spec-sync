use super::*;
use crate::manifest::cargo::parse_cargo_toml_with_access;
use crate::manifest::discover_from_manifests_checked_with_root;
use crate::manifest::identity::{GradleFilesystemIdentity, gradle_filesystem_identity};
use crate::manifest::node::{
    parse_package_json_with_access_and_hook, parse_package_json_with_access_and_hooks,
};
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use std::cell::Cell;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
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
fn retained_root_manifest_discovery_accepts_matching_capability_and_rejects_mismatch() {
    let project = tempdir().unwrap();
    fs::create_dir_all(project.path().join("member/src/main/kotlin")).unwrap();
    fs::write(
        project.path().join("settings.gradle.kts"),
        "include(\":member\")\n",
    )
    .unwrap();
    let retained = Dir::open_ambient_dir(project.path(), ambient_authority()).unwrap();

    let discovery = discover_from_manifests_checked_with_root(project.path(), &retained).unwrap();
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

    let error = discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

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

    let error = discover_from_manifests_checked_with_root(project.path(), &retained).unwrap_err();

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
    let error = discover_from_manifests_checked_with_root(oversized.path(), &retained).unwrap_err();
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
        replace_directory_for_race_test(&workspace, &detached, &replacement, &replacement_refused);
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
        replace_directory_for_race_test(&workspace, &detached, &replacement, &replacement_refused);
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
