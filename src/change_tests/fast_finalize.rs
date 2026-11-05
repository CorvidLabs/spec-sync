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

// REQ-change-124: finalize validates the history of the change it archives and reads every other
// archive from its own bytes, so other archives cost it no workflow-version history walk.

/// Git processes one other archive may add to a finalize whose persistent read cache is cold: the
/// three reads of the immutable commit that introduced it (its `diff-tree`, its `state.json` and
/// its `approvals.json`) for the archive-introduction index the closing check consults. The
/// persistent cache keeps them after the first command (REQ-change-119), so a warm finalize pays
/// nothing per archive.
const COLD_GIT_PROCESSES_PER_ARCHIVE_INTRODUCTION: usize = 3;

/// Git processes more archived changes may add, in total, to one finalize whose cache is warm.
const WARM_FINALIZE_GIT_PROCESS_SLACK: usize = 2;

/// Git processes one finalize spawns. `warm_with` names an archive whose status is read first, the
/// way an operator's earlier command warms the persistent read cache; `None` deletes the cache so
/// nothing remembered can answer for another archive.
fn git_processes_for_one_finalize(root: &Path, warm_with: Option<&str>) -> usize {
    let record = reviewed_change_ready_to_finalize(root);
    match warm_with {
        Some(id) => {
            git_processes_for_one_scoped_read(root, id).0.unwrap();
        }
        None => {
            if let Some(path) = persistent_read_cache_path(root) {
                let _ = fs::remove_file(path);
            }
        }
    }
    reset_test_git_process_count();
    finalize_change(root, &record.id).unwrap();
    let processes = test_git_process_count();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive change"]);
    processes
}

#[test]
fn cold_finalize_git_processes_do_not_grow_with_other_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    let with_few = git_processes_for_one_finalize(root, None);
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    // Five more archives since the first measurement: the four above and the change it archived.
    let added = 5;
    let with_more = git_processes_for_one_finalize(root, None);
    eprintln!("cold finalize Git processes: {with_few} with few archives, {with_more} with {added} more");
    assert!(
        with_more <= with_few + added * COLD_GIT_PROCESSES_PER_ARCHIVE_INTRODUCTION,
        "{added} more archived changes added {} Git processes to one cold finalize \
         ({with_few} with few archives, {with_more} with {added} more)",
        with_more.saturating_sub(with_few)
    );
}

#[test]
fn warm_finalize_git_processes_do_not_grow_with_other_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let other = finalized_archived_change(root);
    let with_few = git_processes_for_one_finalize(root, Some(&other.id));
    for _ in 0..4 {
        finalized_archived_change(root);
    }
    let with_more = git_processes_for_one_finalize(root, Some(&other.id));
    eprintln!("warm finalize Git processes: {with_few} with few archives, {with_more} with five more");
    assert!(
        with_more <= with_few + WARM_FINALIZE_GIT_PROCESS_SLACK,
        "five more archived changes added {} Git processes to one warm finalize \
         ({with_few} with few archives, {with_more} with five more)",
        with_more.saturating_sub(with_few)
    );
}

#[test]
fn finalize_judges_other_archives_from_their_own_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let other = finalized_archived_change(root);
    let other_state = find_change_dir(root, &other.id).unwrap().join("state.json");
    commit_hidden_workflow_downgrade(root, &other_state);
    let record = reviewed_change_ready_to_finalize(root);
    // The other archive's history is `change audit --strict`'s to re-walk (REQ-change-124); the
    // change being archived is still judged in full.
    finalize_change(root, &record.id).unwrap();
    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Archived);
}
