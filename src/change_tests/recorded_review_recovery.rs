fn recorded_review_recovery_git(root: &Path, args: &[&str]) {
    let output = Command::new("git").args(args).current_dir(root).output().unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

fn recorded_review_recovery_fixture(record_on_default: bool) -> (TempDir, ChangeRecord, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    recorded_review_recovery_git(root, &["config", "user.email", "test@example.com"]);
    recorded_review_recovery_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "base"]);
    recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "implementation"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Original reviewer".into()).unwrap();
    fs::write(root.join("src/lib.rs"), "// later verification\npub fn ready() -> bool { true }\n").unwrap();
    recorded_review_recovery_git(root, &["add", "src/lib.rs"]);
    recorded_review_recovery_git(root, &["commit", "-m", "later implementation"]);
    check_change(root, Some(&record.id)).unwrap();
    // Deliberately reproduce the historical bad writer: finalization fields bind the
    // later verification, while the review still describes the earlier workspace.
    let accepted = accept_change_with_gate(root, &record.id, None, None, "finalization", true, false)
        .unwrap();
    let verification = load_verification(root, &accepted).unwrap();
    assert_eq!(validate_finalization_evidence(root, &accepted, &verification).unwrap_err(),
        "finalization scoped review does not bind the accepted verification");
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "record historical inconsistent package"]);
    let anchor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    if record_on_default {
        recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    }
    (temp, accepted, anchor)
}

#[test]
fn recorded_review_recovery_preserves_preimage_and_requires_fresh_check_and_review() {
    let (temp, accepted, anchor) = recorded_review_recovery_fixture(true);
    let root = temp.path();
    let workspace = change_dir(root, &accepted.id);
    let subtree = portable_project_path(root, &workspace);
    let before = archive_workspace_snapshot(root, &workspace, &subtree).unwrap();
    let reopened = reopen_change(root, &accepted.id, "Recovery operator".into(),
        "Recover the recorded inconsistent review without accepting it".into()).unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(reopened.audit.recorded_review_mismatch_anchor.as_deref(), Some(anchor.as_str()));
    assert_eq!(git_tree_snapshot(root, &anchor, &subtree).unwrap(), before);
    assert!(finalize_change(root, &accepted.id).is_err(), "fresh verification is mandatory");
    assert!(check_change(root, Some(&accepted.id)).unwrap().unwrap().passed);
    let error = finalize_change(root, &accepted.id).unwrap_err();
    assert!(error.contains("review"), "fresh scoped review is mandatory: {error}");
    record_scoped_review(root, &accepted.id, "Fresh reviewer".into()).unwrap();
    let archive = finalize_change(root, &accepted.id).unwrap();
    assert!(archive.is_dir());
    assert_eq!(load_change(root, &accepted.id).unwrap().state, ChangeState::Archived);
}

#[test]
fn recorded_review_recovery_refuses_unanchored_or_modified_packages_without_writes() {
    for tamper in ["unanchored", "no-remote", "unrelated-default", "missing-file", "extra-file", "changed-mode"] {
        #[cfg(not(unix))]
        if tamper == "changed-mode" { continue; }
        let (temp, accepted, _) = recorded_review_recovery_fixture(tamper != "unanchored");
        let root = temp.path();
        let workspace = change_dir(root, &accepted.id);
        if tamper == "no-remote" {
            recorded_review_recovery_git(root, &["update-ref", "-d", "refs/remotes/origin/main"]);
        }
        if tamper == "unrelated-default" {
            let orphan = git_output(root, &["commit-tree", "HEAD^{tree}", "-m", "unrelated tree anchor"]).unwrap();
            recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", &orphan]);
        }
        if tamper == "missing-file" {
            fs::remove_file(workspace.join("context.md")).unwrap();
        }
        if tamper == "extra-file" {
            fs::write(workspace.join("uncommitted-note.md"), "not in remote-default history\n").unwrap();
        }
        #[cfg(unix)]
        if tamper == "changed-mode" {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(workspace.join("finalization.json"), fs::Permissions::from_mode(0o755))
                .unwrap();
        }
        let subtree = portable_project_path(root, &workspace);
        let before = archive_workspace_snapshot(root, &workspace, &subtree).unwrap();
        assert!(reopen_change(root, &accepted.id, "Recovery operator".into(),
            "The package must be exactly recorded".into()).is_err(), "{tamper}");
        assert_eq!(archive_workspace_snapshot(root, &workspace, &subtree).unwrap(), before, "{tamper}");
    }
}

#[test]
fn recorded_review_recovery_does_not_relax_other_finalization_digests() {
    let (temp, accepted, _) = recorded_review_recovery_fixture(true);
    let root = temp.path();
    let workspace = change_dir(root, &accepted.id);
    let mut finalization = load_finalization(root, &accepted).unwrap();
    finalization.finalization_digest = sha256_hex(b"tampered finalization");
    fs::write(workspace.join("finalization.json"), json_content(&finalization).unwrap()).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "record a differently corrupt package"]);
    recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let subtree = portable_project_path(root, &workspace);
    let before = archive_workspace_snapshot(root, &workspace, &subtree).unwrap();
    let error = reopen_change(root, &accepted.id, "Recovery operator".into(),
        "Other invalid evidence must remain refused".into()).unwrap_err();
    assert!(error.contains("finalization digest is invalid"), "{error}");
    assert_eq!(archive_workspace_snapshot(root, &workspace, &subtree).unwrap(), before);
}
