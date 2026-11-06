// REQ-change-127 (#694). `upstream_archive_of` names an archive the remote default holds and
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
    assert!(message.contains("merge or rebase onto `origin/main`"), "{message}");
}
