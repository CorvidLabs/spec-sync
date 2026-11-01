
/// DISCRIMINATOR for #741's diagnostic. On the unfixed binary this message names `approve` and
/// stops there.
///
/// That remedy was worse than incomplete: re-approving recorded a digest for the current body,
/// satisfied this very check, and handed the author straight to the `canonical_applied`
/// short-circuit, which discarded the correction in silence and reported success. The message
/// steered into the defect it was reporting. Approval binds the wording; only `check` puts it in
/// the canonical spec, and a remedy that names one step of two is a trap.
#[test]
fn the_refusal_for_a_changed_delta_names_the_second_step_that_finishes_the_job() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    fs::write(delta_path(root, &record, "auth"), SWAPPED_DELTA_BODY).unwrap();
    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("specsync change approve"),
        "the remedy must still name the approval that binds the wording: {error}"
    );
    assert!(
        error.contains("specsync change check"),
        "the remedy must name the step that puts the approved wording in the canonical spec; \
         naming only `approve` walked the author into the silent skip: {error}"
    );
}

#[test]
fn path_coverage_uses_current_remote_base_after_rebase() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success(), "git command failed: {args:?}");
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    let original_base = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(root.join(".github/workflows/ci.yml"), "name: CI\n").unwrap();
    git(&["add", ".github/workflows/ci.yml"]);
    git(&["commit", "-m", "upstream workflow"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);

    let mut record = completed_record(root);
    record.base_commit = Some(original_base);
    record.state = ChangeState::Implementing;
    record.affected_paths = vec!["src/".into(), SEQUENCE_PATH.into(), POLICY_PATH.into()];
    let policy = SddPolicy::default();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[record])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn path_coverage_uses_non_main_remote_default_branch() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "trunk"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/trunk", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/trunk",
    ]);
    git(&["switch", "-c", "feature"]);
    assert_eq!(pull_request_diff_base(root, &[]), "origin/trunk...HEAD");
}

#[test]
fn detached_head_verification_and_acceptance_are_supported() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "base"]);
    git(&["switch", "--detach"]);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
}

#[test]
fn loaded_change_rejects_mismatched_or_unsafe_persisted_identity() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_record(root);
    let state_path = change_dir(root, &record.id).join("state.json");
    let original = fs::read_to_string(&state_path).unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&original).unwrap();

    state["id"] = serde_json::Value::String("CHG-9999-other-workspace".into());
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    assert!(
        load_change(root, &record.id)
            .unwrap_err()
            .contains("does not match workspace")
    );
    assert!(
        list_changes_checked(root)
            .unwrap_err()
            .contains("does not match workspace")
    );

    state["id"] = serde_json::Value::String("../../escape".into());
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    assert!(
        load_change(root, &record.id)
            .unwrap_err()
            .contains("invalid change ID")
    );
}

#[test]
fn loaded_change_rejects_unsafe_persisted_spec_and_artifact_scopes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_record(root);
    let state_path = change_dir(root, &record.id).join("state.json");
    let original = fs::read_to_string(&state_path).unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&original).unwrap();

    state["affected_specs"] = serde_json::json!(["../../escape"]);
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    assert!(
        load_change(root, &record.id)
            .unwrap_err()
            .contains("invalid affected spec")
    );

    state = serde_json::from_str(&original).unwrap();
    state["selected_artifacts"] = serde_json::json!([{"custom": "../../escape"}]);
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    assert!(
        load_change(root, &record.id)
            .unwrap_err()
            .contains("unsafe custom artifact")
    );
}

#[test]
fn historical_tombstone_corruption_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_record(root);
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
        )
        .unwrap();
    let historical = root
        .join(ARCHIVE_PATH)
        .join("2026-01-01-CHG-0000-old/deltas");
    fs::create_dir_all(&historical).unwrap();
    fs::write(historical.join("auth.md"), [0xff, 0xfe]).unwrap();

    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(
        error.contains("historical semantic delta"),
        "unexpected error: {error}"
    );

    fs::write(historical.join("auth.md"), "plain garbage\n").unwrap();
    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(
        error.contains("historical semantic delta"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("no recognized operation headings"),
        "populated historical garbage must not be reported as empty: {error}"
    );
    assert!(
        !error.contains("is empty"),
        "populated historical garbage must not say is empty: {error}"
    );

    fs::write(
        historical.join("auth.md"),
        "## REMVOED\n### REQUIREMENT REQ-auth-000\nRetired.\n",
    )
    .unwrap();
    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(
        error.contains("invalid delta operation heading"),
        "unexpected error: {error}"
    );
}

#[test]
fn definition_approval_preserves_a_corrupt_ledger() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_record(root);
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            prose_or_plan(artifact, "# Complete\n\nReviewed.\n"),
        )
        .unwrap();
    }
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
        )
        .unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    fs::write(&ledger_path, b"{corrupt").unwrap();

    assert!(approve_definition(root, &record.id, Some("Reviewer".into()), None).is_err());
    assert_eq!(fs::read(&ledger_path).unwrap(), b"{corrupt");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Draft
    );
}

#[test]
fn verifying_state_requires_recorded_evidence_in_unified_checks() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands.clear();
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    record.state = ChangeState::Verifying;
    save_change(root, &record).unwrap();

    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("verification evidence is missing"))
    );
}

fn quiet_git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn commit_paths(root: &Path, paths: &[&str], message: &str) {
    let output = Command::new("git")
        .args(["add", "--"])
        .args(paths)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    quiet_git(root, &["commit", "-m", message]);
}

fn verification_history_fixture() -> (TempDir, String, VerificationRecord) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands.clear();
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    // Persistence + scoped-review tests assert v2 review/finalize next_action.
    // completed_no_spec_record is workflow v1; keep v1 covered by
    // workflow_v1_verifying_next_action_names_verify_accept_archive.
    let mut record = completed_no_spec_current_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement"]);
    let evidence = verify_change(root, &record.id).unwrap();
    let record = load_change(root, &record.id).unwrap();
    assert!(verification_is_current(root, &record, &evidence));
    (temp, record.id, evidence)
}

fn verification_persistence_paths(id: &str) -> [String; 3] {
    [
        format!(".specsync/changes/{id}/state.json"),
        format!(".specsync/changes/{id}/verification.json"),
        format!(".specsync/changes/{id}/verification-attempts.json"),
    ]
}

#[test]
fn exact_and_multiple_verification_persistence_commits_remain_current() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    let paths = verification_persistence_paths(&id);
    for (index, path) in paths.iter().enumerate() {
        commit_paths(root, &[path], &format!("persist verification {index}"));
    }
    let record = load_change(root, &id).unwrap();
    assert!(verification_is_current(root, &record, &evidence));
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!(
            "run `specsync change review {id} --reviewer <human>` after the PR's scoped review passes"
        )
    );
    assert!(
        !check_project(root).errors.iter().any(|error| {
            error.contains(&id) && error.contains("verification evidence is stale")
        })
    );
}

// Verifies REQ-change-013, REQ-change-016, and REQ-change-046.
#[test]
fn scoped_review_persistence_commit_keeps_verification_current() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    let verification_paths = verification_persistence_paths(&id);
    let verification_path_refs: Vec<&str> = verification_paths.iter().map(String::as_str).collect();
    commit_paths(root, &verification_path_refs, "persist verification");
    let review = record_scoped_review(root, &id, "Independent Reviewer".into()).unwrap();
    let review_paths = [
        format!(".specsync/changes/{id}/{SCOPED_REVIEW_FILE}"),
        format!(".specsync/changes/{id}/{SCOPED_REVIEW_ATTEMPTS_FILE}"),
    ];
    let review_path_refs: Vec<&str> = review_paths.iter().map(String::as_str).collect();
    commit_paths(root, &review_path_refs, "persist scoped review");
    let record = load_change(root, &id).unwrap();
    assert!(verification_is_current(root, &record, &evidence));
    assert!(scoped_review_is_current(root, &record, &review));
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!("run `specsync change finalize {id}`")
    );
    assert!(
        !check_project(root).errors.iter().any(|error| {
            error.contains(&id) && error.contains("verification evidence is stale")
        })
    );
}

// Verifies REQ-change-013, REQ-change-016, and REQ-change-046.
#[test]
fn scoped_review_persistence_mixed_with_source_change_is_stale() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    let verification_paths = verification_persistence_paths(&id);
    let verification_path_refs: Vec<&str> = verification_paths.iter().map(String::as_str).collect();
    commit_paths(root, &verification_path_refs, "persist verification");
    record_scoped_review(root, &id, "Independent Reviewer".into()).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let review_paths = [
        format!(".specsync/changes/{id}/{SCOPED_REVIEW_FILE}"),
        format!(".specsync/changes/{id}/{SCOPED_REVIEW_ATTEMPTS_FILE}"),
    ];
    let mut committed: Vec<&str> = review_paths.iter().map(String::as_str).collect();
    committed.push("src/lib.rs");
    commit_paths(root, &committed, "mix scoped review and source");
    let record = load_change(root, &id).unwrap();
    assert!(!verification_is_current(root, &record, &evidence));
}

#[test]
fn persisted_scoped_review_allows_scope_approver_as_reviewer() {
    let (temp, id, _) = verification_history_fixture();
    let root = temp.path();
    let verification_paths = verification_persistence_paths(&id);
    let verification_path_refs: Vec<&str> = verification_paths.iter().map(String::as_str).collect();
    commit_paths(root, &verification_path_refs, "persist verification");
    record_scoped_review(root, &id, "Reviewer".into()).unwrap();
    let record = load_change(root, &id).unwrap();
    let loaded = load_scoped_review(root, &record).unwrap();
    assert_eq!(loaded.reviewer, "Reviewer");
    assert_eq!(loaded.verdict, ScopedReviewVerdict::Pass);
}

#[test]
fn mixed_persistence_and_source_commit_is_stale() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let paths = verification_persistence_paths(&id);
    let mut committed: Vec<&str> = paths.iter().map(String::as_str).collect();
    committed.push("src/lib.rs");
    commit_paths(root, &committed, "mix evidence and source");
    let record = load_change(root, &id).unwrap();
    assert!(!verification_is_current(root, &record, &evidence));
}

// Verifies REQ-change-013 and REQ-change-016.
#[test]
fn malicious_state_contract_mutation_is_stale() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    let paths = verification_persistence_paths(&id);
    let path_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    commit_paths(root, &path_refs, "persist verification");
    let mut record = load_change(root, &id).unwrap();
    record.title = "mutated contract".into();
    save_change(root, &record).unwrap();
    commit_paths(root, &[&paths[0]], "mutate state contract");
    let record = load_change(root, &id).unwrap();
    assert!(!verification_is_current(root, &record, &evidence));
}

#[test]
fn evidence_only_merge_checks_every_parent_and_remains_current() {
    let (temp, id, evidence) = verification_history_fixture();
    let root = temp.path();
    let paths = verification_persistence_paths(&id);
    let path_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    commit_paths(root, &path_refs, "persist verification on main");
    quiet_git(
        root,
        &[
            "switch",
            "-c",
            "evidence-side",
            evidence.commit.as_deref().unwrap(),
        ],
    );
    let mut checkout = Command::new("git");
    let output = checkout
        .args(["checkout", "main", "--"])
        .args(&path_refs)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    commit_paths(root, &path_refs, "persist verification on side");
    quiet_git(root, &["merge", "--no-ff", "main", "-m", "merge evidence"]);
    let record = load_change(root, &id).unwrap();
    assert!(verification_is_current(root, &record, &evidence));
}

#[test]
fn no_spec_change_rejects_a_declared_public_contract_change() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record
        .answers
        .insert("public_contract".into(), "yes".into());
    save_change(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(error.contains("no_spec_change"));
    assert!(load_approvals(root, &record).unwrap().approvals.is_empty());
    record.state = ChangeState::Accepted;
    let error = validate_definition(root, &record).unwrap_err();
    assert!(error.contains("no_spec_change"));
}

#[test]
fn accepted_evidence_tracks_scoped_post_acceptance_inputs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();

    let evidence = load_verification(root, &record).unwrap();
    assert!(evidence.acceptance_input_digest.is_some());
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    fs::write(root.join("notes.txt"), "unrelated\n").unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    git(&["add", "notes.txt"]);
    git(&["commit", "-m", "unrelated"]);
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 3 }\n").unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_err());
    assert!(archive_change(root, &record.id).is_err());
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn subproject_policy_and_diff_paths_are_project_relative() {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    let root = repository.join("packages/app");
    let git = |dir: &Path, args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .unwrap()
                .success()
        );
    };
    git(repository, &["init", "-b", "main"]);
    git(repository, &["config", "user.email", "test@example.com"]);
    git(repository, &["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("contracts/auth")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(repository.join("other")).unwrap();
    write_json(&root.join(POLICY_PATH), &SddPolicy::default()).unwrap();
    fs::write(root.join("src/lib.rs"), "base\n").unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"contracts\"\n",
    )
    .unwrap();
    fs::write(root.join("contracts/auth/auth.spec.md"), "base spec\n").unwrap();
    fs::write(repository.join("other/file.rs"), "base\n").unwrap();
    git(repository, &["add", "."]);
    git(repository, &["commit", "-m", "base"]);
    git(
        repository,
        &["update-ref", "refs/remotes/origin/main", "HEAD"],
    );

    let mut weakened = SddPolicy::default();
    weakened.enabled = false;
    write_json(&root.join(POLICY_PATH), &weakened).unwrap();
    assert!(policy_at_comparison_base(&root).unwrap().unwrap().enabled);
    write_json(&root.join(POLICY_PATH), &SddPolicy::default()).unwrap();
    fs::write(root.join("src/lib.rs"), "changed\n").unwrap();
    fs::write(root.join("contracts/auth/auth.spec.md"), "changed spec\n").unwrap();
    fs::write(repository.join("other/file.rs"), "outside\n").unwrap();

    let uncovered = uncovered_meaningful_paths(&root, &SddPolicy::default(), &[]).unwrap();
    assert_eq!(
        uncovered,
        vec![
            "contracts/auth/auth.spec.md".to_string(),
            "src/lib.rs".to_string()
        ]
    );
    let mut record = create_change(
        &root,
        CreateChangeRequest {
            description: "Update auth".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    record.state = ChangeState::Implementing;
    assert!(
        uncovered_meaningful_paths(&root, &SddPolicy::default(), &[record])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn adopt_re_pins_the_bootstrap_record_it_invalidates() {
    // `init` records a digest over the policy it wrote with SDD OFF, and
    // `bootstrap_digest` covers `enabled`. Flipping that field invalidates the
    // record — which does not fail, it silently stops exempting the file it was
    // written to exempt. Adoption re-pins its own bytes.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    // What `specsync init` does: policy off, then pin it.
    write_default_policy(root, Vec::new()).unwrap();
    record_bootstrap_paths(root).unwrap();
    assert!(!load_policy(root).unwrap().enabled);

    adopt(root, false, None).unwrap();

    let policy = load_policy(root).unwrap();
    assert!(policy.enabled, "adopt is the on-switch");
    let gated = SddPolicy {
        require_change_for_meaningful_files: true,
        ..policy
    };
    assert!(
        !uncovered_meaningful_paths(root, &gated, &[])
            .unwrap()
            .contains(&POLICY_PATH.to_string()),
        "the exemption init recorded must survive the flip adoption makes"
    );
}

#[test]
fn adoption_bootstrap_covers_only_the_original_policy() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    adopt(root, false, None).unwrap();
    let policy = load_policy(root).unwrap();
    assert!(check_project(root).errors.is_empty());
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .is_empty()
    );
    let report_path = root.join(".specsync/adoption-report.json");
    let report = fs::read(&report_path).unwrap();
    fs::remove_file(&report_path).unwrap();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .contains(&POLICY_PATH.to_string())
    );
    fs::write(&report_path, report).unwrap();
    let tree = git_output(root, &["rev-parse", "HEAD^{tree}"]).unwrap();
    let unrelated = git_output(root, &["commit-tree", &tree, "-m", "unrelated"]).unwrap();
    let mut parsed: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    parsed["bootstrap_policy"]["base_commit"] = serde_json::Value::String(unrelated);
    write_json(&report_path, &parsed).unwrap();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .contains(&POLICY_PATH.to_string())
    );
    adopt(root, false, None).unwrap();
    let mut changed = policy;
    changed.meaningful_paths.push("private/".into());
    write_json(&root.join(POLICY_PATH), &changed).unwrap();
    assert!(
        uncovered_meaningful_paths(root, &changed, &[])
            .unwrap()
            .contains(&POLICY_PATH.to_string())
    );
}

#[test]
fn bootstrap_records_exempt_only_newly_created_protected_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "base\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-qm", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    write_json(&root.join(POLICY_PATH), &SddPolicy::default()).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/version"), "5.0.0\n").unwrap();
    record_bootstrap_paths(root).unwrap();
    let policy = SddPolicy::default();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .is_empty(),
        "a fresh bootstrap covers the protected files it created"
    );

    // A forged record cannot lend coverage to product source.
    let digest = content_digest(b"forged\n");
    fs::write(root.join("src/lib.rs"), "forged\n").unwrap();
    let head = git_output(root, &["rev-parse", "--verify", "HEAD"]).unwrap();
    write_json(
        &root.join(BOOTSTRAP_RECORD_PATH),
        &serde_json::json!({
            "version": 1,
            "base_commit": head,
            "paths": [{"path": "src/lib.rs", "digest": digest}],
        }),
    )
    .unwrap();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .contains(&"src/lib.rs".to_string()),
        "a bootstrap record must never exempt product source"
    );

    // Nor can it exempt a policy file that already exists at the base.
    git(&["add", "-A"]);
    git(&["commit", "-qm", "bootstrap"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::write(root.join(".specsync/version"), "5.0.1\n").unwrap();
    record_bootstrap_paths(root).unwrap();
    assert!(
        uncovered_meaningful_paths(root, &policy, &[])
            .unwrap()
            .contains(&".specsync/version".to_string()),
        "re-recording must not exempt an edit to an already-tracked policy file"
    );
}

#[test]
fn uncovered_paths_error_names_the_escape_hatch_and_ignore_precedence() {
    let policy = SddPolicy::default();
    let message = uncovered_paths_error(
        &policy,
        &["src/lib.rs".to_string(), "specs/zzz/note.md".to_string()],
    );
    assert!(message.contains("specsync change new"));
    assert!(message.contains("--kind bug-fix --spec"));
    assert!(
        !message.contains("--kind fix --spec"),
        "remediation must use the clap kind bug-fix, got: {message}"
    );
    assert!(message.contains("--path src/lib.rs"));
    assert!(message.contains("--no-spec-change"));
    assert!(
        message.contains("an ignored_paths entry covers specs/zzz/note.md")
            && message.contains("always meaningful")
            && !message.contains("covers src/lib.rs"),
        "only paths shadowed by an ignored_paths entry get the precedence note: {message}"
    );
}

#[test]
fn workflow_v1_verifying_next_action_names_verify_accept_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let mut record = record;
    record.state = ChangeState::Verifying;
    save_change(root, &record).unwrap();
    let next = summarize_change(root, &record).next_action;
    assert!(
        next.contains("change verify")
            && next.contains("change accept")
            && next.contains("change archive"),
        "v1 Verifying must name verify then accept then archive"
    );
    assert!(
        !next.contains("change check") && !next.contains("change finalize"),
        "v1 Verifying must not advertise v2 verbs"
    );
}

#[test]
fn overlapping_changes_cannot_lend_archive_attribution() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    write_lifecycle_test_policy(root);
    let mut legacy_policy = load_policy(root).unwrap();
    legacy_policy.version = 1;
    write_json(&root.join(POLICY_PATH), &legacy_policy).unwrap();

    let mut records = vec![
        completed_no_spec_record(root),
        completed_no_spec_record(root),
    ];
    for record in &mut records {
        append_approval(
            root,
            record,
            "definition",
            Some("Reviewer".into()),
            definition_digest(root, record).unwrap(),
            None,
        )
        .unwrap();
        record.state = ChangeState::Accepted;
        save_change(root, record).unwrap();
        let verification = VerificationRecord {
            timestamp: now(),
            commit: git_output(root, &["rev-parse", "HEAD"]),
            contract_digest: definition_digest(root, record).unwrap(),
            execution_digest: None,
            workspace_digest: project_input_digest(root).unwrap(),
            acceptance_input_digest: None,
            acceptance_manifest: None,
            semantic_succession: None,
            passed: true,
            commands: Vec::new(),
            requirement_ids: Vec::new(),
        };
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        append_approval(
            root,
            record,
            "acceptance",
            Some("Reviewer".into()),
            closing_digest(record, &verification),
            None,
        )
        .unwrap();
    }
    fs::write(root.join("src/lib.rs"), "delivery\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "delivery"]);
    for record in &records {
        let mut verification = load_verification(root, record).unwrap();
        verification.acceptance_input_digest =
            Some(acceptance_input_digest(root, record, &[]).unwrap());
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        append_approval(
            root,
            record,
            "acceptance",
            Some("Reviewer".into()),
            closing_digest(record, &verification),
            Some("Bind current delivery inputs".into()),
        )
        .unwrap();
    }

    assert!(archive_change(root, &records[0].id).is_err());
    git(&["add", "."]);
    git(&["commit", "-m", "record accepted lifecycle evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    archive_change(root, &records[0].id).unwrap();
}

#[cfg(unix)]
#[test]
fn foreign_import_rejects_symlinked_markdown() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let outside = root.join("outside.md");
    fs::write(&outside, "secret\n").unwrap();
    fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
    symlink(&outside, root.join("openspec/specs/auth/spec.md")).unwrap();

    let error = adopt(root, false, Some("openspec")).unwrap_err();
    assert!(error.contains("symlinked foreign import"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(
        !root
            .join(".specsync/imports/openspec/canonical/auth/spec.md")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn foreign_import_rejects_symlinked_ancestor_directories() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir_all(outside.join("openspec/specs/auth")).unwrap();
    fs::write(
        outside.join("openspec/specs/auth/spec.md"),
        "external contract\n",
    )
    .unwrap();
    fs::create_dir_all(&root).unwrap();
    symlink(outside.join("openspec"), root.join("openspec")).unwrap();

    let error = adopt(&root, false, Some("openspec")).unwrap_err();
    assert!(error.contains("symlinked foreign import"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(
        !root
            .join(".specsync/imports/openspec/canonical/auth/spec.md")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn speckit_import_rejects_a_symlinked_constitution_ancestor() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir_all(outside.join("memory")).unwrap();
    fs::write(outside.join("memory/constitution.md"), "external rules\n").unwrap();
    fs::create_dir_all(&root).unwrap();
    symlink(&outside, root.join(".specify")).unwrap();

    let error = adopt(&root, false, Some("speckit")).unwrap_err();
    assert!(error.contains("symlinked foreign import"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(
        !root
            .join(".specsync/imports/speckit/constitution.md")
            .exists()
    );
}

/// Issue #660: an archived package must be authenticated by WHERE its evidence entered the
/// reachable history, never by the mere fact that some commit contains the bytes being checked.
///
/// The fixture is deliberately the hard one: a workflow-v2 change taken through the real
/// lifecycle on a branch, squash-merged into `main`, then inspected from a fresh clone in which
/// the original acceptance-transition commits do not exist. That is the shape 142 of this
/// repository's 161 archived packages actually have, and it is the shape any fix must not break.
///
/// Every scenario is evaluated and reported together rather than short-circuiting, so a reviewer
/// sees which specific properties a candidate fix satisfies instead of only the first failure.
#[test]
fn an_archived_package_is_authenticated_only_by_where_its_evidence_entered_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("source");
    fs::create_dir_all(&root).unwrap();
    let git = |directory: &Path, args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(directory)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "core.autocrlf", "false"]);
    git(&root, &["config", "core.eol", "lf"]);
    fs::write(
        root.join(".gitattributes"),
        "*.json text eol=lf\n*.md text eol=lf\n",
    )
    .unwrap();
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&root, &["add", "README.md", ".gitattributes"]);
    git(&root, &["commit", "-m", "base"]);
    git(&root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(
        &root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    git(&root, &["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "// implementation\n").unwrap();
    git(&root, &["add", "src/lib.rs"]);
    git(&root, &["commit", "-m", "implementation"]);

    let record = current_workflow_record(&root, completed_no_spec_record(&root));
    approve_definition(&root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(&root, Some(&record.id)).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Implement approved change"]);
    check_change(&root, Some(&record.id)).unwrap();
    record_scoped_review(&root, &record.id, "Independent reviewer".into()).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Record passing scoped review"]);
    let acceptance_commit = git_output(&root, &["rev-parse", "HEAD"]).unwrap();
    finalize_change(&root, &record.id).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Finalize change"]);

    git(&root, &["switch", "main"]);
    git(&root, &["merge", "--squash", "feature"]);
    git(&root, &["commit", "-m", "Squash feature"]);
    git(&root, &["branch", "-D", "feature"]);

    let fresh = temp.path().join("fresh");
    let root_text = root.to_string_lossy().to_string();
    let fresh_text = fresh.to_string_lossy().to_string();
    git(
        temp.path(),
        &[
            "-c",
            "core.autocrlf=false",
            "-c",
            "core.eol=lf",
            "clone",
            "--no-local",
            "--single-branch",
            "--branch",
            "main",
            &root_text,
            &fresh_text,
        ],
    );
    git(&fresh, &["config", "user.email", "test@example.com"]);
    git(&fresh, &["config", "user.name", "Test"]);
    git(&fresh, &["config", "core.autocrlf", "false"]);
    git(&fresh, &["config", "core.eol", "lf"]);

    // ---- vacuity control -------------------------------------------------------------------
    // The squash discarded the acceptance-transition commit, so the only history this clone has
    // is the squash commit that created the archive package. If a fix cannot authenticate THIS,
    // it has un-authenticated the majority of the corpus. Must hold identically before and after.
    assert!(
        git_output(
            &fresh,
            &[
                "rev-parse",
                "--verify",
                &format!("{acceptance_commit}^{{commit}}"),
            ],
        )
        .is_none(),
        "fixture must reproduce the squash-orphaned shape: the acceptance commit must be absent"
    );
    let archived = load_change(&fresh, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    validate_archived_integrity(&fresh, &archived)
        .expect("a pristine squash-merged archive must authenticate from a fresh clone");

    let base = git_output(&fresh, &["rev-parse", "HEAD"]).unwrap();
    let archive_relative = find_change_dir(&fresh, &record.id)
        .unwrap()
        .strip_prefix(&fresh)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let relocated_relative = format!("{archive_relative}-relocated");
    let active_relative = format!(".specsync/changes/{}", record.id);
    let pristine_state = fs::read(fresh.join(&archive_relative).join("state.json")).unwrap();

    let reset = |directory: &Path| {
        git(directory, &["reset", "--hard", base.as_str()]);
        git(directory, &["clean", "-fdxq"]);
    };
    // Rewrites the approving actor: a field no workflow-v2 digest covers, so only the anchor
    // stands between it and an authenticated archive.
    let tamper = |package: &Path| {
        let path = package.join("approvals.json");
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("Scope owner"),
            "fixture must record the scope owner in approvals.json"
        );
        fs::write(&path, text.replace("Scope owner", "Impostor")).unwrap();
    };
    let resolve = |directory: &Path| -> Result<String, String> {
        let current = load_change(directory, &record.id)?;
        authenticated_accepted_transition(directory, &current).map(|(anchor, _, _)| anchor)
    };

    let mut failures: Vec<String> = Vec::new();
    let mut record_outcome = |name: &str,
                              must_authenticate: bool,
                              outcome: Result<String, String>| {
        match (must_authenticate, outcome) {
            (true, Err(error)) => {
                failures.push(format!("{name}: expected an anchor, got Err({error})"));
            }
            (false, Ok(anchor)) => {
                failures.push(format!("{name}: expected refusal, got anchor `{anchor}`"));
            }
            _ => {}
        }
    };

    // 1. Pristine squash-merged archive resolves an anchor. (Passes before and after.)
    record_outcome("pristine-squash-merged-archive", true, resolve(&fresh));

    // 2. A legitimate relocation with ZERO content change must still authenticate. This is the
    //    assertion that a fix which simply refuses relocations -- or refuses any commit whose
    //    diff is empty -- cannot satisfy.
    reset(&fresh);
    git(&fresh, &["mv", &archive_relative, &relocated_relative]);
    git(&fresh, &["commit", "-qm", "relocate the archived package"]);
    record_outcome("legitimate-relocation", true, resolve(&fresh));

    // 3. Rewriting archived evidence and committing it must be refused, with no relocation
    //    involved. A working-tree fallback that reads the bytes it is authenticating cannot
    //    satisfy this.
    reset(&fresh);
    tamper(&fresh.join(&archive_relative));
    git(&fresh, &["add", "."]);
    git(
        &fresh,
        &["commit", "-qm", "rewrite archived approval evidence"],
    );
    record_outcome("committed-tamper", false, resolve(&fresh));

    // 4. The issue's reproduction: tamper, then relocate in a separate commit that changes no
    //    content at all.
    reset(&fresh);
    tamper(&fresh.join(&archive_relative));
    git(&fresh, &["add", "."]);
    git(
        &fresh,
        &["commit", "-qm", "rewrite archived approval evidence"],
    );
    git(&fresh, &["mv", &archive_relative, &relocated_relative]);
    git(&fresh, &["commit", "-qm", "relocate the archived package"]);
    record_outcome("tamper-then-relocate", false, resolve(&fresh));

    // 5. Tamper and relocate in ONE commit. A fix that recognises the reproduction by its empty
    //    diff never sees this one.
    reset(&fresh);
    tamper(&fresh.join(&archive_relative));
    git(&fresh, &["mv", &archive_relative, &relocated_relative]);
    git(&fresh, &["add", "."]);
    git(
        &fresh,
        &["commit", "-qm", "rewrite and relocate in one commit"],
    );
    record_outcome("tamper-and-relocate-in-one-commit", false, resolve(&fresh));

    // 6. A forged reopen/re-archive pair. The archive directory keeps its original name
    //    throughout, so no rename rule of any kind can see this; the laundering happens at the
    //    ACTIVE workspace path, which is where `reopen` legitimately writes.
    reset(&fresh);
    fs::create_dir_all(fresh.join(".specsync/changes")).unwrap();
    git(&fresh, &["mv", &archive_relative, &active_relative]);
    let accepted_snapshot =
        fs::read(fresh.join(&active_relative).join("accepted-state.json")).unwrap();
    fs::write(
        fresh.join(&active_relative).join("state.json"),
        &accepted_snapshot,
    )
    .unwrap();
    tamper(&fresh.join(&active_relative));
    git(&fresh, &["add", "-A", "."]);
    git(
        &fresh,
        &[
            "commit",
            "-qm",
            "chore(lifecycle): reopen for fresh verification",
        ],
    );
    git(&fresh, &["mv", &active_relative, &archive_relative]);
    fs::write(
        fresh.join(&archive_relative).join("state.json"),
        &pristine_state,
    )
    .unwrap();
    git(&fresh, &["add", "-A", "."]);
    git(&fresh, &["commit", "-qm", "chore(lifecycle): archive"]);
    record_outcome("forged-reopen-and-re-archive", false, resolve(&fresh));

    assert!(
        failures.is_empty(),
        "an archived package must be authenticated only by where its evidence entered history:\n{}",
        failures.join("\n")
    );
}

/// Issues #540 and #660 in one fixture: a reopened change must be able to close again, and
/// restoring that path must not restore the laundering the introduction bound exists to refuse.
///
/// The fixture is deliberately the shape
/// `an_archived_package_is_authenticated_only_by_where_its_evidence_entered_history` cannot build.
/// That test squash-merges and reads from a fresh clone, so the acceptance commit is absent and
/// `verification_commit_is_accepted_current` can never hold -- which leaves the closing-evidence
/// fallback unreachable there and every claim about it unwitnessed. This change closes on `main`
/// with its acceptance commits intact, so the fallback is live and these scenarios reach it.
///
/// Every scenario runs over ONE fixture and they are reported together, so a fix that recognises
/// this fixture and waives the bound for it fails the adversarial scenarios in the same function
/// in which it passes the genuine one.
#[test]
fn a_reopened_change_closes_again_without_reopening_what_the_bound_refuses() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::write(
        root.join(".gitattributes"),
        "*.json text eol=lf\n*.md text eol=lf\n",
    )
    .unwrap();
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md", ".gitattributes"]);
    git(&["commit", "-m", "base"]);

    let record = current_workflow_record(&root, completed_no_spec_record(&root));
    assert!(record.workflow_version >= 2);
    approve_definition(&root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(&root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    let accepted_verification = check_change(&root, Some(&record.id)).unwrap().unwrap();
    // `review` and `finalize` are committed together, which is what `specsync change finalize`
    // followed by one commit produces and what drill 049 does. It also matters for what this test
    // can witness: committing the review at the ACTIVE path first makes `load_scoped_review` --
    // and therefore `validate_finalization_evidence`, and therefore the whole closing-evidence
    // fallback -- fail on today's build for an unrelated reason, which would silently turn every
    // adversarial scenario below into a pass for the wrong reason.
    record_scoped_review(&root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(&root, &record.id).expect("the first close must archive");
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "chore(lifecycle): scoped review and archive",
    ]);
    let closed = git_output(&root, &["rev-parse", "HEAD"]).unwrap();

    // The property that makes this fixture able to witness the closing-evidence fallback at all:
    // the acceptance commit is IN history, so `verification_commit_is_accepted_current` holds and
    // the fallback's other preconditions are satisfiable. Without this the adversarial scenarios
    // below would be refused by ancestry rather than by the bound under test, and the test would
    // certify a property it never exercised.
    let verification_commit = accepted_verification
        .commit
        .clone()
        .expect("workflow-v2 verification must record its commit");
    assert!(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", &verification_commit, "HEAD"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success(),
        "fixture must keep the acceptance commit reachable from HEAD"
    );

    let archive_relative = find_change_dir(&root, &record.id)
        .unwrap()
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let relocated_relative = format!("{archive_relative}-relocated");

    let reset = || {
        git(&["reset", "--hard", closed.as_str()]);
        git(&["clean", "-fdxq"]);
    };
    // Rewrites the approving actor: a field no workflow-v2 digest covers, so only the anchor
    // stands between it and an authenticated archive.
    let tamper = |package: &Path| {
        let path = package.join("approvals.json");
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("Scope owner"),
            "fixture must record the scope owner in approvals.json"
        );
        fs::write(&path, text.replace("Scope owner", "Impostor")).unwrap();
    };
    // The hand-written generation bump: everything `reopen` would have written, produced by
    // copying the package's own terminal approval and verification record. Nothing in the schema
    // is signed, so this is exactly as forgeable as the real thing -- which is the point.
    let forge_reopen_record = |package: &Path| {
        let path = package.join("approvals.json");
        let mut ledger: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap())
            .expect("archived approvals must parse");
        let verification: serde_json::Value =
            serde_json::from_slice(&fs::read(package.join("verification.json")).unwrap())
                .expect("archived verification must parse");
        let terminal = ledger["approvals"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .find(|approval| {
                matches!(
                    approval["gate"].as_str(),
                    Some("acceptance") | Some("finalization")
                )
            })
            .cloned()
            .expect("archived ledger must record a terminal approval");
        let forged = serde_json::json!({
            "schema_version": 1,
            "change_id": record.id,
            "actor": "Impostor",
            "reason": "forged reopen",
            "timestamp": 1_u64,
            "from_state": "accepted",
            "to_state": "verifying",
            "superseded_approval": terminal,
            "prior_verification": verification,
            "stale_acceptance_input_digest": verification["acceptance_input_digest"].clone(),
            "current_acceptance_input_digest": "f".repeat(64),
        });
        ledger["reopenings"]
            .as_array_mut()
            .expect("ledger must carry a reopenings array")
            .push(forged);
        fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
    };
    let drift = |note: &str| {
        fs::write(
            root.join("src/lib.rs"),
            format!("// implementation\npub fn ready() -> bool {{ {note} }}\n"),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "post-archive drift"]);
    };
    // Two resolvers on purpose. `anchor` isolates the bound under test, so an adversarial
    // scenario cannot be scored a pass because some unrelated validator happened to refuse it
    // first -- which is exactly how a laundering vector hides. `authenticates` is the whole read
    // path a person actually sees, asserted where authentication must SUCCEED.
    let anchor = |directory: &Path| -> Result<String, String> {
        let current = load_change(directory, &record.id)?;
        authenticated_accepted_transition(directory, &current).map(|(found, _, _)| found)
    };
    let authenticates = |directory: &Path| -> Result<String, String> {
        let current = load_change(directory, &record.id)?;
        validate_archived_integrity(directory, &current)?;
        anchor(directory)
    };

    // One reopen round trip, run end to end through the product's own verbs. `rewrite_committed`
    // is the only difference between the honest round trip and the front-door laundering, so the
    // two scenarios below cannot diverge anywhere except in the property under test.
    //
    // The reviewer is never the definition approver: otherwise the laundering would be refused by
    // separation of duties before the anchor bound is ever consulted.
    let reopen_and_reclose = |actor: &str, rewrite_committed: bool| -> Result<String, String> {
        reopen_change(
            &root,
            &record.id,
            actor.to_string(),
            "delivery evidence went stale".into(),
        )?;
        if rewrite_committed {
            tamper(&change_dir(&root, &record.id));
        }
        check_change(&root, Some(&record.id))?;
        git(&["add", "."]);
        git(&["commit", "-m", "chore(lifecycle): re-verify"]);
        check_change(&root, Some(&record.id))?;
        record_scoped_review(&root, &record.id, "Independent reviewer".into())?;
        finalize_change(&root, &record.id).map(|path| path.display().to_string())
    };

    let mut failures: Vec<String> = Vec::new();
    let mut record_outcome =
        |name: &str, must_authenticate: bool, outcome: Result<String, String>| {
            // Every scenario reports, pass or fail. A refusal that arrives for the wrong reason is
            // the failure mode this whole area keeps producing, so the reason is always printed.
            eprintln!(
                "SCENARIO {name} must_authenticate={must_authenticate} -> {}",
                match &outcome {
                    Ok(value) => format!("Ok({value})"),
                    Err(error) => format!("Err({error})"),
                }
            );
            match (must_authenticate, outcome) {
                (true, Err(error)) => {
                    failures.push(format!("{name}: expected an anchor, got Err({error})"));
                }
                (false, Ok(anchor)) => {
                    failures.push(format!("{name}: expected refusal, got `{anchor}`"));
                }
                _ => {}
            }
        };

    // 1. VACUITY CONTROL. A pristine close whose acceptance commits are still in history must
    //    authenticate. Holds on every build in this review, including the ones that ship the bug;
    //    a candidate that fails it has un-authenticated the ordinary case.
    record_outcome("pristine-in-history-archive", true, authenticates(&root));

    // 2. VACUITY CONTROL. An honest relocation with zero content change must still authenticate,
    //    the assertion no "refuse relocated archives" shortcut can satisfy.
    reset();
    git(&["mv", &archive_relative, &relocated_relative]);
    git(&["commit", "-qm", "relocate the archived package"]);
    record_outcome("honest-relocation", true, authenticates(&root));

    // 3. #660 vector 3 on the shape that can witness it. Rewriting a committed approval actor and
    //    committing it must be refused. This is the scenario that fails if the introduction bound
    //    is simply reverted: with the closing-evidence fallback ungated, the working tree -- which
    //    in a clean checkout is the tampered bytes -- authenticates itself, and the fallback never
    //    inspects `approvals.json` for workflow v2.
    reset();
    tamper(&root.join(&archive_relative));
    git(&["add", "."]);
    git(&["commit", "-qm", "rewrite archived approval evidence"]);
    record_outcome("committed-ledger-rewrite", false, anchor(&root));

    // 4. The same rewrite, promoted by a hand-written reopen event and a relocation. A rule that
    //    admits a re-introduction because the ledger next to it claims a higher reopen count is
    //    trusting the attacker's own arithmetic; the successor must be shown to CONTAIN the ledger
    //    history committed, which a rewritten one does not.
    reset();
    tamper(&root.join(&archive_relative));
    forge_reopen_record(&root.join(&archive_relative));
    git(&["add", "."]);
    git(&[
        "commit",
        "-qm",
        "rewrite archived evidence and claim a reopen",
    ]);
    git(&["mv", &archive_relative, &relocated_relative]);
    git(&["commit", "-qm", "relocate the archived package"]);
    record_outcome("forged-generation-then-relocate", false, anchor(&root));

    // 5. #540. A genuine reopen closes again, and the package it writes authenticates -- both
    //    through the command that creates it and, once committed, through the ordinary read path
    //    with no knowledge of that command.
    reset();
    drift("false");
    let reclosed = reopen_and_reclose("Release reviewer", false);
    let reclosed_ok = reclosed.is_ok();
    record_outcome("genuine-reopen-then-close", true, reclosed);
    if reclosed_ok {
        git(&["add", "."]);
        git(&["commit", "-m", "chore(lifecycle): archive again"]);
        record_outcome(
            "genuine-reopen-then-close-reads-back",
            true,
            authenticates(&root),
        );
    }
    // Vacuity guard for scenario 5: the reopen really did put a SECOND introduction of this
    // package into history. Without this a fix that quietly declined to re-archive, or a fixture
    // that never reached the bound, would satisfy the scenario above by doing nothing.
    let introductions = archive_introduction_index(&root)
        .unwrap()
        .get(&record.id)
        .cloned()
        .unwrap_or_default();
    record_outcome(
        "genuine-reopen-then-close-second-introduction",
        true,
        match introductions.len() {
            2 => Ok("two introductions".to_string()),
            found => Err(format!(
                "history holds {found} introductions of the package"
            )),
        },
    );

    // 6. The closing-evidence fallback must stay unavailable to a package this process merely
    //    FOUND in the archive. Flipping a committed package's `state.json` back to `accepted` and
    //    re-running the close is the shape a post-move resume also has, so a fallback keyed on
    //    "am I archiving something" rather than "am I closing the active workspace" blesses it.
    reset();
    let archive_package = root.join(&archive_relative);
    fs::copy(
        archive_package.join("accepted-state.json"),
        archive_package.join("state.json"),
    )
    .unwrap();
    tamper(&archive_package);
    record_outcome(
        "reanimated-committed-package",
        false,
        finalize_change(&root, &record.id).map(|path| path.display().to_string()),
    );

    // 7. The front door. `reopen` legitimately mints a new generation, so an attacker who can run
    //    the product can reach a re-close with the ledger open in front of them. Appending to it is
    //    their right; rewriting what an earlier archive of this change already committed is not.
    reset();
    drift("true");
    record_outcome(
        "reopen-then-rewrite-a-committed-approval",
        false,
        reopen_and_reclose("Impostor", true),
    );

    assert!(
        failures.is_empty(),
        "a reopened change must close again without reopening what the bound refuses:\n{}",
        failures.join("\n")
    );
}

#[test]
fn p5_corpus_census() {
    let Ok(root) = std::env::var("P5_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let _scope = ensure_change_read_scope(root);
    let records = list_all_changes_uncached(root).expect("load all records");
    let mut integ = Vec::new();
    let mut cache = ArchivedIntegrityCache::default();
    let mut n = 0;
    for record in records
        .values()
        .filter(|r| r.state == ChangeState::Archived)
    {
        n += 1;
        match validate_archived_integrity_with_cache(root, record, &mut cache) {
            Ok(()) => integ.push(format!("INTEGRITY_OK {}", record.id)),
            Err(e) => integ.push(format!("INTEGRITY_ERR {} :: {}", record.id, e)),
        }
    }
    // Reconciliation census: recompute the SEQUENCE_PATH manifest entry each archive signed.
    let live = std::fs::read(root.join(SEQUENCE_PATH)).ok();
    let mut seq = Vec::new();
    for record in records.values() {
        let dir = find_change_dir(root, &record.id).expect("dir");
        let vpath = dir.join("verification.json");
        let Ok(raw) = std::fs::read_to_string(&vpath) else {
            continue;
        };
        let v: serde_json::Value = serde_json::from_str(&raw).expect("verification json");
        let Some(entries) = v
            .pointer("/acceptance_manifest/entries")
            .and_then(|e| e.as_array())
        else {
            seq.push(format!("NOMANIFEST {}", record.id));
            continue;
        };
        let Some(signed) = entries
            .iter()
            .find(|e| e.get("path").and_then(|p| p.as_str()) == Some(SEQUENCE_PATH))
        else {
            seq.push(format!("NOSEQENTRY {}", record.id));
            continue;
        };
        let signed_digest = signed
            .get("payload_digest")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();
        match historical_sequence_ledger_acceptance_content(root, record) {
            Err(e) => seq.push(format!("RECON_ERR {} :: {}", record.id, e)),
            Ok(Some(bytes)) => {
                let d = sha256_hex(&bytes);
                if d == signed_digest {
                    seq.push(format!("RECONSTRUCTED {}", record.id));
                } else {
                    seq.push(format!("MISMATCH_RECON {}", record.id));
                }
            }
            Ok(None) => {
                let d = live.as_ref().map(|b| sha256_hex(b)).unwrap_or_default();
                if d == signed_digest {
                    seq.push(format!("LIVE_BYTES {}", record.id));
                } else {
                    seq.push(format!("MISMATCH_LIVE {}", record.id));
                }
            }
        }
    }
    integ.sort();
    seq.sort();
    let mut out = String::new();
    for l in &integ {
        out.push_str(l);
        out.push('\n');
    }
    for l in &seq {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(&format!("SUMMARY archived={} seq_lines={}\n", n, seq.len()));
    std::fs::write(std::env::var("P5_OUT").unwrap(), out).unwrap();
}

/// A minimal located record: what the archive carries, without any lifecycle verb.
#[cfg(test)]
fn write_located_fixture(root: &Path, dir: &Path, id: &str, state: ChangeState) {
    let record = ChangeRecord {
        schema_version: 1,
        workflow_version: 1,
        workflow_origin_version: Some(1),
        id: id.to_string(),
        slug: id.to_string(),
        title: id.to_string(),
        description: id.to_string(),
        kind: ChangeKind::Operations,
        state,
        canonical_applied: false,
        correction_count: 0,
        base_commit: None,
        created_at: 1_752_364_800,
        updated_at: 1_752_364_800,
        affected_specs: Vec::new(),
        affected_paths: vec!["ops/fixture".into()],
        no_spec_change: true,
        no_spec_change_rationale: Some("fixture".into()),
        acceptance_criteria: vec!["fixture".into()],
        selected_artifacts: Vec::new(),
        dependencies: Vec::new(),
        supersedes: Vec::new(),
        acceptance_owner_corrections: Vec::new(),
        legacy_archive_baseline_digest: None,
        answers: BTreeMap::new(),
    };

    let _ = root;
    fs::create_dir_all(dir).unwrap();
    write_json(&dir.join("state.json"), &record).unwrap();
}

/// DISCRIMINATOR. Fails on `b3f3201a` with `invalid change ID`.
///
/// One ordinal-free workspace anywhere on disk made `validate_change_sequences` refuse the
/// whole repository, which took `change new`, `change audit` and — because both reconciliation
/// functions call it first — every one of the 120 archives that signed the sequence ledger
/// down with it.
#[test]
fn ordinal_free_change_ids_do_not_block_numeric_sequence_validation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let frozen_owner = "CHG-0163-frozen-owner";
    write_located_fixture(
        root,
        &root
            .join(ARCHIVE_PATH)
            .join("2026-08-20-CHG-0163-frozen-owner"),
        frozen_owner,
        ChangeState::Archived,
    );
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 163,
            id: frozen_owner.into(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    // Vacuity control: the fixture must already validate, or a validator that always returned
    // Ok would satisfy the assertion below without doing anything.
    validate_change_sequences(root).expect("the frozen ledger fixture must validate on its own");

    write_located_fixture(
        root,
        &change_dir(root, "a-slug-only-change"),
        "a-slug-only-change",
        ChangeState::Draft,
    );
    validate_change_sequences(root)
        .expect("an ID that claims no ordinal must not fail numeric sequence validation");

    // Two of them. A skip implemented as `unwrap_or(0)` passes the single-record case and then
    // reports `duplicate numeric change sequence CHG-0000` for the second slug-only change ever
    // created, with remediation that is impossible once nothing mints.
    write_located_fixture(
        root,
        &change_dir(root, "a-second-slug-only-change"),
        "a-second-slug-only-change",
        ChangeState::Draft,
    );
    validate_change_sequences(root)
        .expect("two IDs that claim no ordinal collide with each other in no numeric sequence");

    let mut located: Vec<String> = located_change_sequences(root)
        .unwrap()
        .into_iter()
        .map(|change| change.id)
        .collect();
    located.sort();
    assert_eq!(
        located,
        vec![
            frozen_owner.to_string(),
            "a-second-slug-only-change".to_string(),
            "a-slug-only-change".to_string(),
        ],
        "an ordinal-free record must still be located; it is only absent from ordinal accounting"
    );
}

/// DISCRIMINATOR. Fails on `b3f3201a`, which reports `invalid change ID` instead.
///
/// The ordinal made a change ID unique by construction and the numeric-collision gate caught a
/// duplicate as a side effect. A slug-only ID is unique only by convention: two clones can
/// archive the same slug on different days into differently dated directories, which git
/// merges without a conflict. `change audit` runs with `include_archive_integrity = false`, so
/// nothing else in the product ever looks at two archives together.
#[test]
fn two_workspaces_claiming_one_change_id_are_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    for date in ["2026-08-20", "2026-08-21"] {
        write_located_fixture(
            root,
            &root
                .join(ARCHIVE_PATH)
                .join(format!("{date}-greeter-must-support-a-custom-salutation")),
            "greeter-must-support-a-custom-salutation",
            ChangeState::Archived,
        );
    }
    let error = validate_change_sequences(root)
        .expect_err("two packages claiming one change ID must fail closed");
    assert!(
        error.contains("duplicate change ID `greeter-must-support-a-custom-salutation`"),
        "got: {error}"
    );
    assert!(error.contains("2026-08-20-greeter"), "got: {error}");
    assert!(error.contains("2026-08-21-greeter"), "got: {error}");
}

/// VACUITY CONTROL. Passes on `b3f3201a` and after.
///
/// Skipping ordinal-free IDs must not neuter the gate for the IDs that do carry one: eleven
/// archived packages in this repository share five ordinals, and the acknowledgement that
/// makes them legal is the only thing standing between them and a hard refusal.
#[test]
fn two_archived_packages_sharing_an_ordinal_are_still_refused_until_acknowledged() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let ids = ["CHG-0007-first-claim", "CHG-0007-second-claim"];
    for (date, id) in ["2026-07-11", "2026-07-13"].iter().zip(ids) {
        write_located_fixture(
            root,
            &root.join(ARCHIVE_PATH).join(format!("{date}-{id}")),
            id,
            ChangeState::Archived,
        );
    }
    let error = validate_change_sequences(root)
        .expect_err("an unacknowledged ordinal collision must still fail closed");
    assert!(
        error.contains("duplicate numeric change sequence CHG-0007"),
        "got: {error}"
    );
    for id in ids {
        assert!(error.contains(id), "got: {error}");
    }

    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 7,
            id: ids[1].into(),
            acknowledged_collisions: vec![ChangeSequenceCollision {
                sequence: 7,
                ids: ids.iter().map(|id| id.to_string()).collect(),
            }],
        },
    )
    .unwrap();
    validate_change_sequences(root)
        .expect("an exact, fully immutable acknowledgement restores validity");
}

/// VACUITY CONTROL. Passes on `b3f3201a` and after.
///
/// An ID that claims an ordinal and gets the notation wrong must keep failing closed. Skipping
/// every unparseable ID — rather than only the ones that claim nothing — would drop such a
/// record out of the acknowledged-collision ID-set check that guards the archived collision
/// members, silently.
#[test]
fn a_non_canonical_ordinal_notation_still_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_located_fixture(
        root,
        &root.join(ARCHIVE_PATH).join("2026-07-13-CHG-16-narrow"),
        "CHG-16-narrow",
        ChangeState::Archived,
    );
    let error = validate_change_sequences(root)
        .expect_err("a malformed ordinal claim must not be silently skipped");
    assert!(error.contains("CHG-16-narrow"), "got: {error}");
}

/// Discriminating test for #673: a verification commit orphaned by a rebase, with
/// byte-identical delivery inputs, must be reopenable.
#[test]
fn orphaned_verification_commit_reopens_even_though_inputs_are_unchanged() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    write_lifecycle_test_policy(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&["switch", "-c", "feature"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept"]);
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    let verification = load_verification(root, &record).unwrap();
    let signed = verification.acceptance_input_digest.clone().unwrap();

    // Orphan the verification commit; leave the remote default behind.
    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);

    // The delivery inputs did NOT drift.
    let live = if let Some(manifest) = &verification.acceptance_manifest {
        let current = acceptance_manifest_with_signed_owners(root, &record, &[], manifest).unwrap();
        acceptance_manifest_digest(&current).unwrap()
    } else {
        acceptance_input_digest(root, &record, &[]).unwrap()
    };
    assert_eq!(live, signed, "delivery inputs must be byte-identical");
    // But the evidence is unanchored, and `check` says so.
    assert!(!accepted_evidence_is_anchored(root, &record, &verification));
    let closing = ensure_closing_approval_valid(root, &record).unwrap_err();
    assert!(closing.contains("not in current history"), "{closing}");
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!(
            "run `specsync change reopen {} --actor <name> --reason <reason>`",
            record.id
        )
    );

    // The verb `status` names must actually work.
    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "the verification commit was orphaned by a rebase".into(),
    )
    .expect("reopen must fire when the verification commit is unreachable");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.stale_acceptance_input_digest,
        reopened.audit.current_acceptance_input_digest
    );
    assert_eq!(
        reopened.audit.stale_evidence_cause,
        Some(ReopenCauseV1::VerificationCommitUnanchored)
    );
    // The sibling sequence-history validator must accept this reopen too.
    let reloaded = load_change(root, &record.id).unwrap();
    assert!(reopened_change_preserves_sequence_history(root, &reloaded));
}

/// #677: a squash-merged workflow-v2 change is recorded on the default branch under its ARCHIVE
/// path, never its active one — `finalize` accepts and archives inside the same pull request, so a
/// squash collapses create-and-archive into one commit where the workspace is already archived.
///
/// DISCRIMINATOR. Before the fix `accepted_change_is_recorded_in_ref` asked only about
/// `.specsync/changes/<id>/state.json`, which such a branch never contains, so this returned false.
/// Measured across this repository's own archives, that made 100 of 172 read as unanchored.
#[test]
fn a_squash_merged_archive_is_recorded_under_its_archive_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    write_lifecycle_test_policy(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["switch", "-c", "feature"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    archive_change(root, &record.id).unwrap();
    record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Archived);
    git(&["add", "-A"]);
    git(&["commit", "-m", "archive"]);

    // Squash the whole branch onto main: main gets ONE commit in which the workspace is already
    // archived, so the ACTIVE path never appears on main at any commit.
    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let active = format!(".specsync/changes/{}/state.json", record.id);
    let log = Command::new("git")
        .args(["log", "--format=%H", "main", "--", &active])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&log.stdout).trim().is_empty(),
        "the active path must be absent from main — that is the whole premise"
    );

    assert!(
        accepted_change_is_recorded_on_remote_default(root, &record),
        "a squash-merged archive is recorded on the default branch under its archive path"
    );
}

/// CONTROL for the above: a record the default branch has never seen in ANY location must still
/// read as unrecorded. Without this, widening the predicate to consult the archive path could be
/// satisfied by matching nothing at all.
#[test]
fn an_archive_absent_from_the_default_branch_is_not_recorded() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    write_lifecycle_test_policy(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    // origin/main is pinned to the base commit and never advances.
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&["switch", "-c", "feature"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    archive_change(root, &record.id).unwrap();
    record = load_change(root, &record.id).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-m", "archive"]);

    assert!(
        !accepted_change_is_recorded_on_remote_default(root, &record),
        "the default branch has never seen this change in any location"
    );
}

/// #689: a squash-merge rewrites the recorded verification commit, so ancestry can never hold
/// again. Readiness is a CONTENT question — the evidence passed, the plan on disk is the plan that
/// was verified, the tree on disk is the tree that was verified.
///
/// HONEST LABEL: this is a CHARACTERIZATION test, not a discriminator. It passes on the unfixed
/// binary too, because `verification_is_current` was ALWAYS content-only — the ancestry walk was
/// removed from these paths long ago, with the reasoning recorded at `verification_is_current`
/// and `validate_verification_for_commit_binding`. Nothing here was broken.
///
/// What #689 fixed is that ship-status never asked this question; it asked about commit
/// reachability instead. That defect lives in `ship_status_report`, and the behaviour change is
/// judged by `ship_status_is_ready_after_a_squash_that_preserves_content` beside it. This test
/// pins the property that fix RELIES ON: that content currency is indifferent to a squash.
#[test]
fn recorded_verification_survives_a_squash_that_preserves_content() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "T"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    write_lifecycle_test_policy(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["switch", "-c", "feature"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record verification"]);
    let verification = load_verification(root, &record).unwrap();

    // Squash the branch onto main: content identical, recorded commit unreachable.
    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);

    assert!(
        !verification_commit_is_accepted_current(root, &verification),
        "the premise: the recorded commit is no longer reachable after the squash"
    );
    assert!(
        recorded_verification_is_current(root, &record),
        "content is unchanged, so the evidence is current regardless of the squash"
    );
}

/// VACUITY CONTROL for the above. Behaviour only, no message text, and it passes on BOTH the fixed
/// and unfixed binaries: evidence that does not match the tree must read as stale.
///
/// Asserts against the comparison directly rather than by mutating the tree and re-reading it.
/// `project_input_digest` memoizes into a thread-local read scope, so a single process cannot
/// observe the digest move — the CLI can, because each invocation is a new process, but a unit
/// test cannot, and a control that measures a cache is not a control. Substituting a
/// non-matching `workspace_digest` tests the same predicate without depending on re-hashing.
#[test]
fn recorded_verification_is_stale_when_the_workspace_digest_does_not_match() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "T"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    write_lifecycle_test_policy(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(root, &record.id).unwrap();

    let mut evidence = load_verification(root, &record).unwrap();
    assert!(
        verification_is_current(root, &record, &evidence),
        "control precondition: the recorded evidence matches the tree"
    );

    // A workspace digest that does not describe this tree is stale, whatever the commit says.
    evidence.workspace_digest = "0".repeat(64);
    assert!(
        !verification_is_current(root, &record, &evidence),
        "a workspace digest that does not match the tree must read as stale on any binary"
    );
}

// Honest label: with ONE horizontal rule this is an INVARIANT, not a discriminator — the old
// `split("---").nth(2).unwrap_or(&text)` also returned the whole string, because `nth(2)` is
// `None` when there are only two fields. It is kept because it pins the documented behaviour,
// and a SECOND rule is added so the case actually discriminates: with two rules the old
// expression returns a mid-document fragment and this assertion fails against it.
#[test]
fn strip_frontmatter_keeps_a_body_whose_horizontal_rules_are_not_frontmatter() {
    let text = "# Notes\n\nFirst lesson.\n\n---\n\nSecond lesson.\n\n---\n\nThird lesson.\n";

    assert_eq!(strip_frontmatter(text), text);
}

#[test]
fn strip_frontmatter_removes_real_frontmatter_and_keeps_later_rules() {
    let text = "---\nspec: change.spec.md\n---\n\nReal prose.\n\n---\n\nMore prose.\n";

    let body = strip_frontmatter(text);

    assert!(!body.contains("spec: change.spec.md"));
    assert!(body.contains("Real prose."));
    assert!(body.contains("More prose."));
}

// A fresh scaffold must not advertise itself as knowledge: pointing an author at a file that
// has learned nothing trains them to ignore the pointer, which kills the surface entirely.
//
// The fixture is the REAL generated companion, not an invented one. A previous version of this
// test used `<!-- What did this module learn? -->`, a string the product never writes for a spec
// companion, so it passed while the shipped behaviour surfaced every untouched scaffold in the
// repository. A fixture the product cannot produce proves nothing about the product.
#[test]
fn accumulated_lessons_ignores_a_context_holding_only_generated_scaffold() {
    let temp = TempDir::new().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join("specs/canary")).expect("mkdir");
    let scaffold = "---\nspec: canary.spec.md\n---\n\n## Key Decisions\n\n\
- Record architectural or design decisions relevant to this spec.\n\n## Files to Read First\n\n\
- List the most important files an agent or new developer should read.\n\n## Current Status\n\n\
- Summarize implemented behavior, active work, and known blockers.\n\n## Notes\n\n\
- Capture useful links, investigation notes, and operational context.\n";
    fs::write(root.join("specs/canary/context.md"), scaffold).expect("write");

    assert!(accumulated_lessons(root, &["canary".to_string()]).is_empty());
}

// The complement: a scaffold an author HAS written into still surfaces, and counts only the
// lines they added — otherwise the fix above would silence real lessons.
#[test]
fn accumulated_lessons_counts_only_what_an_author_added_to_a_scaffold() {
    let temp = TempDir::new().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join("specs/canary")).expect("mkdir");
    let written = "---\nspec: canary.spec.md\n---\n\n## Key Decisions\n\n\
- Record architectural or design decisions relevant to this spec.\n\
- The retry budget is per-host, not per-request; a shared budget starved slow hosts.\n";
    fs::write(root.join("specs/canary/context.md"), written).expect("write");

    assert_eq!(
        accumulated_lessons(root, &["canary".to_string()]),
        vec![("specs/canary/context.md".to_string(), 1)]
    );
}

#[test]
fn accumulated_lessons_counts_substantive_prose_and_skips_absent_modules() {
    let temp = TempDir::new().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join("specs/canary")).expect("mkdir");
    fs::write(
        root.join("specs/canary/context.md"),
        "---\nspec: canary.spec.md\n---\n\n# Context\n\nOne lesson.\nAnother lesson.\n",
    )
    .expect("write");

    let found = accumulated_lessons(root, &["canary".to_string(), "no_such_module".to_string()]);

    assert_eq!(found, vec![("specs/canary/context.md".to_string(), 2)]);
}

// The defect an end-to-end sandbox found and every LF unit fixture missed: a Windows-authored
// companion kept its frontmatter, so a PRISTINE scaffold was reported as knowledge and every new
// adopter on CRLF was pointed at a file that had learned nothing.
//
// The fixture is the REAL generated artifact with its line endings converted — per the lesson
// folded into specs/generator/context.md, a scaffold defect is invisible to any fixture this
// repository can produce by hand, because no untouched scaffold exists here.
#[test]
fn accumulated_lessons_ignores_a_crlf_generated_scaffold() {
    let temp = TempDir::new().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join("specs/winmod")).expect("mkdir");
    let scaffold = crate::generator::generated_context_scaffold("winmod").replace('\n', "\r\n");
    fs::write(root.join("specs/winmod/context.md"), scaffold).expect("write");

    assert!(accumulated_lessons(root, &["winmod".to_string()]).is_empty());
}

// The complement: CRLF must not silence a real lesson either. Suppression and counting have to
// agree, and before the fix they did not — counting worked on CRLF while suppression did not.
#[test]
fn accumulated_lessons_counts_authored_crlf_prose() {
    let temp = TempDir::new().expect("temp");
    let root = temp.path();
    fs::create_dir_all(root.join("specs/winmod")).expect("mkdir");
    let written = format!(
        "{}\r\n- The retry budget is per-host; a shared budget starved slow hosts.\r\n",
        crate::generator::generated_context_scaffold("winmod").replace('\n', "\r\n")
    );
    fs::write(root.join("specs/winmod/context.md"), written).expect("write");

    assert_eq!(
        accumulated_lessons(root, &["winmod".to_string()]),
        vec![("specs/winmod/context.md".to_string(), 1)]
    );
}

// The placeholder hazard on its own: the raw template's `spec: {module}.spec.md` never equals a
// real file's `spec: winmod.spec.md`, so comparing against the UNEXPANDED template leaks that one
// line the instant frontmatter survives. Asserted directly so the expansion cannot be dropped.
#[test]
fn generated_scaffold_expands_the_module_placeholder() {
    let scaffold = crate::generator::generated_context_scaffold("winmod");

    assert!(scaffold.contains("spec: winmod.spec.md"));
    assert!(!scaffold.contains("{module}"));
}

#[test]
fn strip_frontmatter_removes_crlf_frontmatter_and_keeps_later_rules() {
    let text =
        "---\r\nspec: winmod.spec.md\r\n---\r\n\r\nReal prose.\r\n\r\n---\r\n\r\nMore prose.\r\n";

    let body = strip_frontmatter(text);

    assert!(!body.contains("spec: winmod.spec.md"));
    assert!(body.contains("Real prose."));
    assert!(body.contains("More prose."));
}

// The regression #699 recorded, with the mechanism corrected: a `###` subheading inside an
// open item used to FLUSH that item before being classified as content, so one section
// carrying several subheadings became several items under the same key and application kept
// only the last. A spec section silently lost everything above its final subheading —
// including documented behaviour the change never touched.
//
// This fixture is the shape that did the damage on cmd_change.spec.md: three scenarios where
// only the third survived. Under the old ordering `items` has three entries keyed
// "Behavioral Examples"; under the fix it has one containing all three.
#[test]
fn a_content_subheading_does_not_split_a_delta_item() {
    let delta = "## MODIFIED\n\n### SPEC SECTION Behavioral Examples\n\n\
### Scenario: first\n\n- **Given** one\n\n\
### Scenario: second\n\n- **Given** two\n\n\
### Scenario: third\n\n- **Given** three\n";

    let items = parse_delta(delta).expect("delta parses");

    assert_eq!(items.len(), 1, "one section, not one item per subheading");
    assert_eq!(items[0].key, "Behavioral Examples");
    assert!(items[0].content.contains("### Scenario: first"));
    assert!(items[0].content.contains("### Scenario: second"));
    assert!(items[0].content.contains("### Scenario: third"));
}

// Honest label: this is the CONTROL. Real item headings must still end the previous item, or
// the fix above would merge distinct sections into one.
#[test]
fn a_real_item_heading_still_ends_the_previous_item() {
    let delta = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nFirst body.\n\n\
### SPEC SECTION Invariants\n\n1. Second body.\n";

    let items = parse_delta(delta).expect("delta parses");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].key, "Purpose");
    assert!(items[0].content.contains("First body."));
    assert!(!items[0].content.contains("Second body."));
    assert_eq!(items[1].key, "Invariants");
}

// The other route into the same silent loss: two items with one key overwrite on application,
// keeping the last. Refused rather than resolved, and the message names what would be lost.
#[test]
fn a_duplicated_section_key_is_refused_rather_than_overwritten() {
    let delta = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nOriginal body that would vanish.\n\n\
### SPEC SECTION Purpose\n\nReplacement body.\n";

    let error = parse_delta(delta).expect_err("duplicate key must be refused");

    assert!(error.contains("more than once"), "got: {error}");
    assert!(error.contains("Purpose"), "got: {error}");
}

// Verifies REQ-change-091.
//
// Honest label: this is the CONTROL for the Cargo build-lock notice. A command
// that never takes the build-directory lock must resolve no lock path at all,
// or every verification command would be probed and reported against.
#[test]
fn a_non_cargo_verification_command_resolves_no_build_lock() {
    let environment = CargoBuildEnvironment::default();

    for command in [
        vec!["python3".to_string(), "validate.py".to_string()],
        vec!["cargo".to_string(), "fmt".to_string()],
        vec!["cargo".to_string(), "tree".to_string()],
        vec![
            "cargo".to_string(),
            "install".to_string(),
            "ripgrep".to_string(),
        ],
        vec!["cargo".to_string()],
    ] {
        let (program, args) = command.split_first().expect("command has a program");
        assert_eq!(
            cargo_build_lock_path(Path::new("/project"), program, args, &environment),
            None,
            "{command:?} takes no build-directory lock"
        );
    }
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR for the path derivation. The notice may only name
// the lock the command will actually contend on; naming `target/release` while
// `cargo test` waits on `target/debug` would restore the ambiguity it removes.
#[test]
fn a_cargo_verification_command_resolves_the_lock_it_will_contend_on() {
    let root = Path::new("/project");
    let environment = CargoBuildEnvironment::default();
    let resolve = |command: &str| {
        let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
        let (program, args) = words.split_first().expect("command has a program");
        cargo_build_lock_path(root, program, args, &environment)
    };

    assert_eq!(
        resolve("cargo test"),
        Some(root.join("target/debug/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo test change::"),
        Some(root.join("target/debug/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo clippy --all-targets -- -D warnings"),
        Some(root.join("target/debug/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo test --release"),
        Some(root.join("target/release/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo bench"),
        Some(root.join("target/release/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo build --profile dev"),
        Some(root.join("target/debug/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo build --profile=coverage"),
        Some(root.join("target/coverage/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo build --target aarch64-apple-darwin"),
        Some(root.join("target/aarch64-apple-darwin/debug/.cargo-lock"))
    );
    assert_eq!(
        resolve("cargo --color never test --target-dir /elsewhere"),
        Some(Path::new("/elsewhere/debug/.cargo-lock").to_path_buf())
    );
    assert_eq!(
        resolve("cargo test --target-dir=out"),
        Some(root.join("out/debug/.cargo-lock"))
    );
    // A `--release` that is really a test-name filter after `--` must not be
    // read as a profile selector.
    assert_eq!(
        resolve("cargo test -- --release"),
        Some(root.join("target/debug/.cargo-lock"))
    );
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR for the silence half of the contract. Two target
// triples, or a target directory this cannot derive, must produce no notice at
// all rather than a notice naming a plausible-looking wrong lock.
#[test]
fn an_underivable_cargo_build_layout_resolves_no_lock() {
    let root = Path::new("/project");

    for command in [
        "cargo build --target x86_64-unknown-linux-gnu --target aarch64-apple-darwin",
        "cargo build --target custom.json",
        // A profile name is one directory component; anything else would probe
        // somewhere other than the build directory.
        "cargo build --profile ../../elsewhere",
        "cargo build --profile=..",
        // `--config` can set the very keys this cannot follow, and
        // `--manifest-path` moves the workspace whose `target/` is used.
        "cargo --config build.target-dir=\"other\" test",
        "cargo build --config profile.dev.debug=false",
        "cargo test --manifest-path crates/inner/Cargo.toml",
        "cargo test --manifest-path=crates/inner/Cargo.toml",
        // `cargo nextest run --profile ci` names a NEXTEST profile, not a Cargo
        // one, so these rules would derive `target/ci` for a run that contends
        // on `target/debug`.
        "cargo nextest run",
        "cargo nextest run --profile ci",
    ] {
        let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
        let (program, args) = words.split_first().expect("command has a program");
        assert_eq!(
            cargo_build_lock_path(root, program, args, &CargoBuildEnvironment::default()),
            None,
            "{command} has no derivable build directory"
        );
    }
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR, and it exists because the first version of this
// change CLAIMED this silence in a canonical spec without implementing it. A
// project that adds `build.target-dir` while a stale `<root>/target` remains is
// exactly the shape that turns the notice into a wrong claim, so the claim is
// asserted here rather than asserted in prose.
#[test]
fn a_cargo_config_that_moves_the_build_directory_resolves_no_lock() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(root.join(".cargo")).unwrap();
    let words = ["cargo".to_string(), "test".to_string()];
    let (program, args) = words.split_first().expect("command has a program");
    // An isolated `CARGO_HOME` so a real one on the host cannot decide this.
    let environment = CargoBuildEnvironment {
        target_dir: None,
        target_triple: None,
        cargo_home: Some(temp.path().join("cargo-home").display().to_string()),
    };

    // Honest label: CONTROL. A config that sets no layout key must not silence
    // the notice, or "there is a `.cargo/config.toml`" would be enough to turn
    // the whole feature off for most real projects.
    fs::write(
        root.join(".cargo/config.toml"),
        "[net]\ngit-fetch-with-cli = true\n",
    )
    .unwrap();
    assert_eq!(
        cargo_build_lock_path(&root, program, args, &environment),
        Some(root.join("target/debug/.cargo-lock"))
    );

    for config in [
        "[build]\ntarget-dir = \"/elsewhere\"\n",
        "[build]\ntarget = \"aarch64-apple-darwin\"\n",
        // Unstable on the pinned toolchain; SpecSync runs against whatever Cargo
        // the operator's project has, so bail rather than settle it here.
        "[build]\nbuild-dir = \"/elsewhere\"\n",
        // Whether Cargo's `[env]` table feeds its own build layout is unsettled,
        // and silence-when-unsure is the rule this notice lives by.
        "[env]\nCARGO_TARGET_DIR = \"/elsewhere\"\n",
        "[env]\nCARGO_BUILD_TARGET = \"aarch64-apple-darwin\"\n",
        // Unparsable is not "says nothing".
        "[build\ntarget-dir =\n",
    ] {
        fs::write(root.join(".cargo/config.toml"), config).unwrap();
        assert_eq!(
            cargo_build_lock_path(&root, program, args, &environment),
            None,
            "a config in scope moves the build directory: {config:?}"
        );
    }

    // A parent directory's config counts too — Cargo merges upward from the
    // working directory, so stopping at the project root would miss it.
    fs::write(
        root.join(".cargo/config.toml"),
        "[net]\ngit-fetch-with-cli = true\n",
    )
    .unwrap();
    fs::create_dir_all(temp.path().join(".cargo")).unwrap();
    fs::write(
        temp.path().join(".cargo/config"),
        "[build]\ntarget-dir = \"/elsewhere\"\n",
    )
    .unwrap();
    assert_eq!(
        cargo_build_lock_path(&root, program, args, &environment),
        None,
        "an ancestor's config moves the build directory"
    );

    // And so does `CARGO_HOME`, which is not on the ancestor path at all. Until
    // this case existed the whole `cargo_home` leg was dead in test: the earlier
    // version of this test pointed it at a directory that never existed.
    fs::remove_file(temp.path().join(".cargo/config")).unwrap();
    let cargo_home = temp.path().join("cargo-home");
    fs::create_dir_all(&cargo_home).unwrap();
    assert_eq!(
        cargo_build_lock_path(&root, program, args, &environment),
        Some(root.join("target/debug/.cargo-lock")),
        "an empty CARGO_HOME must not silence the notice"
    );
    fs::write(
        cargo_home.join("config.toml"),
        "[build]\ntarget-dir = \"/elsewhere\"\n",
    )
    .unwrap();
    assert_eq!(
        cargo_build_lock_path(&root, program, args, &environment),
        None,
        "a CARGO_HOME config moves the build directory"
    );
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR for environment-driven layout. `CARGO_TARGET_DIR`
// moves the lock, and a notice pointing at `<root>/target` there would name a
// file nothing is waiting on.
#[test]
fn cargo_target_dir_environment_moves_the_resolved_lock() {
    let root = Path::new("/project");
    let words = ["cargo".to_string(), "test".to_string()];
    let (program, args) = words.split_first().expect("command has a program");

    let environment = CargoBuildEnvironment {
        target_dir: Some("/shared/target".to_string()),
        target_triple: None,
        cargo_home: None,
    };
    assert_eq!(
        cargo_build_lock_path(root, program, args, &environment),
        Some(Path::new("/shared/target/debug/.cargo-lock").to_path_buf())
    );

    let environment = CargoBuildEnvironment {
        target_dir: None,
        target_triple: Some("aarch64-apple-darwin".to_string()),
        cargo_home: None,
    };
    assert_eq!(
        cargo_build_lock_path(root, program, args, &environment),
        Some(root.join("target/aarch64-apple-darwin/debug/.cargo-lock"))
    );
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR, and the load-bearing one for the notice itself.
// The lock is held for real by a second open file description in this process —
// `flock` treats those independently — so the held branch is exercised
// deterministically, with no dependence on timing or on host load.
#[test]
fn a_held_cargo_build_lock_produces_a_notice_naming_the_lock() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let build = root.join("target/debug");
    fs::create_dir_all(&build).unwrap();
    let lock_path = build.join(".cargo-lock");
    fs::write(&lock_path, b"").unwrap();
    let words = ["cargo".to_string(), "test".to_string()];
    let (program, args) = words.split_first().expect("command has a program");
    let environment = CargoBuildEnvironment::default();

    // Honest label: CONTROL. Nothing holds the lock yet, so there is nothing to
    // report; without this the test could not tell a working probe from one
    // that reports contention unconditionally.
    assert_eq!(
        cargo_build_lock_wait_notice(root, program, args, &environment),
        None,
        "an unheld lock must produce no notice"
    );

    let holder = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    holder.lock_exclusive().unwrap();

    let notice = cargo_build_lock_wait_notice(root, program, args, &environment)
        .expect("a held lock must be reported");
    assert!(
        notice.contains("waiting on target/debug/.cargo-lock"),
        "the notice must name the lock: {notice}"
    );
    assert!(
        notice.contains("blocked rather than compiling"),
        "the notice must distinguish blocked from compiling: {notice}"
    );

    // Deliberately NOT asserted: that releasing `holder` immediately stops the
    // notice. It was, and it failed once on a heavily loaded host — a `flock`
    // lives on the open file description, and this test binary is multithreaded
    // and spawns processes constantly, so a descriptor duplicated by a
    // concurrent spawn keeps the lock alive past this thread's `drop` until that
    // child execs. "Released" is therefore not observable at a deterministic
    // instant from inside this process, and an assertion on it is a gate that
    // depends on machine load, which is the shape #707 and #702 are about.
    drop(holder);
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR for the Linux holder lookup, exercised through the
// pure parser so it runs on every platform. The wiring that reads `/proc/locks`
// and stats the lock file is Linux-only and is not covered here.
#[test]
fn proc_locks_names_only_the_flock_write_holder_of_the_lock_file() {
    let contents = "\
1: POSIX  ADVISORY  WRITE 4001 08:01:7654321 0 EOF
2: FLOCK  ADVISORY  WRITE 75733 08:01:7654321 0 EOF
2: -> FLOCK  ADVISORY  WRITE 75999 08:01:7654321 0 EOF
3: FLOCK  ADVISORY  READ 4100 08:01:7654321 0 EOF
4: FLOCK  ADVISORY  WRITE 4200 08:01:9999999 0 EOF
5: OFDLCK ADVISORY  WRITE -1 08:01:7654321 0 EOF
";

    assert_eq!(
        proc_locks_flock_holders(contents, 8, 1, 7_654_321),
        vec![75733],
        "only the FLOCK write holder of this inode is a holder"
    );

    // Honest label: CONTROL. An inode nothing locks must name nobody, so an
    // empty result reads as "no holder found" rather than "parser broken".
    assert_eq!(
        proc_locks_flock_holders(contents, 8, 1, 1_111_111),
        Vec::<u32>::new()
    );
}

// ─── Handoff readiness (REQ-change-093) ──────────────────────────────────────

fn handoff_signals(state: ChangeState) -> HandoffSignals {
    HandoffSignals {
        state,
        workflow_version: 2,
        sequence_frozen: false,
        open_questions: false,
        artifacts_complete: true,
        approval_valid: true,
        correction_valid: true,
        scoped_edits_uncommitted: Some(false),
        verification_current: true,
        scoped_review_current: true,
        terminal_evidence_stale: false,
    }
}

fn assert_handoff_has_no_digest(summary: &HandoffSummary) {
    let text = format!(
        "{} {} {}",
        summary.reason,
        summary.resume,
        summary.before_clearing.join(" ")
    );
    assert!(
        !text
            .split(|c: char| !c.is_ascii_hexdigit())
            .any(|word| word.len() >= 40),
        "handoff prose must never carry a digest: {text}"
    );
}

// Verifies REQ-change-093.
#[test]
fn handoff_resume_is_always_change_status_and_never_a_digest() {
    for state in [
        ChangeState::Draft,
        ChangeState::Approved,
        ChangeState::Implementing,
        ChangeState::Verifying,
        ChangeState::Accepted,
        ChangeState::Archived,
    ] {
        let summary = classify_handoff("add-passkeys", &handoff_signals(state));
        assert_eq!(summary.resume, "specsync change status add-passkeys");
        assert_handoff_has_no_digest(&summary);
        if summary.readiness != HandoffReadiness::Safe {
            assert!(
                !summary.before_clearing.is_empty(),
                "{state:?}: a verdict that is not safe must name what to do first"
            );
        }
    }
}

// Verifies REQ-change-093.
#[test]
fn handoff_sequence_freeze_outranks_every_state() {
    for state in [
        ChangeState::Draft,
        ChangeState::Verifying,
        ChangeState::Archived,
    ] {
        let mut signals = handoff_signals(state);
        signals.sequence_frozen = true;
        let summary = classify_handoff("add-passkeys", &signals);
        assert_eq!(summary.readiness, HandoffReadiness::NotYet, "{state:?}");
        assert!(summary.before_clearing[0].contains("freeze"));
    }
}

// Verifies REQ-change-093.
#[test]
fn handoff_draft_is_never_safe_and_names_the_next_boundary() {
    let mut signals = handoff_signals(ChangeState::Draft);
    signals.open_questions = true;
    signals.artifacts_complete = false;
    let questions = classify_handoff("add-passkeys", &signals);
    assert_eq!(questions.readiness, HandoffReadiness::Conditional);
    assert!(questions.before_clearing[0].contains("specsync change answer add-passkeys"));
    assert!(questions.before_clearing[1].contains(".specsync/changes/add-passkeys/change.md"));

    signals.open_questions = false;
    let stubs = classify_handoff("add-passkeys", &signals);
    assert_eq!(stubs.readiness, HandoffReadiness::Conditional);
    assert!(stubs.reason.contains("stubs"));
    assert!(stubs.before_clearing[0].contains("specsync change approve add-passkeys"));

    signals.artifacts_complete = true;
    let unapproved = classify_handoff("add-passkeys", &signals);
    assert_eq!(unapproved.readiness, HandoffReadiness::Conditional);
    assert_eq!(
        unapproved.before_clearing,
        vec!["run `specsync change approve add-passkeys --actor <name>`".to_string()]
    );
}

// Verifies REQ-change-093.
#[test]
fn handoff_stale_approval_is_not_yet_in_every_approved_state() {
    for state in [
        ChangeState::Approved,
        ChangeState::Implementing,
        ChangeState::Verifying,
    ] {
        let mut signals = handoff_signals(state);
        signals.approval_valid = false;
        // A dirty tree and stale evidence must not hide the stale approval behind a softer verdict.
        signals.scoped_edits_uncommitted = Some(true);
        signals.verification_current = false;
        let summary = classify_handoff("add-passkeys", &signals);
        assert_eq!(summary.readiness, HandoffReadiness::NotYet, "{state:?}");
        assert!(summary.reason.contains("changed after it was approved"));
        assert!(summary.before_clearing[0].contains("specsync change approve add-passkeys"));
    }
}

// Verifies REQ-change-093.
#[test]
fn handoff_uncommitted_scoped_edits_are_conditional_and_name_change_md() {
    for state in [
        ChangeState::Approved,
        ChangeState::Implementing,
        ChangeState::Verifying,
    ] {
        let mut signals = handoff_signals(state);
        signals.scoped_edits_uncommitted = Some(true);
        let summary = classify_handoff("add-passkeys", &signals);
        assert_eq!(
            summary.readiness,
            HandoffReadiness::Conditional,
            "{state:?}"
        );
        assert_eq!(summary.before_clearing[0], "commit the work in progress");
        assert!(summary.before_clearing[1].contains(".specsync/changes/add-passkeys/change.md"));
    }
    // Outside a Git repository nothing is committed anyway; the unknown reads as clean.
    let mut signals = handoff_signals(ChangeState::Approved);
    signals.scoped_edits_uncommitted = None;
    assert_eq!(
        classify_handoff("add-passkeys", &signals).readiness,
        HandoffReadiness::Safe
    );
}

// Verifies REQ-change-093.
#[test]
fn handoff_approved_and_implementing_with_a_clean_tree_are_safe() {
    for state in [ChangeState::Approved, ChangeState::Implementing] {
        let summary = classify_handoff("add-passkeys", &handoff_signals(state));
        assert_eq!(summary.readiness, HandoffReadiness::Safe, "{state:?}");
        assert!(summary.reason.contains("`change check`"));
        assert!(summary.reason.contains("this tree"));
        assert!(
            !summary.reason.contains("committed"),
            "a clean affected-path list is not a commit: {summary:?}"
        );
        assert!(summary.before_clearing.is_empty());
    }
}

// Verifies REQ-change-093.
#[test]
fn handoff_verifying_follows_evidence_currency() {
    let mut signals = handoff_signals(ChangeState::Verifying);
    signals.verification_current = false;
    signals.scoped_review_current = false;
    let stale = classify_handoff("add-passkeys", &signals);
    assert_eq!(stale.readiness, HandoffReadiness::Conditional);
    assert_eq!(
        stale.before_clearing,
        vec!["run `specsync change check add-passkeys --commit`".to_string()]
    );

    signals.verification_current = true;
    let awaiting_review = classify_handoff("add-passkeys", &signals);
    assert_eq!(awaiting_review.readiness, HandoffReadiness::Safe);
    assert!(awaiting_review.reason.contains("independent review"));
    assert!(awaiting_review.reason.contains("this working tree"));
    // `HandoffSignals` carries no commit signal. Currency is this working tree,
    // and `change check` without `--commit` leaves the evidence untracked.
    // The reason may only claim what the classifier checked.
    assert!(
        !awaiting_review.reason.contains("committed"),
        "verification currency is not a commit: {awaiting_review:?}"
    );
    assert!(awaiting_review.reason.contains("verification is current"));

    signals.scoped_review_current = true;
    let ready = classify_handoff("add-passkeys", &signals);
    assert_eq!(ready.readiness, HandoffReadiness::Safe);
    assert!(ready.reason.contains("finalize"));
    assert!(ready.reason.contains("do not commit"));
    assert!(!ready.reason.contains("verification is committed"));
}

// Verifies REQ-change-093.
#[test]
fn handoff_accepted_depends_on_workflow_and_ledgers() {
    let mut signals = handoff_signals(ChangeState::Accepted);
    signals.correction_valid = false;
    let corrupt = classify_handoff("add-passkeys", &signals);
    assert_eq!(corrupt.readiness, HandoffReadiness::NotYet);
    assert!(corrupt.before_clearing[0].contains("corrections.json"));

    signals.correction_valid = true;
    // Workflow v2 acceptance is a recorded boundary even when legacy evidence would be stale.
    signals.terminal_evidence_stale = true;
    let v2 = classify_handoff("add-passkeys", &signals);
    assert_eq!(v2.readiness, HandoffReadiness::Safe);
    assert!(v2.reason.contains("finalize"));

    signals.workflow_version = 1;
    let legacy_stale = classify_handoff("add-passkeys", &signals);
    assert_eq!(legacy_stale.readiness, HandoffReadiness::NotYet);
    assert!(legacy_stale.before_clearing[0].contains("specsync change reopen add-passkeys"));

    signals.terminal_evidence_stale = false;
    let legacy_current = classify_handoff("add-passkeys", &signals);
    assert_eq!(legacy_current.readiness, HandoffReadiness::Safe);
    assert!(legacy_current.reason.contains("archive"));
}

// Verifies REQ-change-093.
#[test]
fn handoff_archived_is_safe_with_nothing_to_do() {
    let summary = classify_handoff("add-passkeys", &handoff_signals(ChangeState::Archived));
    assert_eq!(summary.readiness, HandoffReadiness::Safe);
    assert!(summary.before_clearing.is_empty());
}

// Verifies REQ-change-093.
#[test]
fn handoff_readiness_serializes_kebab_case_and_prints_two_words() {
    assert_eq!(
        serde_json::to_string(&HandoffReadiness::NotYet).unwrap(),
        "\"not-yet\""
    );
    assert_eq!(HandoffReadiness::NotYet.as_str(), "not yet");
    let summary = classify_handoff("add-passkeys", &handoff_signals(ChangeState::Archived));
    let json = serde_json::to_value(&summary).unwrap();
    assert!(
        json.get("before_clearing").is_none(),
        "an empty step list is omitted, not printed as []"
    );
}

/// The only signal the classifier cannot be handed by a test: whether the TREE under this
/// change's paths is dirty. Evidence under `.specsync/` must never count — `review` then
/// `finalize` runs with `review.json` uncommitted by design — while one edit under
/// `affected_paths` must.
// Verifies REQ-change-093.
#[test]
fn handoff_follows_the_lifecycle_and_ignores_uncommitted_lifecycle_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);

    let record = current_workflow_record(root, completed_no_spec_record(root));
    let draft = handoff_summary(root, &record);
    assert_eq!(draft.readiness, HandoffReadiness::Conditional);
    assert!(draft.before_clearing[0].contains("specsync change approve"));

    let approved = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    // `src/` is still untracked: the work is on disk, its intent is not.
    let dirty = handoff_summary(root, &approved);
    assert_eq!(dirty.readiness, HandoffReadiness::Conditional);
    assert_eq!(dirty.before_clearing[0], "commit the work in progress");

    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    let clean = handoff_summary(root, &approved);
    assert_eq!(clean.readiness, HandoffReadiness::Safe, "{clean:?}");
    assert!(clean.reason.contains("`change check`"));

    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    git(&["add", "."]);
    git(&["commit", "-m", "Record verification"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    let verifying = load_change(root, &record.id).unwrap();
    assert_eq!(verifying.state, ChangeState::Verifying);
    let awaiting_review = handoff_summary(root, &verifying);
    assert_eq!(
        awaiting_review.readiness,
        HandoffReadiness::Safe,
        "{awaiting_review:?}"
    );
    assert!(awaiting_review.reason.contains("independent review"));

    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    // review.json is uncommitted, exactly as the lifecycle wants it before finalize.
    let reviewed = handoff_summary(root, &verifying);
    assert_eq!(reviewed.readiness, HandoffReadiness::Safe, "{reviewed:?}");
    assert!(reviewed.reason.contains("finalize"));

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let edited = handoff_summary(root, &verifying);
    assert_eq!(
        edited.readiness,
        HandoffReadiness::Conditional,
        "{edited:?}"
    );
    assert!(edited.reason.contains("uncommitted edits"));
    git(&["checkout", "--", "src/lib.rs"]);

    let destination = finalize_change(root, &record.id).unwrap();
    assert!(destination.is_dir());
    let archived = load_change(root, &record.id).unwrap();
    let done = handoff_summary(root, &archived);
    assert_eq!(done.readiness, HandoffReadiness::Safe);
    assert!(done.before_clearing.is_empty());
}

// Verifies REQ-change-093.
#[test]
fn change_summary_carries_the_same_handoff_the_domain_computes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let summary = summarize_change(root, &record);
    assert_eq!(summary.handoff, handoff_summary(root, &record));
    let json = serde_json::to_value(&summary).unwrap();
    assert_eq!(json["handoff"]["readiness"], "conditional");
    assert_eq!(
        json["handoff"]["resume"],
        format!("specsync change status {}", record.id)
    );
}

#[test]
fn bundled_lifecycle_limits_match_the_github_scripts_copy() {
    let bundled = include_str!("../lifecycle-validation-limits.json");
    let github = include_str!("../../.github/scripts/lifecycle-validation-limits.json");
    assert_eq!(
        bundled, github,
        "src/lifecycle-validation-limits.json must stay byte-identical to the CI copy so crates.io and release.yml cannot drift"
    );
}

#[test]
fn canonical_definition_payload_folds_crlf_to_lf() {
    let lf = b"## Purpose\n\nHello\n";
    let crlf = b"## Purpose\r\n\r\nHello\r\n";
    assert_eq!(
        canonical_definition_artifact_payload("change.md", lf),
        canonical_definition_artifact_payload("change.md", crlf)
    );
    assert_eq!(
        canonical_definition_artifact_payload("change.md", crlf),
        b"## Purpose\n\nHello\n"
    );
}

#[test]
fn canonical_definition_payload_preserves_a_lone_carriage_return() {
    let payload = b"progress\rbar\n";
    assert_eq!(
        canonical_definition_artifact_payload("change.md", payload),
        payload
    );
}

#[test]
fn canonical_tasks_payload_folds_crlf_before_checkbox_rewrite() {
    let crlf = b"- [x] done\r\n- [ ] open\r\n";
    let lf = b"- [x] done\n- [ ] open\n";
    assert_eq!(
        canonical_definition_artifact_payload("pkg/tasks.md", crlf),
        canonical_definition_artifact_payload("pkg/tasks.md", lf)
    );
    let rewritten = canonical_definition_artifact_payload("pkg/tasks.md", crlf);
    assert!(
        rewritten.windows(4).any(|w| w == b"[ ] "),
        "checkbox rewrite must still run after CRLF fold, got {rewritten:?}"
    );
}

#[test]
fn deleting_verification_attempts_refuses_adopted_finalization() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);

    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();

    let attempts_path = change_dir(root, &record.id).join("verification-attempts.json");
    assert!(attempts_path.is_file(), "verify must have written a ledger");
    fs::remove_file(&attempts_path).unwrap();

    git(&[
        "commit",
        "--allow-empty",
        "-m",
        "move HEAD after verification",
    ]);

    let error = accept_change_with_gate(root, &record.id, None, None, "finalization", true, false)
        .expect_err("missing attempts ledger must not adopt HEAD");
    assert!(
        error.contains("cannot adopt the implementation commit"),
        "expected empty-ledger adoption refusal, got {error}"
    );
}

#[test]
fn emptying_verification_attempts_refuses_adopted_finalization() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);

    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();

    let attempts_path = change_dir(root, &record.id).join("verification-attempts.json");
    fs::write(&attempts_path, "{\"schema_version\":1,\"attempts\":[]}\n").unwrap();

    git(&[
        "commit",
        "--allow-empty",
        "-m",
        "move HEAD after verification",
    ]);

    let error = accept_change_with_gate(root, &record.id, None, None, "finalization", true, false)
        .expect_err("empty attempts ledger must not adopt HEAD");
    assert!(
        error.contains("cannot adopt the implementation commit"),
        "expected empty-ledger adoption refusal, got {error}"
    );
}

#[test]
fn verifying_change_refuses_to_recreate_a_missing_attempts_ledger() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_current_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);

    let attempts_path = change_dir(root, &record.id).join("verification-attempts.json");
    fs::remove_file(&attempts_path).unwrap();
    let verification = load_verification(root, &record).unwrap();
    let error = record_verification_attempt(root, &record, &verification)
        .expect_err("Verifying must not mint a fresh empty ledger");
    assert!(
        error.contains("verification attempt history is missing"),
        "got {error}"
    );
}

// Verifies REQ-change-102.
//
// Honest label: DISCRIMINATOR for the ownership answer the lifecycle staging path trusts. The
// exact-set assertion is the point. A version that returned `affected_paths` would hand the
// staging path `src/` (or `.`) and bring back the `git add -A` sweep through the side door, and a
// version that returned the spec directory would own any stray file dropped beside a spec.
#[test]
fn lifecycle_commit_scope_names_exactly_what_the_change_owns() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "base"]);
    ensure_auth_spec_owns_its_source(root);
    let record = completed_current_record(root);
    let workspace = format!(".specsync/changes/{}", record.id);

    let mut expected = vec![
        workspace.clone(),
        SEQUENCE_PATH.to_string(),
        WORKFLOW_V2_BASELINE_PATH.to_string(),
        LEGACY_BASELINE_PATH.to_string(),
        BOOTSTRAP_RECORD_PATH.to_string(),
        HASH_CACHE_PATH.to_string(),
        "specs/auth/auth.spec.md".to_string(),
    ];
    expected.extend(
        canonical_companion_names("auth")
            .into_iter()
            .map(|companion| format!("specs/auth/{companion}")),
    );
    expected.sort();
    let scope = lifecycle_commit_scope(root, &record.id).unwrap();
    assert_eq!(scope.owned, expected);
    assert!(
        !scope
            .owned
            .iter()
            .any(|path| path == "src/auth.rs" || path == "src" || path == "specs/auth"),
        "affected paths and spec directories are not lifecycle-owned"
    );
    assert_eq!(
        scope.runtime,
        vec![LOCK_PATH.to_string(), TRANSACTION_PATH.to_string()],
        "the lock and journal are runtime files, never owned delivery"
    );
    assert!(
        !scope.owned.iter().any(|path| scope.runtime.contains(path)),
        "no path may be both committed and never-committed"
    );

    // Once `finalize` has moved the workspace, the package it moved into is owned as well, and
    // the vacated workspace stays owned so the removal of its tracked files can be staged.
    let package = format!("{ARCHIVE_PATH}/2026-01-01-{}", record.id);
    fs::create_dir_all(root.join(ARCHIVE_PATH)).unwrap();
    fs::rename(change_dir(root, &record.id), root.join(&package)).unwrap();
    let archived = lifecycle_commit_scope(root, &record.id).unwrap().owned;
    assert!(archived.contains(&package), "{archived:?}");
    assert!(archived.contains(&workspace), "{archived:?}");

    assert!(lifecycle_commit_scope(root, "no-such-change").is_err());
}

fn interview_complete_bugfix(root: &Path, module: &str) -> ChangeRecord {
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: format!("note a constraint for {module}"),
            kind: ChangeKind::BugFix,
            affected_specs: vec![module.into()],
            affected_paths: vec!["src/lib.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("The note does not move a contract".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["The constraint is visible".into()];
    record.answers.insert("public_contract".into(), "no".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    save_change(root, &record).unwrap();
    record
}

// Verifies REQ-change-104.
#[test]
fn constraints_question_is_asked_only_for_a_draft_that_has_lessons_or_principles() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/context.md"),
        "---\nspec: auth.spec.md\n---\n\n## Notes\n\nKeep the lock around the write.\n",
    )
    .unwrap();
    let mut record = interview_complete_bugfix(root, "auth");

    let questions = next_questions_for(root, &record);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].id, "constraints");

    record.state = ChangeState::Approved;
    assert!(
        next_questions_for(root, &record).is_empty(),
        "a change that has left draft is not sent back to the interview"
    );
}

// Verifies REQ-change-104.
#[test]
fn constraints_question_is_absent_when_the_context_is_only_a_scaffold() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/context.md"),
        crate::generator::generated_context_scaffold("auth"),
    )
    .unwrap();
    let record = interview_complete_bugfix(root, "auth");

    assert!(next_questions_for(root, &record).is_empty());
}

// Verifies REQ-change-104.
#[test]
fn answering_none_completes_the_constraints_question() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/context.md"),
        "---\nspec: auth.spec.md\n---\n\nKeep the lock around the write.\n",
    )
    .unwrap();
    let record = interview_complete_bugfix(root, "auth");
    let error = answer_question(root, &record.id, "constraints", "   ").unwrap_err();
    assert!(error.contains("must not be empty"), "{error}");

    let record = answer_question(root, &record.id, "constraints", "none").unwrap();
    assert!(next_questions_for(root, &record).is_empty());
    assert_eq!(
        record.answers.get("constraints").map(String::as_str),
        Some("none")
    );
}

// Verifies REQ-change-104.
#[test]
fn changing_affected_specs_asks_the_constraints_question_again() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/context.md"),
        "---\nspec: auth.spec.md\n---\n\nKeep the lock around the write.\n",
    )
    .unwrap();
    let record = interview_complete_bugfix(root, "auth");
    let record = answer_question(root, &record.id, "constraints", "none").unwrap();
    let record = answer_question(root, &record.id, "affected_specs", "auth").unwrap();

    assert!(!record.answers.contains_key("constraints"));
    assert_eq!(next_questions_for(root, &record)[0].id, "constraints");
}

// Verifies REQ-change-104.
#[test]
fn a_constraints_answer_changes_the_scope_digest() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = interview_complete_bugfix(root, "auth");
    let before = scope_digest(root, &record).unwrap();
    record.answers.insert("constraints".into(), "none".into());
    let none = scope_digest(root, &record).unwrap();
    record
        .answers
        .insert("constraints".into(), "keep the lock".into());
    let named = scope_digest(root, &record).unwrap();

    assert_ne!(before, none);
    assert_ne!(none, named);
}

// Verifies REQ-change-105.
#[test]
fn a_plan_of_empty_headings_is_incomplete() {
    let empty = "# Plan\n\n## Approach\n\n## Out of scope\n\n## Steps\n\n## Risks\n\n## Constraints consulted\n";
    assert!(plan_sections_incomplete(empty));
    assert!(artifact_is_incomplete(&ArtifactKind::Plan, empty));
    assert!(!artifact_is_incomplete(
        &ArtifactKind::Context,
        "# Complete\n\nReviewed.\n"
    ));
}

// Verifies REQ-change-105.
#[test]
fn a_plan_with_substantive_sections_is_complete() {
    assert!(!plan_sections_incomplete(SUBSTANTIVE_PLAN));
    assert!(!artifact_is_incomplete(
        &ArtifactKind::Plan,
        SUBSTANTIVE_PLAN
    ));
    let mixed_case = "# Plan\n\n## APPROACH\n\nDo it.\n\n## Out of Scope\n\nNot that.\n\n## steps\n\nOne.\n\n## RISKS\n\nnone\n\n## Constraints Consulted\n\nnone\n";
    assert!(!plan_sections_incomplete(mixed_case));
}

// Verifies REQ-change-105. The generated heading must be `## Approach`.
// A path prefix left over from a file move makes the section invisible, so a
// plan the author filled still fails approval.
#[test]
fn a_filled_generated_plan_template_passes_artifact_validation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = interview_complete_bugfix(root, "auth");
    let template = artifact_template(root, &ArtifactKind::Plan, &record);
    assert!(
        template.contains("\n## Approach\n"),
        "generated plan heading was not recognized:\n{template}"
    );
    assert!(
        !template.contains("../"),
        "generated plan template gained a path prefix:\n{template}"
    );
    let context = artifact_template(root, &ArtifactKind::Context, &record);
    assert!(
        context.contains("<!-- What led here:"),
        "generated context prompt was not recognized:\n{context}"
    );
    assert!(
        !context.contains("../"),
        "generated context template gained a path prefix:\n{context}"
    );

    let mut filled = template.replace(
        "<!-- TODO: complete this artifact or remove it from selected_artifacts before approval. -->",
        "",
    );
    for prose in [
        "Do the work.",
        "Leave unrelated work alone.",
        "Land the change.",
        "none",
        "none",
    ] {
        let start = filled
            .find("<!--")
            .unwrap_or_else(|| panic!("generated plan is missing a section prompt:\n{filled}"));
        let end = filled[start..]
            .find("-->")
            .unwrap_or_else(|| panic!("generated plan has an unclosed prompt:\n{filled}"));
        filled.replace_range(start..start + end + 3, prose);
    }
    assert!(
        !artifact_is_incomplete(&ArtifactKind::Plan, &filled),
        "filled generated plan is still incomplete:\n{filled}"
    );
    let path = change_dir(root, &record.id).join(ArtifactKind::Plan.file_name());
    fs::write(&path, &filled).unwrap();
    validate_artifact_bodies(root, &record.id, &[ArtifactKind::Plan])
        .unwrap_or_else(|error| panic!("{error}\n{filled}"));
}

// Verifies REQ-change-105.
#[test]
fn a_plan_section_does_not_borrow_the_next_heading_or_an_html_comment() {
    let borrowed = "# Plan\n\n## Approach\n\n\n## Rollout\n\nShip it.\n\n## Out of scope\n\nNot that.\n\n## Steps\n\nOne.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n";
    assert!(plan_sections_incomplete(borrowed));
    let commented = "# Plan\n\n## Approach\n\nDo it.\n\n## Out of scope\n\nNot that.\n\n## Steps\n\nOne.\n\n## Risks\n\n<!--\nplaceholder\n-->\n\n## Constraints consulted\n\nnone\n";
    assert!(plan_sections_incomplete(commented));
    let lessons = context_substantive_lines(
        "auth",
        "---\nspec: auth.spec.md\n---\n# Context\n\n<!--\nhidden lesson\n-->\n",
    );
    assert!(lessons.is_empty(), "{lessons:?}");
}

// Verifies REQ-change-105.
#[test]
fn a_fenced_plan_cannot_be_completed_from_inside_a_code_block() {
    let quoted = r#"# Plan

## Approach

```markdown
## Steps

Looks done.
## Out of scope

no
## Risks

none
## Constraints consulted

none
```
"#;
    assert!(plan_sections_incomplete(quoted));
    assert!(artifact_is_incomplete(&ArtifactKind::Plan, quoted));
    assert!(plan_sections_incomplete(&quoted.replace("```", "~~~")));

    // ```markdown is an opening fence with an info string, not a close.
    let info_closer = "# Plan\n\n## Approach\n\n```\n```markdown\n## Steps\n\nDo it.\n\n## Out of scope\n\nNot that.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n```\n";
    assert!(plan_sections_incomplete(info_closer));

    // Four leading spaces is an indented code block, not a fence.
    let indented = "# Plan\n\n## Approach\n\nDo it.\n\n    ```\n    ## Steps\n    hidden\n    ```\n\n## Out of scope\n\nNot that.\n\n## Steps\n\nOne.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n";
    assert!(!plan_sections_incomplete(indented));

    let only_fenced_prose = r#"# Plan

## Approach

```
Do the work.
```

## Out of scope

Leave it.

## Steps

One.

## Risks

none

## Constraints consulted

none
"#;
    assert!(plan_sections_incomplete(only_fenced_prose));

    let unclosed = r#"# Plan

## Approach

Do the work.

```
## Out of scope

Leave it.

## Steps

One.

## Risks

none

## Constraints consulted

none
"#;
    assert!(plan_sections_incomplete(unclosed));

    let outside = r#"# Plan

## Approach

Do the work.

```markdown
## Approach

not this
## Steps

Looks done.
## Out of scope

no
```

## Out of scope

Leave unrelated work alone.

~~~
```
## Risks

none of these count
~~~

## Steps

Land the change.

````
## Constraints consulted

quoted
```
still quoted
````

## Risks

none

## Constraints consulted

none
"#;
    assert!(!plan_sections_incomplete(outside));
    assert!(!artifact_is_incomplete(&ArtifactKind::Plan, outside));

    let longer_close = r#"# Plan

## Approach

Do the work.

```
## Steps

example only
```` end

## Out of scope

Leave it.

## Steps

One.

## Risks

none

## Constraints consulted

none
"#;
    assert!(
        plan_sections_incomplete(longer_close),
        "an info string on a longer marker does not close the fence"
    );

    let bare_longer_close = r#"# Plan

## Approach

Do the work.

```
## Steps

example only
````

## Out of scope

Leave it.

## Steps

One.

## Risks

none

## Constraints consulted

none
"#;
    assert!(!plan_sections_incomplete(bare_longer_close));
}

// Verifies REQ-change-105.
#[test]
fn a_bug_fix_does_not_grow_a_plan() {
    let bug = adaptive_artifacts(
        ChangeKind::BugFix,
        &["auth".into()],
        &["src/auth.rs".into()],
    );
    assert!(!bug.contains(&ArtifactKind::Plan));
    let docs = adaptive_artifacts(ChangeKind::Documentation, &[], &["README.md".into()]);
    assert!(!docs.contains(&ArtifactKind::Plan));
}

// Verifies REQ-change-106.
#[test]
fn open_change_briefing_quotes_substantive_lessons_and_omits_a_scaffold() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join("specs/plain")).unwrap();
    let mut lesson = String::from("---\nspec: auth.spec.md\n---\n\n## Notes\n\n");
    for index in 0..30 {
        lesson.push_str(&format!("Lesson line {index} stays in the module.\n"));
    }
    fs::write(root.join("specs/auth/context.md"), lesson).unwrap();
    fs::write(
        root.join("specs/plain/context.md"),
        crate::generator::generated_context_scaffold("plain"),
    )
    .unwrap();
    let mut record = interview_complete_bugfix(root, "auth");
    record.affected_specs = vec!["auth".into(), "plain".into()];

    let briefing = open_change_briefing(root, &record);
    assert!(briefing.principles.is_none());
    assert_eq!(briefing.lessons.len(), 1);
    assert_eq!(briefing.lessons[0].path, "specs/auth/context.md");
    assert_eq!(briefing.lessons[0].lines, 30);
    assert!(briefing.lessons[0].truncated);
    assert!(briefing.lessons[0].excerpt.contains("Lesson line 0"));
    assert!(!briefing.lessons[0].excerpt.contains("Lesson line 29"));
}

// Verifies REQ-change-106.
#[test]
fn open_change_briefing_names_a_configured_principles_file() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.principles_file = Some("PRINCIPLES.md".into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::write(
        root.join("PRINCIPLES.md"),
        "# Principles\n\nDo not withhold a merge on folding lessons.\n",
    )
    .unwrap();
    let record = interview_complete_bugfix(root, "auth");

    let briefing = open_change_briefing(root, &record);
    let principles = briefing.principles.expect("principles");
    assert_eq!(principles.path, "PRINCIPLES.md");
    assert!(!principles.missing);
    assert!(principles.excerpt.contains("Do not withhold a merge"));
    assert!(
        next_questions_for(root, &record)
            .iter()
            .any(|question| question.id == "constraints")
    );
}

// Verifies REQ-change-106.
#[test]
fn a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/context.md"),
        "---\nspec: auth.spec.md\n---\n\nThis context.md lesson must stay hidden once the module moves.\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.3md"),
        "---\n3md: 1.0\naxis: layers\n---\n@plane z=5 label=\"Context\"\nThe retry budget is a decision, not a lesson.\n\n@plane z=1 label=\"Lessons\"\nDo not say to merge while a change is still open.\n",
    )
    .unwrap();
    let mut record = interview_complete_bugfix(root, "auth");
    record.affected_specs = vec!["auth".into()];

    let briefing = open_change_briefing(root, &record);
    assert_eq!(briefing.lessons.len(), 1);
    assert_eq!(briefing.lessons[0].path, "specs/auth/auth.3md");
    assert!(briefing.lessons[0].excerpt.contains("Do not say to merge"));
    assert!(!briefing.lessons[0].excerpt.contains("context.md lesson"));
    assert!(!briefing.lessons[0].excerpt.contains("retry budget"));
    assert_eq!(
        accumulated_lessons(root, &["auth".to_string()]),
        vec![("specs/auth/auth.3md".to_string(), 1)]
    );
    assert_eq!(
        lesson_fold_targets(root, &record.id),
        vec!["specs/auth/auth.3md".to_string()]
    );
    let archive = root.join("bundle");
    fs::create_dir_all(&archive).unwrap();
    write_lesson_bundle(root, &record, &archive).unwrap();
    let bundle = fs::read_to_string(archive.join(LESSON_BUNDLE_FILE)).unwrap();
    assert!(bundle.contains("specs/auth/auth.3md"), "{bundle}");
    assert!(!bundle.contains("specs/auth/context.md"), "{bundle}");

    fs::write(root.join("specs/auth/auth.3md"), "this is not 3md\n").unwrap();
    let fallback = open_change_briefing(root, &record);
    assert_eq!(fallback.lessons.len(), 1);
    assert_eq!(fallback.lessons[0].path, "specs/auth/context.md");
    assert!(fallback.lessons[0].excerpt.contains("context.md lesson"));
}

// Verifies REQ-change-108.
#[test]
fn change_new_refuses_a_broken_sdd_policy() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();
    let error = create_change(
        root,
        CreateChangeRequest {
            description: "note a broken policy".into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: Vec::new(),
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("the policy file itself is the defect".into()),
        },
    )
    .unwrap_err();
    assert!(error.contains("invalid SDD policy"), "{error}");
    assert!(!root.join(".specsync/changes").exists());
}

// Verifies REQ-change-108. An adopted baseline must not skip the policy parse.
#[test]
fn change_new_refuses_a_broken_policy_when_the_v2_baseline_exists() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/workflow-v2-baseline.json"),
        "{\n  \"schema_version\": 1,\n  \"domain\": \"specsync.workflow-v2-baseline.v1\",\n  \"cutoff_commit\": \"fc091c88f72a6d2fb2df168f4baa4370579ff8a2\"\n}\n",
    )
    .unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();
    let error = create_change(
        root,
        CreateChangeRequest {
            description: "note a broken policy beside a baseline".into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: Vec::new(),
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("the policy file itself is the defect".into()),
        },
    )
    .unwrap_err();
    assert!(error.contains("invalid SDD policy"), "{error}");
    assert!(!root.join(".specsync/changes").exists());
}

#[test]
fn an_escaping_principles_path_is_not_a_present_file() {
    let temp = TempDir::new().unwrap();
    let error = principles_file_is_present(temp.path(), "/etc/passwd").unwrap_err();
    assert!(error.contains("escapes the project root"), "{error}");
    let dotted = principles_file_is_present(temp.path(), "../outside.md").unwrap_err();
    assert!(dotted.contains("escapes the project root"), "{dotted}");
}

// Verifies REQ-cmd-change-021. `change status` returns this error and does not print a briefing.
#[test]
fn status_reads_the_same_broken_policy_error() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();
    let error = configured_principles_file(root).unwrap_err();
    assert!(error.contains("invalid SDD policy"), "{error}");
}

// Verifies REQ-change-106.
#[test]
fn open_change_briefing_survives_an_unreadable_context() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.principles_file = Some("missing-principles.md".into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let record = interview_complete_bugfix(root, "absent");

    let briefing = open_change_briefing(root, &record);
    assert!(briefing.lessons.is_empty());
    let principles = briefing.principles.expect("configured file");
    assert!(principles.missing);
    assert!(principles.excerpt.is_empty());
}
