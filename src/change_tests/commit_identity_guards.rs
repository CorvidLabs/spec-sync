// Verifies REQ-change-013 for commit identities read from change records: a value that is not a
// full commit id is never handed to Git as a revision.

fn commit_identity_repository() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let output = Command::new("git").args(args).current_dir(root).output().unwrap();
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "one\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "one"]);
    fs::write(root.join("README.md"), "two\n").unwrap();
    git(&["commit", "-q", "-am", "two"]);
    let head = git(&["rev-parse", "HEAD"]);
    (temp, head)
}

#[test]
fn a_recorded_commit_that_is_not_a_full_commit_id_is_never_handed_to_git() {
    let (temp, head) = commit_identity_repository();
    let root = temp.path();
    let short = &head[..12];
    for value in ["--not-a-commit", "-x", short, "HEAD", "main"] {
        assert!(!commit_is_ancestor_of_head(root, value), "{value}");
        let error = acceptance_entry_digest_at_commit(root, value, "predecessor", "src/a.ts")
            .expect_err("only a full commit id is checked out");
        assert!(error.contains("is not a full commit id"), "{value}: {error}");
    }
    assert!(
        !root.join(".git/worktrees").exists(),
        "no worktree is added for a value that is not a commit id"
    );
    assert!(commit_is_ancestor_of_head(root, &head));

    // The diff base falls back to the no-record base when the recorded one is not a commit id.
    let mut record = completed_no_spec_record(root);
    record.base_commit = Some("--not-a-commit".into());
    assert_eq!(recorded_diff_base(root, std::slice::from_ref(&record)), "HEAD~1...HEAD");
    record.base_commit = Some(head.clone());
    assert_eq!(recorded_diff_base(root, std::slice::from_ref(&record)), head);

    // Only the oldest change's base counts: a later change's valid base never stands in for it.
    let mut oldest = record.clone();
    oldest.base_commit = Some("--not-a-commit".into());
    let mut later = record.clone();
    later.id = "CHG-0002-later".into();
    later.base_commit = Some(head.clone());
    assert_eq!(recorded_diff_base(root, &[oldest, later]), "HEAD~1...HEAD");
}
