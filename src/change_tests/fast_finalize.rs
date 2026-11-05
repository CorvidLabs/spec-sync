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

#[test]
fn strict_audit_still_refuses_a_hidden_downgrade_in_another_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let other = finalized_archived_change(root);
    finalized_archived_change(root);
    // A passing strict audit stores its verdicts in the persistent cache, so the refusal below is
    // also proof that a warm cache cannot hide the downgrade.
    let clean = audit_project_with_strict(root, true);
    assert!(clean.errors.is_empty(), "{:?}", clean.errors);

    let other_state = find_change_dir(root, &other.id).unwrap().join("state.json");
    commit_hidden_workflow_downgrade(root, &other_state);

    let report = audit_project_with_strict(root, true);
    assert!(
        report.errors.iter().any(|error| error.contains(&format!("`{}`", other.id))
            && error.contains("workflow-version history changed immutable identity")),
        "strict audit did not name the hidden downgrade: {:?}",
        report.errors
    );
    assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
    assert_eq!(audit_project_with_strict(root, true).errors, report.errors);
    // The default audit stays active-only, exactly as before (REQ-change-audit-project-001).
    assert_eq!(audit_project(root).errors, audit_project_with_strict(root, false).errors);
}

/// The timed finalize benchmark against a Raven-sized project: 124 archived changes
/// (`SPECSYNC_BENCH_ARCHIVES` changes the count). Run on demand with
/// `cargo test --release --bin specsync -- --ignored raven_sized_finalize_benchmark --nocapture`.
/// With `SPECSYNC_BENCH_FIXTURE=<dir>` it builds the project there and stops at a reviewed change
/// ready to finalize, so two release binaries can be timed against copies of the same tree.
#[test]
#[ignore = "timed benchmark; run on demand"]
fn raven_sized_finalize_benchmark() {
    let archives = std::env::var("SPECSYNC_BENCH_ARCHIVES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(124);
    let kept = std::env::var_os("SPECSYNC_BENCH_FIXTURE").map(PathBuf::from);
    let temp = TempDir::new().unwrap();
    let root = kept.clone().unwrap_or_else(|| temp.path().to_path_buf());
    fs::create_dir_all(&root).unwrap();
    let root = root.as_path();
    fast_read_repository(root);
    let built = Instant::now();
    for _ in 0..archives {
        finalized_archived_change(root);
    }
    eprintln!(
        "fixture: {archives} archived changes, built in {:.1}s",
        built.elapsed().as_secs_f64()
    );
    if kept.is_some() {
        let record = reviewed_change_ready_to_finalize(root);
        eprintln!("fixture ready: `specsync change finalize {}`", record.id);
        return;
    }
    for warm in [false, true] {
        let record = reviewed_change_ready_to_finalize(root);
        if !warm && let Some(path) = persistent_read_cache_path(root) {
            let _ = fs::remove_file(path);
        }
        reset_test_git_process_count();
        let started = Instant::now();
        finalize_change(root, &record.id).unwrap();
        eprintln!(
            "{} finalize: {:.3}s, {} Git processes",
            if warm { "warm" } else { "cold" },
            started.elapsed().as_secs_f64(),
            test_git_process_count()
        );
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "archive change"]);
    }
}
