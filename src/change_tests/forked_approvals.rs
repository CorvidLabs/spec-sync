// #532: two clones approve the same change, each commits its approval, and the branches merge.
// History then holds two approvals ledgers that neither extends. Definition-approval validation
// refuses that, naming both approvers, until one fresh approval is recorded on the merged branch.

/// A change approved separately on two branches, each approval committed, merged with the first
/// branch's ledger kept (`-s ours`), as a hand resolution that drops one side would.
fn change_approved_on_two_branches() -> (TempDir, ChangeRecord) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "draft the change"]);
    quiet_git(root, &["checkout", "-q", "-b", "clone-b"]);
    approve_definition(root, &record.id, Some("clone-b".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "clone B approves"]);
    quiet_git(root, &["checkout", "-q", "main"]);
    approve_definition(root, &record.id, Some("clone-a".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "clone A approves"]);
    quiet_git(root, &["merge", "-q", "-s", "ours", "clone-b", "-m", "merge clone B"]);
    (temp, record)
}

#[test]
fn approvals_forked_across_clones_are_refused_until_one_fresh_approval() {
    let (temp, record) = change_approved_on_two_branches();
    let root = temp.path();
    let record = load_change(root, &record.id).unwrap();

    let refused = ensure_definition_approval_valid(root, &record).unwrap_err();
    assert!(
        refused.contains("clone-a") && refused.contains("clone-b"),
        "the refusal names both approvers: {refused}"
    );
    assert!(
        refused.contains(&format!("specsync change approve {}", record.id)),
        "the refusal names the one fresh approval that settles it: {refused}"
    );
    let summary = summarize_change(root, &record);
    assert!(!summary.approval_valid, "status must not report the forked approval valid");

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let record = load_change(root, &record.id).unwrap();
    ensure_definition_approval_valid(root, &record).expect("one fresh approval settles the fork");
}

#[test]
fn approvals_that_only_append_are_not_a_fork() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve"]);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve again"]);
    let record = load_change(root, &record.id).unwrap();
    ensure_definition_approval_valid(root, &record).expect("an appended approval is no fork");
}
