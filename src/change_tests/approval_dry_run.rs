// `change approve --dry-run` (#1091, REQ-change-163): what stays in memory, and what is skipped.
// The refusals themselves, and the unchanged project tree, are driven through the built binary in
// `tests/integration/approve_dry_run.rs`.

fn approval_dry_run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success(),
        "git {args:?}"
    );
}

fn approval_dry_run_repository(root: &Path) {
    approval_dry_run_git(root, &["init", "-b", "main"]);
    approval_dry_run_git(root, &["config", "user.email", "test@example.com"]);
    approval_dry_run_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed"), "seed\n").unwrap();
    approval_dry_run_git(root, &["add", "seed"]);
    approval_dry_run_git(root, &["commit", "-m", "seed"]);
}

/// The dry run of a plain approval by `Reviewer`, with no relayed decision.
fn plain_approval_dry_run(root: &Path, id: &str) -> ApprovalDryRun {
    approve_definition_dry_run(root, id, Some("Reviewer".into()), None, false, Ok(None))
}

fn dry_run_check<'a>(report: &'a ApprovalDryRun, name: &str) -> &'a ApprovalCheckReport {
    report
        .checks
        .iter()
        .find(|check| check.check == name)
        .unwrap_or_else(|| panic!("no `{name}` check in {report:?}"))
}

/// Every step is reported once, in the approval's order, and a step only ever waits on steps
/// that come before it.
#[test]
fn every_approval_step_is_reported_in_order_and_waits_only_on_earlier_steps() {
    let names: Vec<&str> = ApprovalStep::ALL.iter().map(|step| step.name()).collect();
    assert_eq!(
        names,
        [
            "relay",
            "lifecycle_lock",
            "change",
            "sequence",
            "state",
            "roster",
            "baseline",
            "definition",
            "deltas",
            "ownership",
            "verification_ignored_paths",
            "approval_record",
            "actor",
        ]
    );
    for (position, step) in ApprovalStep::ALL.iter().enumerate() {
        for needed in step.waits_on() {
            let earlier = ApprovalStep::ALL
                .iter()
                .position(|candidate| candidate == needed)
                .unwrap();
            assert!(
                earlier < position,
                "`{}` waits on `{}`, which runs later",
                step.name(),
                needed.name()
            );
        }
    }
    // The three skip rules, and nothing else: the lock, the change and its state gate what
    // follows them, and the approval record also waits on the deltas it digests.
    assert_eq!(
        ApprovalStep::ApprovalRecord.waits_on().last(),
        Some(&ApprovalStep::Deltas)
    );
    for step in [
        ApprovalStep::Definition,
        ApprovalStep::Deltas,
        ApprovalStep::Ownership,
        ApprovalStep::VerificationIgnoredPaths,
        ApprovalStep::Actor,
    ] {
        assert_eq!(
            step.waits_on(),
            [
                ApprovalStep::LifecycleLock,
                ApprovalStep::Change,
                ApprovalStep::State
            ],
            "`{}` runs on its own once the change is loaded and its state admits approval",
            step.name()
        );
    }
}

/// The legacy-baseline binding changes the record. The dry run makes it in memory, so the later
/// checks see what the approval would, and `state.json` keeps its bytes. The real approval then
/// binds and saves it.
#[test]
fn the_dry_run_keeps_the_legacy_baseline_binding_in_memory() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    approval_dry_run_repository(root);
    let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let mut authority = completed_no_spec_record(root);
    authority.base_commit = Some(cutoff.clone());
    authority
        .affected_paths
        .push(LEGACY_BASELINE_PATH.to_string());
    save_change(root, &authority).unwrap();
    write_change_markdown(root, &authority).unwrap();
    fs::create_dir_all(root.join(".specsync/archive")).unwrap();
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: authority.id.clone(),
        cutoff_commit: cutoff,
        entries: Vec::new(),
    };
    let bytes = json_content(&baseline).unwrap();
    fs::write(root.join(LEGACY_BASELINE_PATH), &bytes).unwrap();
    let (_, expected) = validate_legacy_archive_baseline_bytes(bytes.as_bytes()).unwrap();
    let state_path = change_dir(root, &authority.id).join("state.json");
    let state_before = fs::read(&state_path).unwrap();
    let ledger_path = change_dir(root, &authority.id).join("approvals.json");
    let ledger_before = fs::read(&ledger_path).unwrap();

    let report = plain_approval_dry_run(root, &authority.id);

    assert!(report.would_approve, "{report:?}");
    assert_eq!(
        dry_run_check(&report, "baseline").result,
        ApprovalCheckResult::Passed
    );
    assert_eq!(fs::read(&state_path).unwrap(), state_before);
    assert_eq!(fs::read(&ledger_path).unwrap(), ledger_before);
    assert_eq!(
        load_change(root, &authority.id)
            .unwrap()
            .legacy_archive_baseline_digest,
        None,
        "the dry run saved the binding"
    );

    let approved = approve_definition(root, &authority.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(
        approved.legacy_archive_baseline_digest.as_deref(),
        Some(expected.as_str())
    );
}

/// A relayed decision raises a workflow-v2 record to the relay-gated version before its approval
/// record is built (REQ-change-158). The dry run raises it in memory only.
#[test]
fn the_dry_run_keeps_a_relayed_decisions_version_raise_in_memory() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    approval_dry_run_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    assert_eq!(record.workflow_version, 2);
    let relay = || RelayedDecision {
        decided_by: "Reviewer".into(),
        words: "Approve it".into(),
        where_: "the team chat".into(),
        recorded_by: "an agent".into(),
    };
    let state_path = change_dir(root, &record.id).join("state.json");
    let state_before = fs::read(&state_path).unwrap();

    let report = approve_definition_dry_run(
        root,
        &record.id,
        Some("Reviewer".into()),
        None,
        false,
        Ok(Some(relay())),
    );

    assert!(report.would_approve, "{report:?}");
    assert_eq!(fs::read(&state_path).unwrap(), state_before);
    assert_eq!(load_change(root, &record.id).unwrap().workflow_version, 2);

    let approved = approve_definition_with_relay(
        root,
        &record.id,
        Some("Reviewer".into()),
        None,
        false,
        Some(relay()),
    )
    .unwrap();
    assert!(
        approved.workflow_version > 2,
        "the real approval raises and saves the version"
    );
}

/// A change whose state does not admit approval is one refusal, the one the approval gives, and
/// every later check is skipped because of it.
#[test]
fn a_state_that_refuses_approval_skips_every_later_check() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    approval_dry_run_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    approval_dry_run_git(root, &["add", "."]);
    approval_dry_run_git(root, &["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id).unwrap();
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Archived
    );

    let report = plain_approval_dry_run(root, &record.id);

    assert!(!report.would_approve, "{report:?}");
    let refused: Vec<&ApprovalCheckReport> = report
        .checks
        .iter()
        .filter(|check| check.result == ApprovalCheckResult::Refused)
        .collect();
    assert_eq!(refused.len(), 1, "{report:?}");
    assert_eq!(refused[0].check, "state");
    let real = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert_eq!(refused[0].message.as_deref(), Some(real.as_str()));
    let position = ApprovalStep::ALL
        .iter()
        .position(|step| *step == ApprovalStep::State)
        .unwrap();
    for step in &ApprovalStep::ALL[position + 1..] {
        let check = dry_run_check(&report, step.name());
        assert_eq!(check.result, ApprovalCheckResult::Skipped, "{report:?}");
        assert_eq!(check.skipped_because, Some("state"), "{report:?}");
    }
    for step in &ApprovalStep::ALL[..position] {
        assert_eq!(
            dry_run_check(&report, step.name()).result,
            ApprovalCheckResult::Passed,
            "{report:?}"
        );
    }
}

/// An interrupted lifecycle write is recovered by the next command that takes the lock. The dry
/// run takes no lock: it refuses at the lock step, runs nothing after it, and leaves the journal
/// and the lock file as it found them.
#[test]
fn an_interrupted_write_refuses_at_the_lock_step_and_nothing_is_recovered() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    approval_dry_run_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let journal = root.join(TRANSACTION_PATH);
    fs::write(&journal, "[]\n").unwrap();
    let lock = root.join(LOCK_PATH);
    let _ = fs::remove_file(&lock);

    let report = plain_approval_dry_run(root, &record.id);

    assert!(!report.would_approve, "{report:?}");
    let lock_check = dry_run_check(&report, "lifecycle_lock");
    assert_eq!(lock_check.result, ApprovalCheckResult::Refused);
    assert!(
        lock_check
            .message
            .as_deref()
            .is_some_and(|message| message.contains(TRANSACTION_PATH)),
        "{report:?}"
    );
    for step in &ApprovalStep::ALL[2..] {
        let check = dry_run_check(&report, step.name());
        assert_eq!(check.result, ApprovalCheckResult::Skipped, "{report:?}");
        assert_eq!(check.skipped_because, Some("lifecycle_lock"), "{report:?}");
    }
    assert_eq!(fs::read_to_string(&journal).unwrap(), "[]\n");
    assert!(!lock.exists(), "the dry run took the lifecycle lock");
}
