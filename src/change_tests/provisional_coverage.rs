// #657: an accepted change's changed input counts as covered for now when a later change that is
// approved, implementing or verifying declares its owner module and path. Audit then warns,
// naming that change, instead of failing; the accepted change may archive, and its archive
// records which change covered the file. With no such change the input is stale, as before.

/// A legacy `auth` change accepted and committed, then `src/auth.rs` edited and committed. With
/// `successor`, an approved change created after it declares `auth` and `src/auth.rs`.
fn accepted_change_with_moved_input(root: &Path, successor: bool) -> (ChangeRecord, Option<ChangeRecord>) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut accepted = completed_section_only_record(root, delta);
    accepted = approve_definition(root, &accepted.id, Some("Reviewer".into()), None).unwrap();
    accepted = start_implementation(root, &accepted.id).unwrap();
    verify_change(root, &accepted.id).unwrap();
    accepted = accept_change(root, &accepted.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the first change"]);
    let in_flight = successor.then(|| {
        let mut later = completed_current_record(root);
        for artifact in &later.selected_artifacts {
            let content = if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Edit the shared source.\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
            };
            fs::write(change_dir(root, &later.id).join(artifact.file_name()), content).unwrap();
        }
        fs::write(
            delta_path(root, &later, "auth"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the later change.\n",
        )
        .unwrap();
        later.affected_paths.push("src/auth.rs".into());
        // The later change edits the `auth` spec too when it is checked, so its scope says so.
        later.affected_paths.push("specs/auth/".into());
        later.affected_paths.sort();
        later.affected_paths.dedup();
        save_change(root, &later).unwrap();
        write_change_markdown(root, &later).unwrap();
        approve_definition(root, &later.id, Some("Scope owner".into()), None).unwrap()
    });
    fs::write(root.join("src/auth.rs"), "// Edited by the later change.\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "the later change edits the shared source"]);
    (accepted, in_flight)
}

fn terminal_validity(root: &Path, record: &ChangeRecord) -> TerminalEvidenceSummary {
    terminal_evidence_summary_with_records(root, record, &list_all_changes_checked(root).unwrap())
}

#[test]
fn an_in_flight_change_that_declares_the_input_covers_it_for_now() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();

    let evidence = terminal_validity(root, &accepted);
    assert_eq!(evidence.validity.as_str(), "provisionally-covered", "{evidence:?}");
    assert!(
        evidence.reason.as_deref().is_some_and(|reason| reason.contains(&in_flight.id)),
        "{evidence:?}"
    );
    let report = audit_project(root);
    assert!(
        !report.errors.iter().any(|error| error.contains(&accepted.id)),
        "{:?}",
        report.errors
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains(&accepted.id) && warning.contains(&in_flight.id)),
        "{:?}",
        report.warnings
    );

    let archived = archive_change(root, &accepted.id).expect("a provisionally covered change may archive");
    let coverage: serde_json::Value =
        serde_json::from_slice(&fs::read(archived.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(coverage["covered_by"][0]["path"], "src/auth.rs", "{coverage}");
    assert_eq!(coverage["covered_by"][0]["change"], in_flight.id.as_str(), "{coverage}");
}

#[test]
fn an_exact_archive_refuses_stray_coverage_and_preserves_the_source() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, false);
    quiet_git(root, &["checkout", "HEAD^", "--", "src/auth.rs"]);
    quiet_git(root, &["commit", "-m", "restore the accepted bytes"]);
    assert_eq!(
        terminal_validity(root, &accepted).validity,
        TerminalEvidenceValidity::Exact
    );

    let source = change_dir(root, &accepted.id);
    let coverage_path = source.join(PROVISIONAL_COVERAGE_FILE);
    let stray = json_content(&serde_json::json!({
        "schema_version": 1,
        "change_id": accepted.id,
        "covered_by": [{"path": "src/auth.rs", "change": "fabricated-coverer"}],
    }))
    .unwrap();
    fs::write(&coverage_path, &stray).unwrap();

    let error = archive_change(root, &accepted.id).unwrap_err();
    assert!(
        error.contains(PROVISIONAL_COVERAGE_FILE) && error.contains("move it aside"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(&coverage_path).unwrap(), stray);
    assert!(!source.join("accepted-state.json").exists());
    assert_eq!(
        load_change(root, &accepted.id).unwrap().state,
        ChangeState::Accepted
    );

    fs::remove_file(coverage_path).unwrap();
    let archived = archive_change(root, &accepted.id).unwrap();
    assert!(!archived.join(PROVISIONAL_COVERAGE_FILE).exists());
}

#[test]
fn a_moved_archive_retries_only_with_current_authoritative_coverage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();
    let source = change_dir(root, &accepted.id);
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("1900-01-01-{}", accepted.id));
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::rename(&source, &destination).unwrap();

    let coverage_path = destination.join(PROVISIONAL_COVERAGE_FILE);
    let stray = "stale coverage bytes";
    fs::write(&coverage_path, stray).unwrap();
    let error = archive_change(root, &accepted.id).unwrap_err();
    assert!(
        error.contains(PROVISIONAL_COVERAGE_FILE) && error.contains("move it aside"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(&coverage_path).unwrap(), stray);
    assert!(!source.exists());

    let computed = json_content(&serde_json::json!({
        "schema_version": 1,
        "change_id": accepted.id,
        "covered_by": [{"path": "src/auth.rs", "change": in_flight.id}],
    }))
    .unwrap();
    fs::write(&coverage_path, &computed).unwrap();
    assert_eq!(archive_change(root, &accepted.id).unwrap(), destination);
    assert_eq!(fs::read_to_string(&coverage_path).unwrap(), computed);
}

#[test]
fn a_failed_archive_integrity_check_rolls_back_new_coverage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, true);
    let source = change_dir(root, &accepted.id);
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{}", today(), accepted.id));
    let state = fs::read(source.join("state.json")).unwrap();
    let markdown = fs::read(source.join("change.md")).unwrap();
    let verification = fs::read(source.join("verification.json")).unwrap();

    let destination_verification = destination.join("verification.json");
    inject_transaction_after_journal_hook(move || {
        fs::write(
            destination_verification,
            "invalid verification after publication starts",
        )
        .unwrap();
    });
    let error = archive_change(root, &accepted.id).unwrap_err();
    assert!(
        error.contains("post-move validation") && error.contains("source restored"),
        "{error}"
    );
    assert_eq!(fs::read(source.join("state.json")).unwrap(), state);
    assert_eq!(fs::read(source.join("change.md")).unwrap(), markdown);
    assert!(!source.join(PROVISIONAL_COVERAGE_FILE).exists());
    assert!(!source.join("accepted-state.json").exists());
    assert!(!destination.exists());

    fs::write(source.join("verification.json"), verification).unwrap();
    let archived = archive_change(root, &accepted.id).unwrap();
    assert!(archived.join(PROVISIONAL_COVERAGE_FILE).is_file());
}

#[test]
fn only_a_current_approval_can_supply_provisional_coverage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let mut covering = in_flight.unwrap();
    covering
        .acceptance_criteria
        .push("Additional unapproved work.".into());
    save_change(root, &covering).unwrap();

    assert!(ensure_definition_approval_valid(root, &covering).is_err());
    assert_eq!(
        terminal_validity(root, &accepted).validity,
        TerminalEvidenceValidity::Stale
    );
    assert!(archive_change(root, &accepted.id).is_err());
}

#[test]
fn provisional_coverage_keeps_an_unproven_historical_transition_visible() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (mut accepted, _) = accepted_change_with_moved_input(root, true);
    // The marker is recorded during historical reconciliation. Here the graph verdict is
    // already known. Exercise its projection without changing the accepted receipt.
    accepted
        .answers
        .insert(HISTORICAL_RECONCILIATION.into(), "unproven".into());
    let mut memo = AcceptedInputValidationMemo::default();
    memo.validity.insert(
        accepted.id.clone(),
        Ok(AcceptedInputValidity::ProvisionallyCovered),
    );
    let evidence = terminal_evidence_summary_with_validation_state(
        root,
        &accepted,
        &BTreeMap::new(),
        &mut BTreeSet::new(),
        &mut memo,
        &mut ArchivedIntegrityCache::default(),
    );
    assert_eq!(
        evidence.validity,
        TerminalEvidenceValidity::HistoricalTransitionUnproven
    );
}

#[test]
fn a_sequence_owner_with_provisional_coverage_cannot_cover_a_predecessor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let mut owner = reidentify_as_ordinal(root, &completed_no_spec_record(root), "CHG-0002-owner");
    owner.affected_paths.push(SEQUENCE_PATH.into());
    save_change(root, &owner).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: owner.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "record the sequence owner"]);
    approve_definition(root, &owner.id, Some("Reviewer".into()), None).unwrap();
    start_implementation(root, &owner.id).unwrap();
    verify_change(root, &owner.id).unwrap();
    let owner = accept_change(root, &owner.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the sequence owner"]);

    let mut earlier = owner.clone();
    earlier.id = "CHG-0001-earlier".into();
    earlier.created_at = owner.created_at - 1;
    let records = BTreeMap::from([(owner.id.clone(), owner.clone())]);
    let mut memo = AcceptedInputValidationMemo::default();
    for (validity, covers) in [
        (AcceptedInputValidity::Exact, true),
        (AcceptedInputValidity::ProvisionallyCovered, false),
    ] {
        memo.validity.insert(owner.id.clone(), Ok(validity));
        assert_eq!(
            later_sequence_owner_covers_historical_input(
                root,
                &earlier,
                &records,
                &mut BTreeSet::new(),
                &mut memo,
                None,
            )
            .unwrap(),
            covers,
            "{validity:?}"
        );
    }
}

#[test]
fn with_no_change_declaring_the_input_it_is_stale_as_before() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, false);
    assert_eq!(terminal_validity(root, &accepted).validity.as_str(), "stale");
    let report = audit_project(root);
    assert!(
        report.errors.iter().any(|error| error.contains(&accepted.id) && error.contains("stale")),
        "{:?}",
        report.errors
    );
}

/// A legacy `auth` change accepted and committed, and a later legacy change that declares `auth`,
/// the shared paths and optionally a succession of it, approved and implementing before acceptance. The
/// later change then edits `src/auth.rs`, committed.
fn accepted_change_with_successor_in_flight(root: &Path, explicit: bool) -> (ChangeRecord, ChangeRecord) {
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut predecessor = completed_section_only_record(root, delta);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the first change"]);
    let manifest = load_verification(root, &predecessor)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let owned = |entry: &&AcceptanceInputEntryV1| {
        entry.owners.iter().any(|owner| owner == "auth")
            && entry.path != "specs/auth/requirements.md"
            && !entry.path.starts_with("specs/auth/history/")
    };
    let mut successor = completed_section_only_record(
        root,
        &delta.replace("remains governed", "is governed by the successor"),
    );
    successor
        .affected_paths
        .extend(manifest.entries.iter().filter(owned).map(|entry| entry.path.clone()));
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    if explicit {
        for entry in manifest.entries.iter().filter(owned) {
            successor = add_supersedes_obligation(
                root,
                &successor.id,
                &predecessor.id,
                &entry.path,
                "auth",
                &entry.entry_digest,
            )
            .unwrap();
        }
    }
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    successor = start_implementation(root, &successor.id).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "the successor edits the shared source"]);
    (predecessor, successor)
}

/// The change's first criterion and SHIP-13.a: neither a draft nor a change that declares the
/// path without its owning module covers it; an approved, implementing or verifying change that
/// declares both does.
#[test]
fn only_an_in_flight_change_that_declares_the_module_and_path_covers_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();
    let mut records = BTreeMap::from([(accepted.id.clone(), accepted.clone())]);
    let mut candidate = in_flight.clone();
    let mut memo = AcceptedInputValidationMemo::default();
    for state in [ChangeState::Approved, ChangeState::Implementing, ChangeState::Verifying] {
        candidate.state = state;
        records.insert(candidate.id.clone(), candidate.clone());
        assert_eq!(
            in_flight_coverer(root, &mut memo, &records, &accepted, "src/auth.rs", "auth"),
            Some(in_flight.id.clone()),
            "{state:?}"
        );
    }
    candidate.state = ChangeState::Draft;
    records.insert(candidate.id.clone(), candidate.clone());
    assert_eq!(in_flight_coverer(root, &mut memo, &records, &accepted, "src/auth.rs", "auth"), None, "a draft");
    candidate.state = ChangeState::Approved;
    candidate.affected_specs.clear();
    records.insert(candidate.id.clone(), candidate.clone());
    assert_eq!(
        in_flight_coverer(root, &mut memo, &records, &accepted, "src/auth.rs", "auth"),
        None,
        "the path without its owning module"
    );
}

/// REQ-change-167: when the covering change is accepted, the earlier change reads
/// `successor-covered`, as before #657.
#[test]
fn when_the_covering_change_is_accepted_the_earlier_one_is_successor_covered() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, successor) = accepted_change_with_successor_in_flight(root, true);
    let evidence = terminal_validity(root, &predecessor);
    assert_eq!(evidence.validity, TerminalEvidenceValidity::ProvisionallyCovered, "{evidence:?}");
    verify_change(root, &successor.id).unwrap();
    accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the successor"]);
    let evidence = terminal_validity(root, &predecessor);
    assert_eq!(evidence.validity, TerminalEvidenceValidity::SuccessorCovered, "{evidence:?}");
}

/// The change's second criterion: `change reopen` still accepts a provisionally covered change.
#[test]
fn a_provisionally_covered_change_can_still_be_reopened() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, true);
    reopen_change(root, &accepted.id, "Operator".into(), "re-verify now".into())
        .expect("a provisionally covered change may be reopened");
}

/// The change's second criterion: a provisionally covered change does not stand as a retirement
/// replacement, because its own currency is only deferred.
#[test]
fn a_provisionally_covered_change_is_no_retirement_replacement() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();
    let records = list_all_changes_checked(root).unwrap();
    let manifest = load_verification(root, &accepted)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let refused = retirement_custody(root, &in_flight, &accepted.id, &manifest, &records, 2)
        .unwrap_err();
    assert!(refused.contains("covered only for now"), "{refused}");
}

/// The change's second criterion: a provisionally covered change does not cover another change's
/// input. The successor is accepted, and then a later in-flight change edits the shared file: the
/// successor is covered only for now, so it covers nothing for its predecessor.
#[test]
fn a_provisionally_covered_change_covers_no_other_change() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, successor) = accepted_change_with_successor_in_flight(root, true);
    verify_change(root, &successor.id).unwrap();
    let successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "accept the successor"]);
    assert_eq!(
        terminal_validity(root, &predecessor).validity,
        TerminalEvidenceValidity::SuccessorCovered
    );
    // The fixture rewrites the spec and the source when it creates a change; put back what the
    // successor's acceptance left, so only the third change's own edit moves an input.
    let spec = fs::read(root.join("specs/auth/auth.spec.md")).unwrap();
    let mut later = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the third change.\n",
    );
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();
    later = approve_definition(root, &later.id, Some("Reviewer".into()), None).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "a third change edits the shared source"]);
    let own = terminal_validity(root, &successor);
    assert_eq!(own.validity, TerminalEvidenceValidity::ProvisionallyCovered, "{own:?}");
    let earlier = terminal_validity(root, &predecessor);
    assert_ne!(earlier.validity, TerminalEvidenceValidity::SuccessorCovered, "{earlier:?}");
    assert!(
        earlier.reason.as_deref().is_some_and(|reason| !reason.contains(&format!("`{}` by", successor.id))),
        "{earlier:?}"
    );
    let _ = later;
}

/// ss-lead's build note: on workflow v2 a change archives through `change finalize`. A finalization
/// that stopped after acceptance resumes onto a provisionally covered change and writes the same
/// `coverage.json` as the literal `change archive` does on a legacy change.
#[test]
fn a_resumed_finalization_writes_coverage_json_like_change_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let record = current_workflow_record(root, completed_section_only_current_record(root, delta));
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement the approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    // The finalization stopped here: accepted, not yet moved into the archive.
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "accepted by a finalization that stopped"]);
    let mut later = completed_section_only_current_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the later change.\n",
    );
    // The fixture rewrites the spec and the source; put back what the accepted change
    // committed, so only the later change's own edit moves an input.
    quiet_git(
        root,
        &["checkout", "HEAD", "--", "specs/auth/auth.spec.md", "src/auth.rs", WORKFLOW_V2_BASELINE_PATH],
    );
    later = approve_definition(root, &later.id, Some("Reviewer".into()), None).unwrap();
    fs::write(root.join("src/auth.rs"), "// Edited by the later change.\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "the later change edits the shared source"]);
    let accepted = load_change(root, &record.id).unwrap();
    assert_eq!(
        terminal_validity(root, &accepted).validity,
        TerminalEvidenceValidity::ProvisionallyCovered
    );
    let archived = finalize_change(root, &record.id).expect("a provisionally covered change may finalize");
    let coverage: serde_json::Value =
        serde_json::from_slice(&fs::read(archived.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(coverage["schema_version"], 1, "{coverage}");
    assert_eq!(coverage["change_id"], record.id.as_str(), "{coverage}");
    assert_eq!(coverage["covered_by"][0]["path"], "src/auth.rs", "{coverage}");
    assert_eq!(coverage["covered_by"][0]["change"], later.id.as_str(), "{coverage}");
}

/// REQ-change-167 and the change's third criterion: after the archive, the archived change reads
/// `authenticated-history` whatever becomes of the covering change. With the covering change sent
/// back, status names the recorded entry and warns, and `change audit --archives` warns without
/// failing; with no `coverage.json`, an archive reads as before.
#[test]
fn an_archive_warns_when_its_covering_change_is_sent_back() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, in_flight) = accepted_change_with_moved_input(root, true);
    let in_flight = in_flight.unwrap();
    archive_change(root, &accepted.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive the covered change"]);
    let archived = load_change(root, &accepted.id).unwrap();
    let summary = summarize_change(root, &archived);
    assert_eq!(
        summary.terminal_evidence.as_ref().map(|evidence| evidence.validity),
        Some(TerminalEvidenceValidity::AuthenticatedHistory)
    );
    assert_eq!(summary.provisional_coverage.len(), 1, "{:?}", summary.provisional_coverage);
    assert_eq!(summary.provisional_coverage[0].change, in_flight.id);
    assert!(summary.provisional_coverage[0].warning.is_none(), "still in flight");
    assert!(
        !audit_project_with_archives(root, true)
            .warnings
            .iter()
            .any(|warning| warning.contains("no accepted change stands behind it")),
        "no warning while the covering change is in flight"
    );

    send_back_change(root, &in_flight.id, "Operator".into(), "re-scope".into()).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "send the covering change back"]);
    let archived = load_change(root, &accepted.id).unwrap();
    let summary = summarize_change(root, &archived);
    assert_eq!(
        summary.terminal_evidence.as_ref().map(|evidence| evidence.validity),
        Some(TerminalEvidenceValidity::AuthenticatedHistory),
        "the archive is history"
    );
    let warning = summary.provisional_coverage[0].warning.clone().unwrap_or_default();
    assert!(
        warning.contains("no accepted change stands behind it")
            && warning.contains(&format!("specsync change reopen {} --actor <name> --reason <text>", accepted.id)),
        "{warning}"
    );
    let report = audit_project_with_archives(root, true);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(
        report.warnings.iter().any(|w| w.contains(&accepted.id) && w.contains("no accepted change stands behind it")),
        "{:?}",
        report.warnings
    );
}

/// REQ-change-168 and the change's fourth criterion: two accepted legacy changes share a delivery
/// input and the later one edited it without covering it. Each archive refusal names the other
/// change and says it is blocked too; both still name `change reopen` for the earlier one, and
/// running it lets the later one archive.
#[test]
fn the_two_archive_refusals_name_each_other_and_the_reopen_frees_both() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (earlier, _) = accepted_change_with_moved_input(root, false);
    // A later legacy change over the same paths that declares no succession of the earlier one.
    let mut later = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication is governed by the later change.\n",
    );
    quiet_git(root, &["checkout", "HEAD", "--", "specs/auth/auth.spec.md", "src/auth.rs"]);
    later.affected_paths.push("specs/auth/".into());
    later.affected_paths.sort();
    later.affected_paths.dedup();
    save_change(root, &later).unwrap();
    write_change_markdown(root, &later).unwrap();
    later = approve_definition(root, &later.id, Some("Reviewer".into()), None).unwrap();
    later = start_implementation(root, &later.id).unwrap();
    verify_change(root, &later.id).unwrap();
    let later = accept_change(root, &later.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "accept the later change, which declares no succession"]);
    assert_eq!(terminal_validity(root, &earlier).validity, TerminalEvidenceValidity::Stale);
    let reopen = format!("specsync change reopen {}", earlier.id);

    let first = archive_change(root, &earlier.id).unwrap_err();
    assert!(
        first.contains(&format!("`{}`", later.id))
            && first.contains("cannot be archived either")
            && first.contains(&reopen),
        "{first}"
    );
    let second = archive_change(root, &later.id).unwrap_err();
    assert!(
        second.contains(&format!("`{}`", earlier.id))
            && second.contains("cannot be archived either")
            && second.contains(&reopen),
        "{second}"
    );

    reopen_change(root, &earlier.id, "Operator".into(), "re-verify".into()).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "reopen the earlier change"]);
    archive_change(root, &later.id).expect("the reopen frees the later change");
}

/// REQ-change-168: with no later change whose scope covers the input, the stale-input refusal
/// reads as it does today, with no blocked-too sentence.
#[test]
fn with_no_later_covering_scope_the_stale_refusal_reads_as_before() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (accepted, _) = accepted_change_with_moved_input(root, false);
    let refused = archive_change(root, &accepted.id).unwrap_err();
    assert!(!refused.contains("cannot be archived either"), "{refused}");
    assert!(refused.contains(&format!("specsync change reopen {}", accepted.id)), "{refused}");
}

#[test]
fn accepting_a_plain_coverer_leaves_the_active_predecessor_stale() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, coverer) = accepted_change_with_successor_in_flight(root, false);
    assert!(verify_change(root, &coverer.id).unwrap().passed);
    accept_change(root, &coverer.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "accept plain coverer"]);
    assert_eq!(terminal_validity(root, &predecessor).validity, TerminalEvidenceValidity::Stale);
}

fn assert_archived_coverer_warning(explicit: bool) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (predecessor, coverer) = accepted_change_with_successor_in_flight(root, explicit);
    archive_change(root, &predecessor.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive temporarily covered predecessor"]);
    let predecessor = load_change(root, &predecessor.id).unwrap();
    assert!(verify_change(root, &coverer.id).unwrap().passed);
    accept_change(root, &coverer.id, Some("Closer".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "accept recorded coverer"]);

    for state in [ChangeState::Accepted, ChangeState::Archived] {
        if state == ChangeState::Archived {
            archive_change(root, &coverer.id).unwrap();
            quiet_git(root, &["add", "--all"]);
            quiet_git(root, &["commit", "-m", "archive recorded coverer"]);
        }
        let _scope = begin_change_read_scope(root);
        let summary = summarize_change(root, &predecessor);
        assert_eq!(summary.terminal_evidence.unwrap().validity, TerminalEvidenceValidity::AuthenticatedHistory);
        let covered = summary.provisional_coverage.iter().find(|entry| entry.path == "src/auth.rs").unwrap();
        assert_eq!(covered.change_state, Some(state));
        assert_eq!(covered.warning.is_some(), !explicit, "{state:?} {explicit}: {covered:?}");
        let report = audit_project_with_archives(root, true);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(
            report.warnings.iter().any(|warning| warning.contains("src/auth.rs") && warning.contains(&coverer.id)),
            !explicit,
            "{:?}", report.warnings
        );
    }
}

#[test]
fn an_archived_predecessor_warns_after_a_plain_coverer_closes() {
    assert_archived_coverer_warning(false);
}

#[test]
fn explicit_succession_clears_the_archived_predecessor_warning() {
    assert_archived_coverer_warning(true);
}
