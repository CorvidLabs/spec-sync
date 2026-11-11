// Synthetic fixtures only: these actor labels are not real human acceptance.
fn retirement_reopened_fixture() -> (TempDir, ChangeRecord) {
    let (temp, old, _) = recorded_review_recovery_fixture(true);
    let root = temp.path();
    let reopened = reopen_change(
        root,
        &old.id,
        "Synthetic recovery actor".into(),
        "Current intent supersedes the obsolete criterion".into(),
    )
    .unwrap()
    .change;
    fs::write(
        change_dir(root, &old.id).join("obsolete-criterion-result.json"),
        r#"{"criterion":"Dormant Hatch","passed":false,"authority":"synthetic"}"#,
    )
    .unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "retain reopened obsolete failure"]);
    (temp, reopened)
}

fn retirement_current_replacement(root: &Path) -> ChangeRecord {
    retirement_current_replacement_scoped(root, None, false)
}

fn retirement_current_replacement_scoped(
    root: &Path,
    paths: Option<&str>,
    other_owner: bool,
) -> ChangeRecord {
    let mut replacement = completed_no_spec_current_record(root);
    if other_owner {
        let target = root.join("specs/other");
        fs::create_dir_all(&target).unwrap();
        let content = fs::read_to_string(root.join("specs/change/change.spec.md"))
            .unwrap()
            .replace("module: change", "module: other");
        fs::write(target.join("other.spec.md"), content).unwrap();
        replacement = answer_question(root, &replacement.id, "affected_specs", "other").unwrap();
    }
    if let Some(paths) = paths {
        replacement = answer_question(root, &replacement.id, "affected_paths", paths).unwrap();
    }
    approve_definition(
        root,
        &replacement.id,
        Some("Synthetic scope actor".into()),
        None,
    )
    .unwrap();
    check_change(root, Some(&replacement.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "current replacement implementation"],
    );
    check_change(root, Some(&replacement.id)).unwrap();
    record_scoped_review(
        root,
        &replacement.id,
        "Synthetic implementation reviewer".into(),
    )
    .unwrap();
    finalize_change(root, &replacement.id).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &[
            "commit",
            "-m",
            "archive genuinely checked synthetic replacement",
        ],
    );
    load_change(root, &replacement.id).unwrap()
}

#[test]
fn recovery_retirement_positive_custody_and_child_route() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let old_dir = find_change_dir(root, &old.id).unwrap();
    let before =
        archive_workspace_snapshot(root, &old_dir, &portable_project_path(root, &old_dir)).unwrap();
    let refused = retire_change(
        root,
        &old.id,
        "Synthetic\noperator".into(),
        "Obsolete Hatch intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic test authority, not a real human decision".into(),
    )
    .unwrap_err();
    assert!(refused.contains("single line"), "{refused}");
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete Hatch intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic test authority, not a real human decision".into(),
    )
    .unwrap();
    assert_eq!(retired.state.as_str(), "retired");
    let retired_dir = find_change_dir(root, &old.id).unwrap();
    let after = archive_workspace_snapshot(
        root,
        &retired_dir,
        &portable_project_path(root, &retired_dir),
    )
    .unwrap();
    for (path, bytes) in &before {
        if path != "state.json" {
            assert_eq!(after.get(path), Some(bytes), "original {path} changed");
        }
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(retired_dir.join("retirement.json")).unwrap()).unwrap();
    assert_eq!(receipt["implementation_accepted"], false);
    assert_eq!(receipt["historical_transition_status"], "unproven");
    assert_eq!(
        receipt["original_state"].as_str().unwrap().as_bytes(),
        before["state.json"].1.as_slice()
    );
    assert!(
        !serde_json::from_slice::<serde_json::Value>(&after["obsolete-criterion-result.json"].1)
            .unwrap()["passed"]
            .as_bool()
            .unwrap()
    );
    let prior = load_approvals(root, &retired)
        .unwrap()
        .reopenings
        .last()
        .unwrap()
        .prior_verification
        .acceptance_manifest
        .clone()
        .unwrap();
    assert_eq!(
        receipt["custody"].as_array().unwrap().len(),
        prior.entries.len(),
        "all formerly accepted input obligations remain in custody"
    );
    assert!(validate_retirement(root, &retired).is_ok());
    let repeated = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete Hatch intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic test authority, not a real human decision".into(),
    )
    .unwrap();
    assert_eq!(repeated.state.as_str(), "retired");
    assert_eq!(
        archive_workspace_snapshot(
            root,
            &retired_dir,
            &portable_project_path(root, &retired_dir)
        )
        .unwrap(),
        after
    );

    let child = completed_no_spec_current_record(root);
    add_dependency(root, &child.id, &old.id).unwrap();
    approve_definition(root, &child.id, Some("Synthetic scope actor".into()), None).unwrap();
    let refused = start_implementation(root, &child.id).unwrap_err();
    assert!(refused.contains("must be accepted"), "{refused}");
    send_back_change(
        root,
        &child.id,
        "Synthetic operator".into(),
        "Bind the real accepted replacement instead of obsolete intent".into(),
    )
    .unwrap();
    answer_question(root, &child.id, "dependencies", &replacement.id).unwrap();
    approve_definition(root, &child.id, Some("Synthetic scope actor".into()), None).unwrap();
    check_change(root, Some(&child.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &[
            "commit",
            "-m",
            "re-scope child to accepted current replacement",
        ],
    );
    check_change(root, Some(&child.id)).unwrap();
    record_scoped_review(root, &child.id, "Synthetic implementation reviewer".into()).unwrap();
    let retained = find_change_dir(root, &old.id)
        .unwrap()
        .join("obsolete-criterion-result.json");
    let retained_bytes = fs::read(&retained).unwrap();
    fs::write(&retained, "tampered retained failure").unwrap();
    assert!(
        !audit_project_with_archives(root, true).errors.is_empty(),
        "explicit archive audit must refuse corrupted custody"
    );
    let refused = finalize_change(root, &child.id).unwrap_err();
    assert!(
        refused.contains("retirement") || refused.contains("original"),
        "{refused}"
    );
    fs::write(&retained, retained_bytes).unwrap();
    finalize_change(root, &child.id).unwrap();
    assert_eq!(
        load_change(root, &child.id).unwrap().state,
        ChangeState::Archived
    );
    assert!(validate_retirement(root, &load_change(root, &old.id).unwrap()).is_ok());
    if let Ok(destination) = std::env::var("SPECSYNC_RECOVERY_FIXTURE_OUT") {
        recorded_review_recovery_git(root, &["add", "."]);
        recorded_review_recovery_git(root, &["commit", "-m", "synthetic qualification archive"]);
        let status = Command::new("/bin/cp")
            .args(["-R", root.to_str().unwrap(), &destination])
            .status()
            .unwrap();
        assert!(
            status.success(),
            "export synthetic fixture for actual reader qualification"
        );
    }
}

#[test]
fn recovery_retirement_requires_authority_and_current_reviewed_replacement_atomically() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = completed_no_spec_current_record(root);
    let dir = find_change_dir(root, &old.id).unwrap();
    let before =
        archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
    for (actor, source, replacement_id, expected) in [
        (
            "",
            "Synthetic authority",
            replacement.id.as_str(),
            "--actor",
        ),
        (
            "Synthetic actor",
            "",
            replacement.id.as_str(),
            "--authority-source",
        ),
        (
            "Synthetic actor",
            "Synthetic authority",
            "missing",
            "not found",
        ),
        (
            "Synthetic actor",
            "Synthetic authority",
            replacement.id.as_str(),
            "accepted",
        ),
    ] {
        let err = retire_change(
            root,
            &old.id,
            actor.into(),
            "Obsolete".into(),
            replacement_id.into(),
            source.into(),
        )
        .unwrap_err();
        assert!(err.contains(expected), "{expected}: {err}");
        assert_eq!(
            archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap(),
            before
        );
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
    let before =
        archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
    let error = retire_change(
        root,
        &old.id,
        "Synthetic actor".into(),
        "Obsolete".into(),
        replacement.id,
        "Synthetic authority".into(),
    )
    .unwrap_err();
    assert!(
        error.contains("definition") || error.contains("approval"),
        "{error}"
    );
    assert_eq!(
        archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap(),
        before
    );
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
        schema_version: 1,
        change_id: record.id.clone(),
        actor: "Synthetic actor".into(),
        reason: "Synthetic frozen-conflict diagnostic only".into(),
        timestamp: now(),
        from_state: ChangeState::Accepted,
        to_state: ChangeState::Verifying,
        superseded_approval: ledger.approvals.last().unwrap().clone(),
        prior_verification: VerificationRecord {
            ..verify_change(root, &record.id).unwrap()
        },
        stale_acceptance_input_digest: "0".repeat(64),
        current_acceptance_input_digest: "1".repeat(64),
        stale_evidence_cause: None,
        recorded_review_mismatch_anchor: None,
    });
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    let p = requirements_path(root);
    let newer = fs::read_to_string(&p).unwrap().replace(
        "The system SHALL keep a session alive across a refresh.",
        "The system SHALL preserve the later approved reader contract.",
    );
    fs::write(&p, &newer).unwrap();
    let err = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        err.contains("frozen") && err.contains("REQ-auth-002"),
        "{err}"
    );
    assert!(!err.contains("use MODIFIED"), "{err}");
    assert_eq!(fs::read_to_string(&p).unwrap(), newer);
}

#[test]
fn recovery_retirement_committed_disposition_cannot_be_resigned_by_editing_current_json() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id,
        "Synthetic authority".into(),
    )
    .unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "anchor immutable nonacceptance disposition"],
    );
    let path = find_change_dir(root, &old.id)
        .unwrap()
        .join(RETIREMENT_FILE);
    let original = fs::read(&path).unwrap();
    let mut receipt: RetirementRecord = serde_json::from_slice(&original).unwrap();
    receipt.actor = "Forged replacement actor".into();
    receipt.reason = "Forged replacement reason".into();
    receipt.digest = retirement_digest(&receipt).unwrap();
    fs::write(&path, json_content(&receipt).unwrap()).unwrap();
    let error = validate_retirement(root, &retired).unwrap_err();
    assert!(
        error.contains("committed") || error.contains("history"),
        "{error}"
    );
    fs::write(&path, original).unwrap();
    assert!(validate_retirement(root, &retired).is_ok());
}

#[test]
fn recovery_retirement_retries_an_interrupted_archive_move_without_reacceptance() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    )
    .unwrap();
    let destination = find_change_dir(root, &old.id).unwrap();
    let active = change_dir(root, &old.id);
    // Exact crash boundary: state and receipt persisted, native package move not completed.
    fs::rename(&destination, &active).unwrap();
    let before =
        archive_workspace_snapshot(root, &active, &portable_project_path(root, &active)).unwrap();
    let retried = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id,
        "Synthetic authority".into(),
    )
    .unwrap();
    assert_eq!(retried.state, retired.state);
    assert!(!active.exists(), "retry must finish the interrupted move");
    assert_eq!(find_change_dir(root, &old.id).unwrap(), destination);
    assert_eq!(
        archive_workspace_snapshot(
            root,
            &destination,
            &portable_project_path(root, &destination)
        )
        .unwrap(),
        before
    );
}

#[test]
fn recovery_retirement_rename_failure_keeps_a_complete_resumable_receipt() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let dir = find_change_dir(root, &old.id).unwrap();
    let before =
        archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
    RETIREMENT_TEST_FAIL_MOVE.with(|value| value.set(true));
    let attempted = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    );
    RETIREMENT_TEST_FAIL_MOVE.with(|value| value.set(false));
    let error = attempted.unwrap_err();
    assert!(
        error.contains("pending") || error.contains("incomplete"),
        "{error}"
    );
    assert!(
        dir.join(RETIREMENT_FILE).is_file(),
        "a complete receipt must survive the interrupted move"
    );
    let receipt: RetirementRecord =
        serde_json::from_slice(&fs::read(dir.join(RETIREMENT_FILE)).unwrap()).unwrap();
    assert_eq!(
        receipt.original_state.as_bytes(),
        before["state.json"].1.as_slice()
    );
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete".into(),
        replacement.id,
        "Synthetic authority".into(),
    )
    .unwrap();
    let archived = find_change_dir(root, &old.id).unwrap();
    assert!(!dir.exists());
    let after =
        archive_workspace_snapshot(root, &archived, &portable_project_path(root, &archived))
            .unwrap();
    for (path, entry) in before
        .iter()
        .filter(|(path, _)| path.as_str() != "state.json")
    {
        assert_eq!(after.get(path), Some(entry), "{path} bytes/mode changed");
    }
    assert!(validate_retirement(root, &retired).is_ok());
}

#[test]
fn recovery_retirement_resumes_the_prior_failed_rollback_crash_boundary() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    )
    .unwrap();
    let archived = find_change_dir(root, &old.id).unwrap();
    let complete =
        archive_workspace_snapshot(root, &archived, &portable_project_path(root, &archived))
            .unwrap();
    let receipt: RetirementRecord =
        serde_json::from_slice(&fs::read(archived.join(RETIREMENT_FILE)).unwrap()).unwrap();
    let active = change_dir(root, &old.id);
    fs::rename(&archived, &active).unwrap();
    // Model the reviewed ee5e failure: old state restored, receipt removal interrupted.
    fs::write(active.join("state.json"), &receipt.original_state).unwrap();
    retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete".into(),
        replacement.id,
        "Synthetic authority".into(),
    )
    .unwrap();
    assert!(
        !active.exists(),
        "the recorded disposition must have a supported retry"
    );
    assert_eq!(
        archive_workspace_snapshot(root, &archived, &portable_project_path(root, &archived))
            .unwrap(),
        complete
    );
}

#[test]
fn recovery_retirement_rejects_stale_unreviewed_circular_and_unknown_inputs_without_writes() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let old_dir = find_change_dir(root, &old.id).unwrap();
    let replacement_dir = find_change_dir(root, &replacement.id).unwrap();
    let ledger_path = old_dir.join("approvals.json");
    let ledger_bytes = fs::read(&ledger_path).unwrap();
    let state_path = old_dir.join("state.json");
    let state_bytes = fs::read(&state_path).unwrap();
    let replacement_state = replacement_dir.join("state.json");
    let replacement_state_bytes = fs::read(&replacement_state).unwrap();
    let review = replacement_dir.join("review.json");
    let review_bytes = fs::read(&review).unwrap();
    let source = root.join("src/lib.rs");
    let source_bytes = fs::read(&source).unwrap();
    for attack in [
        "stale",
        "unreviewed",
        "cycle",
        "unknown-format",
        "unknown-anchor",
    ] {
        match attack {
            "stale" => fs::write(&source, "// unaccepted later delivery\n").unwrap(),
            "unreviewed" => fs::remove_file(&review).unwrap(),
            "cycle" => {
                let mut forged = load_change(root, &replacement.id).unwrap();
                forged.dependencies.push(old.id.clone());
                fs::write(&replacement_state, json_content(&forged).unwrap()).unwrap();
            }
            "unknown-format" => {
                let mut forged: serde_json::Value = serde_json::from_slice(&state_bytes).unwrap();
                forged["workflow_version"] = serde_json::json!(99);
                fs::write(&state_path, serde_json::to_vec_pretty(&forged).unwrap()).unwrap();
            }
            "unknown-anchor" => {
                let mut forged: serde_json::Value = serde_json::from_slice(&ledger_bytes).unwrap();
                forged["reopenings"][0]["recorded_review_mismatch_anchor"] =
                    serde_json::json!("0".repeat(40));
                fs::write(&ledger_path, serde_json::to_vec_pretty(&forged).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        let before =
            archive_workspace_snapshot(root, &old_dir, &portable_project_path(root, &old_dir))
                .unwrap();
        let error = retire_change(
            root,
            &old.id,
            "Synthetic operator".into(),
            "Obsolete".into(),
            replacement.id.clone(),
            "Synthetic authority".into(),
        )
        .unwrap_err();
        assert!(
            !error.is_empty(),
            "{attack} must refuse with an actual reason"
        );
        if attack == "cycle" {
            assert!(error.contains("depend"), "{error}");
        }
        assert_eq!(
            archive_workspace_snapshot(root, &old_dir, &portable_project_path(root, &old_dir))
                .unwrap(),
            before,
            "{attack} mutated old evidence"
        );
        fs::write(&source, &source_bytes).unwrap();
        fs::write(&review, &review_bytes).unwrap();
        fs::write(&replacement_state, &replacement_state_bytes).unwrap();
        fs::write(&state_path, &state_bytes).unwrap();
        fs::write(&ledger_path, &ledger_bytes).unwrap();
    }
}

#[test]
fn recovery_retirement_requires_each_real_signed_input_and_compatible_owner() {
    for different_owner in [false, true] {
        let (temp, old) = retirement_reopened_fixture();
        let root = temp.path();
        let replacement = retirement_current_replacement_scoped(
            root,
            if different_owner {
                Some("src/,specs/change/change.spec.md")
            } else {
                Some("README.md")
            },
            different_owner,
        );
        let dir = find_change_dir(root, &old.id).unwrap();
        let before =
            archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap();
        let error = retire_change(
            root,
            &old.id,
            "Synthetic operator".into(),
            "Obsolete".into(),
            replacement.id,
            "Synthetic authority".into(),
        )
        .unwrap_err();
        assert!(
            error.contains(if different_owner {
                "signed owners"
            } else {
                "no custody"
            }),
            "{error}"
        );
        assert_eq!(
            archive_workspace_snapshot(root, &dir, &portable_project_path(root, &dir)).unwrap(),
            before
        );
    }
}

fn retirement_dirty_accepted_reopening_fixture() -> (TempDir, ChangeRecord, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    recorded_review_recovery_git(root, &["config", "user.email", "synthetic@example.com"]);
    recorded_review_recovery_git(root, &["config", "user.name", "Synthetic fixture"]);
    fs::write(root.join("README.md"), "Synthetic base\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "synthetic base"]);
    recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let record = completed_no_spec_current_record(root);
    approve_definition(root, &record.id, Some("Synthetic scope actor".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "synthetic implementation, still verifying"],
    );
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Synthetic reviewer".into()).unwrap();
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    let failed = archive_change_with_same_pr_finalize_failure(root, &record.id).unwrap_err();
    assert!(
        failed.contains("forced") || failed.contains("failure"),
        "{failed}"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
    assert!(
        accepted_recording_anchors(root, &record)
            .unwrap()
            .is_empty()
    );
    fs::write(
        root.join("src/lib.rs"),
        "// later current delivery\npub fn ready() -> bool { true }\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &record.id,
        "Synthetic operator".into(),
        "Retain a real dirty-Accepted failed archive as explicitly unproven history".into(),
    )
    .unwrap()
    .change;
    let opening = load_approvals(root, &reopened)
        .unwrap()
        .reopenings
        .last()
        .unwrap()
        .clone();
    assert!(opening.recorded_review_mismatch_anchor.is_none());
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &[
            "commit",
            "-m",
            "commit actual audited reopening, never Accepted",
        ],
    );
    let anchor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    assert!(
        accepted_recording_anchors(root, &reopened)
            .unwrap()
            .is_empty()
    );
    (temp, reopened, anchor)
}

#[test]
fn recovery_refinement_explicit_committed_reopening_retains_unproven_source_without_accepted_snapshot()
 {
    let (temp, original, anchor) = retirement_dirty_accepted_reopening_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let source = find_change_dir(root, &original.id).unwrap();
    let before =
        archive_workspace_snapshot(root, &source, &portable_project_path(root, &source)).unwrap();
    let retired = retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap();
    let directory = find_change_dir(root, &original.id).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join(RETIREMENT_FILE)).unwrap()).unwrap();
    assert_eq!(receipt["schema_version"], 2);
    assert_eq!(
        receipt["source_kind"],
        "committed-audited-reopening-unproven"
    );
    assert_eq!(receipt["source_anchor"], anchor);
    assert_eq!(
        receipt["source_directory"],
        format!("{CHANGES_PATH}/{}", original.id)
    );
    assert_eq!(receipt["implementation_accepted"], false);
    assert_eq!(receipt["historical_transition_status"], "unproven");
    let after =
        archive_workspace_snapshot(root, &directory, &portable_project_path(root, &directory))
            .unwrap();
    for (path, entry) in before
        .iter()
        .filter(|(path, _)| path.as_str() != "state.json")
    {
        assert_eq!(after.get(path), Some(entry), "{path} was rewritten");
    }
    assert!(validate_retirement(root, &retired).is_ok());
    assert!(
        load_approvals(root, &retired)
            .unwrap()
            .reopenings
            .last()
            .unwrap()
            .recorded_review_mismatch_anchor
            .is_none()
    );
    retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id,
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap();
}

#[test]
fn recovery_refinement_committed_mode_never_downgrades_an_existing_accepted_anchor() {
    let (temp, original) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let directory = find_change_dir(root, &original.id).unwrap();
    let before =
        archive_workspace_snapshot(root, &directory, &portable_project_path(root, &directory))
            .unwrap();
    let error = retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Must not downgrade corrupt accepted history".into(),
        replacement.id,
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap_err();
    assert!(
        error.contains("accepted") || error.contains("mismatch"),
        "{error}"
    );
    assert_eq!(
        archive_workspace_snapshot(root, &directory, &portable_project_path(root, &directory))
            .unwrap(),
        before
    );
}

fn retirement_policy_entry(
    path: &str,
    kind: AcceptanceInputKind,
    mode: u32,
    payload: &[u8],
    owners: &[&str],
) -> AcceptanceInputEntryV1 {
    let payload_digest = sha256_hex(payload);
    AcceptanceInputEntryV1 {
        path: path.into(),
        entry_digest: acceptance_entry_digest(path, &kind, mode, &payload_digest),
        kind,
        mode,
        payload_digest,
        owners: owners.iter().map(|s| (*s).into()).collect(),
    }
}

#[test]
fn recovery_refinement_owner_variant_is_unchanged_exact_ordinary_file_only() {
    for path in ["README.md", "src/pages/docs.astro"] {
        let old = retirement_policy_entry(
            path,
            AcceptanceInputKind::File,
            0o100644,
            b"unchanged",
            &["@exact:delivery"],
        );
        let mut current = old.clone();
        current.owners = vec!["studio".into()];
        assert_eq!(
            retirement_entry_classification(&old, &current).unwrap(),
            RetirementCustodyClassification::UnchangedExactFileCurrentOwner
        );
        for attack in [
            "kind",
            "mode",
            "payload",
            "entry",
            "reserved",
            "mixed",
            "empty",
            "named-prior",
            "test-prior",
        ] {
            let mut old = old.clone();
            let mut next = current.clone();
            match attack {
                "kind" => next.kind = AcceptanceInputKind::Missing,
                "mode" => next.mode = 0o100755,
                "payload" => next.payload_digest = sha256_hex(b"changed"),
                "entry" => next.entry_digest = "0".repeat(64),
                "reserved" => next.owners = vec!["@exact:test".into()],
                "mixed" => next.owners.push("@exact:delivery".into()),
                "empty" => next.owners.clear(),
                "named-prior" => old.owners = vec!["old-module".into()],
                "test-prior" => old.owners = vec!["@exact:test".into()],
                _ => unreachable!(),
            }
            assert!(
                retirement_entry_classification(&old, &next).is_err(),
                "{path}: {attack}"
            );
        }
    }
    let old = retirement_policy_entry(
        ".chough/session.json",
        AcceptanceInputKind::File,
        0o100644,
        b"old",
        &["@exact:delivery"],
    );
    let next = retirement_policy_entry(
        ".chough/session.json",
        AcceptanceInputKind::Missing,
        0,
        b"",
        &["studio"],
    );
    assert!(
        retirement_entry_classification(&old, &next).is_err(),
        "File->Missing must not use owner exception"
    );
}

#[test]
fn recovery_refinement_committed_source_rejects_incomplete_or_changed_packets_without_writes() {
    let (temp, original, _) = retirement_dirty_accepted_reopening_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let directory = find_change_dir(root, &original.id).unwrap();
    let relative = portable_project_path(root, &directory);
    let pristine = archive_workspace_snapshot(root, &directory, &relative).unwrap();
    for attack in [
        "missing-ledger",
        "missing-finalization",
        "missing-review",
        "missing-manifest",
        "missing-closing",
        "extra-uncommitted-file",
    ] {
        match attack {
            "missing-ledger" => fs::remove_file(directory.join("approvals.json")).unwrap(),
            "missing-finalization" => fs::remove_file(directory.join("finalization.json")).unwrap(),
            "missing-review" => fs::remove_file(directory.join("review.json")).unwrap(),
            "missing-manifest" | "missing-closing" => {
                let mut ledger: serde_json::Value =
                    serde_json::from_slice(&pristine["approvals.json"].1).unwrap();
                if attack == "missing-manifest" {
                    ledger["reopenings"][0]["prior_verification"]["acceptance_manifest"] =
                        serde_json::Value::Null;
                } else {
                    ledger["approvals"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|a| a["gate"] != "finalization" && a["gate"] != "acceptance");
                }
                fs::write(
                    directory.join("approvals.json"),
                    serde_json::to_vec_pretty(&ledger).unwrap(),
                )
                .unwrap();
            }
            "extra-uncommitted-file" => {
                fs::write(directory.join("never-committed.md"), "Not an anchor").unwrap()
            }
            _ => unreachable!(),
        }
        let before = archive_workspace_snapshot(root, &directory, &relative).unwrap();
        assert!(
            retire_change_with_source(
                root,
                &original.id,
                "Synthetic operator".into(),
                "Obsolete".into(),
                replacement.id.clone(),
                "Synthetic authority".into(),
                RetirementSourceKind::CommittedAuditedReopeningUnproven
            )
            .is_err(),
            "{attack}"
        );
        assert_eq!(
            archive_workspace_snapshot(root, &directory, &relative).unwrap(),
            before,
            "{attack}"
        );
        if directory.join("never-committed.md").exists() {
            fs::remove_file(directory.join("never-committed.md")).unwrap();
        }
        for (path, (_, bytes)) in &pristine {
            fs::write(directory.join(path), bytes).unwrap();
        }
    }
}

#[test]
fn recovery_refinement_recomputed_unproven_receipt_cannot_use_missing_or_nonancestor_anchor() {
    let (temp, original, _) = retirement_dirty_accepted_reopening_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Obsolete".into(),
        replacement.id,
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap();
    let path = find_change_dir(root, &original.id)
        .unwrap()
        .join(RETIREMENT_FILE);
    let original_bytes = fs::read(&path).unwrap();
    let tree = git_output(root, &["rev-parse", "HEAD^{tree}"]).unwrap();
    let unrelated = git_output(
        root,
        &["commit-tree", &tree, "-m", "synthetic unrelated root"],
    )
    .unwrap();
    for anchor in ["0".repeat(40), unrelated] {
        let mut receipt: RetirementRecord = serde_json::from_slice(&original_bytes).unwrap();
        receipt.source_anchor = anchor;
        receipt.digest = retirement_digest(&receipt).unwrap();
        fs::write(&path, json_content(&receipt).unwrap()).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(validate_retirement(root, &retired).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    fs::write(&path, original_bytes).unwrap();
    assert!(validate_retirement(root, &retired).is_ok());
}

#[test]
fn recovery_refinement_legacy_receipt_keeps_its_original_canonical_digest() {
    let raw = include_bytes!("fixtures/retirement-v1.json");
    let receipt: RetirementRecord = serde_json::from_slice(raw).unwrap();
    assert_eq!(receipt.schema_version, 1);
    assert!(receipt.source_kind.is_none() && receipt.source_directory.is_none());
    assert!(
        receipt
            .custody
            .iter()
            .all(|entry| entry.classification.is_none())
    );
    assert_eq!(
        retirement_digest(&receipt).unwrap(),
        receipt.digest,
        "new optional fields must not reinterpret the original v1 preimage"
    );
}

#[test]
fn recovery_retirement_history_distinguishes_unborn_from_unreadable_head() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    assert!(
        retirement_history(root).unwrap().is_empty(),
        "a valid new repository cannot have a committed retirement event"
    );
    let reference = root.join(".git/refs/heads/main");
    fs::write(&reference, format!("{}\n", "0".repeat(40))).unwrap();
    assert!(
        retirement_history(root).is_err(),
        "an existing corrupt branch ref is unavailable history, not an unborn repository"
    );
    fs::remove_file(&reference).unwrap();
    recorded_review_recovery_git(root, &["config", "user.name", "Synthetic test"]);
    recorded_review_recovery_git(root, &["config", "user.email", "test@example.invalid"]);
    recorded_review_recovery_git(
        root,
        &["commit", "--allow-empty", "-m", "committed history"],
    );
    let commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let object = root
        .join(".git/objects")
        .join(&commit[..2])
        .join(&commit[2..]);
    assert!(
        object.is_file(),
        "fixture must remove a real committed object"
    );
    fs::remove_file(object).unwrap();
    assert!(
        retirement_history(root).is_err(),
        "a missing committed object must never be treated as unborn or empty history"
    );
}

/// The ordinary happy path: `finalize` accepts and archives in one command, so a change
/// reopened afterwards legitimately records `from_state: archived` (PR #1018 review).
fn retirement_archived_origin_reopening_fixture() -> (TempDir, ChangeRecord, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    recorded_review_recovery_git(root, &["config", "user.email", "synthetic@example.com"]);
    recorded_review_recovery_git(root, &["config", "user.name", "Synthetic fixture"]);
    fs::write(root.join("README.md"), "Synthetic base\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "synthetic base"]);
    recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let record = completed_no_spec_current_record(root);
    approve_definition(root, &record.id, Some("Synthetic scope actor".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "synthetic implementation"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Synthetic reviewer".into()).unwrap();
    finalize_change(root, &record.id).unwrap();
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Archived
    );
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "archive the finalized change"]);
    // The accepted evidence goes stale: an accepted input moves after archival.
    fs::write(
        root.join("src/lib.rs"),
        "// later current delivery\npub fn ready() -> bool { true }\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &record.id,
        "Synthetic operator".into(),
        "Recover the archived obsolete intent for retirement".into(),
    )
    .unwrap()
    .change;
    assert_eq!(reopened.state, ChangeState::Verifying);
    let opening = load_approvals(root, &reopened)
        .unwrap()
        .reopenings
        .last()
        .unwrap()
        .clone();
    assert_eq!(opening.from_state, ChangeState::Archived);
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "commit archived-origin audited reopening"],
    );
    let anchor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    (temp, reopened, anchor)
}

#[test]
fn recovery_retirement_reopening_from_the_archive_retires_through_committed_custody() {
    let (temp, original, anchor) = retirement_archived_origin_reopening_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap();
    assert_eq!(retired.state, ChangeState::Retired);
    let directory = find_change_dir(root, &original.id).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join(RETIREMENT_FILE)).unwrap()).unwrap();
    assert_eq!(receipt["source_anchor"], anchor);
    assert_eq!(receipt["implementation_accepted"], false);
    assert!(validate_retirement(root, &retired).is_ok());
}

#[test]
fn recovery_retirement_refuses_a_reopening_that_proves_no_staleness() {
    // The dirty fixture's reopening is drift-driven (digests differ) with no
    // review-mismatch anchor, so equalizing the digests leaves a record no
    // `reopen_change` run could have produced.
    let (temp, original, _anchor) = retirement_dirty_accepted_reopening_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    // Hand-edit the recorded reopening so the stale and current digests agree with no
    // stale-evidence cause: no `reopen_change` run produces that, so the record cannot
    // be audited reopening evidence and retirement must refuse before building custody.
    let path = find_change_dir(root, &original.id)
        .unwrap()
        .join("approvals.json");
    let mut ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let reopening = ledger["reopenings"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    let stale = reopening["stale_acceptance_input_digest"]
        .as_str()
        .unwrap()
        .to_string();
    reopening["current_acceptance_input_digest"] = serde_json::Value::String(stale);
    reopening["stale_evidence_cause"] = serde_json::Value::Null;
    fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
    let error = retire_change_with_source(
        root,
        &original.id,
        "Synthetic operator".into(),
        "Obsolete intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
        RetirementSourceKind::CommittedAuditedReopeningUnproven,
    )
    .unwrap_err();
    assert!(
        error.contains("intact audited reopening evidence"),
        "{error}"
    );
}

#[test]
fn recovery_retirement_history_refuses_a_shallow_clone_boundary() {
    // A depth-limited clone must not pass retirement custody off an unreadable
    // walk. The `-G` walk's boundary commit diffs against the empty tree, so any
    // receipt present at the tip still matches; what must fail closed is custody
    // validation and deleted-working-tree archives.
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    )
    .unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "commit the retirement"]);
    fs::write(root.join("README.md"), "later boundary\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "move the shallow boundary past the retirement"],
    );

    let clone_dir = temp.path().join("shallow");
    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &format!("file://{}", root.display()),
            clone_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "clone: {output:?}");

    // The retired record is present but its authenticating history sits beyond
    // the boundary: custody validation refuses.
    let records = list_all_change_metadata(&clone_dir, false).unwrap();
    assert!(
        records.values().any(|record| record.state == ChangeState::Retired),
        "fixture must hold a retired record"
    );
    let error = validate_retired_records(&clone_dir, &records).unwrap_err();
    assert!(error.contains("shallow"), "{error}");

    // Deleting the retired archive locally does not erase it either: the
    // boundary commit still carries the receipt, so the walk finds it and the
    // removal is named.
    let retired_dir = find_change_dir(&clone_dir, &old.id).unwrap();
    fs::remove_dir_all(&retired_dir).unwrap();
    let records = list_all_change_metadata(&clone_dir, false).unwrap();
    assert!(
        !records.values().any(|record| record.state == ChangeState::Retired),
        "the deletion removed the record from the working tree"
    );
    let error = validate_retired_records(&clone_dir, &records).unwrap_err();
    assert!(
        error.contains("was removed"),
        "the committed receipt outlives the local deletion: {error}"
    );

    let full_dir = temp.path().join("full");
    let output = Command::new("git")
        .args([
            "clone",
            &format!("file://{}", root.display()),
            full_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "clone: {output:?}");
    let records = list_all_change_metadata(&full_dir, false).unwrap();
    assert!(
        validate_retired_records(&full_dir, &records).is_ok(),
        "a complete clone authenticates the same custody"
    );
}

#[test]
fn recovery_refinement_same_exact_owner_requires_unchanged_bytes() {
    let prior = retirement_policy_entry(
        "src/pages/docs.astro",
        AcceptanceInputKind::File,
        0o100644,
        b"exact bytes",
        &["@exact:delivery"],
    );
    assert_eq!(
        retirement_entry_classification(&prior, &prior.clone()).unwrap(),
        RetirementCustodyClassification::SameSignedOwners,
        "identical reserved exact custody of unchanged bytes still transfers"
    );
    for attack in ["kind", "mode", "payload", "entry"] {
        let mut next = prior.clone();
        match attack {
            "kind" => next.kind = AcceptanceInputKind::Missing,
            "mode" => next.mode = 0o100755,
            "payload" => next.payload_digest = sha256_hex(b"rewritten"),
            "entry" => next.entry_digest = "0".repeat(64),
            _ => unreachable!(),
        }
        assert!(
            retirement_entry_classification(&prior, &next).is_err(),
            "reserved exact owners cannot adopt changed bytes: {attack}"
        );
    }
    // Named same-owner custody keeps the ordinary successor rule.
    let named = retirement_policy_entry(
        "src/lib.rs",
        AcceptanceInputKind::File,
        0o100644,
        b"a",
        &["studio"],
    );
    let mut renamed = named.clone();
    renamed.payload_digest = sha256_hex(b"b");
    renamed.entry_digest = "1".repeat(64);
    assert_eq!(
        retirement_entry_classification(&named, &renamed).unwrap(),
        RetirementCustodyClassification::SameSignedOwners
    );
}

#[test]
fn recovery_send_back_supersedes_a_historical_reopening_in_the_frozen_test() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_requirement_delta(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    let reopened_at = now();
    ledger.reopenings.push(ReopenRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: "Synthetic actor".into(),
        reason: "Synthetic reopen-supersession diagnostic only".into(),
        timestamp: reopened_at,
        from_state: ChangeState::Accepted,
        to_state: ChangeState::Verifying,
        superseded_approval: ledger.approvals.last().unwrap().clone(),
        prior_verification: VerificationRecord {
            ..verify_change(root, &record.id).unwrap()
        },
        stale_acceptance_input_digest: "0".repeat(64),
        current_acceptance_input_digest: "1".repeat(64),
        stale_evidence_cause: None,
        recorded_review_mismatch_anchor: None,
    });
    assert!(latest_reopening_still_freezes_canonical(&ledger));
    let send_back = |timestamp: u64| SendBackRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: "Synthetic actor".into(),
        reason: "Synthetic re-scope".into(),
        timestamp,
        from_state: ChangeState::Verifying,
        to_state: ChangeState::Draft,
        cancelled_approvals: ledger.approvals.len(),
        cancelled_review_attempts: Some(0),
        canonical_applied: true,
        applied_items: Vec::new(),
        applied_modules: Vec::new(),
    };
    // An earlier send-back does not supersede the reopening.
    ledger.send_backs.push(send_back(reopened_at - 1));
    assert!(latest_reopening_still_freezes_canonical(&ledger));
    // One recorded at the reopening's own second does: the change was re-scoped
    // and re-approved, so the historical reopening freezes nothing any more.
    ledger.send_backs.push(send_back(reopened_at));
    assert!(!latest_reopening_still_freezes_canonical(&ledger));
}

#[test]
fn recovery_retired_record_keeps_its_historical_sequence_classification() {
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    )
    .unwrap();
    let sequences = located_change_sequences(root).unwrap();
    let entry = sequences
        .iter()
        .find(|entry| entry.id == retired.id)
        .unwrap();
    assert!(
        entry.historical,
        "retirement is immutable terminal nonacceptance; the sequence stays historical"
    );
}

#[test]
fn recovery_retirement_custody_survives_audited_custodian_drift_and_reacceptance() {
    // The #1021 site shape: a retired record whose custodian's accepted inputs legitimately
    // drift after acceptance. Custody is the recorded binding, not the custodian's
    // freshness, so the record still validates and the reopen remedy stays reachable.
    let (temp, old) = retirement_reopened_fixture();
    let root = temp.path();
    let replacement = retirement_current_replacement(root);
    let retired = retire_change(
        root,
        &old.id,
        "Synthetic operator".into(),
        "Obsolete intent replaced by current intent".into(),
        replacement.id.clone(),
        "Synthetic authority".into(),
    )
    .unwrap();
    assert!(validate_retirement(root, &retired).is_ok());

    // A receipt claiming custody under owners the custodian does not sign is refused,
    // even before any committed history pins the receipt bytes.
    let dir = find_change_dir(root, &old.id).unwrap();
    let path = dir.join(RETIREMENT_FILE);
    let original_bytes = fs::read(&path).unwrap();
    let mut receipt: RetirementRecord = serde_json::from_slice(&original_bytes).unwrap();
    receipt.custody[0].replacement_owners = vec!["forged-owner".into()];
    receipt.digest = retirement_digest(&receipt).unwrap();
    fs::write(&path, json_content(&receipt).unwrap()).unwrap();
    let error = validate_retirement(root, &retired).unwrap_err();
    assert!(error.contains("custody"), "{error}");
    fs::write(&path, original_bytes).unwrap();
    assert!(validate_retirement(root, &retired).is_ok());

    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();

    assert!(
        validate_retirement(root, &retired).is_ok(),
        "custodian drift is reported on the custodian, never a custody failure"
    );
    let reopened = reopen_change(
        root,
        &replacement.id,
        "Synthetic recovery actor".into(),
        "Custody input drifted after acceptance".into(),
    )
    .unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);

    // The whole recovery window keeps loading: custody is judged against the custodian's
    // last accepted authenticated manifest while the custodian is being re-verified.
    assert!(
        validate_retirement(root, &load_change(root, &old.id).unwrap()).is_ok(),
        "custody holds while the custodian is reopened"
    );
    assert!(
        list_all_changes_uncached(root).is_ok(),
        "roster load keeps working while the custodian is reopened"
    );

    assert!(check_change(root, Some(&replacement.id)).unwrap().unwrap().passed);
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "evolve custodian delivery"]);
    check_change(root, Some(&replacement.id)).unwrap();
    record_scoped_review(
        root,
        &replacement.id,
        "Synthetic implementation reviewer".into(),
    )
    .unwrap();
    finalize_change(root, &replacement.id).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "archive evolved custodian"]);
    assert_eq!(
        load_change(root, &replacement.id).unwrap().state,
        ChangeState::Archived
    );

    // Custody survives the audited evolution: new digests, same signed owners.
    let retired = load_change(root, &old.id).unwrap();
    let custody = validate_retirement(root, &retired);
    assert!(
        custody.is_ok(),
        "custody holds across the custodian's audited re-acceptance: {}",
        custody.err().unwrap_or_default()
    );

    // The receipt is now committed history: an owner tamper is refused as edited history.
    let path = find_change_dir(root, &old.id).unwrap().join(RETIREMENT_FILE);
    let original_bytes = fs::read(&path).unwrap();
    let mut receipt: RetirementRecord = serde_json::from_slice(&original_bytes).unwrap();
    receipt.custody[0].replacement_owners = vec!["forged-owner".into()];
    receipt.digest = retirement_digest(&receipt).unwrap();
    fs::write(&path, json_content(&receipt).unwrap()).unwrap();
    assert!(
        validate_retirement(root, &retired).is_err(),
        "committed custody tamper is refused"
    );
    fs::write(&path, original_bytes).unwrap();
    assert!(validate_retirement(root, &retired).is_ok());

    // A send-back during recovery moves the custodian to Draft with cancelled approvals;
    // custody still holds from the anchored last accepted evidence.
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    reopen_change(
        root,
        &replacement.id,
        "Synthetic recovery actor".into(),
        "Custody input drifted again".into(),
    )
    .unwrap();
    send_back_change(
        root,
        &replacement.id,
        "Synthetic operator".into(),
        "Re-scope during recovery".into(),
    )
    .unwrap();
    assert_eq!(
        load_change(root, &replacement.id).unwrap().state,
        ChangeState::Draft
    );
    let custody = validate_retirement(root, &load_change(root, &old.id).unwrap());
    assert!(
        custody.is_ok(),
        "custody holds through the send-back window: {}",
        custody.err().unwrap_or_default()
    );
    assert!(
        list_all_changes_uncached(root).is_ok(),
        "roster load keeps working through the send-back window"
    );
}

