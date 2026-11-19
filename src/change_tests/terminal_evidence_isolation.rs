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

/// A legacy change accepted on the current branch and committed: an accepted change whose
/// verdict searches for successors when one of its delivery inputs changes.
fn accepted_legacy_change(root: &Path) -> ChangeRecord {
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve and implement the legacy change"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "accept the legacy change"]);
    record
}

#[test]
fn an_accepted_change_is_judged_on_its_own_inputs_and_a_failed_candidate_is_named() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let accepted = accepted_legacy_change(root);
    // A later change over the same paths, archived and then left open and archived at one commit.
    let broken = finalized_archived_change(root);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    leave_open_and_archived_at_one_commit(root, &broken, &before_archive);
    for (read, summary) in ["scoped", "unscoped"]
        .iter()
        .zip(scoped_and_unscoped_evidence(root, &accepted.id))
    {
        assert_eq!(
            summary.validity,
            TerminalEvidenceValidity::Exact,
            "unchanged inputs, {read}: {:?}",
            summary.reason
        );
    }

    // Its input changes: the broken change would be a candidate successor, and covers nothing.
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "change the accepted change's input"]);
    for (read, summary) in ["scoped", "unscoped"]
        .iter()
        .zip(scoped_and_unscoped_evidence(root, &accepted.id))
    {
        assert_eq!(summary.validity, TerminalEvidenceValidity::Stale, "{read}");
        let reason = summary.reason.unwrap_or_default();
        assert!(
            reason.contains("changed after acceptance")
                && reason.contains(&format!("candidate successor `{}` fails validation", broken.id))
                && reason.contains("duplicates state at commit"),
            "{read}: the reason names the failed candidate and its failure: {reason}"
        );
    }
}

#[test]
fn an_unreadable_state_counts_only_against_a_successor_search() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let accepted = accepted_legacy_change(root);
    let archived = finalized_archived_change(root);
    let unreadable = root.join(".specsync/changes/unreadable-change");
    fs::create_dir_all(&unreadable).unwrap();
    fs::write(unreadable.join("state.json"), "not json\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "a change whose state does not read"]);
    for (case, id, validity) in [
        ("an archive never searches", &archived.id, TerminalEvidenceValidity::AuthenticatedHistory),
        ("unchanged inputs never search", &accepted.id, TerminalEvidenceValidity::Exact),
    ] {
        for (read, summary) in ["scoped", "unscoped"]
            .iter()
            .zip(scoped_and_unscoped_evidence(root, id))
        {
            assert_eq!(summary.validity, validity, "{case}, {read}: {:?}", summary.reason);
        }
    }
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "change the accepted change's input"]);
    let [scoped, unscoped] = scoped_and_unscoped_evidence(root, &accepted.id);
    for (read, summary) in [("scoped", scoped), ("unscoped", unscoped)] {
        let reason = summary.reason.unwrap_or_default();
        assert!(
            reason.contains("candidate successor `unreadable-change` fails validation"),
            "{read}: a search counts the unreadable change: {reason}"
        );
    }
}

// #1104, criterion 4: a retired change's validity reads its replacement, so a replacement whose
// records fail is still reported in the retired change's own status. This read is not changed by
// #1104; the test holds it there.
#[test]
fn a_retired_change_whose_replacement_fails_still_reports_it() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic test authority, not a real human decision".into(),
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "retire the obsolete change"]);
    leave_open_and_archived_at_one_commit(root, &replacement, &before_archive);
    let status = load_change(root, &old.id).and_then(|record| {
        serde_json::to_string(&summarize_change_with_strict(root, &record, false))
            .map_err(|error| error.to_string())
    });
    let reported = match status {
        Ok(json) => json,
        Err(error) => error,
    };
    assert!(
        reported.contains("duplicates state at commit"),
        "the retired change's status reports its replacement's failure: {reported}"
    );
}
