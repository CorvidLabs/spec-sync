use super::*;
use crate::manifest::MAX_GRADLE_MANIFEST_BYTES;
use crate::mcp::MAX_PROJECT_FILE_BYTES;
use crate::mcp::tests::{open_test_directory, setup_project};
use std::{fs, io};
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
