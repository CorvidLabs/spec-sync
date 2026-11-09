// Synthetic fixtures only: these actor labels are not real human acceptance.
fn retirement_reopened_fixture() -> (TempDir, ChangeRecord) {
    let (temp, old, _) = recorded_review_recovery_fixture(true);
    let root = temp.path();
    let reopened = reopen_change(root, &old.id, "Synthetic recovery actor".into(),
        "Current intent supersedes the obsolete criterion".into()).unwrap().change;
    fs::write(change_dir(root, &old.id).join("obsolete-criterion-result.json"),
        r#"{"criterion":"Dormant Hatch","passed":false,"authority":"synthetic"}"#).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "retain reopened obsolete failure"]);
    (temp, reopened)
}

fn retirement_current_replacement(root: &Path) -> ChangeRecord {
    let replacement = completed_no_spec_current_record(root);
    approve_definition(root, &replacement.id, Some("Synthetic scope actor".into()), None).unwrap();
    check_change(root, Some(&replacement.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "current replacement implementation"]);
    check_change(root, Some(&replacement.id)).unwrap();
    record_scoped_review(root, &replacement.id, "Synthetic implementation reviewer".into()).unwrap();
    finalize_change(root, &replacement.id).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "archive genuinely checked synthetic replacement"]);
    load_change(root, &replacement.id).unwrap()
}

#[test]
fn recovery_retirement_positive_custody_and_child_route() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let old_dir = find_change_dir(root, &old.id).unwrap();
    let before = archive_workspace_snapshot(root, &old_dir,
        &portable_project_path(root, &old_dir)).unwrap();
    let retired = retire_change(root, &old.id, "Synthetic operator".into(),
        "Obsolete Hatch intent replaced by current intent".into(),
        replacement.id.clone(), "Synthetic test authority, not a real human decision".into()).unwrap();
    assert_eq!(retired.state.as_str(), "retired");
    let retired_dir = find_change_dir(root, &old.id).unwrap();
    let after = archive_workspace_snapshot(root, &retired_dir,
        &portable_project_path(root, &retired_dir)).unwrap();
    for (path, bytes) in &before {
        if path != "state.json" {
            assert_eq!(after.get(path), Some(bytes), "original {path} changed");
        }
    }
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(retired_dir.join("retirement.json")).unwrap()).unwrap();
    assert_eq!(receipt["implementation_accepted"], false);
    assert_eq!(receipt["historical_transition_status"], "unproven");
    assert_eq!(receipt["original_state"].as_str().unwrap().as_bytes(),
        before["state.json"].1.as_slice());
    assert!(!serde_json::from_slice::<serde_json::Value>(
        &after["obsolete-criterion-result.json"].1).unwrap()["passed"].as_bool().unwrap());
    let prior = load_approvals(root, &retired).unwrap().reopenings.last()
        .unwrap().prior_verification.acceptance_manifest.clone().unwrap();
    assert_eq!(receipt["custody"].as_array().unwrap().len(), prior.entries.len(),
        "all formerly accepted input obligations remain in custody");
    assert!(validate_retirement(root, &retired).is_ok());
    let repeated = retire_change(root, &old.id, "Synthetic operator".into(),
        "Obsolete Hatch intent replaced by current intent".into(),
        replacement.id.clone(), "Synthetic test authority, not a real human decision".into()).unwrap();
    assert_eq!(repeated.state.as_str(), "retired");
    assert_eq!(archive_workspace_snapshot(root, &retired_dir,
        &portable_project_path(root, &retired_dir)).unwrap(), after);

    let child = completed_no_spec_current_record(root);
    add_dependency(root, &child.id, &old.id).unwrap();
    approve_definition(root, &child.id, Some("Synthetic scope actor".into()), None).unwrap();
    let refused = start_implementation(root, &child.id).unwrap_err();
    assert!(refused.contains("must be accepted"), "{refused}");
    send_back_change(root, &child.id, "Synthetic operator".into(),
        "Bind the real accepted replacement instead of obsolete intent".into()).unwrap();
    answer_question(root, &child.id, "dependencies", &replacement.id).unwrap();
    approve_definition(root, &child.id, Some("Synthetic scope actor".into()), None).unwrap();
    check_change(root, Some(&child.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "re-scope child to accepted current replacement"]);
    check_change(root, Some(&child.id)).unwrap();
    record_scoped_review(root, &child.id, "Synthetic implementation reviewer".into()).unwrap();
    finalize_change(root, &child.id).unwrap();
    assert_eq!(load_change(root, &child.id).unwrap().state, ChangeState::Archived);
    assert!(validate_retirement(root, &load_change(root, &old.id).unwrap()).is_ok());
}

#[test]
fn recovery_retirement_requires_authority_and_current_reviewed_replacement_atomically() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = completed_no_spec_current_record(root);
    let dir = find_change_dir(root, &old.id).unwrap();
    let before = archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
    for (actor, source, replacement_id, expected) in [
        ("", "Synthetic authority", replacement.id.as_str(), "--actor"),
        ("Synthetic actor", "", replacement.id.as_str(), "--authority-source"),
        ("Synthetic actor", "Synthetic authority", "missing", "not found"),
        ("Synthetic actor", "Synthetic authority", replacement.id.as_str(), "accepted"),
    ] {
        let err = retire_change(root, &old.id, actor.into(), "Obsolete".into(),
            replacement_id.into(), source.into()).unwrap_err();
        assert!(err.contains(expected), "{expected}: {err}");
        assert_eq!(archive_workspace_snapshot(root, &dir,
            &portable_project_path(root, &dir)).unwrap(), before);
    }
}

#[test]
fn recovery_retirement_refuses_changed_reopened_definition() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let mut modified = load_change(root, &old.id).unwrap();
    modified.acceptance_criteria.push("Forged scope".into());
    save_change(root, &modified).unwrap();
    let dir = find_change_dir(root, &old.id).unwrap();
    let before = archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
    let error = retire_change(root, &old.id, "Synthetic actor".into(), "Obsolete".into(),
        replacement.id, "Synthetic authority".into()).unwrap_err();
    assert!(error.contains("definition") || error.contains("approval"), "{error}");
    assert_eq!(archive_workspace_snapshot(root, &dir,
        &portable_project_path(root, &dir)).unwrap(), before);
}

#[test]
fn recovery_frozen_added_conflict_preserves_newer_canonical_bytes_and_names_hold() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let mut applied = load_change(root, &record.id).unwrap();
    applied.canonical_applied = true;
    save_change(root, &applied).unwrap();
    // A recorded reopen is present, so mutable "use MODIFIED" is not legitimate guidance.
    let mut ledger = load_approvals(root, &applied).unwrap();
    ledger.reopenings.push(ReopenRecord {
        schema_version: 1, change_id: record.id.clone(), actor: "Synthetic actor".into(),
        reason: "Synthetic frozen-conflict diagnostic only".into(), timestamp: now(),
        from_state: ChangeState::Accepted, to_state: ChangeState::Verifying,
        superseded_approval: ledger.approvals.last().unwrap().clone(),
        prior_verification: VerificationRecord {
            ..verify_change(root, &record.id).unwrap()
        },
        stale_acceptance_input_digest: "0".repeat(64), current_acceptance_input_digest: "1".repeat(64),
        stale_evidence_cause: None, recorded_review_mismatch_anchor: None,
    });
    write_json(&change_dir(root, &record.id).join("approvals.json"), &ledger).unwrap();
    let p = requirements_path(root);
    let newer = fs::read_to_string(&p).unwrap().replace(
        "The system SHALL keep a session alive across a refresh.",
        "The system SHALL preserve the later approved reader contract.");
    fs::write(&p, &newer).unwrap();
    let err = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(err.contains("frozen") && err.contains("REQ-auth-002"), "{err}");
    assert!(!err.contains("use MODIFIED"), "{err}");
    assert_eq!(fs::read_to_string(&p).unwrap(), newer);
}

#[test]
fn recovery_retirement_committed_disposition_cannot_be_resigned_by_editing_current_json() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(root, &old.id, "Synthetic operator".into(), "Obsolete intent".into(),
        replacement.id, "Synthetic authority".into()).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "anchor immutable nonacceptance disposition"]);
    let path = find_change_dir(root, &old.id).unwrap().join(RETIREMENT_FILE);
    let original = fs::read(&path).unwrap();
    let mut receipt: RetirementRecord = serde_json::from_slice(&original).unwrap();
    receipt.actor = "Forged replacement actor".into();
    receipt.reason = "Forged replacement reason".into();
    receipt.digest = retirement_digest(&receipt).unwrap();
    fs::write(&path, json_content(&receipt).unwrap()).unwrap();
    let error = validate_retirement(root, &retired).unwrap_err();
    assert!(error.contains("committed") || error.contains("history"), "{error}");
    fs::write(&path, original).unwrap();
    assert!(validate_retirement(root, &retired).is_ok());
}

#[test]
fn recovery_retirement_retries_an_interrupted_archive_move_without_reacceptance() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(root, &old.id, "Synthetic operator".into(), "Obsolete intent".into(),
        replacement.id.clone(), "Synthetic authority".into()).unwrap();
    let destination = find_change_dir(root, &old.id).unwrap();
    let active = change_dir(root, &old.id);
    // Exact crash boundary: state and receipt persisted, native package move not completed.
    fs::rename(&destination, &active).unwrap();
    let before = archive_workspace_snapshot(root, &active, &portable_project_path(root, &active)).unwrap();
    let retried = retire_change(root, &old.id, "Synthetic operator".into(), "Obsolete intent".into(),
        replacement.id, "Synthetic authority".into()).unwrap();
    assert_eq!(retried.state, retired.state);
    assert!(!active.exists(), "retry must finish the interrupted move");
    assert_eq!(find_change_dir(root, &old.id).unwrap(), destination);
    assert_eq!(archive_workspace_snapshot(root, &destination,
        &portable_project_path(root, &destination)).unwrap(), before);
}
