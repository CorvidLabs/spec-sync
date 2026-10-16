use super::super::confinement::{MAX_CONFINEMENT_ENTRIES, MAX_PROJECT_FILE_BYTES};
use super::super::snapshot_manifest::collect_snapshot_manifest_inputs;
use super::super::test_support::{open_test_directory, setup_project};
use super::*;
use crate::manifest::MAX_GRADLE_MANIFEST_BYTES;
use std::fs;
use tempfile::TempDir;

#[test]
fn snapshot_root_source_dir_includes_otherwise_ignored_inputs() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("target")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["."]}"#,
    )
    .unwrap();
    fs::File::create(tmp.path().join("target/oversized.rs"))
        .unwrap()
        .set_len(MAX_PROJECT_FILE_BYTES + 1)
        .unwrap();

    let error = ProjectSnapshot::create(tmp.path())
        .err()
        .expect("explicit root inputs must not disappear from the snapshot");
    assert!(error.contains("per-file limit"));
}

#[cfg(unix)]
#[test]
fn snapshot_skips_ignored_and_configured_exclusion_symlinks_before_following() {
    use std::os::unix::fs::symlink;

    let source_temp = TempDir::new().unwrap();
    let destination_temp = TempDir::new().unwrap();
    let explicit_destination_temp = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("secret.rs"), "outside\n").unwrap();
    fs::create_dir_all(source_temp.path().join("kept")).unwrap();
    fs::write(source_temp.path().join("kept/module.rs"), "inside\n").unwrap();
    symlink(outside.path(), source_temp.path().join("target")).unwrap();
    symlink("kept", source_temp.path().join("generated")).unwrap();
    let source = open_test_directory(source_temp.path());
    let destination = open_test_directory(destination_temp.path());
    let mut budget = SnapshotBudget::default();

    copy_snapshot_directory(
        &source,
        &destination,
        Path::new("."),
        &HashSet::from(["generated".to_string()]),
        &[],
        &mut budget,
    )
    .expect("excluded symlink names must be skipped without following their targets");

    assert!(!destination_temp.path().join("target").exists());
    assert!(!destination_temp.path().join("generated").exists());

    let explicit_destination = open_test_directory(explicit_destination_temp.path());
    let mut explicit_budget = SnapshotBudget::default();
    let error = copy_snapshot_directory(
        &source,
        &explicit_destination,
        Path::new("."),
        &HashSet::from(["generated".to_string()]),
        &[PathBuf::from("generated")],
        &mut explicit_budget,
    )
    .expect_err("an explicit input must inspect and reject the excluded symlink");
    assert!(
        error.contains("regular file or directory"),
        "unexpected explicit symlink rejection: {error}"
    );
}

#[test]
fn snapshot_includes_manifest_derived_member_under_ignored_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
}

#[test]
fn snapshot_normalizes_confined_cargo_sibling_dependency() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/a\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/b/Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
}

#[test]
fn snapshot_normalizes_confined_windows_native_cargo_paths() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = ['crates\\a']\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/b/Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("crates/a/src/lib.rs").is_file());
    assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
}

#[test]
fn snapshot_ignores_nonsemantic_cargo_metadata_paths() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"metadata-path\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"vendor/lib.rs\"\n\n[package.metadata.example]\npath = \"/benign/absolute/metadata/value\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(
        tmp.path().join("vendor/lib.rs"),
        "pub fn custom_target() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path())
        .expect("non-semantic Cargo metadata paths must not affect snapshot discovery");

    assert!(snapshot.root().join("src/lib.rs").is_file());
    assert!(snapshot.root().join("vendor/lib.rs").is_file());
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_confined_sibling_dependency_symlink_escape() {
    use std::os::unix::fs::symlink;

    let tmp = setup_project();
    let outside = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
    fs::create_dir_all(outside.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/a\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
    )
    .unwrap();
    fs::write(
        outside.path().join("Cargo.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(outside.path().join("src/lib.rs"), "pub fn outside() {}\n").unwrap();
    symlink(outside.path(), tmp.path().join("crates/b")).unwrap();

    ProjectSnapshot::create(tmp.path())
        .err()
        .expect("a normalized sibling dependency must not follow a symlink outside root");
}

#[test]
fn snapshot_includes_multiline_cargo_members_beneath_ignored_directories() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"vendor/member\", # kept\n]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::File::create(tmp.path().join("vendor/member/src/lib.rs"))
        .unwrap()
        .set_len(MAX_PROJECT_FILE_BYTES + 1)
        .unwrap();

    let error = ProjectSnapshot::create(tmp.path())
        .err()
        .expect("a multiline workspace member must remain visible and bounded");

    assert!(error.contains("per-file limit"));
    let expected_path = PathBuf::from("vendor")
        .join("member")
        .join("src")
        .join("lib.rs")
        .display()
        .to_string();
    assert!(error.contains(&expected_path));
}

#[test]
fn snapshot_uses_toml_structure_when_comments_precede_workspace_header() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "# [workspace]\n# members = [\"ignored\"]\n\n[workspace]\nmembers = [\"vendor/member\"]\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("vendor/member/src/lib.rs"),
        "pub fn member() {}\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
}

#[test]
fn snapshot_rejects_malformed_cargo_toml_without_partial_discovery() {
    let tmp = setup_project();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/member\"\n",
    )
    .unwrap();

    let error = ProjectSnapshot::create(tmp.path())
        .err()
        .expect("malformed Cargo TOML must make snapshot discovery inconclusive");

    assert!(error.contains("Cannot parse MCP Cargo workspace manifest Cargo.toml as TOML"));
}

#[test]
fn manifest_discovery_charges_the_shared_cumulative_snapshot_budget() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("vendor/one")).unwrap();
    fs::create_dir_all(tmp.path().join("vendor/two")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"vendor/one\", \"vendor/two\"]\n",
    )
    .unwrap();
    fs::write(tmp.path().join("vendor/one/Cargo.toml"), vec![b' '; 300]).unwrap();
    fs::write(tmp.path().join("vendor/two/Cargo.toml"), vec![b' '; 300]).unwrap();
    let source = open_test_directory(tmp.path());
    let mut inputs = Vec::new();
    let mut budget = SnapshotBudget {
        max_bytes: 512,
        ..SnapshotBudget::default()
    };

    let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
        .expect_err("manifest parsing must not bypass the cumulative byte budget");

    assert!(error.contains("512 bytes cumulative limit"));
}

#[test]
fn snapshot_workspace_declarations_are_charged_before_deduplication() {
    let cargo = TempDir::new().unwrap();
    fs::create_dir_all(cargo.path().join("crates/member")).unwrap();
    fs::write(
        cargo.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
    )
    .unwrap();
    fs::write(
        cargo.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let cargo_source = open_test_directory(cargo.path());
    let mut cargo_inputs = Vec::new();
    let mut cargo_budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };

    collect_snapshot_manifest_inputs(&cargo_source, &mut cargo_inputs, &mut cargo_budget)
        .expect("two duplicate Cargo declarations must fit exactly at the traversal limit");

    assert_eq!(cargo_budget.entries, MAX_CONFINEMENT_ENTRIES);
    assert_eq!(
        cargo_inputs
            .iter()
            .filter(|path| path.as_path() == Path::new("crates/member"))
            .count(),
        1
    );

    let node = TempDir::new().unwrap();
    fs::write(
        node.path().join("package.json"),
        r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
    )
    .unwrap();
    let node_source = open_test_directory(node.path());
    let mut node_inputs = Vec::new();
    let mut node_budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };

    let error = collect_snapshot_manifest_inputs(&node_source, &mut node_inputs, &mut node_budget)
        .expect_err("a duplicate Node declaration beyond the traversal limit must be rejected");

    assert!(error.contains("exceeds 100000 entries"), "{error}");
    assert_eq!(node_budget.entries, MAX_CONFINEMENT_ENTRIES + 1);
    assert_eq!(
        node_inputs
            .iter()
            .filter(|path| path.as_path() == Path::new("packages"))
            .count(),
        0
    );
}

#[test]
fn snapshot_package_json_fails_closed_and_charges_entries_before_type_validation() {
    for (content, expected) in [
        ("{", "Cannot parse MCP package.json as JSON"),
        ("[]", "root must be a JSON object"),
        (
            r#"{"workspaces":7}"#,
            "`workspaces` must be an array or an object",
        ),
        (
            r#"{"workspaces":{"packages":7}}"#,
            "`workspaces.packages` must be an array",
        ),
        (
            r#"{"workspaces":{}}"#,
            "`workspaces` object must contain a `packages` array",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), content).unwrap();
        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("invalid package.json must make snapshot discovery inconclusive");
        assert!(error.contains(expected), "{content}: {error}");
    }

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":[7,"packages/*"]}"#,
    )
    .unwrap();
    let source = open_test_directory(tmp.path());
    let mut inputs = Vec::new();
    let mut budget = SnapshotBudget {
        entries: MAX_CONFINEMENT_ENTRIES - 2,
        ..SnapshotBudget::default()
    };
    let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
        .expect_err("wrong-typed workspace entries must fail after declaration charging");

    assert!(
        error.contains("workspace entries must be strings"),
        "{error}"
    );
    assert_eq!(budget.entries, MAX_CONFINEMENT_ENTRIES);
}

#[test]
fn snapshot_copies_the_exact_manifest_bytes_charged_during_discovery() {
    let source_temp = TempDir::new().unwrap();
    let destination_temp = TempDir::new().unwrap();
    let original = b"[workspace]\nmembers = []\n";
    fs::write(source_temp.path().join("Cargo.toml"), original).unwrap();
    let source = open_test_directory(source_temp.path());
    let destination = open_test_directory(destination_temp.path());
    let mut budget = SnapshotBudget {
        max_bytes: original.len() as u64,
        ..SnapshotBudget::default()
    };

    let loaded =
        read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget).unwrap();
    assert!(loaded.is_some());
    fs::write(
        source_temp.path().join("Cargo.toml"),
        vec![b'x'; MAX_PROJECT_FILE_BYTES as usize],
    )
    .unwrap();

    copy_preloaded_snapshot_files(&destination, &mut budget).unwrap();
    copy_snapshot_directory(
        &source,
        &destination,
        Path::new("."),
        &HashSet::new(),
        &[PathBuf::from("Cargo.toml")],
        &mut budget,
    )
    .unwrap();

    assert_eq!(
        fs::read(destination_temp.path().join("Cargo.toml")).unwrap(),
        original
    );
    assert_eq!(budget.bytes, original.len() as u64);
}

#[cfg(unix)]
#[test]
fn snapshot_manifest_reader_rejects_special_files_without_blocking() {
    use std::process::Command;
    use std::time::{Duration, Instant};

    let tmp = TempDir::new().unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(tmp.path().join("Cargo.toml"))
            .status()
            .unwrap()
            .success()
    );
    let source = open_test_directory(tmp.path());
    let mut budget = SnapshotBudget::default();
    let started = Instant::now();

    let error = read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget)
        .expect_err("special-file manifests must fail before a blocking open");

    assert!(error.contains("regular file"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[cfg(unix)]
#[test]
fn generic_snapshot_rejects_fifo_and_socket_sources_without_blocking() {
    use std::os::unix::net::UnixListener;
    use std::process::Command;
    use std::time::{Duration, Instant};

    let fifo = setup_project();
    assert!(
        Command::new("mkfifo")
            .arg(fifo.path().join("src/special.rs"))
            .status()
            .unwrap()
            .success()
    );
    let started = Instant::now();
    let fifo_error = ProjectSnapshot::create(fifo.path())
        .err()
        .expect("a configured FIFO source must fail the generic snapshot");
    assert!(
        fifo_error.contains("regular file or directory"),
        "{fifo_error}"
    );
    assert!(started.elapsed() < Duration::from_secs(2));

    let socket = setup_project();
    match UnixListener::bind(socket.path().join("src/special.rs")) {
        Ok(_listener) => {
            let socket_error = ProjectSnapshot::create(socket.path())
                .err()
                .expect("a configured socket source must fail the generic snapshot");
            assert!(
                socket_error.contains("regular file or directory"),
                "{socket_error}"
            );
        }
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
        Err(error) => panic!("cannot create generic snapshot socket fixture: {error}"),
    }
}

#[cfg(unix)]
#[test]
fn generic_snapshot_reader_rejects_fifo_symlink_and_regular_replacements() {
    use std::os::unix::fs::symlink;
    use std::process::Command;
    use std::time::{Duration, Instant};

    const ATTACKER_BYTES: &str = "GENERIC_SNAPSHOT_ATTACKER_BYTES";
    for replacement in ["fifo", "symlink", "regular"] {
        let tmp = setup_project();
        let root = tmp.path();
        let source_path = root.join("src/lib.rs");
        let attacker_path = root.join("attacker.rs");
        fs::write(&source_path, "pub fn retained() {}\n").unwrap();
        fs::write(&attacker_path, ATTACKER_BYTES).unwrap();
        let source = open_test_directory(root);
        let started = Instant::now();

        let result = read_snapshot_project_file_with_hook(&source, Path::new("src/lib.rs"), || {
            fs::rename(&source_path, root.join("src/original.rs")).unwrap();
            match replacement {
                "fifo" => {
                    assert!(
                        Command::new("mkfifo")
                            .arg(&source_path)
                            .status()
                            .unwrap()
                            .success()
                    );
                }
                "symlink" => symlink(&attacker_path, &source_path).unwrap(),
                "regular" => fs::rename(&attacker_path, &source_path).unwrap(),
                _ => unreachable!(),
            }
        });

        let error = result.expect_err("every post-open path replacement must fail closed");
        assert!(
            error.contains("changed during inspection"),
            "{replacement}: {error}"
        );
        assert!(!error.contains(ATTACKER_BYTES), "{replacement}: {error}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "{replacement} replacement blocked the retained reader"
        );
    }
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_directory_replacement_after_enumeration_before_recursion() {
    let source_temp = setup_project();
    let destination_temp = TempDir::new().unwrap();
    let source_root = source_temp.path();
    let original = source_root.join("src");
    let retained = source_root.join("retained-src");
    let destination = open_test_directory(destination_temp.path());
    let source = open_test_directory(source_root);
    let mut budget = SnapshotBudget::default();
    let mut replaced = false;

    let error = copy_snapshot_directory_with_hook(
        &source,
        &destination,
        Path::new("."),
        &HashSet::new(),
        &[PathBuf::from("src"), PathBuf::from("specs")],
        &mut budget,
        |relative| {
            if relative == Path::new(".") && !replaced {
                fs::rename(&original, &retained).unwrap();
                fs::create_dir(&original).unwrap();
                fs::write(original.join("attacker.rs"), "pub fn attacker() {}\n").unwrap();
                replaced = true;
            }
            Ok(())
        },
    )
    .expect_err("a post-enumeration directory replacement must fail closed");

    assert!(
        error.contains("changed during snapshot traversal"),
        "{error}"
    );
    assert!(error.contains("src"), "{error}");
    assert!(!destination_temp.path().join("src/attacker.rs").exists());
}

#[cfg(unix)]
#[test]
fn snapshot_preflight_rejects_special_and_linked_gradle_manifests_without_blocking() {
    use std::os::unix::fs::symlink;
    use std::os::unix::net::UnixListener;
    use std::process::Command;
    use std::time::{Duration, Instant};

    let fifo = TempDir::new().unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(fifo.path().join("build.gradle.kts"))
            .status()
            .unwrap()
            .success()
    );
    let started = Instant::now();
    let error = ProjectSnapshot::create(fifo.path())
        .err()
        .expect("a Gradle FIFO must fail before a blocking snapshot open");
    assert!(error.contains("regular file"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));

    let socket = TempDir::new().unwrap();
    match UnixListener::bind(socket.path().join("settings.gradle.kts")) {
        Ok(_listener) => {
            let error = ProjectSnapshot::create(socket.path())
                .err()
                .expect("a Gradle socket must fail snapshot preflight");
            assert!(error.contains("regular file"), "{error}");
        }
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
        Err(error) => panic!("cannot create Gradle socket fixture: {error}"),
    }

    for manifest in ["build.gradle.kts", "settings.gradle.kts"] {
        let linked = TempDir::new().unwrap();
        fs::write(linked.path().join("real.gradle"), "plugins {}\n").unwrap();
        symlink(
            linked.path().join("real.gradle"),
            linked.path().join(manifest),
        )
        .unwrap();
        let error = ProjectSnapshot::create(linked.path())
            .err()
            .expect("a linked Gradle manifest must fail snapshot preflight");
        assert!(error.contains("symlink or reparse point"), "{error}");
    }
}

#[test]
fn snapshot_preflight_applies_the_gradle_limit_before_discovery() {
    for manifest in [
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
    ] {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(manifest),
            vec![b' '; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
        )
        .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("an oversized Gradle manifest must fail snapshot preflight");
        assert!(error.contains("4 MiB"), "{manifest}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn gradle_manifest_path_replacement_cannot_block_or_change_preloaded_bytes() {
    use std::process::Command;
    use std::time::{Duration, Instant};

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    let source = open_test_directory(root);
    let mut budget = SnapshotBudget::default();
    let started = Instant::now();

    let result = read_capability_text_if_exists_with_limit_and_hook(
        &source,
        Path::new("build.gradle.kts"),
        &mut budget,
        MAX_GRADLE_MANIFEST_BYTES,
        || {
            fs::rename(
                root.join("build.gradle.kts"),
                root.join("original.gradle.kts"),
            )
            .unwrap();
            assert!(
                Command::new("mkfifo")
                    .arg(root.join("build.gradle.kts"))
                    .status()
                    .unwrap()
                    .success()
            );
        },
    );

    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn snapshot_manifest_reader_binds_the_retained_file_identity() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(
        root.join("replacement.toml"),
        "[workspace]\nmembers = [\"outside\"]\n",
    )
    .unwrap();
    let source = open_test_directory(root);
    let mut budget = SnapshotBudget::default();

    let result = read_capability_text_if_exists_with_hook(
        &source,
        Path::new("Cargo.toml"),
        &mut budget,
        || {
            fs::rename(root.join("Cargo.toml"), root.join("original.toml")).unwrap();
            fs::rename(root.join("replacement.toml"), root.join("Cargo.toml")).unwrap();
        },
    );

    assert!(
        result.is_err(),
        "a manifest path replaced after the retained open must be rejected"
    );
}

#[test]
fn selected_config_directory_identity_stays_bound_to_the_pre_open_observation() {
    assert!(snapshot_configuration_identities_match(
        &1_u64,
        &[Some(1), Some(1), Some(1)]
    ));
    assert!(!snapshot_configuration_identities_match(
        &1_u64,
        &[Some(2), Some(2), Some(2)]
    ));
    assert!(!snapshot_configuration_identities_match(
        &1_u64,
        &[Some(1), None, Some(1)]
    ));
}

#[test]
fn selected_config_parent_open_binds_the_pre_open_identity() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir(root.join("selected")).unwrap();
    fs::create_dir(root.join("replacement")).unwrap();
    let source = open_test_directory(root);

    let result = open_snapshot_configuration_directory_with_hook(
        &source,
        OsStr::new("selected"),
        Path::new("selected"),
        || {
            fs::rename(root.join("selected"), root.join("original")).unwrap();
            fs::rename(root.join("replacement"), root.join("selected")).unwrap();
        },
    );

    let error =
        result.expect_err("a replacement opened after metadata inspection must be rejected");
    assert!(error.contains("changed during inspection"), "{error}");
}

#[test]
fn selected_config_open_binds_the_retained_file_identity() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("other.json"),
        r#"{"specsDir":"other","sourceDirs":["other"]}"#,
    )
    .unwrap();
    let source = open_test_directory(root);

    let result = read_snapshot_configuration_with_hook(&source, Path::new("specsync.json"), || {
        fs::rename(root.join("specsync.json"), root.join("original.json")).unwrap();
        fs::rename(root.join("other.json"), root.join("specsync.json")).unwrap();
    });

    assert!(
        result.is_err(),
        "a regular selected config replaced after discovery must not be read"
    );
}

#[test]
fn snapshot_includes_python_package_derived_under_ignored_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("vendor")).unwrap();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("pyproject.toml"),
        "[project]\nname = \"vendor\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("vendor/module.py"), "def public(): pass\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    assert!(snapshot.root().join("vendor/module.py").is_file());
}

#[test]
fn snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle"),
        r#"
include ':vendor:groovy',
        ':nested:second'
include(
    ":kotlin:member",
    ':override'
)
project(':override').projectDir = file('vendor/custom')
"#,
    )
    .unwrap();
    for path in [
        "vendor/groovy/src/main/kotlin/one.kt",
        "nested/second/src/main/java/Two.java",
        "kotlin/member/src/main/kotlin/three.kt",
        "vendor/custom/src/main/kotlin/four.kt",
    ] {
        let path = tmp.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "public class Module {}\n").unwrap();
    }

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(
        snapshot
            .root()
            .join("vendor/groovy/src/main/kotlin/one.kt")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("nested/second/src/main/java/Two.java")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("kotlin/member/src/main/kotlin/three.kt")
            .is_file()
    );
    assert!(
        snapshot
            .root()
            .join("vendor/custom/src/main/kotlin/four.kt")
            .is_file()
    );
}

#[test]
fn test_snapshot_excludes_mixed_case_git_metadata() {
    let tmp = setup_project();
    fs::create_dir_all(tmp.path().join(".GIT")).unwrap();
    fs::write(tmp.path().join(".GIT/config"), "gitdir = outside\n").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

    assert!(!snapshot.root().join(".GIT").exists());
    assert!(!snapshot.root().join(".git").exists());
    assert!(!snapshot.git_freshness_available());
}
