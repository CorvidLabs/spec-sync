fn historical_reconciliation_fixture() -> (TempDir, ChangeRecord, ChangeRecord) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    recorded_review_recovery_git(root, &["config", "user.email", "test@example.com"]);
    recorded_review_recovery_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("README.md"), "base\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "base"]);
    let old = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n",
    );
    approve_definition(root, &old.id, Some("Fixture scope owner".into()), None).unwrap();
    start_implementation(root, &old.id).unwrap();
    verify_change(root, &old.id).unwrap();
    let old = accept_change(root, &old.id, Some("Fixture closer".into()), None).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "record old receipt"]);
    let obligations = load_verification(root, &old)
        .unwrap()
        .acceptance_manifest
        .unwrap()
        .entries
        .into_iter()
        .filter(|e| e.owners.iter().any(|o| o == "auth") && e.path != "specs/auth/requirements.md")
        .collect::<Vec<_>>();
    fs::write(
        root.join("src/auth.rs"),
        "// Already delivered before reconciliation.\n",
    )
    .unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "earlier delivered source change"]);
    recorded_review_recovery_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let mut successor = current_workflow_record(root, completed_current_record(root));
    for artifact in &successor.selected_artifacts {
        let body = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Reconcile the exact old receipt.\n"
        } else {
            prose_or_plan(
                artifact,
                "# Evidence\n\nPreserve old evidence; do not claim historical transitions were proven.\n",
            )
        };
        fs::write(
            change_dir(root, &successor.id).join(artifact.file_name()),
            body,
        )
        .unwrap();
    }
    fs::write(delta_path(root, &successor, "auth"), "## MODIFIED\n### SPEC SECTION Invariants\n\nCurrent authentication is governed; the old transition remains unproven.\n").unwrap();
    successor
        .affected_paths
        .extend(obligations.iter().map(|e| e.path.clone()));
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for entry in obligations {
        successor = add_supersedes_obligation(
            root,
            &successor.id,
            &old.id,
            &entry.path,
            "auth",
            &entry.entry_digest,
        )
        .unwrap();
    }
    approve_definition(
        root,
        &successor.id,
        Some("Fixture scope owner".into()),
        None,
    )
    .unwrap();
    check_change(root, Some(&successor.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &["commit", "-m", "prepare present-day reconciliation"],
    );
    check_change(root, Some(&successor.id)).unwrap();
    record_scoped_review(root, &successor.id, "Fixture reviewer".into()).unwrap();
    (temp, old, successor)
}

#[test]
fn historical_reconciliation_preserves_strict_normal_failure_and_requires_fresh_gates() {
    let (temp, old, successor) = historical_reconciliation_fixture();
    let root = temp.path();
    let old_package = archive_workspace_snapshot(
        root,
        &change_dir(root, &old.id),
        &format!("{CHANGES_PATH}/{}", old.id),
    )
    .unwrap();
    let normal = finalize_change(root, &successor.id).unwrap_err();
    assert!(normal.contains("definition-signed base tree"), "{normal}");
    let refused = enable_historical_reconciliation(
        root,
        &successor.id,
        "Fixture\nmigration owner".into(),
        "Acknowledge unproven history, preserving the original receipt.".into(),
    )
    .unwrap_err();
    assert!(refused.contains("single line"), "{refused}");
    enable_historical_reconciliation(
        root,
        &successor.id,
        "Fixture migration owner".into(),
        "Acknowledge unproven history, preserving the original receipt.".into(),
    )
    .unwrap();
    assert!(
        finalize_change(root, &successor.id).is_err(),
        "old approval must not authorize the new basis"
    );
    approve_definition(
        root,
        &successor.id,
        Some("Fixture migration scope owner".into()),
        None,
    )
    .unwrap();
    assert!(
        record_scoped_review(root, &successor.id, "Fixture reviewer".into()).is_err(),
        "fresh verification is required"
    );
    check_change(root, Some(&successor.id)).unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(
        root,
        &[
            "commit",
            "-m",
            "approve explicit historical acknowledgement",
        ],
    );
    check_change(root, Some(&successor.id)).unwrap();
    assert!(
        finalize_change(root, &successor.id).is_err(),
        "old scoped review must not authorize closure"
    );
    record_scoped_review(root, &successor.id, "Fresh fixture reviewer".into()).unwrap();
    finalize_change(root, &successor.id).unwrap();
    let closed = load_change(root, &successor.id).unwrap();
    let evidence = load_verification(root, &closed)
        .unwrap()
        .semantic_succession
        .unwrap();
    assert_eq!(evidence.schema_version, 2);
    let historical = evidence.historical_reconciliation.as_ref().unwrap();
    assert_eq!(historical.transition_status, "unproven");
    assert_eq!(historical.anchor_commit, successor.base_commit.unwrap());
    assert_eq!(historical.predecessors.len(), 1);
    assert_eq!(
        old_package,
        archive_workspace_snapshot(
            root,
            &change_dir(root, &old.id),
            &format!("{CHANGES_PATH}/{}", old.id)
        )
        .unwrap()
    );
    assert!(ensure_closing_approval_valid(root, &load_change(root, &old.id).unwrap()).is_ok());
    assert_eq!(
        terminal_evidence_summary(root, &closed).validity,
        TerminalEvidenceValidity::HistoricalTransitionUnproven
    );
    assert_eq!(
        terminal_evidence_summary(root, &load_change(root, &old.id).unwrap()).validity,
        TerminalEvidenceValidity::HistoricalTransitionUnproven
    );
    // Authenticate each shared receipt/anchor once per pure graph pass, not once per input.
    let tuple = evidence.tuples.first().unwrap();
    let mut memo = AcceptedInputValidationMemo::default();
    reset_test_git_process_count();
    assert!(semantic_tuple_transition_is_valid(
        root,
        &closed,
        tuple,
        Some(&evidence),
        None,
        &mut memo
    )
    .unwrap());
    let first_reads = test_git_process_count();
    reset_test_git_process_count();
    assert!(semantic_tuple_transition_is_valid(
        root,
        &closed,
        tuple,
        Some(&evidence),
        None,
        &mut memo
    )
    .unwrap());
    let repeated_reads = test_git_process_count();
    assert!(
        repeated_reads < first_reads,
        "shared proof reads were repeated: first={first_reads}, repeated={repeated_reads}"
    );
    assert_eq!(memo.historical_receipts.len(), 1);
    assert_eq!(memo.accepted_transitions.len(), 1);
    drop(memo);
    // A new pass must notice raw receipt changes, even if the JSON remains semantically valid.
    let old_verification = change_dir(root, &old.id).join("verification.json");
    let original = fs::read(&old_verification).unwrap();
    let mut altered = original.clone();
    altered.push(b'\n');
    fs::write(&old_verification, altered).unwrap();
    assert!(AcceptedInputValidationMemo::default()
        .historical_receipt(root, &historical.anchor_commit, &old.id)
        .is_err());
    fs::write(&old_verification, original).unwrap();
    // Witness and classification changes are never interchangeable with proven succession.
    let mut forged = evidence.clone();
    forged
        .historical_reconciliation
        .as_mut()
        .unwrap()
        .transition_status = "proven".into();
    assert!(validate_semantic_succession(&closed, &forged).is_err());
    let mut forged = evidence.clone();
    forged.schema_version = 1;
    assert!(validate_semantic_succession(&closed, &forged).is_err());
    let mut forged = evidence.clone();
    forged
        .historical_reconciliation
        .as_mut()
        .unwrap()
        .predecessors[0]
        .package_digest = "0".repeat(64);
    let tuple = forged.tuples.first().unwrap();
    assert!(semantic_tuple_transition_is_valid(
        root,
        &closed,
        tuple,
        Some(&forged),
        None,
        &mut AcceptedInputValidationMemo::default()
    )
    .is_err());
    // The preserved original can now be archived natively, with its accepted-state intact.
    archive_change(root, &old.id).unwrap();
    assert!(validate_archived_integrity(root, &load_change(root, &old.id).unwrap()).is_ok());
    let (_, archived_witness) =
        historical_predecessor_receipt(root, &historical.anchor_commit, &old.id).unwrap();
    assert_eq!(
        archived_witness, historical.predecessors[0],
        "native metadata projection must retain the exact original package witness"
    );
    let archived_markdown = find_change_dir(root, &old.id).unwrap().join("change.md");
    let native_markdown = fs::read(&archived_markdown).unwrap();
    let mut altered_markdown = native_markdown.clone();
    altered_markdown.push(b'\n');
    fs::write(&archived_markdown, altered_markdown).unwrap();
    assert!(
        historical_predecessor_receipt(root, &historical.anchor_commit, &old.id).is_err(),
        "arbitrary archived markdown is not a native metadata projection"
    );
    fs::write(&archived_markdown, native_markdown).unwrap();
    fs::write(root.join("src/auth.rs"), "// unsigned later source\n").unwrap();
    assert!(
        ensure_closing_approval_valid(root, &load_change(root, &successor.id).unwrap()).is_err()
    );
}

#[test]
fn historical_reconciliation_refuses_tampered_or_unanchored_packages_without_mutation() {
    let (temp, old, successor) = historical_reconciliation_fixture();
    let root = temp.path();
    let state = change_dir(root, &successor.id).join("state.json");
    let before = fs::read(&state).unwrap();
    let artifact = change_dir(root, &old.id).join("unrecorded-extra.md");
    fs::write(&artifact, "not at the anchor\n").unwrap();
    assert!(enable_historical_reconciliation(
        root,
        &successor.id,
        "Fixture owner".into(),
        "Preserve unknown history.".into()
    )
    .is_err());
    assert_eq!(fs::read(&state).unwrap(), before);
    fs::remove_file(artifact).unwrap();
    let mut record = load_change(root, &successor.id).unwrap();
    record.base_commit = Some("0".repeat(40));
    save_change(root, &record).unwrap();
    let invalid = fs::read(&state).unwrap();
    assert!(enable_historical_reconciliation(
        root,
        &successor.id,
        "Fixture owner".into(),
        "Preserve unknown history.".into()
    )
    .is_err());
    assert_eq!(fs::read(&state).unwrap(), invalid);
}

#[test]
fn historical_reconciliation_refuses_protected_and_unsupported_inputs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    for path in [
        ".specsync/config.toml",
        ".github/workflows/ci.yml",
        ".trust.toml",
        "AGENTS.md",
        "hi/security.md",
        "../outside",
    ] {
        assert!(historical_supported_path(root, path).is_err(), "{path}");
    }
    for path in [
        "src/auth.rs",
        "specs/auth/auth.spec.md",
        ".env.example",
        ".gitignore",
    ] {
        assert!(historical_supported_path(root, path).is_ok(), "{path}");
    }
    for kind in [AcceptanceInputKind::Symlink, AcceptanceInputKind::Gitlink] {
        let entry = AcceptanceInputEntryV1 {
            path: "src/auth.rs".into(),
            kind,
            mode: 0o120000,
            payload_digest: "0".repeat(64),
            entry_digest: "0".repeat(64),
            owners: vec!["auth".into()],
        };
        assert!(historical_supported_entry(&entry).is_err());
    }
}

#[test]
fn historical_reconciliation_leaves_normal_schema_and_digest_shape_unchanged() {
    let evidence = SemanticSuccessionEvidenceV1 {
        schema_version: 1,
        tuples: Vec::new(),
        historical_reconciliation: None,
    };
    assert_eq!(
        serde_json::to_value(&evidence).unwrap(),
        serde_json::json!({"schema_version":1,"tuples":[]})
    );
    let mut digest = FramedDigest::new(SEMANTIC_SUCCESSION_DOMAIN);
    digest.frame(b"schema-version", &1_u32.to_be_bytes());
    assert_eq!(
        semantic_succession_digest(&evidence).unwrap(),
        digest.finish()
    );
    assert!(historical_package_digest(&BTreeMap::new()).is_err());
    assert!(historical_package_digest(&BTreeMap::from([(
        "state.json".into(),
        (0o120000, b"outside".to_vec())
    )]))
    .is_err());
}

#[test]
fn historical_reconciliation_reads_exact_successor_inputs_in_nested_projects() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    recorded_review_recovery_git(root, &["init", "-b", "main"]);
    recorded_review_recovery_git(root, &["config", "user.email", "test@example.com"]);
    recorded_review_recovery_git(root, &["config", "user.name", "Test"]);
    let project = root.join("site");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("src/auth.rs"), "// current nested input\n").unwrap();
    recorded_review_recovery_git(root, &["add", "."]);
    recorded_review_recovery_git(root, &["commit", "-m", "nested input"]);
    let commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    assert_eq!(
        historical_entry_digest_at_commit(&project, &commit, "src/auth.rs").unwrap(),
        acceptance_entry_digest(
            "src/auth.rs",
            &AcceptanceInputKind::File,
            0o100644,
            &sha256_hex(b"// current nested input\n")
        )
    );
    assert_eq!(
        historical_entry_digest_at_commit(&project, &commit, "missing.rs").unwrap(),
        acceptance_entry_digest(
            "missing.rs",
            &AcceptanceInputKind::Missing,
            0,
            &sha256_hex(b"")
        )
    );
    assert_eq!(
        historical_entry_digest_at_commit(&project, &commit, "src").unwrap(),
        acceptance_entry_digest("src", &AcceptanceInputKind::NonFile, 0, &sha256_hex(b""))
    );
}
