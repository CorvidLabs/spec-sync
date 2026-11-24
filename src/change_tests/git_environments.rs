#[test]
fn partial_clone_recovery_accepts_only_complete_missing_object_ids() {
    let sha1 = "1234567890abcdef1234567890abcdef12345678";
    let sha256 = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
    let listing = [
        format!("{sha1} specs/change/requirements.md"),
        "?0123456789abcdef0123456789abcdef0123456".to_string(),
        format!("?{sha1} extra"),
        "?0123456789abcdef0123456789abcdef0123456z".to_string(),
        format!("?{sha1}"),
        format!("?{sha256}"),
    ]
    .join("\n");

    assert_eq!(
        missing_git_objects(listing.as_bytes()).collect::<Vec<_>>(),
        [sha1, sha256]
    );
}

#[test]
fn partial_clone_checks_the_subcommand_after_git_config_overrides() {
    let mut args = vec!["-c", "core.quotePath=false", "-c", "color.ui=never"];
    let options = args.len();
    args.extend(["cat-file", "-e", "HEAD:specs/change/requirements.md"]);
    assert!(reads_history_objects(&args));

    args.truncate(options);
    args.extend(["rev-parse", "--is-shallow-repository"]);
    assert!(!reads_history_objects(&args));
}

#[cfg(unix)]
#[test]
fn safe_directory_recovery_round_trips_raw_unix_bytes() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let temp = TempDir::new().unwrap();
    let config = temp.path().join("user-config");
    for bytes in [b"/tmp/user's ordinary repository".as_slice(), b"/tmp/user's repo \xff\n"] {
        let path = Path::new(OsStr::from_bytes(bytes));
        let command = safe_directory_recovery_command(path);
        let shell = Command::new("sh")
            .args(["-c", &command])
            .env("GIT_CONFIG_GLOBAL", &config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(shell.status.success(), "{}", String::from_utf8_lossy(&shell.stderr));
    }
    let stored = Command::new("git")
        .args(["config", "--global", "--null", "--get-all", "safe.directory"])
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(stored.status.success());
    assert_eq!(
        stored.stdout,
        b"/tmp/user's ordinary repository\0/tmp/user's repo \xff\n\0"
    );
}

#[test]
fn a_partial_clone_history_reader_waits_for_the_running_download() {
    use std::sync::mpsc::{RecvTimeoutError, channel};
    use std::time::Duration;

    for result in [Ok(()), Err("recover the missing objects with the user's Git".into())] {
        let pending = TempDir::new().unwrap();
        let other = TempDir::new().unwrap();
        let root = pending.path();
        let other_root = other.path();
        let git = git_in(root);
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("tracked.txt"), "locally readable\n").unwrap();
        git(&["add", "--", "tracked.txt"]);
        git(&["-c", "commit.gpgSign=false", "commit", "-qm", "baseline"]);
        let expected = result.clone().err();
        let (started, first_is_waiting) = channel();
        let (release, finish) = channel();
        let (attempt, attempted) = channel();
        let (observed, observation) = channel();
        let (independent, unaffected) = channel();
        let (early, second, first, unrelated) = std::thread::scope(|threads| {
            let first = threads.spawn(move || {
                ensure_partial_clone_objects_with(root, || {
                    started.send(()).unwrap();
                    finish.recv().unwrap();
                    result
                });
                partial_clone_download_refusal(root)
            });
            let timeout = Duration::from_secs(3);
            first_is_waiting.recv_timeout(timeout).unwrap();
            threads.spawn(move || {
                attempt.send(()).unwrap();
                ensure_partial_clone_objects_with(root, || panic!("one download per root"));
                observed.send(partial_clone_download_refusal(root)).unwrap();
            });
            threads.spawn(move || {
                ensure_partial_clone_objects_with(other_root, || Ok(()));
                independent.send(partial_clone_download_refusal(other_root)).unwrap();
            });
            attempted.recv_timeout(timeout).unwrap();
            let early = observation.recv_timeout(Duration::from_millis(150));
            let unrelated = unaffected.recv_timeout(timeout);
            release.send(()).unwrap();
            let second = if early.is_err() { observation.recv_timeout(timeout) } else { early.clone() };
            (early, second, first.join().unwrap(), unrelated)
        });
        assert_eq!(early, Err(RecvTimeoutError::Timeout));
        assert_eq!(second, Ok(expected.clone()));
        assert_eq!(first, expected);
        assert_eq!(unrelated, Ok(None));
        ensure_partial_clone_objects_with(root, || panic!("the terminal result must be cached"));
        let readable = run_git_required(root, &["show", "HEAD:tracked.txt"], None, 1_024).unwrap();
        assert_eq!(readable, b"locally readable\n");
    }
}

#[cfg(unix)]
#[test]
fn partial_clone_recovery_keeps_the_raw_root_in_both_fetch_commands() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;

    let scratch = TempDir::new().unwrap();
    let git = scratch.path().join("git");
    fs::write(&git, "#!/bin/sh\nprintf '%s\\0' \"$@\"\n").unwrap();
    fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).unwrap();
    let raw = b"/specsync nonexisting path's \xff\n";
    let root = Path::new(OsStr::from_bytes(raw));
    let oid = "1234567890abcdef1234567890abcdef12345678";
    let recovery = partial_clone_listing_refusal(root, "origin", &["HEAD".into()], "no tree");
    let refetch = recovery.split('`').nth(1).unwrap();
    let missing = partial_clone_fetch_command(root, "origin", &[oid], "--no-filter");

    for (command, expected_option) in [
        (&missing[..], b"--stdin".as_slice()),
        (refetch, b"--refetch".as_slice()),
    ] {
        let output = Command::new("/bin/sh")
            .args(["-c", command])
            .env("PATH", scratch.path())
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let args = output.stdout.split(|byte| *byte == 0).collect::<Vec<_>>();
        assert_eq!(args[..3], [b"-C".as_slice(), raw.as_slice(), b"fetch".as_slice()]);
        assert!(args.contains(&expected_option), "{args:?}");
    }
}

#[cfg(unix)]
#[test]
fn partial_clone_download_tolerates_non_utf8_historical_requirement_paths() {
    use std::io::Write;
    use std::process::Stdio;

    let scratch = TempDir::new().unwrap();
    let root = scratch.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let write_object = |args: &[&str], input: &[u8]| {
        let mut child = Command::new("git")
            .current_dir(root)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    let blob = write_object(&["hash-object", "-w", "--stdin"], b"# Requirement history\n");
    let file = format!("100644 blob {blob}\trequirements.md\0");
    let files = write_object(&["mktree", "-z"], file.as_bytes());
    let mut dirs = format!("040000 tree {files}\tspecs\0").into_bytes();
    dirs.extend_from_slice(format!("040000 tree {files}\tignored-").as_bytes());
    dirs.extend_from_slice(b"\xff\0");
    let tree = write_object(&["mktree", "-z"], &dirs);
    let commit = write_object(&["commit-tree", &tree], b"Requirements in two directories\n");
    git(&["update-ref", "refs/heads/main", &commit]);
    git(&["config", "remote.origin.promisor", "true"]);

    assert_eq!(download_partial_clone_objects(root), Ok(()));
}
