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
    assert_eq!(upstream.reference, "origin/main");
    assert_eq!(upstream.commit, archive_commit);
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
            upstream_archive_of(&root, &early).map(|found| found.commit),
            Some(archive_commit.clone()),
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
    assert_eq!(upstream.reference, "origin/main");
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
    let found = upstream_archive_of(&root, &unarchived);
    assert!(
        found.is_some(),
        "an unreadable upstream archive listing must not read as no upstream archive"
    );
    let message = upstream_archive_message(&unarchived.id, &found.unwrap());
    assert!(message.contains("cannot tell"), "{message}");
    assert!(message.contains("origin/main"), "{message}");
    assert!(message.contains("git fetch"), "{message}");
}
