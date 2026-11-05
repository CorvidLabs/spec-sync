// Finalize stays cheap however many archives a project holds (REQ-change-121).
//
// Counted in Git processes, so the guard is hardware independent. Each archive may still cost a
// few live queries (its own committed state and a live `HEAD` resolution); what must not come back
// is a full workflow history walk per archive, which cost over a hundred processes each.

/// Git processes one more archived change may add to a finalize.
const FINALIZE_GIT_PROCESSES_PER_ARCHIVE: usize = 12;

/// Drives one no-spec change through approve, check and review, leaving it ready to finalize.
fn reviewed_change_ready_to_finalize(root: &Path) -> ChangeRecord {
    let record = current_workflow_record(root, completed_no_spec_record(root));
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    } else {
        quiet_git(root, &["checkout", "--", WORKFLOW_V2_BASELINE_PATH]);
    }
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    record
}

/// Git processes one finalize spawns once earlier archives have been seen by a previous command.
fn git_processes_for_one_warm_finalize(root: &Path) -> usize {
    let other = finalized_archived_change(root);
    git_processes_for_one_scoped_read(root, &other.id).0.unwrap();
    let record = reviewed_change_ready_to_finalize(root);
    reset_test_git_process_count();
    finalize_change(root, &record.id).unwrap();
    let processes = test_git_process_count();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive change"]);
    processes
}

#[test]
fn finalize_git_processes_do_not_grow_with_other_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    let with_few = git_processes_for_one_warm_finalize(root);
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    let with_more = git_processes_for_one_warm_finalize(root);
    eprintln!("finalize Git processes: {with_few} with few archives, {with_more} with four more");
    assert!(
        with_more <= with_few + 4 * FINALIZE_GIT_PROCESSES_PER_ARCHIVE,
        "four more archived changes added {} Git processes to one finalize \
         ({with_few} with few archives, {with_more} with four more)",
        with_more.saturating_sub(with_few)
    );
}
