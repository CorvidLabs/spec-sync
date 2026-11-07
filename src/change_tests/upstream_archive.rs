// REQ-change-129 (#694). `upstream_archive_of` names an archive the remote default holds and
// HEAD does not, and stays silent for an archive HEAD already contains: that is the
// reopen-and-rearchive path, which must keep working.
#[test]
fn upstream_archive_is_reported_only_when_head_does_not_contain_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    quiet_git(&root, &["init", "-b", "main"]);
    quiet_git(&root, &["config", "user.email", "test@example.com"]);
    quiet_git(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(&root, &["add", "seed.txt"]);
    quiet_git(&root, &["commit", "-m", "seed"]);
    let base = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let archived = archived_workflow_record(&root);
    let archive_commit = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();

    let origin = temp.path().join("origin.git");
    quiet_git(
        temp.path(),
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );
    quiet_git(&root, &["remote", "add", "origin", origin.to_str().unwrap()]);
    quiet_git(&root, &["push", "origin", "HEAD:refs/heads/main"]);
    quiet_git(&root, &["fetch", "origin"]);
    quiet_git(&root, &["remote", "set-head", "origin", "main"]);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;

    // HEAD contains the archive: the reopen-and-rearchive shape. Nothing is reported.
    assert_eq!(upstream_archive_of(&root, &unarchived), None);
    // An archived record is never reported either.
    assert_eq!(upstream_archive_of(&root, &archived), None);

    // A branch from before the archive: the remote default holds it and HEAD does not.
    quiet_git(&root, &["checkout", "-q", "-b", "stale", &base]);
    let upstream = upstream_archive_of(&root, &unarchived)
        .expect("an archive only the remote default holds must be reported");
    assert_eq!(
        upstream,
        UpstreamArchive::Archived {
            reference: "origin/main".into(),
            commit: archive_commit.clone(),
        }
    );
    let message = upstream_archive_message(&unarchived.id, &upstream);
    assert!(message.contains("already archived on `origin/main`"), "{message}");
    assert!(message.contains(&archive_commit), "{message}");
    assert!(message.contains("rebase this branch onto `origin/main`"), "{message}");
    assert!(
        message.contains(&format!("`.specsync/changes/{}/`", unarchived.id)),
        "{message}"
    );

    // Every unarchived state is checked (review thread on #889): a stale copy is named while it
    // is still a draft, before `check --commit` adds another lifecycle commit.
    for state in [ChangeState::Draft, ChangeState::Approved, ChangeState::Implementing] {
        let mut early = unarchived.clone();
        early.state = state;
        assert_eq!(
            upstream_archive_of(&root, &early),
            Some(UpstreamArchive::Archived {
                reference: "origin/main".into(),
                commit: archive_commit.clone(),
            }),
            "{state:?}"
        );
    }
    let mut accepted = unarchived.clone();
    accepted.state = ChangeState::Accepted;
    assert!(upstream_archive_of(&root, &accepted).is_some());
}

// #907. An upstream archive is identified by the id in its `state.json`, as `find_change_dir`
// does, not by a `YYYY-MM-DD-<id>` directory name: a renamed or undated package still counts.
#[test]
fn upstream_archive_is_found_by_its_state_record_not_its_directory_name() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    quiet_git(&root, &["init", "-b", "main"]);
    quiet_git(&root, &["config", "user.email", "test@example.com"]);
    quiet_git(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(&root, &["add", "seed.txt"]);
    quiet_git(&root, &["commit", "-m", "seed"]);
    let base = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let archived = archived_workflow_record(&root);
    let dated = format!("{ARCHIVE_PATH}/2026-07-14-{}", archived.id);
    let renamed = format!("{ARCHIVE_PATH}/legacy-package");
    quiet_git(&root, &["mv", &dated, &renamed]);
    quiet_git(&root, &["commit", "-m", "rename the archived package"]);

    let origin = temp.path().join("origin.git");
    quiet_git(
        temp.path(),
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );
    quiet_git(&root, &["remote", "add", "origin", origin.to_str().unwrap()]);
    quiet_git(&root, &["push", "origin", "HEAD:refs/heads/main"]);
    quiet_git(&root, &["fetch", "origin"]);
    quiet_git(&root, &["remote", "set-head", "origin", "main"]);
    quiet_git(&root, &["checkout", "-q", "-b", "stale", &base]);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let upstream = upstream_archive_of(&root, &unarchived)
        .expect("a renamed upstream package must still be found by its state record");
    assert!(
        matches!(&upstream, UpstreamArchive::Archived { reference, .. } if reference == "origin/main"),
        "{upstream:?}"
    );
}

// #946, SHIP-3.c. When the remote default's archived state records cannot be read, the lookup
// cannot tell whether this change was already archived there. That must not read as "no
// upstream archive": it is reported, and its message says how to find out.
#[test]
fn an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    quiet_git(&root, &["init", "-b", "main"]);
    quiet_git(&root, &["config", "user.email", "test@example.com"]);
    quiet_git(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(&root, &["add", "seed.txt"]);
    quiet_git(&root, &["commit", "-m", "seed"]);
    let base = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let archived = archived_workflow_record(&root);
    let state_path = format!("{ARCHIVE_PATH}/2026-07-14-{}/state.json", archived.id);

    let origin = temp.path().join("origin.git");
    quiet_git(
        temp.path(),
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );
    quiet_git(&root, &["remote", "add", "origin", origin.to_str().unwrap()]);
    quiet_git(&root, &["push", "origin", "HEAD:refs/heads/main"]);
    quiet_git(&root, &["fetch", "origin"]);
    quiet_git(&root, &["remote", "set-head", "origin", "main"]);
    quiet_git(&root, &["checkout", "-q", "-b", "stale", &base]);

    // The batched read of the archived records fails: their blob is gone from this clone.
    let blob = quiet_git(&root, &["rev-parse", &format!("origin/main:{state_path}")])
        .trim()
        .to_string();
    fs::remove_file(root.join(".git/objects").join(&blob[..2]).join(&blob[2..])).unwrap();

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let found = upstream_archive_of(&root, &unarchived)
        .expect("an unreadable upstream archive listing must not read as no upstream archive");
    assert_eq!(
        found,
        UpstreamArchive::CannotTell {
            reference: "origin/main".into(),
            project_root: root.to_path_buf(),
        }
    );
    let message = upstream_archive_message(&unarchived.id, &found);
    assert!(message.contains("cannot tell"), "{message}");
    assert!(message.contains("origin/main"), "{message}");
    assert!(message.contains("git fetch"), "{message}");
}

// #962. A project in a repository subdirectory whose name starts with pathspec magic. Passed to
// `git ls-tree` as a plain pathspec, `:!app/.specsync/archive/` reads as an exclude, the listing
// fails, and the upstream archive is missed. Read literally from the top, it is found. The
// project is built as `app` and renamed on both branches, so only the lookup meets the name.
#[test]
fn upstream_archive_is_found_in_a_project_directory_named_like_pathspec_magic() {
    let (temp, app) = subdirectory_project_named("app");
    let repository = temp.path();
    let base = quiet_git(repository, &["rev-parse", "HEAD"]).trim().to_string();
    let archived = archived_workflow_record(&app);
    let rename_to_magic = |message: &str| {
        // Ignored files the checkout left behind under the old name go first.
        let _ = fs::remove_dir_all(repository.join(":!app"));
        fs::rename(repository.join("app"), repository.join(":!app")).unwrap();
        quiet_git(repository, &["add", "--all"]);
        quiet_git(repository, &["commit", "-q", "-m", message]);
        quiet_git(repository, &["rev-parse", "HEAD"]).trim().to_string()
    };
    let archive_commit = rename_to_magic("move the project under a pathspec-magic name");

    let remote = TempDir::new().unwrap();
    let origin = remote.path().join("origin.git");
    quiet_git(
        remote.path(),
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );
    quiet_git(repository, &["remote", "add", "origin", origin.to_str().unwrap()]);
    quiet_git(repository, &["push", "origin", "HEAD:refs/heads/main"]);
    quiet_git(repository, &["fetch", "origin"]);
    quiet_git(repository, &["remote", "set-head", "origin", "main"]);
    quiet_git(repository, &["checkout", "-q", "-b", "stale", &base]);
    rename_to_magic("the stale branch moves the project too");
    let root = repository.join(":!app");

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let upstream = upstream_archive_of(&root, &unarchived)
        .expect("an archive under a pathspec-magic directory name must still be found");
    assert_eq!(
        upstream,
        UpstreamArchive::Archived {
            reference: "origin/main".into(),
            commit: archive_commit,
        }
    );
}

/// Publishes `root`'s history as `origin/main` and checks out `stale` from `base`, the shape every
/// upstream-archive test shares.
fn publish_and_branch_from(temp: &TempDir, root: &Path, base: &str) {
    let origin = temp.path().join("origin.git");
    quiet_git(
        temp.path(),
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );
    quiet_git(root, &["remote", "add", "origin", origin.to_str().unwrap()]);
    quiet_git(root, &["push", "origin", "HEAD:refs/heads/main"]);
    quiet_git(root, &["fetch", "origin"]);
    quiet_git(root, &["remote", "set-head", "origin", "main"]);
    quiet_git(root, &["checkout", "-q", "-b", "stale", base]);
}

/// A repository at `temp/work` with one seed commit; returns the root and that commit.
fn seeded_repository(temp: &TempDir) -> (PathBuf, String) {
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    quiet_git(&root, &["init", "-b", "main"]);
    quiet_git(&root, &["config", "user.email", "test@example.com"]);
    quiet_git(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(&root, &["add", "seed.txt"]);
    quiet_git(&root, &["commit", "-m", "seed"]);
    let base = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    (root, base)
}

/// The shell commands a message offers: every backtick span that starts with `git `.
fn offered_git_commands(message: &str) -> Vec<String> {
    message
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| span.starts_with("git "))
        .map(str::to_string)
        .collect()
}

fn cannot_tell_message(root: &Path, id: &str, upstream: &UpstreamArchive) -> String {
    let message = upstream_archive_message(id, upstream);
    assert!(
        message.contains(&format!(
            "then `specsync --root {} change status -- {}`",
            shell_quote(&root.to_string_lossy()),
            shell_quote(id)
        )),
        "{message}"
    );
    message
}

fn run_printed_git_recovery(message: &str, current_dir: &Path) {
    let commands = offered_git_commands(message);
    assert_eq!(
        commands,
        ["git fetch --refetch origin", "git remote set-head origin --auto"]
    );
    for command in commands {
        let output = std::process::Command::new("sh")
            .args(["-c", &command])
            .current_dir(current_dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "`{command}` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn remove_loose_object(root: &Path, object: &str) {
    let git_dir = quiet_git(root, &["rev-parse", "--absolute-git-dir"]);
    fs::remove_file(
        Path::new(git_dir.trim())
            .join("objects")
            .join(&object[..2])
            .join(&object[2..]),
    )
    .unwrap();
}

// Codex P1 on #967. An archived `state.json` that Git returns whole but that does not decode as a
// change record could be this change's archive: the lookup cannot tell, so it must not read as no
// upstream archive.
#[test]
fn an_undecodable_upstream_state_record_is_reported_as_cannot_tell() {
    let temp = TempDir::new().unwrap();
    let (root, base) = seeded_repository(&temp);
    let archived = archived_workflow_record(&root);
    // The remote default's only archive of this change no longer decodes, so nothing can say
    // whose archive it is.
    let state_path = root
        .join(ARCHIVE_PATH)
        .join(format!("2026-07-14-{}", archived.id))
        .join("state.json");
    fs::write(&state_path, "{ not a change record\n").unwrap();
    quiet_git(&root, &["add", "--all"]);
    quiet_git(&root, &["commit", "-m", "an undecodable archived record"]);
    publish_and_branch_from(&temp, &root, &base);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    assert_eq!(
        upstream_archive_of(&root, &unarchived),
        Some(UpstreamArchive::CannotTell {
            reference: "origin/main".into(),
            project_root: root.to_path_buf(),
        })
    );
}

// Codex round 2 on #967. The recovery repairs the clone, then reruns the lookup rather than
// guessing whether a grep hit is a stale copy or the supported reopen-and-rearchive path.
#[test]
fn the_cannot_tell_recovery_repairs_the_clone_and_reruns_the_lookup() {
    let temp = TempDir::new().unwrap();
    let (root, base) = seeded_repository(&temp);
    let archived = archived_workflow_record(&root);
    let archive_commit = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let state_path = format!("{ARCHIVE_PATH}/2026-07-14-{}/state.json", archived.id);
    publish_and_branch_from(&temp, &root, &base);
    let blob = quiet_git(&root, &["rev-parse", &format!("origin/main:{state_path}")])
        .trim()
        .to_string();
    remove_loose_object(&root, &blob);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let found = upstream_archive_of(&root, &unarchived).expect("cannot tell");
    let message = cannot_tell_message(&root, &unarchived.id, &found);
    run_printed_git_recovery(&message, &root);
    assert_eq!(
        upstream_archive_of(&root, &unarchived),
        Some(UpstreamArchive::Archived {
            reference: "origin/main".into(),
            commit: archive_commit,
        })
    );
}

// The same printed recovery works from the repository root when SpecSync's project root is its
// `app/` subdirectory. The old grep searched the shell's directory and missed the app's archive.
#[test]
fn the_cannot_tell_recovery_supports_a_project_root_below_the_shell_directory() {
    let (temp, root) = subdirectory_project_named("app");
    let base = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let archived = archived_workflow_record(&root);
    let archive_commit = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let state_path = format!("app/{ARCHIVE_PATH}/2026-07-14-{}/state.json", archived.id);
    publish_and_branch_from(&temp, &root, &base);
    let blob = quiet_git(&root, &["rev-parse", &format!("origin/main:{state_path}")])
        .trim()
        .to_string();
    remove_loose_object(&root, &blob);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let found = upstream_archive_of(&root, &unarchived).expect("cannot tell");
    let message = cannot_tell_message(&root, &unarchived.id, &found);
    run_printed_git_recovery(&message, temp.path());
    assert_eq!(
        upstream_archive_of(&root, &unarchived),
        Some(UpstreamArchive::Archived {
            reference: "origin/main".into(),
            commit: archive_commit,
        })
    );
}

// A fetch does not repair `origin/HEAD` after the remote default is renamed and the old tracking
// branch is pruned. `git remote set-head origin --auto` does, before the lookup runs again.
#[test]
fn the_cannot_tell_recovery_repairs_a_dangling_remote_default() {
    let temp = TempDir::new().unwrap();
    let (root, base) = seeded_repository(&temp);
    let archived = archived_workflow_record(&root);
    let archive_commit = quiet_git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let state_path = format!("{ARCHIVE_PATH}/2026-07-14-{}/state.json", archived.id);
    publish_and_branch_from(&temp, &root, &base);
    let origin = temp.path().join("origin.git");
    quiet_git(&origin, &["branch", "-m", "main", "trunk"]);
    quiet_git(&root, &["fetch", "--prune", "origin"]);
    quiet_git(
        &root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let blob = quiet_git(&root, &["rev-parse", &format!("origin/trunk:{state_path}")])
        .trim()
        .to_string();
    remove_loose_object(&root, &blob);

    let mut unarchived = archived.clone();
    unarchived.state = ChangeState::Verifying;
    let found = upstream_archive_of(&root, &unarchived).expect("cannot tell");
    assert_eq!(
        found,
        UpstreamArchive::CannotTell {
            reference: "origin/main".into(),
            project_root: root.to_path_buf(),
        }
    );
    let message = cannot_tell_message(&root, &unarchived.id, &found);
    run_printed_git_recovery(&message, &root);
    assert_eq!(
        upstream_archive_of(&root, &unarchived),
        Some(UpstreamArchive::Archived {
            reference: "origin/trunk".into(),
            commit: archive_commit,
        })
    );
}

// The id is data in the status command, even when it contains shell metacharacters.
#[test]
fn the_cannot_tell_recovery_quotes_the_change_id_for_the_status_rerun() {
    let id = "add-auth$(touch pwned)'\"y";
    let root = Path::new("/project with 'quotes'");
    let message = cannot_tell_message(
        root,
        id,
        &UpstreamArchive::CannotTell {
            reference: "origin/main".into(),
            project_root: root.to_path_buf(),
        },
    );
    assert!(message.contains(&format!("-- {}", shell_quote(id))), "{message}");
    let command = message
        .split('`')
        .find_map(|span| span.strip_prefix("specsync "))
        .expect("printed status arguments");
    let temp = TempDir::new().unwrap();
    let output = std::process::Command::new("sh")
        .args(["-c", &format!("set -- {command}; printf '%s\\0' \"$@\"")])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let arguments: Vec<_> = output.stdout.split(|byte| *byte == 0).collect();
    assert_eq!(
        arguments,
        [b"--root".as_slice(), root.as_os_str().as_encoded_bytes(), b"change", b"status", b"--", id.as_bytes(), b""]
    );
    assert!(!temp.path().join("pwned").exists());
}
