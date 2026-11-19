// #1104: one change whose records fail validation no longer decides another change's terminal
// evidence. REQ-change-119 computes an archived change's evidence for that change alone, and
// REQ-change-012 and REQ-change-024 have other changes enter only as selected successors. tester2's
// S1 and S2 (`tester2/prs/g298`): a merge left one change both open and archived at one commit, and
// every other archived change then read `corrupt-history` with that change's reason.

/// Leaves `broken`, just archived and committed, open and archived at one commit, then removes the
/// open copy again, as the starling merge did. Its own workflow-version history then fails with
/// "duplicates state at commit". `before_archive` is the commit that still held the open package.
fn leave_open_and_archived_at_one_commit(
    root: &Path,
    broken: &ChangeRecord,
    before_archive: &str,
) -> String {
    let active = format!(".specsync/changes/{}", broken.id);
    quiet_git(root, &["checkout", before_archive, "--", &active]);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "a merge keeps the open copy beside the archive"]);
    let both = quiet_git(root, &["rev-parse", "HEAD"]).trim().to_string();
    quiet_git(root, &["rm", "-r", "-q", &active]);
    quiet_git(root, &["commit", "-m", "drop the open copy the merge kept"]);
    both
}

/// The terminal evidence of `id`, read the way a status command reads it inside its read scope,
/// and read again with no scope.
fn scoped_and_unscoped_evidence(root: &Path, id: &str) -> [TerminalEvidenceSummary; 2] {
    let scoped = {
        let _scope = begin_change_read_scope(root);
        let record = load_change(root, id).unwrap();
        // Twice, so the second read comes from the scope's cache.
        let _ = terminal_evidence_summary(root, &record);
        terminal_evidence_summary(root, &record)
    };
    let record = load_change(root, id).unwrap();
    let unscoped = terminal_evidence_summary(root, &record);
    [scoped, unscoped]
}

/// S1 and S2: an archive committed before another change's bad commit, and one committed after
/// it and its cleanup.
fn s1_and_s2() -> (TempDir, ChangeRecord, ChangeRecord, ChangeRecord, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let before = finalized_archived_change(root);
    let broken = finalized_archived_change(root);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    let both = leave_open_and_archived_at_one_commit(root, &broken, &before_archive);
    let after = finalized_archived_change(root);
    (temp, before, broken, after, both)
}

#[test]
fn another_change_s_broken_history_does_not_decide_an_archive_s_evidence() {
    let (temp, before, _broken, after, _) = s1_and_s2();
    let root = temp.path();
    for (case, record) in [("archived before (S1)", &before), ("archived after (S2)", &after)] {
        for (read, summary) in ["scoped", "unscoped"]
            .iter()
            .zip(scoped_and_unscoped_evidence(root, &record.id))
        {
            assert_eq!(
                summary.validity,
                TerminalEvidenceValidity::AuthenticatedHistory,
                "{case}, {read}: {:?}",
                summary.reason
            );
        }
    }
}

#[test]
fn the_broken_change_still_reports_its_own_failure() {
    let (temp, _before, broken, _after, both) = s1_and_s2();
    let root = temp.path();
    let error = load_change(root, &broken.id)
        .map(|record| format!("{:?}", terminal_evidence_summary(root, &record)))
        .unwrap_or_else(|error| error);
    assert!(
        error.contains("duplicates state at commit") && error.contains(&both),
        "{error}"
    );
}
