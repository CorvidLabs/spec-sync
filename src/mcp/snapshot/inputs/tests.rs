use super::super::copy_preloaded_snapshot_files;
use super::super::traversal::copy_snapshot_directory;
use super::*;
use crate::manifest::MAX_GRADLE_MANIFEST_BYTES;
use crate::mcp::capabilities::directory_identity;
use crate::mcp::tests::{open_test_directory, setup_project};
use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;

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
