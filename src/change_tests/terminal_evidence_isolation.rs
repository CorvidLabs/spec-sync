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

/// The broken change here is a no-spec-change archive over the same paths: `successor_covers_input`
/// never takes such a change as a successor, so it could not have covered the input and is not
/// named (Codex on #1114).
#[test]
fn an_accepted_change_is_judged_on_its_own_inputs_and_a_broken_change_that_could_not_cover_them_is_not_named()
{
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

    // Its input changes. The broken change declares no succession of it, so it is no candidate.
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "change the accepted change's input"]);
    for (read, summary) in ["scoped", "unscoped"]
        .iter()
        .zip(scoped_and_unscoped_evidence(root, &accepted.id))
    {
        assert_eq!(summary.validity, TerminalEvidenceValidity::Stale, "{read}");
        let reason = summary.reason.unwrap_or_default();
        assert!(reason.contains(CHANGED_AFTER_ACCEPTANCE), "{read}: {reason}");
        assert!(
            !reason.contains(&broken.id) && !reason.contains("duplicates state at commit"),
            "{read}: a change that could not cover the input is not named: {reason}"
        );
    }
}

/// A broken change that does declare the input as its successor is named, with its failure: it
/// would have covered the input (#1104's third criterion).
#[test]
fn a_broken_change_that_declares_the_input_is_named_as_its_failed_candidate() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, successor) = accepted_change_with_covering_successor(root);
    archive_change(root, &successor.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive the successor"]);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    leave_open_and_archived_at_one_commit(root, &successor, &before_archive);
    for (read, summary) in ["scoped", "unscoped"]
        .iter()
        .zip(scoped_and_unscoped_evidence(root, &predecessor.id))
    {
        assert_eq!(summary.validity, TerminalEvidenceValidity::Stale, "{read}");
        let reason = summary.reason.unwrap_or_default();
        assert!(
            reason.contains(CHANGED_AFTER_ACCEPTANCE)
                && reason.contains(&format!("candidate successor `{}` fails validation", successor.id))
                && reason.contains("duplicates state at commit"),
            "{read}: the reason names the failed candidate and its failure: {reason}"
        );
    }
}

/// #1104's review: `name_failed_candidates` knows a successor search by the words the two
/// writers of a changed-input reason put in it. One constant holds those words for all three, so
/// rewording a writer cannot quietly stop the reader from naming a failed candidate.
#[test]
fn the_successor_search_reads_the_words_both_writers_write() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = accepted_legacy_change(root);
    let rejected = |workflow_version| {
        BTreeMap::from([(
            "a-successor".to_string(),
            RejectedSuccessor {
                workflow_version,
                reason: "its own delivery-input evidence is stale".into(),
            },
        )])
    };
    let reasons = [
        stale_input_remediation_reason(&record, "src/lib.rs", "lib", &BTreeMap::new()),
        stale_input_remediation_reason(&record, "src/lib.rs", "lib", &rejected(2)),
        stale_input_remediation_reason(&record, "src/lib.rs", "lib", &rejected(1)),
        exact_only_input_remediation_reason(&record, "README.md"),
    ];
    for reason in reasons {
        assert!(reason.contains(CHANGED_AFTER_ACCEPTANCE), "{reason}");
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

/// The reopened fixture change, retired under its current replacement and committed, with the
/// bytes of its `state.json` from just before the retirement.
fn retired_fixture() -> (TempDir, ChangeRecord, ChangeRecord, Vec<u8>) {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let before_retirement = fs::read(change_dir(root, &old.id).join("state.json")).unwrap();
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
    let old = load_change(root, &old.id).unwrap();
    let replacement = load_change(root, &replacement.id).unwrap();
    (temp, old, replacement, before_retirement)
}

/// corvid-agent's case on #1114: a retirement rolled back, by putting back the state the change
/// had before it, is that change's own failure. Its evidence names the rollback in the words
/// `validate_retired_records` uses, and its replacement still reads as its own history says.
#[test]
fn a_rolled_back_retirement_names_the_rollback_in_its_own_evidence() {
    let (temp, old, replacement, before_retirement) = retired_fixture();
    let root = temp.path();
    let archived = find_change_dir(root, &old.id).unwrap();
    fs::write(archived.join("state.json"), &before_retirement).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "roll the retirement back"]);
    let records = list_all_change_metadata(root, false).unwrap();
    let expected = validate_retired_records(root, &records).unwrap_err();
    assert_eq!(
        expected,
        format!("committed retirement history for {} was rolled back", old.id)
    );
    let rolled_back = load_change(root, &old.id).unwrap();
    assert_ne!(rolled_back.state, ChangeState::Retired);
    let own = terminal_evidence_summary(root, &rolled_back);
    assert_eq!(own.validity, TerminalEvidenceValidity::Stale, "{own:?}");
    assert_eq!(own.reason.as_deref(), Some(expected.as_str()), "{own:?}");
    let other = terminal_evidence_summary(root, &replacement);
    assert_eq!(
        other.validity,
        TerminalEvidenceValidity::AuthenticatedHistory,
        "{other:?}"
    );
}

/// Two changes carry committed retirement receipts, and only one still holds its retirement: the
/// other stayed archived. That is the other change's own failure, in the words
/// `validate_retired_records` uses; the change that is still retired reads exactly as it did
/// before (corvid-agent and Codex on #1114).
#[test]
fn a_retired_change_is_not_blamed_for_another_change_s_rolled_back_retirement() {
    let (temp, old, replacement, _) = retired_fixture();
    let root = temp.path();
    let clean = terminal_evidence_summary(root, &old);
    let mut receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(find_change_dir(root, &old.id).unwrap().join(RETIREMENT_FILE)).unwrap(),
    )
    .unwrap();
    receipt["change_id"] = replacement.id.clone().into();
    fs::write(
        find_change_dir(root, &replacement.id)
            .unwrap()
            .join(RETIREMENT_FILE),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(
        root,
        &["commit", "-m", "a retirement receipt for a change that stays archived"],
    );
    let records = list_all_change_metadata(root, false).unwrap();
    let expected = validate_retired_records(root, &records).unwrap_err();
    assert_eq!(
        expected,
        format!(
            "committed retirement history for {} was rolled back",
            replacement.id
        )
    );
    assert_eq!(terminal_evidence_summary(root, &old), clean);
    let own = terminal_evidence_summary(root, &replacement);
    assert_eq!(own.validity, TerminalEvidenceValidity::CorruptHistory, "{own:?}");
    assert_eq!(own.reason.as_deref(), Some(expected.as_str()), "{own:?}");
}

/// Codex on #1114: while one commit holds a change both open and archived, the open change reads
/// `stale` with its failure, as before #1104; `corrupt-history` is an archive's verdict. The
/// verdict follows the record asked about, not the order the roster found the two copies in.
#[test]
fn an_open_copy_beside_its_archive_reads_stale_with_its_own_failure() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let archived = finalized_archived_change(root);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    let active = format!(".specsync/changes/{}", archived.id);
    quiet_git(root, &["checkout", &before_archive, "--", &active]);
    quiet_git(root, &["add", "--all"]);
    quiet_git(
        root,
        &["commit", "-m", "a merge keeps the open copy beside the archive"],
    );
    let open: ChangeRecord =
        serde_json::from_str(&fs::read_to_string(root.join(&active).join("state.json")).unwrap())
            .unwrap();
    assert_ne!(open.state, ChangeState::Archived);
    let own = terminal_evidence_summary(root, &open);
    assert_eq!(own.validity, TerminalEvidenceValidity::Stale, "{own:?}");
    assert!(own.reason.is_some_and(|reason| !reason.is_empty()));
    let archive = terminal_evidence_summary(root, &archived);
    assert_eq!(
        archive.validity,
        TerminalEvidenceValidity::CorruptHistory,
        "{archive:?}"
    );
}

/// Codex on #1114 (REQ-change-149): the per-change roster walks the archive with the same
/// deadline boundary as the roster load, so an expired deadline stops the read instead of setting
/// an archive aside as failed.
#[test]
fn the_per_change_roster_stops_at_an_expired_deadline_in_the_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    finalized_archived_change(root);
    let _scope = begin_change_read_scope(root);
    // The active changes are read, and kept in the scope, before the deadline passes.
    list_changes(root).unwrap();
    let _deadline = expired_worker_deadline();
    let result = list_all_changes_one_by_one(root);
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("archived change validation")),
        "an expired deadline let the per-change roster answer: {:?}",
        result.map(|roster| roster.failed.into_keys().collect::<Vec<_>>())
    );
}

/// The per-change roster's retirement checks put each failure on the change it names, in the words
/// `validate_retired_records` uses for the roster load: here an edited receipt.
#[test]
fn the_per_change_retirement_checks_say_what_the_roster_load_says() {
    let (temp, old, _, _) = retired_fixture();
    let root = temp.path();
    let receipt = find_change_dir(root, &old.id).unwrap().join(RETIREMENT_FILE);
    let mut bytes = fs::read(&receipt).unwrap();
    bytes.push(b'\n');
    fs::write(&receipt, bytes).unwrap();
    let records = list_all_change_metadata(root, false).unwrap();
    let expected = validate_retired_records(root, &records).unwrap_err();
    assert!(expected.contains("was edited"), "{expected}");
    let roster = TerminalRoster {
        records,
        failed: BTreeMap::new(),
    };
    assert_eq!(
        retirement_failures(root, &roster).unwrap(),
        vec![(old.id.clone(), expected)]
    );
}

/// Codex's second round on #1114: a healthy legacy accepted change beside another change whose
/// records fail validation reads `exact`, and its status says to archive it, not to reopen it: its
/// closing-approval check is judged on the changes that load, as its terminal evidence is.
#[test]
fn a_healthy_legacy_accepted_change_beside_a_broken_one_is_told_to_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let accepted = accepted_legacy_change(root);
    let broken = finalized_archived_change(root);
    let before_archive = quiet_git(root, &["rev-parse", "HEAD~1"]).trim().to_string();
    leave_open_and_archived_at_one_commit(root, &broken, &before_archive);
    let record = load_change(root, &accepted.id).unwrap();
    let summary = summarize_change_with_strict(root, &record, false);
    assert_eq!(
        summary.terminal_evidence.as_ref().map(|evidence| evidence.validity),
        Some(TerminalEvidenceValidity::Exact),
        "{:?}",
        summary.terminal_evidence
    );
    assert_eq!(
        summary.next_action,
        format!("run `specsync change archive {}`", accepted.id)
    );
    let handoff = serde_json::to_string(&summary.handoff).unwrap();
    assert!(!handoff.contains("change reopen"), "{handoff}");
}

/// Codex's second round on #1114: a failed archive's own verdict is kept in the read scope like
/// any other archive's, so a second read in the scope answers from it instead of loading the
/// roster again.
#[test]
fn a_failed_archive_s_verdict_is_kept_in_the_read_scope() {
    let (temp, _before, broken, _after, _) = s1_and_s2();
    let root = temp.path();
    let directory = find_change_dir(root, &broken.id).unwrap();
    let record: ChangeRecord =
        serde_json::from_str(&fs::read_to_string(directory.join("state.json")).unwrap()).unwrap();
    let _scope = begin_change_read_scope(root);
    let first = terminal_evidence_summary(root, &record);
    assert!(
        first
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("duplicates state at commit")),
        "{first:?}"
    );
    // A fresh read of the roster would no longer find this archive at all.
    fs::remove_dir_all(&directory).unwrap();
    assert_eq!(terminal_evidence_summary(root, &record), first);
}

/// Codex's second round on #1114: a retirement rolled back by restoring the change's open
/// workspace, with no archive left at all, is still that change's own failure: the per-change
/// roster runs the retirement checks with no archive directory too.
#[test]
fn a_rollback_to_an_open_workspace_with_no_archive_left_still_names_the_rollback() {
    let (temp, old, _replacement, _) = retired_fixture();
    let root = temp.path();
    let active = format!(".specsync/changes/{}", old.id);
    quiet_git(root, &["checkout", "HEAD~1", "--", &active]);
    quiet_git(root, &["rm", "-r", "-q", ARCHIVE_PATH]);
    quiet_git(
        root,
        &["commit", "-m", "put the open workspace back and remove the archive"],
    );
    // Untracked leftovers go too, so no archive directory is left at all.
    if root.join(ARCHIVE_PATH).exists() {
        fs::remove_dir_all(root.join(ARCHIVE_PATH)).unwrap();
    }
    let open: ChangeRecord =
        serde_json::from_str(&fs::read_to_string(root.join(&active).join("state.json")).unwrap())
            .unwrap();
    let own = terminal_evidence_summary(root, &open);
    assert_eq!(
        own.reason.as_deref(),
        Some(format!("committed retirement history for {} was rolled back", old.id).as_str()),
        "{own:?}"
    );
}

#[test]
fn a_terminal_roster_attributes_an_oversized_archived_state_to_its_own_package() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let directory = "2026-11-22-oversized";
    let path = root.join(ARCHIVE_PATH).join(directory);
    fs::create_dir_all(&path).unwrap();
    fs::File::create(path.join("state.json"))
        .unwrap()
        .set_len(crate::util::MAX_STATE_FILE_BYTES + 1)
        .unwrap();

    let roster = terminal_evidence_roster(root).unwrap();
    assert_eq!(roster.failed.len(), 1);
    assert!(roster.failed[directory].reason.contains("larger than"), "{roster:?}");
}

fn pre_snapshot_archive_with_approved_authority(root: &Path) -> (ChangeRecord, ChangeRecord) {
    fast_read_repository(root);
    let mut archived = accepted_legacy_change(root);
    let workspace = change_dir(root, &archived.id);
    let mut verification = load_verification(root, &archived).unwrap();
    verification.acceptance_manifest = None;
    write_json(&workspace.join("verification.json"), &verification).unwrap();
    archived.state = ChangeState::Archived;
    write_json(&workspace.join("state.json"), &archived).unwrap();

    let archive_path = format!("{ARCHIVE_PATH}/2026-07-11-{}", archived.id);
    fs::create_dir_all(root.join(ARCHIVE_PATH)).unwrap();
    fs::rename(workspace, root.join(&archive_path)).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "introduce the pre-snapshot archive"]);
    let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let mut authority = completed_no_spec_record(root);
    authority.base_commit = Some(cutoff.clone());
    authority.affected_paths.push(LEGACY_BASELINE_PATH.into());
    save_change(root, &authority).unwrap();
    write_change_markdown(root, &authority).unwrap();
    let paths = BTreeSet::from([archive_path.clone()]);
    let (paths, evidence) =
        stable_discovered_evidence(root, Some(&paths), &BTreeSet::new(), false, false).unwrap();
    let snapshot = archive_snapshot_from_evidence(&paths, &evidence, &archive_path).unwrap();
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: authority.id.clone(),
        cutoff_commit: cutoff.clone(),
        entries: vec![LegacyArchiveBaselineEntryV1 {
            id: archived.id.clone(),
            archive_path,
            introduction_commit: cutoff,
            subtree_digest: legacy_archive_subtree_digest(&snapshot).unwrap(),
        }],
    };
    write_json(&root.join(LEGACY_BASELINE_PATH), &baseline).unwrap();
    let authority = approve_definition(root, &authority.id, Some("Scope owner".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "record the approved legacy baseline"]);
    (archived, authority)
}

#[test]
fn a_pre_snapshot_archive_ignores_broken_siblings_but_requires_its_baseline_authority() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (archived, authority) = pre_snapshot_archive_with_approved_authority(root);
    let assert_authenticated = || {
        for summary in scoped_and_unscoped_evidence(root, &archived.id) {
            assert_eq!(
                summary.validity,
                TerminalEvidenceValidity::AuthenticatedHistory,
                "{:?}",
                summary.reason
            );
        }
    };
    assert_authenticated();
    let clean_tip = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let broken = finalized_archived_change(root);
    let before_archive = git_output(root, &["rev-parse", "HEAD~1"]).unwrap();
    let both = leave_open_and_archived_at_one_commit(root, &broken, &before_archive);
    assert_authenticated();
    assert!(load_change(root, &broken.id).unwrap_err().contains(&both));
    let strict_error = validate_archived_integrity(root, &archived).unwrap_err();
    assert!(strict_error.contains(&both), "{strict_error}");
    quiet_git(root, &["switch", "--detach", &clean_tip]);

    let malformed = change_dir(root, "malformed-sibling");
    fs::create_dir_all(&malformed).unwrap();
    fs::write(malformed.join("state.json"), "{ invalid").unwrap();
    assert_authenticated();
    let strict_error = validate_archived_integrity(root, &archived).unwrap_err();
    assert!(strict_error.contains("malformed-sibling"), "{strict_error}");
    let audit = audit_project_with_archives(root, true);
    assert!(
        audit.errors.iter().any(|error| error.contains("malformed-sibling")),
        "{:?}",
        audit.errors
    );

    fs::write(change_dir(root, &authority.id).join("state.json"), "{ invalid").unwrap();
    for summary in scoped_and_unscoped_evidence(root, &archived.id) {
        assert_eq!(summary.validity, TerminalEvidenceValidity::CorruptHistory);
        let reason = summary.reason.unwrap();
        assert!(reason.contains("baseline authority") && reason.contains(&authority.id), "{reason}");
    }
}
