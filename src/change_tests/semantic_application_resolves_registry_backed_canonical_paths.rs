
#[test]
fn no_spec_change_with_module_scope_needs_no_delta() {
    let temp = TempDir::new().unwrap();
    let record = completed_no_spec_record(temp.path());
    assert!(
        collect_requirement_ids(temp.path(), &record)
            .unwrap()
            .is_empty()
    );
    assert!(
        validate_effective_contracts(temp.path(), &[record])
            .errors
            .is_empty()
    );
}

#[test]
fn reopened_canonical_change_validates_current_canonical_contract() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## ADDED\n\n### SPEC SECTION Invariants\n\nDuplicate invariant.\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Verifying;
    record.canonical_applied = true;
    save_change(root, &record).unwrap();

    let canonical = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    let item = parse_delta(delta).unwrap().remove(0);
    assert!(
        apply_markdown_block(&canonical, "## ", &item.key, &item.content, item.operation,).is_err(),
        "the fixture must fail if an already-applied delta is replayed"
    );
    assert!(
        validate_effective_contracts(root, &[record.clone()])
            .errors
            .is_empty()
    );

    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "# Invalid current contract\n",
    )
    .unwrap();

    let errors = validate_effective_contracts(root, &[record]).errors;
    assert!(
        errors
            .iter()
            .any(|error| error.contains("effective contract `auth`")),
        "{errors:?}"
    );
}

/// The canonical spec a fresh `specsync new` leaves behind: real structure,
/// scaffold placeholder prose in `## Purpose` and `## Dependencies`.
fn write_scaffolded_auth_spec(root: &Path) {
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nDocument this module's responsibility, inputs, outputs, and ownership boundaries.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nList runtime dependencies and the specific symbols, services, or data they provide.\n\n## Legacy Notes\n\nRetained for compatibility.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
}

// Verifies REQ-change-059. The content-loss path stays closed: a `## MODIFIED`
// block with an empty body blanks the section in the canonical spec, and the
// stub warning is the only gate that sees it.
#[test]
fn effective_contract_keeps_authored_emptied_section_fatal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Dependencies\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Approved;

    let errors = validate_effective_contracts(root, &[record]).errors;

    assert!(
            errors.iter().any(|error| error
                == "effective contract `auth`: Section ## Dependencies contains only unfinished draft text"),
            "{errors:?}"
        );
}

// Verifies REQ-change-059. Same content loss through the `canonical_applied`
// path, where the delta is never replayed: authorship comes from the delta
// file, so the emptied section stays fatal.
#[test]
fn effective_contract_keeps_applied_authored_emptied_section_fatal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Dependencies\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Verifying;
    record.canonical_applied = true;
    save_change(root, &record).unwrap();
    let canonical = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    let item = parse_delta(delta).unwrap().remove(0);
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        apply_markdown_block(&canonical, "## ", &item.key, &item.content, item.operation).unwrap(),
    )
    .unwrap();

    let errors = validate_effective_contracts(root, &[record]).errors;

    assert!(
            errors.iter().any(|error| error
                == "effective contract `auth`: Section ## Dependencies contains only unfinished draft text"),
            "{errors:?}"
        );
}

// Verifies REQ-change-059.
#[test]
fn effective_contract_exempts_stub_sections_no_active_change_authored() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nPasskey material never leaves the enclave.\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Approved;
    write_scaffolded_auth_spec(root);

    let outcome = validate_effective_contracts(root, &[record]);

    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    let suppressions = outcome.suppressions;
    for section in ["Purpose", "Dependencies"] {
        assert!(
            suppressions.iter().any(|note| note.contains(&format!(
                "Section ## {section} contains only unfinished draft text"
            )) && note
                .contains(&format!("no active change authored ## {section}"))),
            "{suppressions:?}"
        );
    }
    assert!(
        !suppressions
            .iter()
            .any(|note| note.contains("## Invariants")),
        "{suppressions:?}"
    );
}

// Verifies REQ-change-059. Authorship that cannot be read exempts nothing.
#[test]
fn effective_contract_exempts_nothing_when_authorship_is_unknown() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nStable.\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Verifying;
    record.canonical_applied = true;
    save_change(root, &record).unwrap();
    write_scaffolded_auth_spec(root);
    fs::remove_file(delta_path(root, &record, "auth")).unwrap();

    let errors = validate_effective_contracts(root, &[record]).errors;

    assert!(
        errors.iter().any(|error| error
            == "effective contract `auth`: Section ## Purpose contains only unfinished draft text"),
        "{errors:?}"
    );
}

// Verifies REQ-change-059. `.specsyncignore` reaches this gate, and the
// suppression it grants is reported rather than silent.
#[test]
fn effective_contract_reports_ignore_rule_suppressions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Dependencies\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Approved;
    fs::write(root.join(".specsyncignore"), "stub-section:specs/auth/\n").unwrap();

    let outcome = validate_effective_contracts(root, &[record]);

    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert!(
        outcome.suppressions.iter().any(|note| note.contains(
            "suppressed `stub-section` warning by path ignore rule: Section ## Dependencies"
        )),
        "{:?}",
        outcome.suppressions
    );
}

// Verifies REQ-change-059. A module that fails still reports what it let
// through, so a suppression is never lost behind an unrelated error.
#[test]
fn effective_contract_reports_suppressions_alongside_errors() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // Authors `## Dependencies` (emptied — fatal) and leaves the scaffolded
    // `## Purpose` alone (suppressed).
    let delta = "## MODIFIED\n\n### SPEC SECTION Dependencies\n";
    let mut record = completed_section_only_record(root, delta);
    record.state = ChangeState::Approved;
    write_scaffolded_auth_spec(root);

    let outcome = validate_effective_contracts(root, &[record]);

    assert!(
            outcome.errors.iter().any(|error| error
                == "effective contract `auth`: Section ## Dependencies contains only unfinished draft text"),
            "{:?}",
            outcome.errors
        );
    assert!(
        outcome
            .suppressions
            .iter()
            .any(|note| note.contains("no active change authored ## Purpose")),
        "{:?}",
        outcome.suppressions
    );
}

// Verifies REQ-change-059. Suppressions reach the surfaces that report them.
#[test]
fn project_check_reports_effective_contract_suppressions_as_warnings() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nPasskey material never leaves the enclave.\n";
    let record = completed_section_only_record(root, delta);
    approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    write_scaffolded_auth_spec(root);

    let report = check_project(root);

    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("effective contract `auth`")
                && warning.contains("Section ## Purpose contains only unfinished draft text")),
        "{:?}",
        report.warnings
    );
    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.contains("unfinished draft text")),
        "{:?}",
        report.errors
    );
}

#[test]
fn unsafe_verification_commands_are_refused() {
    assert!(shell_words("cargo test; rm -rf .").is_err());
    assert!(shell_words("cargo test | tee out").is_err());
    assert!(shell_words("cargo test '").is_err());
    assert_eq!(
        shell_words("fledge run test").unwrap(),
        vec!["fledge", "run", "test"]
    );
    assert_eq!(
        shell_words("cargo run --manifest-path 'tools/spec sync/Cargo.toml' -- check # safe")
            .unwrap(),
        vec![
            "cargo",
            "run",
            "--manifest-path",
            "tools/spec sync/Cargo.toml",
            "--",
            "check"
        ]
    );
}

#[test]
fn verification_detection_prefers_portable_project_commands() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();
    fs::write(temp.path().join("fledge.toml"), "[tasks.test]\n").unwrap();
    assert_eq!(detect_verification_commands(temp.path()), ["cargo test"]);
}

#[test]
fn portable_paths_normalize_windows_separators() {
    assert_eq!(
        portable_project_path(Path::new(""), Path::new(r"openspec\changes\add-passkeys")),
        "openspec/changes/add-passkeys"
    );
}

#[test]
fn definition_digest_is_portable_across_checkout_roots() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    let record = completed_record(first.path());
    fs::create_dir_all(change_dir(second.path(), &record.id).join("deltas")).unwrap();
    save_change(second.path(), &record).unwrap();
    for artifact in &record.selected_artifacts {
        let content = format!("# {}\n\nComplete.\n", artifact.file_name());
        fs::write(
            change_dir(first.path(), &record.id).join(artifact.file_name()),
            &content,
        )
        .unwrap();
        fs::write(
            change_dir(second.path(), &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    let delta = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n";
    fs::write(delta_path(first.path(), &record, "auth"), delta).unwrap();
    fs::write(delta_path(second.path(), &record, "auth"), delta).unwrap();
    assert_eq!(
        definition_digest(first.path(), &record).unwrap(),
        definition_digest(second.path(), &record).unwrap()
    );
}

#[test]
fn task_completion_progress_does_not_invalidate_scope_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let tasks = change_dir(root, &record.id).join("tasks.md");
    let approved_digest = definition_digest(root, &record).unwrap();

    fs::write(&tasks, "# Tasks\n\n- [ ] Complete\n").unwrap();
    assert_eq!(definition_digest(root, &record).unwrap(), approved_digest);

    fs::write(
        &tasks,
        "# Tasks\n\n- [x] Complete\n- [x] Newly added scope\n",
    )
    .unwrap();
    assert_ne!(definition_digest(root, &record).unwrap(), approved_digest);
}

#[test]
fn task_progress_digest_accepts_both_historical_checkbox_encodings() {
    let before_completion = TempDir::new().unwrap();
    let root = before_completion.path();
    let record = completed_no_spec_record(root);
    let tasks = change_dir(root, &record.id).join("tasks.md");
    fs::write(&tasks, "# Tasks\n\n- [ ] Complete\n").unwrap();
    let unchecked_digest =
        legacy_task_definition_digest_for_correction_count(root, &record, 0, false).unwrap();
    append_approval(
        root,
        &record,
        "definition",
        Some("Scope owner".into()),
        unchecked_digest,
        None,
    )
    .unwrap();
    fs::write(&tasks, "# Tasks\n\n- [x] Complete\n").unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_ok());

    let after_completion = TempDir::new().unwrap();
    let root = after_completion.path();
    let record = completed_no_spec_record(root);
    let checked_digest =
        legacy_task_definition_digest_for_correction_count(root, &record, 0, false).unwrap();
    assert_ne!(checked_digest, definition_digest(root, &record).unwrap());
    append_approval(
        root,
        &record,
        "definition",
        Some("Scope owner".into()),
        checked_digest,
        None,
    )
    .unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn task_progress_and_completion_share_a_markdown_aware_parser() {
    let payload =
        b"# Tasks\n\n+ [ ] Plus task\n* [x] Star task\n\n```md\n- [ ] Example only\n```\n";
    let offsets = markdown_task_checkbox_offsets(payload);
    assert_eq!(offsets.len(), 2);
    assert_eq!(payload[offsets[0]], b' ');
    assert_eq!(payload[offsets[1]], b'x');

    let canonical = canonical_definition_artifact_payload(
        ".specsync/changes/CHG-0001-example/tasks.md",
        payload,
    );
    assert_eq!(canonical[offsets[0]], b' ');
    assert_eq!(canonical[offsets[1]], b' ');
    assert!(
        String::from_utf8(canonical)
            .unwrap()
            .contains("- [ ] Example only")
    );
}

#[test]
fn verification_routing_fails_closed_without_any_validator() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let policy = SddPolicy {
        verification_commands: Vec::new(),
        ..SddPolicy::default()
    };
    fs::remove_file(root.join(POLICY_PATH)).unwrap();

    let error = verification_commands_for_change(root, &policy, &record, false).unwrap_err();
    assert!(error.contains("no verification commands"));
}

/// Write a routing config with one routed module, and return a policy whose
/// project-wide list is distinguishable from it.
fn routed_policy_fixture(root: &Path) -> SddPolicy {
    let policy = SddPolicy {
        verification_commands: vec!["project-wide".into()],
        ..SddPolicy::default()
    };
    // Serialize the real policy and inject the routing key, rather than writing
    // JSON by hand: `component_verification_commands` is a top-level sibling of
    // the policy fields, read by a second deserialization, and a hand-written
    // object omits everything else `SddPolicy` requires.
    let mut document = serde_json::to_value(&policy).unwrap();
    document.as_object_mut().unwrap().insert(
        "component_verification_commands".into(),
        serde_json::json!({ "routed": ["component-routed"] }),
    );
    if let Some(parent) = root.join(POLICY_PATH).parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        root.join(POLICY_PATH),
        serde_json::to_string_pretty(&document).unwrap(),
    )
    .unwrap();
    policy
}

/// Declaring an additional module must never remove verification (#617).
///
/// The old rule was `if commands.is_empty()` over the whole change: one routed
/// module made the list non-empty, which suppressed the project-wide commands
/// for every module in that scope, routed or not. So naming a second, unrouted
/// module *reduced* what ran — the SDD lifecycle punishing an author for
/// declaring scope accurately.
#[test]
fn declaring_an_unrouted_module_never_reduces_verification() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let policy = routed_policy_fixture(root);
    let mut record = completed_no_spec_record(root);
    record.affected_paths.clear();

    // Baseline: an unrouted module alone gets the project-wide list.
    record.affected_specs = vec!["unrouted".into()];
    let unrouted_only = verification_commands_for_change(root, &policy, &record, false).unwrap();
    assert!(unrouted_only.contains(&"project-wide".to_string()));

    // Baseline: a routed module alone gets its component command. This is the
    // targeted-verification feature and stays intact.
    record.affected_specs = vec!["routed".into()];
    let routed_only = verification_commands_for_change(root, &policy, &record, false).unwrap();
    assert!(routed_only.contains(&"component-routed".to_string()));

    // The property: adding a module is monotonic. The union of the two scopes
    // must be a superset of each, never a subset of either.
    record.affected_specs = vec!["routed".into(), "unrouted".into()];
    let both = verification_commands_for_change(root, &policy, &record, false).unwrap();
    for command in unrouted_only.iter().chain(routed_only.iter()) {
        assert!(
            both.contains(command),
            "declaring both modules dropped `{command}`; got {both:?}"
        );
    }
}

/// The targeted-verification feature must survive the fix (#617).
///
/// Without this, "always run the project-wide list" passes the monotonicity
/// test above while deleting the optimisation the routing exists for.
#[test]
fn a_fully_routed_change_still_runs_only_its_component_commands() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let policy = routed_policy_fixture(root);
    let mut record = completed_no_spec_record(root);
    record.affected_paths.clear();
    record.affected_specs = vec!["routed".into()];

    let commands = verification_commands_for_change(root, &policy, &record, false).unwrap();
    assert_eq!(commands, vec!["component-routed".to_string()]);
    assert!(
        !commands.contains(&"project-wide".to_string()),
        "a fully routed change must not fall back to the project list: {commands:?}"
    );
}

#[test]
fn broad_scope_overlap_triggers_strict_validation() {
    let temp = TempDir::new().unwrap();
    let mut record = completed_no_spec_record(temp.path());
    record.affected_specs.clear();
    record.affected_paths = vec!["src/".into()];

    assert!(change_requires_strict_validation(
        &record,
        &VerificationRouting::default()
    ));
}

#[test]
fn optional_companions_are_selected_by_policy() {
    let feature = adaptive_artifacts(
        ChangeKind::Feature,
        &["auth".into()],
        &["src/auth.rs".into()],
    );
    assert!(feature.contains(&ArtifactKind::Requirements));
    assert!(feature.contains(&ArtifactKind::Testing));
    let docs = adaptive_artifacts(ChangeKind::Documentation, &[], &["README.md".into()]);
    assert!(docs.contains(&ArtifactKind::Docs));
    assert!(!docs.contains(&ArtifactKind::Requirements));
}

#[test]
fn full_lifecycle_applies_contract_and_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    fs::write(
        root.join("specs/auth/requirements.md"),
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/auth.rs"),
        "// Verifies REQ-auth-001\n#[test]\nfn passkey_authentication() {}\n",
    )
    .unwrap();
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
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    let mut record = completed_record(root);
    for artifact in &record.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Implement passkeys\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    fs::write(
            delta_path(root, &record, "auth"),
            "# Auth delta\n\n## ADDED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL support passkey authentication.\n\nAcceptance Criteria\n- A registered passkey authenticates the user.\n\n## MODIFIED\n\n### SPEC SECTION Invariants\n\n1. Passkey authentication is supported and traced to `REQ-auth-001`.\n",
        )
        .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    let mut verification = verify_change(root, &record.id).unwrap();
    assert!(verification.passed);
    fs::write(
        change_dir(root, &record.id).join("context.md"),
        "# Complete\n\nUpdated during verification.\n",
    )
    .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Implementing);
    verification = verify_change(root, &record.id).unwrap();
    assert!(verification.passed);
    let tombstone = root.join(".specsync/archive/changes/old/deltas");
    fs::create_dir_all(&tombstone).unwrap();
    fs::write(
        tombstone.join("auth.md"),
        "## REMOVED\n### REQUIREMENT REQ-auth-001\nRetired.\n",
    )
    .unwrap();
    let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(error.contains("permanent tombstone"), "{error}");
    fs::remove_dir_all(root.join(".specsync/archive/changes/old")).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(spec.contains("version: 1.0.1"));
    assert!(spec.contains("REQ-auth-001"));
    assert!(spec.contains(&record.id));
    // The row is dated when it is written. Comparing it to a second `today()`
    // fails when the write and the assertion fall on opposite sides of midnight.
    let spec_lf = spec.replace("\r\n", "\n");
    let marker = "| 2026-01-01 | Initial |\n| ";
    let row = spec_lf.split_once(marker).unwrap_or_else(|| {
        panic!(
            "changelog row was not appended after Initial:\n{}",
            spec_lf.rsplit("## Change Log").next().unwrap_or(&spec_lf)
        )
    });
    let date: String = row.1.chars().take(10).collect();
    assert!(
        date.len() == 10 && date.as_bytes()[4] == b'-' && date.as_bytes()[7] == b'-',
        "changelog date {date:?} is not YYYY-MM-DD"
    );
    let requirements = fs::read_to_string(root.join("specs/auth/requirements.md")).unwrap();
    assert!(requirements.contains("### REQ-auth-001"));
    git(&["add", "."]);
    git(&["commit", "-m", "record accepted lifecycle evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let archived = archive_change(root, &record.id).unwrap();
    assert!(archived.is_dir());
    assert!(!change_dir(root, &record.id).exists());
}

#[test]
fn section_only_semantic_delta_can_satisfy_acceptance_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and explicitly documented.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();

    assert!(verification.passed);
    assert!(verification.requirement_ids.is_empty());
}

#[test]
fn missing_semantic_acceptance_evidence_is_not_reported_as_command_failure() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_section_only_record(
        root,
        "## REMOVED\n### SPEC SECTION Legacy Notes\n\nRetired.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let error = verify_change(root, &record.id).unwrap_err();

    assert!(
        error.contains("semantic acceptance evidence is missing"),
        "{error}"
    );
    assert!(
        !error.contains("configured verification command failed"),
        "{error}"
    );
}

const PHANTOM_AUTH_SPEC: &str = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n| `does_not_exist` | Phantom. |\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n";

#[test]
fn change_check_does_not_execute_configured_project_commands() {
    // Honest label: DISCRIMINATOR. On the unfixed binary this command runs
    // `verification_commands` (`true`, `cargo test`, …). A sentinel file must
    // not appear.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();
    assert!(
        verification.passed,
        "matching specs and code must pass without running project commands"
    );
    assert_eq!(
        verification.commands[0].command, "specsync check --spec auth",
        "evidence must name the scoped pass a reader can rerun"
    );
    assert!(
        !root.join("was-run").exists(),
        "configured verification_commands must not be spawned"
    );
}

#[test]
fn change_check_fails_when_specs_and_code_drift() {
    // Honest label: CONTROL. A verifier that always passes (or ignores specs)
    // would go green here. Phantom export is an error, not a warning.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["true".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    fs::write(root.join("specs/auth/auth.spec.md"), PHANTOM_AUTH_SPEC).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let error = verify_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("does_not_exist")
            || error.contains("out of sync")
            || error.contains("phantom")
            || error.contains("missing"),
        "spec↔code drift must fail change check, got {error}"
    );
}

#[test]
fn change_check_ignores_drift_in_an_unowned_spec() {
    // Honest label: DISCRIMINATOR for scoping. On the unfixed binary
    // `evaluate_spec_code_sync` walked every spec under `specs_dir`, so a
    // phantom export in a module this change never declared failed its
    // verification. `change check` is scoped: another module's drift is another
    // change's problem, and the whole-project answer is `specsync check`.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["true".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    // `other` is neither in `affected_specs` (["auth"]) nor mapped by
    // `affected_paths` (["src/auth.rs"]).
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: other")
            .replace("src/auth.rs", "src/other.rs")
            .replace("# Auth", "# Other"),
    )
    .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();
    assert!(
        verification.passed,
        "drift in a module this change does not own must not fail its scoped check"
    );
    assert_eq!(
        verification.commands[0].command,
        "specsync check --spec auth"
    );
}

#[test]
fn scope_includes_a_spec_mapping_a_declared_path_with_no_declared_module() {
    // A `--no-spec-change` delivery declares no module. The specs mapping its
    // source are still the contracts it can break, so scope is the union of
    // declared modules AND specs whose `files:` fall inside a declared path.
    // Without the second half this change would verify against nothing.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ensure_test_verification_policy(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::create_dir_all(root.join("vendor")).unwrap();
    fs::create_dir_all(root.join("specs/vendored")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(root.join("vendor/thing.rs"), "// vendored\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: other")
            .replace("src/auth.rs", "src/other.rs")
            .replace("# Auth", "# Other"),
    )
    .unwrap();
    // CONTROL: mapped outside the declared scope, so it must stay out.
    fs::write(
        root.join("specs/vendored/vendored.spec.md"),
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: vendored")
            .replace("src/auth.rs", "vendor/thing.rs")
            .replace("# Auth", "# Vendored"),
    )
    .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Refactor internals without a contract change".into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();

    let config = crate::config::load_config(root);
    let scope = scoped_spec_files(root, &record, &config);

    assert_eq!(scope.filters, vec!["other".to_string()]);
    assert!(scope.unresolved.is_empty(), "{:?}", scope.unresolved);
    assert!(
        scope
            .files
            .iter()
            .any(|file| file.ends_with("specs/other/other.spec.md")),
        "{:?}",
        scope.files
    );
    assert!(
        !scope
            .files
            .iter()
            .any(|file| file.ends_with("specs/vendored/vendored.spec.md")),
        "a spec mapping only paths outside the declared scope is not this change's: {:?}",
        scope.files
    );
}

#[test]
fn the_spec_filter_name_is_the_stem_filter_specs_matches_not_the_declared_module() {
    // `filter_specs` matches `--spec` against the file stem with `.spec`
    // stripped, never against frontmatter `module:`. Recording the declared
    // module for a spec whose filename differs writes a command that selects
    // nothing — the evidence would name a pass no one can reproduce.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ensure_test_verification_policy(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        clean_spec("other", "src/other.rs").replace("module: other", "module: differently_named"),
    )
    .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Touch src with a mismatched module name".into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();

    let scope = scoped_spec_files(root, &record, &crate::config::load_config(root));

    assert_eq!(
        scope.filters,
        vec!["other".to_string()],
        "the filter is the stem `other`, not the declared module `differently_named`"
    );
}

#[test]
fn mixed_scope_evidence_names_the_missing_module_but_does_not_rerun_faithfully() {
    // Honest label: CHARACTERIZATION of a KNOWN RESIDUAL. It asserts what the
    // evidence string can and cannot promise, so nobody restores the comment
    // that claimed a faithful rerun in every case.
    //
    // With one declared module resolved and one missing, `change check` FAILS
    // (the missing module is an error) and the recorded filters name both — the
    // only place the persisted record says which module was missing. But
    // `filter_specs` demotes an unmatched filter to a stderr warning as soon as
    // any other filter matches, and check's exit-1 gate fires only on an empty
    // match set, so rerunning that literal command can exit 0. If this ever
    // fails, check why before changing it.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ensure_test_verification_policy(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        clean_spec("other", "src/other.rs"),
    )
    .unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "Declare one resolvable and one missing module".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["other".into(), "absent".into()],
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Mixed scope is recorded honestly".into()];

    let scope = scoped_spec_files(root, &record, &crate::config::load_config(root));

    assert_eq!(
        scope.unresolved.len(),
        1,
        "the missing module is an error, not a silently smaller scope: {:?}",
        scope.unresolved
    );
    assert!(
        scope.unresolved[0].contains("`absent`"),
        "{:?}",
        scope.unresolved
    );
    assert_eq!(
        scope.filters,
        vec!["absent".to_string(), "other".to_string()],
        "evidence names BOTH, so the record says which module was missing"
    );
    // The residual: this command names a filter that matches nothing alongside
    // one that matches, which `filter_specs` treats as a warning, not an error.
    assert_eq!(
        scoped_check_command(&scope.filters, false),
        "specsync check --spec absent --spec other"
    );
}

#[test]
fn change_check_fails_when_a_declared_module_has_no_spec_on_disk() {
    // Honest label: DISCRIMINATOR. Before this fix `scoped_spec_files` dropped a
    // declared module whose spec was not on disk, so a change could name a
    // contract, never write (or delete) its spec, and verify GREEN against zero
    // of the modules it declared. A clean spec in path scope made the pass look
    // real. `no_spec_change` is what isolates this gate: the effective-contract
    // walk skips those records, so the scoped spec↔code pass is the only thing
    // standing between this record and a vacuous green.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();

    let mut record = completed_no_spec_record(root);
    // Another spec IS in path scope (`affected_paths` is `src/`), so the run has
    // real work to do and cannot pass merely for lack of anything to check.
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        clean_spec("other", "src/other.rs"),
    )
    .unwrap();
    // The declared module's spec is gone.
    fs::remove_file(root.join("specs/change/change.spec.md")).unwrap();

    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let error = verify_change(root, &record.id).unwrap_err();

    assert!(
        error.contains("`change`"),
        "the failure must name the module whose spec is missing: {error}"
    );
    assert!(
        error.contains("verification.json"),
        "a missing declared spec is a recorded spec↔code failure, so the retry is \
         append-only like any other: {error}"
    );
    let recorded = load_verification(root, &record).unwrap();
    assert!(!recorded.passed);
    assert!(
        recorded.commands[0].command.contains("--spec change"),
        "evidence names the unresolved module so the command reproduces the verdict: {}",
        recorded.commands[0].command
    );
    assert!(
        !root.join("was-run").exists(),
        "configured verification_commands must not be spawned"
    );
}

#[test]
fn change_check_passes_a_no_spec_change_delivery_that_maps_no_spec_at_all() {
    // Honest label: CONTROL for the fix above. A change that declared NO module
    // and whose declared paths map no spec has claimed no contract — an empty
    // scope here is honest, not evaporation, and must stay a pass. A fix that
    // failed every empty scope would go red here.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();

    let mut record = no_spec_change_record_over_src(root, "Refactor with no spec anywhere");
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();

    assert!(verification.passed);
    assert_eq!(
        verification.commands[0].command, "specsync check (no spec in scope)",
        "a change that claimed no spec records exactly that"
    );
    assert!(!root.join("was-run").exists());
}

/// A `--no-spec-change` delivery over `src/` that declares NO module.
///
/// `completed_no_spec_record` declares `change`, so its scope survives even if
/// scoping were to read `affected_specs` alone. The record that proves the union
/// clause is load-bearing is the one that names no module at all.
fn no_spec_change_record_over_src(root: &Path, description: &str) -> ChangeRecord {
    ensure_test_verification_policy(root);
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: description.into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Internals are refactored without a contract change".into()];
    record.answers.insert("public_contract".into(), "no".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    persist_legacy_test_record(root, &mut record);
    write_change_markdown(root, &record).unwrap();
    for artifact in &record.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    record
}

/// A spec that documents nothing and therefore cannot drift.
fn clean_spec(module: &str, source: &str) -> String {
    format!(
        "---\nmodule: {module}\nversion: 1.0.0\nstatus: stable\nfiles:\n  - {source}\n---\n\n# {module}\n\n## Purpose\n\nFixture.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
    )
}

#[test]
fn change_check_fails_a_no_spec_change_delivery_whose_mapped_spec_drifted() {
    // Honest label: DISCRIMINATOR for the union clause, end to end through
    // `verify_change` rather than through `scoped_spec_files` alone. A scoping
    // rule that read only `affected_specs` would make this record — which names
    // no module — verify against nothing and pass vacuously. Asserting the
    // helper in isolation cannot catch that; only the verb can.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: other")
            .replace("src/auth.rs", "src/other.rs")
            .replace("# Auth", "# Other"),
    )
    .unwrap();

    let mut record = no_spec_change_record_over_src(root, "Refactor internals over src");
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let error = verify_change(root, &record.id).unwrap_err();

    assert!(
        error.contains("does_not_exist") || error.contains("no matching export"),
        "{error}"
    );
    // The effective-contract gate walks declared MODULES and returns before any
    // attempt is recorded. Naming `verification.json` is what proves the failure
    // came from the scoped spec↔code pass, which recorded an attempt.
    assert!(
        error.contains("verification.json"),
        "the failure must be the recorded spec↔code pass, not an earlier gate: {error}"
    );
    assert_eq!(
        load_verification(root, &record).unwrap().commands[0].command,
        "specsync check --spec other",
        "evidence must name the module reached through the declared path"
    );
    assert!(
        !root.join("was-run").exists(),
        "configured verification_commands must not be spawned"
    );
}

#[test]
fn change_check_passes_a_no_spec_change_delivery_when_only_an_unmapped_spec_drifted() {
    // Honest label: CONTROL for the test above. A verifier that simply fails
    // everything, or that widened scope back to the whole project, would go red
    // here. The in-scope spec is clean and the drifted one maps `vendor/`, which
    // this change never declared — and the recorded command proves the pass was
    // over a real scope rather than an empty one.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("vendor")).unwrap();
    fs::create_dir_all(root.join("specs/other")).unwrap();
    fs::create_dir_all(root.join("specs/vendored")).unwrap();
    fs::write(root.join("src/other.rs"), "// other\n").unwrap();
    fs::write(root.join("vendor/thing.rs"), "// vendored\n").unwrap();
    fs::write(
        root.join("specs/other/other.spec.md"),
        clean_spec("other", "src/other.rs"),
    )
    .unwrap();
    fs::write(
        root.join("specs/vendored/vendored.spec.md"),
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: vendored")
            .replace("src/auth.rs", "vendor/thing.rs")
            .replace("# Auth", "# Vendored"),
    )
    .unwrap();

    let mut record = no_spec_change_record_over_src(root, "Refactor internals over src only");
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();

    assert!(
        verification.passed,
        "drift in a spec mapping no declared path is not this change's failure"
    );
    assert_eq!(
        verification.commands[0].command, "specsync check --spec other",
        "the pass must be over a real scope, not an empty one"
    );
    assert!(
        !root.join("was-run").exists(),
        "configured verification_commands must not be spawned"
    );
}

#[test]
fn an_empty_scope_is_named_rather_than_written_as_a_project_wide_pass() {
    // Recording a bare `specsync check` here would claim a project-wide pass
    // that never ran.
    assert_eq!(
        scoped_check_command(&[], false),
        "specsync check (no spec in scope)"
    );
    assert_eq!(
        scoped_check_command(&["auth".to_string(), "billing".to_string()], false),
        "specsync check --spec auth --spec billing"
    );
    assert_eq!(
        scoped_check_command(&["auth".to_string()], true),
        "specsync check --spec auth --strict"
    );
}

#[test]
fn status_does_not_advertise_strict_unless_strict_was_requested() {
    // Honest label: DISCRIMINATOR for F4. `change` is one of the modules
    // `change_requires_strict_validation` classifies as high-risk, so on the
    // unfixed binary the summary advertised `specsync check --strict` while
    // `verify_change` recorded plain `specsync check`. Status and evidence must
    // name the same command.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let summary = summarize_change(root, &record);
    assert!(
        summary.strict_validation_required,
        "the high-risk classification itself is still reported"
    );
    assert_eq!(
        summary.verification_commands,
        vec!["specsync check --spec change".to_string()],
        "a classification is not a `--strict` invocation"
    );
    assert_eq!(
        summarize_change_with_strict(root, &record, true).verification_commands,
        vec!["specsync check --spec change --strict".to_string()]
    );

    let verification = verify_change(root, &record.id).unwrap();
    assert_eq!(
        vec![verification.commands[0].command.clone()],
        summary.verification_commands,
        "evidence must be the command status advertised"
    );
}

// Verifies REQ-change-030.
#[test]
fn native_cargo_check_argument_is_not_misclassified_as_specsync() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"native-cli\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    assert!(reject_direct_lifecycle_verification(root, "cargo run -- check").is_ok());
    assert!(
        reject_direct_lifecycle_verification(root, "cargo run --bin specsync -- check").is_err()
    );
    assert!(reject_direct_lifecycle_verification(root, "cargo run -p specsync -- check").is_err());
    assert!(
        reject_direct_lifecycle_verification(root, "cargo run --package specsync -- check")
            .is_err()
    );
    assert!(reject_direct_lifecycle_verification(root, "specsync --strict").is_err());

    fs::write(
        root.join("Cargo.toml"),
        "[ package ]\nname = 'specsync' # lifecycle CLI\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    assert!(reject_direct_lifecycle_verification(root, "cargo run -- check").is_err());
    assert_eq!(
        cargo_package_value(
            "[ package ]\nname = 'specsync' # lifecycle CLI\n[dependencies]\nname = \"ignored\"\n",
            "name"
        ),
        Some("specsync".into())
    );
}

// Verifies REQ-change-030.
#[test]
fn cargo_manifest_path_detects_recursive_specsync_before_state_mutation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("tools/specsync")).unwrap();
    fs::write(
        root.join("tools/specsync/Cargo.toml"),
        "[package]\nname = \"specsync\"\nversion = \"5.0.2\"\n",
    )
    .unwrap();

    for command in [
        "cargo run --manifest-path tools/specsync/Cargo.toml -- check",
        "cargo run --manifest-path=tools/specsync/Cargo.toml -- check",
        "cargo --manifest-path tools/specsync/Cargo.toml run -- check",
    ] {
        assert!(reject_direct_lifecycle_verification(root, command).is_err());
    }
    fs::create_dir_all(root.join("tools/spec sync")).unwrap();
    fs::write(
        root.join("tools/spec sync/Cargo.toml"),
        "[package]\nname = \"specsync\" # nested CLI\nversion = \"5.0.2\"\n",
    )
    .unwrap();
    assert!(
        reject_direct_lifecycle_verification(
            root,
            "cargo run --manifest-path 'tools/spec sync/Cargo.toml' -- check # lifecycle"
        )
        .is_err()
    );
    fs::create_dir_all(root.join("tools/default-run")).unwrap();
    fs::write(
        root.join("tools/default-run/Cargo.toml"),
        "[package]\nname = \"wrapper\"\ndefault-run = \"specsync\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    assert!(
        reject_direct_lifecycle_verification(
            root,
            "cargo run --manifest-path=tools/default-run/Cargo.toml -- check"
        )
        .is_err()
    );

    fs::create_dir_all(root.join("tools/native")).unwrap();
    fs::write(
        root.join("tools/native/Cargo.toml"),
        "[package]\nname = \"native-cli\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    assert!(
        reject_direct_lifecycle_verification(
            root,
            "cargo run --manifest-path tools/native/Cargo.toml -- check"
        )
        .is_ok()
    );
    assert!(
        reject_direct_lifecycle_verification(
            root,
            "cargo run --manifest-path ../outside/Cargo.toml -- check"
        )
        .is_err()
    );

    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands =
        vec!["cargo run --manifest-path tools/specsync/Cargo.toml -- check".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let verification = verify_change(root, &record.id).unwrap();
    assert!(
        verification.passed,
        "configured recursive cargo-run is not spawned, so spec↔code sync still runs"
    );
    assert_eq!(
        verification.commands[0].command,
        "specsync check --spec auth"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Verifying
    );
}

// Verifies REQ-change-030.
#[test]
fn generated_sequence_scope_does_not_suppress_delivery_scope_question() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Describe real delivery scope".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: Vec::new(),
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();

    assert!(
        next_questions(&record)
            .iter()
            .any(|question| question.id == "affected_paths")
    );
    let answered = answer_question(root, &record.id, "affected_paths", "src/lib.rs").unwrap();
    assert_eq!(answered.affected_paths, vec!["src/lib.rs".to_string()]);
    // The interview records exactly what the author said. It used to append the generated
    // sequence-ledger claim, which made every change sign an `@exact:delivery` obligation on a
    // file it never touched — and made a change scoped only to the ledger unable to leave the
    // interview, because the delivery-scope question filtered that one path back out.
    assert!(
        !next_questions(&answered)
            .iter()
            .any(|question| question.id == "affected_paths")
    );
}

// Verifies REQ-change-031.
#[test]
fn interview_preserves_prose_and_requires_explicit_multiple_criteria() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Preserve interview intent".into(),
            kind: ChangeKind::BugFix,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/change.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Interview-only fixture".into()),
        },
    )
    .unwrap();
    let prose = "Literal strings, spaced headers, and trailing comments\nremain one criterion.";

    let answered = answer_question(root, &record.id, "acceptance_criteria", prose).unwrap();

    assert_eq!(answered.acceptance_criteria, [prose]);
    let persisted = load_change(root, &record.id).unwrap();
    assert_eq!(persisted.acceptance_criteria, [prose]);
    let markdown = fs::read_to_string(change_dir(root, &record.id).join("change.md")).unwrap();
    assert!(markdown.contains(&format!("- {prose}")));

    let answered = answer_question(
        root,
        &record.id,
        "acceptance_criteria",
        r#"["First, exactly", "Second criterion"]"#,
    )
    .unwrap();
    assert_eq!(
        answered.acceptance_criteria,
        ["First, exactly", "Second criterion"]
    );

    let answered =
        answer_question(root, &record.id, "affected_specs", "change, registry\ncli").unwrap();
    assert_eq!(answered.affected_specs, ["change", "registry", "cli"]);

    let answered = answer_question(
        root,
        &record.id,
        "affected_paths",
        "src/change.rs,\ntests/integration/change.rs",
    )
    .unwrap();
    assert!(answered.affected_paths.contains(&"src/change.rs".into()));
    assert!(
        answered
            .affected_paths
            .contains(&"tests/integration/change.rs".into())
    );
    assert!(!answered.affected_paths.contains(&SEQUENCE_PATH.into()));
}

// Verifies REQ-change-030.
#[test]
fn disabled_policy_skips_sequence_validation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.enabled = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::write(root.join(SEQUENCE_PATH), "not valid json\n").unwrap();

    let report = check_project(root);

    assert!(!report.enabled);
    assert!(report.errors.is_empty());
}

#[test]
fn failed_spec_sync_is_retryable_with_append_only_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["python3 -c \"open('was-run','w').write('ran')\"".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    // The drift has to be inside this change's scope, or scoped verification
    // ignores it. It also has to be outside `affected_specs`, because the
    // effective-contract gate walks the declared MODULES and returns before any
    // verification attempt is recorded — and the append-only attempt ledger is
    // what this test is about. `auth_cli` maps `src/auth.rs`, the change's
    // declared path, so the scoped spec↔code pass is the gate that fails.
    let drifted_spec = root.join("specs/auth_cli/auth_cli.spec.md");
    fs::create_dir_all(drifted_spec.parent().unwrap()).unwrap();
    fs::write(
        &drifted_spec,
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: auth_cli")
            .replace("# Auth", "# Auth CLI"),
    )
    .unwrap();

    let first_error = verify_change(root, &record.id).unwrap_err();
    assert!(
        first_error.contains("does_not_exist") || first_error.contains("no matching export"),
        "{first_error}"
    );
    assert!(first_error.contains("verification.json"), "{first_error}");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Verifying
    );
    assert!(
        !root.join("was-run").exists(),
        "configured verification_commands must not be spawned on a failed spec↔code pass"
    );

    fs::remove_file(&drifted_spec).unwrap();
    let successful = verify_change(root, &record.id).unwrap();
    assert!(successful.passed);
    let history: VerificationAttemptLedger = serde_json::from_slice(
        &fs::read(change_dir(root, &record.id).join("verification-attempts.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(history.schema_version, 1);
    assert_eq!(history.attempts.len(), 2);
    assert!(!history.attempts[0].passed);
    assert!(history.attempts[1].passed);
    assert!(load_verification(root, &record).unwrap().passed);
}

#[test]
fn added_block_reapplies_when_content_is_identical() {
    let source = "# Requirements\n\n### REQ-auth-001\n\nUsers sign in.\n";
    // Re-deriving an already-applied ADDED block converges instead of failing,
    // so a partially-applied run can still be reconciled.
    let unchanged = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "Users sign in.",
        DeltaOperation::Added,
    )
    .unwrap();
    assert_eq!(unchanged, source);

    // A block present with different text is a real conflict and must be declared.
    let error = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "Users sign in with passkeys.",
        DeltaOperation::Added,
    )
    .unwrap_err();
    assert!(error.contains("different content"), "{error}");
    assert!(error.contains("## MODIFIED"), "{error}");
}

#[test]
fn change_ordinals_identify_independently_allocated_workspaces() {
    // Two workspaces that allocated CHG-0078 for different work share an ordinal,
    // which is what makes them a collision once both land on the same base.
    assert_eq!(
        change_sequence_number("CHG-0078-delete-the-ci-reimplementation"),
        Some(78)
    );
    assert_eq!(
        change_sequence_number("CHG-0078-todo-artifact-markers"),
        Some(78)
    );
    assert_ne!(
        change_sequence_number("CHG-0079-something-else"),
        change_sequence_number("CHG-0078-todo-artifact-markers")
    );
    // Non-ordinal identifiers never raise a collision.
    assert!(change_sequence_number("CHG-not-a-number").is_none());
    assert!(change_sequence_number("legacy-change").is_none());
}

#[test]
fn overlapping_active_deltas_are_blocked() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut first = current_workflow_record(root, completed_record(root));
    let mut second = create_change(
        root,
        CreateChangeRequest {
            description: "add recovery passkeys".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    second.acceptance_criteria = vec!["Recovery works".into()];
    second
        .answers
        .insert("public_contract".into(), "yes".into());
    second
        .answers
        .insert("architecture_risk".into(), "no".into());
    for record in [&mut first, &mut second] {
        record.state = ChangeState::Approved;
        save_change(root, record).unwrap();
        fs::write(
                delta_path(root, record, "auth"),
                "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
            )
            .unwrap();
    }
    let error = ensure_no_delta_conflicts(root, &first).unwrap_err();
    assert!(error.contains(&second.id));
    first.dependencies.push(second.id.clone());
    assert!(ensure_no_delta_conflicts(root, &first).is_ok());
}

#[test]
fn unified_gate_validates_code_against_effective_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    fs::write(root.join("tests/auth.rs"), "// REQ-auth-001\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\n| Function | Description |\n|----------|-------------|\n| `login` | Login |\n\n## Invariants\n\n1. Stable.\n\n## Behavioral Examples\n\n### Scenario: Login\n\n- **Given** a user\n- **When** login runs\n- **Then** it succeeds\n\n## Error Cases\n\n| Condition | Behavior |\n|-----------|----------|\n| Invalid | Error |\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = completed_record(root);
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Done\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
            },
        )
        .unwrap();
    }
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL support secure login.\n\nAcceptance Criteria\n- Login is tested.\n\n## MODIFIED\n### SPEC SECTION Public API\n| Function | Description |\n|----------|-------------|\n| `login` | Login |\n| `phantom` | Missing implementation |\n",
        )
        .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Approved);
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("phantom") && error.contains("effective contract")),
        "expected effective-contract phantom error, got {:?}",
        report.errors
    );
    record = start_implementation(root, &record.id).unwrap();
    let error = verify_change(root, &record.id).unwrap_err();
    assert!(error.contains("phantom") && error.contains("effective contract"));
}

#[test]
fn openspec_adoption_imports_canonical_and_active_but_not_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
    fs::create_dir_all(root.join("openspec/changes/add-passkeys")).unwrap();
    fs::create_dir_all(root.join("openspec/changes/archive/old-change")).unwrap();
    fs::write(root.join("openspec/specs/auth/spec.md"), "# Auth\n").unwrap();
    fs::write(
        root.join("openspec/changes/add-passkeys/proposal.md"),
        "# Add passkeys\n",
    )
    .unwrap();
    fs::write(
        root.join("openspec/changes/archive/old-change/proposal.md"),
        "# Old\n",
    )
    .unwrap();
    let actions = adopt(root, false, Some("openspec")).unwrap();
    assert!(actions.iter().any(|action| action.contains("openspec")));
    assert!(
        root.join(".specsync/imports/openspec/canonical/auth/spec.md")
            .is_file()
    );
    let roster = list_changes(root).unwrap();
    assert!(!roster.is_degraded(), "unreadable: {:?}", roster.unreadable);
    let records = roster.records;
    assert_eq!(records.len(), 1);
    assert!(
        change_dir(root, &records[0].id)
            .join("imported/proposal.md")
            .is_file()
    );
}

#[test]
fn speckit_adoption_imports_constitution_and_feature_workspaces_only() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specify/memory")).unwrap();
    fs::create_dir_all(root.join("specs/001-passkeys")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join(".specify/memory/constitution.md"),
        "# Constitution\n",
    )
    .unwrap();
    fs::write(root.join("specs/001-passkeys/spec.md"), "# Passkeys\n").unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), "# Native spec\n").unwrap();
    fs::write(root.join("specs/auth/tasks.md"), "# Native tasks\n").unwrap();
    adopt(root, false, Some("speckit")).unwrap();
    assert!(
        root.join(".specsync/imports/speckit/constitution.md")
            .is_file()
    );
    let roster = list_changes(root).unwrap();
    assert!(!roster.is_degraded(), "unreadable: {:?}", roster.unreadable);
    let records = roster.records;
    assert_eq!(records.len(), 1);
    assert!(
        change_dir(root, &records[0].id)
            .join("imported/spec.md")
            .is_file()
    );
}

#[test]
fn custom_artifact_templates_are_scoped_and_rendered() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::write(
        root.join("templates/risk.md"),
        "# Risk for {{title}}\n\nChange: {{change_id}}\n",
    )
    .unwrap();
    let mut policy = SddPolicy::default();
    policy
        .custom_artifacts
        .insert("risk".into(), "templates/risk.md".into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Assess authentication risk".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: vec![ArtifactKind::parse("../../risk")],
            no_spec_change: true,
            rationale: Some("Risk assessment only".into()),
        },
    )
    .unwrap();
    let rendered = fs::read_to_string(change_dir(root, &record.id).join("risk.md")).unwrap();
    assert!(rendered.contains(&record.id));
    assert!(rendered.contains(&record.title));
    assert!(safe_project_path(root, "../../secret").is_err());
}

#[test]
fn project_principles_are_part_of_the_approval_digest() {
    let temp = TempDir::new().unwrap();
    ensure_auth_spec_owns_its_source(temp.path());
    let root = temp.path();
    fs::write(
        root.join("PRINCIPLES.md"),
        "# Principles\n\nPrefer safety.\n",
    )
    .unwrap();
    let mut policy = SddPolicy::default();
    policy.principles_file = Some("PRINCIPLES.md".into());
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_record(root);
    record.answers.insert("constraints".into(), "none".into());
    save_change(root, &record).unwrap();
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
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
    fs::write(
        root.join("PRINCIPLES.md"),
        "# Principles\n\nPrefer speed.\n",
    )
    .unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_err());
}

#[test]
fn definition_approval_rejects_an_invalid_semantic_delta() {
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
        "## ADDED\n### REQUIREMENT REQ-auth-001\nMissing normative language.\n",
    )
    .unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(error.contains("SHALL") || error.contains("Acceptance Criteria"));
    assert!(load_approvals(root, &record).unwrap().approvals.is_empty());
}

#[test]
fn prepared_write_failure_rolls_back_prior_files() {
    let temp = TempDir::new().unwrap();
    let first = temp.path().join("first.md");
    let second = temp.path().join("second.md");
    fs::write(&first, "original\n").unwrap();
    fs::create_dir(&second).unwrap();
    let result = write_prepared_files(
        temp.path(),
        &[
            (first.clone(), "changed\n".into()),
            (second, "cannot write a directory\n".into()),
        ],
    );
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(first).unwrap(), "original\n");
}

#[test]
fn pending_transaction_is_recovered_before_next_lifecycle_write() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let canonical = root.join("specs/auth/auth.spec.md");
    fs::write(&canonical, "corrupted partial write\n").unwrap();
    write_json(
        &root.join(TRANSACTION_PATH),
        &[TransactionEntry {
            path: "specs/auth/auth.spec.md".into(),
            original: Some("original canonical content\n".into()),
        }],
    )
    .unwrap();
    let lock = acquire_project_lock(root).unwrap();
    drop(lock);
    assert_eq!(
        fs::read_to_string(canonical).unwrap(),
        "original canonical content\n"
    );
    assert!(!root.join(TRANSACTION_PATH).exists());
}

#[test]
fn torn_transaction_staging_file_is_never_treated_as_the_journal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let canonical = root.join(".specsync/state.json");
    fs::write(&canonical, "original\n").unwrap();
    fs::write(
        root.join(".specsync/.specsync-transaction-torn"),
        b"{\"incomplete\":",
    )
    .unwrap();

    let lock = acquire_project_lock(root).unwrap();
    drop(lock);

    assert_eq!(fs::read_to_string(canonical).unwrap(), "original\n");
    assert!(!root.join(TRANSACTION_PATH).exists());
}

#[test]
fn torn_canonical_transaction_journal_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let canonical = root.join(".specsync/state.json");
    fs::write(&canonical, "partial payload\n").unwrap();
    let journal = TransactionJournal {
        schema_version: 1,
        entry_count: 2,
        entries_digest: "0".repeat(64),
        entries: vec![TransactionEntry {
            path: ".specsync/state.json".into(),
            original: Some("original\n".into()),
        }],
    };
    write_json(&root.join(TRANSACTION_PATH), &journal).unwrap();

    let error = acquire_project_lock(root).err().unwrap();
    assert!(error.contains("transaction journal integrity"));
    assert_eq!(fs::read_to_string(&canonical).unwrap(), "partial payload\n");
    assert!(root.join(TRANSACTION_PATH).exists());
}

#[test]
fn unreadable_transaction_backup_aborts_before_journal_publication() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let target = root.join("state.json");
    fs::write(&target, [0xff, 0xfe]).unwrap();

    let error =
        write_prepared_files(root, &[(target.clone(), "replacement\n".into())]).unwrap_err();
    assert!(error.contains("failed to preserve transaction target"));
    assert_eq!(fs::read(&target).unwrap(), vec![0xff, 0xfe]);
    assert!(!root.join(TRANSACTION_PATH).exists());
}

#[test]
fn change_dependencies_reject_cycles() {
    let temp = TempDir::new().unwrap();
    let request = |description: &str| CreateChangeRequest {
        description: description.into(),
        kind: ChangeKind::Operations,
        affected_specs: Vec::new(),
        affected_paths: vec!["ci/".into()],
        requested_artifacts: Vec::new(),
        no_spec_change: true,
        rationale: Some("Operational ordering".into()),
    };
    let first = create_change(temp.path(), request("First operation")).unwrap();
    let second = create_change(temp.path(), request("Second operation")).unwrap();
    add_dependency(temp.path(), &first.id, &second.id).unwrap();
    let error = add_dependency(temp.path(), &second.id, &first.id).unwrap_err();
    assert!(error.contains("cycle"));
}

#[test]
fn removed_requirement_ids_cannot_be_reused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync/archive/changes/old/deltas")).unwrap();
    fs::write(
        root.join(".specsync/archive/changes/old/deltas/auth.md"),
        "## REMOVED\n### REQUIREMENT REQ-auth-007\nRetired requirement.\n",
    )
    .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Reuse retired requirement".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-007\nThe system SHALL reuse an ID.\n\nAcceptance Criteria\n- Reused.\n",
        )
        .unwrap();
    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(error.contains("permanent tombstone"));
}

#[test]
fn draft_requirement_removals_are_not_permanent_tombstones() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let removal = create_change(
        root,
        CreateChangeRequest {
            description: "Consider retiring requirement".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
        delta_path(root, &removal, "auth"),
        "## REMOVED\n### REQUIREMENT REQ-auth-007\nRetired requirement.\n",
    )
    .unwrap();
    let addition = create_change(
        root,
        CreateChangeRequest {
            description: "Add active requirement".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
            delta_path(root, &addition, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-007\nThe system SHALL add an active requirement.\n\nAcceptance Criteria\n- Active.\n",
        )
        .unwrap();

    assert!(validate_delta_files(root, &addition).is_ok());
}

#[test]
fn requirement_ids_must_match_their_delta_module() {
    let temp = TempDir::new().unwrap();
    let record = completed_record(temp.path());
    fs::write(
            delta_path(temp.path(), &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-billing-001\nThe system SHALL authenticate.\n\nAcceptance Criteria\n- Works.\n",
        )
        .unwrap();
    let error = validate_delta_files(temp.path(), &record).unwrap_err();
    assert!(error.contains("must match affected module `auth`"));
}

#[test]
fn draft_next_action_prefers_complete_artifacts_over_approve() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            format!(
                "---\nchange: {}\nartifact: {}\n---\n\n# {}\n\n<!-- TODO: fill -->\n",
                record.id,
                artifact.file_name().trim_end_matches(".md"),
                artifact.file_name()
            ),
        )
        .unwrap();
    }
    let summary = summarize_change(root, &record);
    assert!(!summary.artifacts_complete);
    assert!(
        summary.next_action.contains("complete"),
        "expected complete-artifacts guidance, got {}",
        summary.next_action
    );
    assert!(
        !summary.next_action.contains("change approve"),
        "must not recommend approve while artifacts incomplete: {}",
        summary.next_action
    );

    for artifact in &record.selected_artifacts {
        let body = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReady for approval.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            body,
        )
        .unwrap();
    }
    let summary = summarize_change(root, &record);
    assert!(summary.artifacts_complete);
    assert!(
        summary.next_action.contains("change approve"),
        "expected approve next, got {}",
        summary.next_action
    );
}

#[test]
fn added_requirement_already_in_living_tree_fails_delta_validation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n| `auth` | Auth entry. |\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    fs::write(
            root.join("specs/auth/requirements.md"),
            "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe auth module SHALL provide an `auth` entry point.\n\nAcceptance Criteria\n\n- Callers can invoke `auth`.\n",
        )
        .unwrap();

    let record = current_workflow_record(root, completed_record(root));
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Complete\n"
            } else {
                prose_or_plan(artifact, "# Complete\n\nReviewed.\n")
            },
        )
        .unwrap();
    }
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe auth module SHALL provide an `auth` entry point.\n\nAcceptance Criteria\n- Callers can invoke `auth`.\n",
        )
        .unwrap();
    let error = validate_delta_files(root, &record).unwrap_err();
    assert!(
        error.contains("cannot add existing block `REQ-auth-001`"),
        "{error}"
    );
    assert!(
        error.contains("## MODIFIED"),
        "error should steer agents to MODIFIED: {error}"
    );
    let approve_error =
        approve_definition(root, &record.id, Some("Owner".into()), None).unwrap_err();
    assert!(
        approve_error.contains("cannot add existing block"),
        "{approve_error}"
    );

    fs::write(
            delta_path(root, &record, "auth"),
            "## MODIFIED\n### REQUIREMENT REQ-auth-001\nThe auth module SHALL provide an `auth` entry point with documented behavior.\n\nAcceptance Criteria\n- Callers can invoke `auth`.\n- Behavior is documented.\n",
        )
        .unwrap();
    assert!(validate_delta_files(root, &record).is_ok());
}

#[test]
fn change_identifiers_and_scope_cannot_escape_project_root() {
    let temp = TempDir::new().unwrap();
    assert!(load_change(temp.path(), "../../Cargo.toml").is_err());
    let result = create_change(
        temp.path(),
        CreateChangeRequest {
            description: "Escape scope".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["../outside".into()],
            affected_paths: vec!["../../secret".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    );
    assert!(result.is_err());
    assert!(!temp.path().join("../outside").exists());
}

#[test]
fn windows_paths_are_normalized_without_allowing_traversal() {
    assert_eq!(
        normalize_project_path(r"src\auth\mod.rs").unwrap(),
        "src/auth/mod.rs"
    );
    assert!(normalize_project_path(r"..\secret.txt").is_err());
    assert!(normalize_project_path(r"C:\secret.txt").is_err());
}

#[cfg(unix)]
#[test]
fn safe_project_paths_reject_symlink_escapes() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
    let error = safe_project_path(root.path(), "linked/secret.md").unwrap_err();
    assert!(error.contains("through a symlink"));
}

#[test]
fn path_scopes_match_component_boundaries() {
    assert!(path_matches_scope("src", "src"));
    assert!(path_matches_scope("src/auth.rs", "src"));
    assert!(path_matches_scope("src/auth.rs", "src/"));
    assert!(!path_matches_scope("src-old/auth.rs", "src"));
    assert!(!path_matches_scope("src2.rs", "src"));
    assert!(!path_matches_scope("Src/auth.rs", "src"));
}

#[test]
fn default_policy_covers_root_action_and_dependency_lockfiles() {
    let policy = SddPolicy::default();
    for path in [
        "action.yml",
        "Cargo.lock",
        "bun.lock",
        "package-lock.json",
        "Package.resolved",
        "go.sum",
        "uv.lock",
    ] {
        assert!(
            path_is_meaningful(path, &policy),
            "{path} should be meaningful"
        );
    }
    let mut hostile = policy;
    hostile.ignored_paths.push(".specsync/".into());
    assert!(path_is_meaningful(".specsync/sdd.json", &hostile));
    assert!(path_is_meaningful(".specsync/config.toml", &hostile));
    assert!(path_is_meaningful(".specsync/registry.toml", &hostile));
    assert!(path_is_meaningful("specsync-registry.toml", &hostile));
    assert!(path_is_meaningful(SEQUENCE_PATH, &hostile));
    assert!(!path_is_meaningful(
        ".specsync/adoption-report.json",
        &hostile
    ));
    assert!(path_matches_scope("root.rs", "."));
}

#[test]
fn workspace_digest_tracks_unicode_and_space_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src/space dir")).unwrap();
    let path = root.join("src/space dir/naïve.rs");
    fs::write(&path, "first\n").unwrap();
    let first = project_input_digest(root).unwrap();
    fs::write(path, "second\n").unwrap();
    let second = project_input_digest(root).unwrap();
    assert_ne!(first, second);
}

#[test]
fn later_sequence_claim_reuses_the_committed_collision_owner_ledger() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);

    let mut first = reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0001-first-collision-member",
    );
    first.state = ChangeState::Accepted;
    save_change(root, &first).unwrap();
    let mut collision_owner = first.clone();
    collision_owner.id = "CHG-0001-collision-owner".into();
    collision_owner.slug = "collision-owner".into();
    collision_owner.title = "Collision owner".into();
    save_change(root, &collision_owner).unwrap();
    let mut collision_ids = vec![first.id.clone(), collision_owner.id.clone()];
    collision_ids.sort();
    let historical = ChangeSequenceLedger {
        schema_version: 1,
        sequence: 1,
        id: collision_owner.id.clone(),
        acknowledged_collisions: vec![ChangeSequenceCollision {
            sequence: 1,
            ids: collision_ids.clone(),
        }],
    };
    write_json(&root.join(SEQUENCE_PATH), &historical).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "record collision owner ledger"]);
    let historical_bytes = fs::read(root.join(SEQUENCE_PATH)).unwrap();

    let mut later = first.clone();
    later.id = "CHG-0002-later-owner".into();
    later.slug = "later-owner".into();
    later.title = "Later owner".into();
    later.state = ChangeState::Draft;
    save_change(root, &later).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: later.id,
            acknowledged_collisions: vec![ChangeSequenceCollision {
                sequence: 1,
                ids: collision_ids,
            }],
        },
    )
    .unwrap();

    assert_eq!(
        historical_sequence_ledger_acceptance_content(root, &first)
            .unwrap()
            .unwrap(),
        historical_bytes
    );
}

// Verifies REQ-change-029.
#[test]
fn valid_later_sequence_claim_preserves_historical_acceptance_input() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0001-earlier-owner",
    );
    let mut record = current_workflow_record(root, record);
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".specsync".into()];
    save_change(root, &record).unwrap();
    // The ledger this test is about is no longer written by `change new`, so the fixture
    // writes the earlier owner's claim itself, exactly as the archived corpus carries it.
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: record.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    let first_workspace = project_input_digest(root).unwrap();
    let first_acceptance = acceptance_input_digest(root, &record, &[]).unwrap();

    let successor = create_change(
        root,
        CreateChangeRequest {
            description: "Later sequence owner".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/later".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();
    let successor = reidentify_as_ordinal(root, &successor, "CHG-0002-later-sequence-owner");
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: successor.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    let second_workspace = project_input_digest(root).unwrap();
    let second_acceptance = acceptance_input_digest(root, &record, &[]).unwrap();

    assert!(change_sequence(&successor.id) > change_sequence(&record.id));
    assert_ne!(first_workspace, second_workspace);
    assert_eq!(first_acceptance, second_acceptance);
    assert!(!project_input_is_volatile(SEQUENCE_PATH));
}

// Verifies REQ-change-029.
#[test]
fn later_collision_acknowledgements_do_not_stale_earlier_sequence_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut predecessor = reidentify_as_ordinal(
        root,
        &completed_no_spec_current_record(root),
        "CHG-0001-earlier-evidence",
    );
    predecessor.state = ChangeState::Implementing;
    predecessor.affected_paths = vec![SEQUENCE_PATH.into()];
    save_change(root, &predecessor).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: predecessor.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    let before = acceptance_input_digest(root, &predecessor, &[]).unwrap();

    let successor = create_change(
        root,
        CreateChangeRequest {
            description: "Later sequence owner".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/later".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();
    let mut successor = reidentify_as_ordinal(root, &successor, "CHG-0002-later-sequence-owner");
    successor.state = ChangeState::Accepted;
    save_change(root, &successor).unwrap();
    let mut duplicate = successor.clone();
    duplicate.id = "CHG-0002-archived-collision".into();
    duplicate.slug = "archived-collision".into();
    duplicate.state = ChangeState::Archived;
    let archived_dir = root
        .join(ARCHIVE_PATH)
        .join("2026-07-14-CHG-0002-archived-collision");
    fs::create_dir_all(&archived_dir).unwrap();
    write_json(&archived_dir.join("state.json"), &duplicate).unwrap();
    let mut ids = vec![successor.id.clone(), duplicate.id.clone()];
    ids.sort();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: successor.id,
            acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 2, ids }],
        },
    )
    .unwrap();

    assert!(validate_change_sequences(root).is_ok());
    assert_eq!(
        before,
        acceptance_input_digest(root, &predecessor, &[]).unwrap()
    );
}

// Verifies REQ-change-029.
#[test]
fn accepted_later_sequence_owner_covers_post_acceptance_collision_acknowledgement() {
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
    write_lifecycle_test_policy(root);
    let mut legacy_policy = load_policy(root).unwrap();
    legacy_policy.version = 1;
    write_json(&root.join(POLICY_PATH), &legacy_policy).unwrap();

    // `change new` used to write this file and add it to the change's own scope. Nothing
    // writes it now, so the fixture plays the allocator: it scopes the ledger into each
    // record and advances the claim at the point the allocator used to, which is what makes
    // an earlier accepted record's exact ledger evidence go stale and need covering.
    let claim = |sequence: u64, id: &str, acknowledged: Vec<ChangeSequenceCollision>| {
        write_json(
            &root.join(SEQUENCE_PATH),
            &ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: id.to_string(),
                acknowledged_collisions: acknowledged,
            },
        )
        .unwrap();
    };
    let scope_ledger = |record: &ChangeRecord| {
        let mut scoped = record.clone();
        scoped.affected_paths.push(SEQUENCE_PATH.into());
        scoped.affected_paths.sort();
        scoped.affected_paths.dedup();
        save_change(root, &scoped).unwrap();
        scoped
    };

    let mut predecessor = scope_ledger(&reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0001-predecessor",
    ));
    claim(1, &predecessor.id, Vec::new());
    git(&["add", "."]);
    git(&["commit", "-m", "base predecessor"]);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let mut intermediate = scope_ledger(&reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0002-intermediate",
    ));
    claim(2, &intermediate.id, Vec::new());
    intermediate =
        approve_definition(root, &intermediate.id, Some("Reviewer".into()), None).unwrap();
    intermediate = start_implementation(root, &intermediate.id).unwrap();
    verify_change(root, &intermediate.id).unwrap();
    intermediate = accept_change(root, &intermediate.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept intermediate owner"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let mut duplicate = predecessor.clone();
    duplicate.id = "CHG-0001-historical-collision".into();
    duplicate.slug = "historical-collision".into();
    duplicate.title = "Historical collision".into();
    save_change(root, &duplicate).unwrap();
    let mut ids = vec![predecessor.id.clone(), duplicate.id];
    ids.sort();
    let acknowledged = vec![ChangeSequenceCollision { sequence: 1, ids }];
    claim(2, &intermediate.id, acknowledged.clone());

    let mut owner = scope_ledger(&reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0003-ledger-owner",
    ));
    claim(3, &owner.id, acknowledged);
    owner = approve_definition(root, &owner.id, Some("Reviewer".into()), None).unwrap();
    owner = start_implementation(root, &owner.id).unwrap();
    verify_change(root, &owner.id).unwrap();
    owner = accept_change(root, &owner.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept collision reconciliation owner"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    for historical in [&predecessor, &intermediate] {
        let evidence = summarize_change(root, historical)
            .terminal_evidence
            .unwrap();
        assert_eq!(
            evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            evidence.reason
        );
    }
    assert_eq!(
        summarize_change(root, &owner)
            .terminal_evidence
            .unwrap()
            .validity,
        TerminalEvidenceValidity::Exact
    );

    let ledger = load_change_sequence_ledger(root).unwrap().unwrap();
    fs::write(
        root.join(SEQUENCE_PATH),
        serde_json::to_string(&ledger).unwrap(),
    )
    .unwrap();
    assert_eq!(
        summarize_change(root, &owner)
            .terminal_evidence
            .unwrap()
            .validity,
        TerminalEvidenceValidity::Stale
    );
}

#[test]
fn current_sequence_owner_binds_exact_ledger_content() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0001-ledger-owner",
    );
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".specsync".into()];
    save_change(root, &record).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: record.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    let canonical = acceptance_input_digest(root, &record, &[]).unwrap();
    let ledger = load_change_sequence_ledger(root).unwrap().unwrap();
    fs::write(
        root.join(SEQUENCE_PATH),
        serde_json::to_string(&ledger).unwrap(),
    )
    .unwrap();

    assert!(validate_change_sequences(root).is_ok());
    assert_ne!(
        canonical,
        acceptance_input_digest(root, &record, &[]).unwrap()
    );
}

#[test]
fn invalid_later_sequence_claim_cannot_replace_historical_ledger_input() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = reidentify_as_ordinal(
        root,
        &completed_no_spec_record(root),
        "CHG-0001-ledger-owner",
    );
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".specsync".into()];
    save_change(root, &record).unwrap();
    // Disk may not run ahead of the ledger high-water mark.
    let mut later = record.clone();
    later.id = "CHG-0002-later-owner".into();
    later.slug = "later-owner".into();
    save_change(root, &later).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: record.id.clone(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();

    let error = acceptance_input_digest(root, &record, &[]).unwrap_err();
    assert!(error.contains("highest recorded sequence"));
}

#[test]
fn abandoned_draft_may_leave_sequence_high_water_without_workspace() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    save_change(root, &record).unwrap();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: "CHG-0002-abandoned-draft".into(),
            acknowledged_collisions: Vec::new(),
        },
    )
    .unwrap();
    assert!(validate_change_sequences(root).is_ok());
}

#[test]
fn framed_workspace_digest_resists_nul_entry_boundary_collisions() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("a"), b"X\0b\0Y").unwrap();
    let one_file = project_input_digest(temp.path()).unwrap();

    fs::write(temp.path().join("a"), b"X").unwrap();
    fs::write(temp.path().join("b"), b"Y").unwrap();
    let two_files = project_input_digest(temp.path()).unwrap();

    assert_ne!(one_file, two_files);
}

#[test]
fn framed_acceptance_digest_resists_nul_entry_boundary_collisions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".".into()];
    fs::write(root.join("a"), b"X\0b\0Y").unwrap();
    let one_file = acceptance_input_digest(root, &record, &[]).unwrap();

    fs::write(root.join("a"), b"X").unwrap();
    fs::write(root.join("b"), b"Y").unwrap();
    let two_files = acceptance_input_digest(root, &record, &[]).unwrap();

    assert_ne!(one_file, two_files);
}

#[test]
fn workspace_digest_preserves_binary_bytes_and_line_endings() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("fixture.bin");
    fs::write(&path, b"first\r\nsecond\0\xff").unwrap();
    let binary = project_input_digest(temp.path()).unwrap();
    fs::write(&path, b"first\r\nsecond\0\xfe").unwrap();
    let changed_binary = project_input_digest(temp.path()).unwrap();
    assert_ne!(binary, changed_binary);

    fs::write(&path, b"first\nsecond\n").unwrap();
    let lf = project_input_digest(temp.path()).unwrap();
    fs::write(&path, b"first\r\nsecond\r\n").unwrap();
    let crlf = project_input_digest(temp.path()).unwrap();
    assert_ne!(lf, crlf, "line endings remain byte-exact digest inputs");
}

#[test]
fn workspace_digest_includes_git_executable_mode() {
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
    git(&["config", "core.autocrlf", "false"]);
    fs::write(root.join("tool.sh"), b"#!/bin/sh\nexit 0\n").unwrap();
    git(&["add", "tool.sh"]);
    let regular = project_input_digest(root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(root.join("tool.sh")).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(root.join("tool.sh"), permissions).unwrap();
    }
    #[cfg(not(unix))]
    git(&["update-index", "--chmod=+x", "tool.sh"]);
    let executable = project_input_digest(root).unwrap();
    assert_ne!(regular, executable);
}

#[cfg(unix)]
#[test]
fn workspace_digest_distinguishes_symlinks_files_and_targets() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("first"), b"same").unwrap();
    fs::write(root.join("second"), b"same").unwrap();
    symlink("first", root.join("entry")).unwrap();
    let first_target = project_input_digest(root).unwrap();
    fs::remove_file(root.join("entry")).unwrap();
    symlink("second", root.join("entry")).unwrap();
    let second_target = project_input_digest(root).unwrap();
    fs::remove_file(root.join("entry")).unwrap();
    fs::write(root.join("entry"), b"second").unwrap();
    let regular_file = project_input_digest(root).unwrap();

    assert_ne!(first_target, second_target);
    assert_ne!(second_target, regular_file);
}

#[cfg(target_os = "linux")]
#[test]
fn workspace_digest_rejects_non_utf8_paths_instead_of_lossy_aliasing() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join(OsString::from_vec(vec![b'f', 0xff]));
    fs::write(path, b"content").unwrap();
    let error = project_input_digest(temp.path()).unwrap_err();
    assert!(error.contains("non-UTF-8"), "{error}");
}

#[test]
fn recorded_base_uses_oldest_change_order_not_hash_order() {
    let mut first = completed_record(TempDir::new().unwrap().path());
    first.base_commit = Some("ffffffff".into());
    let mut second = first.clone();
    second.id = "CHG-0002-second".into();
    second.base_commit = Some("00000000".into());
    let temp = TempDir::new().unwrap();
    assert_eq!(
        recorded_diff_base(temp.path(), &[first, second]),
        "ffffffff"
    );
}

#[test]
fn dependent_changes_are_topologically_ordered() {
    let temp = TempDir::new().unwrap();
    let mut dependent = completed_record(temp.path());
    dependent.id = "CHG-0001-dependent".into();
    dependent.dependencies = vec!["CHG-0002-prerequisite".into()];
    let mut prerequisite = dependent.clone();
    prerequisite.id = "CHG-0002-prerequisite".into();
    prerequisite.dependencies.clear();
    let ordered = dependency_ordered_changes(vec![&dependent, &prerequisite]).unwrap();
    assert_eq!(ordered[0].id, prerequisite.id);
    assert_eq!(ordered[1].id, dependent.id);
}

#[test]
fn dependency_order_is_deterministic_for_twenty_change_chain() {
    let temp = TempDir::new().unwrap();
    let seed = completed_record(temp.path());
    let records = (0..20)
        .map(|index| {
            let mut record = seed.clone();
            record.id = format!("CHG-{:04}-chain", index + 1);
            record.dependencies = (index > 0)
                .then(|| format!("CHG-{index:04}-chain"))
                .into_iter()
                .collect();
            record
        })
        .collect::<Vec<_>>();
    let expected = records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();
    let ascending = (0..records.len()).collect::<Vec<_>>();
    let descending = (0..records.len()).rev().collect::<Vec<_>>();
    let interleaved = (0..records.len())
        .step_by(2)
        .chain((1..records.len()).step_by(2).rev())
        .collect::<Vec<_>>();

    for indices in [ascending, descending, interleaved] {
        let input = indices
            .iter()
            .map(|index| &records[*index])
            .collect::<Vec<_>>();
        let ordered = dependency_ordered_changes(input)
            .unwrap()
            .into_iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ordered, expected);
    }
}

#[test]
fn transitive_dependencies_order_overlapping_deltas() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut prerequisite = completed_current_record(root);
    let mut middle = completed_current_record(root);
    let mut dependent = completed_current_record(root);
    prerequisite.state = ChangeState::Implementing;
    middle.state = ChangeState::Implementing;
    dependent.state = ChangeState::Implementing;
    middle.dependencies = vec![prerequisite.id.clone()];
    dependent.dependencies = vec![middle.id.clone()];
    for (record, requirement) in [
        (&prerequisite, "REQ-auth-900"),
        (&middle, "REQ-auth-901"),
        (&dependent, "REQ-auth-900"),
    ] {
        save_change(root, record).unwrap();
        fs::write(
                delta_path(root, record, "auth"),
                format!("## ADDED\n### REQUIREMENT {requirement}\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n"),
            )
            .unwrap();
    }
    assert!(ensure_no_delta_conflicts(root, &dependent).is_ok());
}

#[test]
fn semantic_application_respects_custom_specs_directory() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.json"),
        r#"{"specsDir":"contracts","sourceDirs":["src"]}"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("contracts/auth")).unwrap();
    fs::write(
            root.join("contracts/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# Auth\n\n## Purpose\nAuth.\n\n## Public API\nNone.\n\n## Invariants\nStable.\n\n## Behavioral Examples\nWorks.\n\n## Error Cases\nNone.\n\n## Dependencies\nNone.\n\n## Change Log\nInitial.\n",
        )
        .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Add auth requirement".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL authenticate.\n\nAcceptance Criteria\n- Works.\n",
        )
        .unwrap();
    let prepared = prepare_pending_delta_application(root, &record)
        .unwrap()
        .files;
    assert!(
        prepared
            .iter()
            .all(|(path, _)| path.starts_with(root.join("contracts")))
    );
}

#[test]
fn semantic_application_resolves_registry_backed_canonical_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/client")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/client.rs"), "// client API\n").unwrap();
    fs::write(
            root.join(".specsync/registry.toml"),
            "[registry]\nname = \"fixture\"\n\n[specs]\nclient-api = \"specs/client/client-api.spec.md\"\n",
        )
        .unwrap();
    fs::write(
            root.join("specs/client/client-api.spec.md"),
            "---\nmodule: client-api\nversion: 1\nstatus: stable\nfiles:\n  - src/client.rs\n---\n\n# Client API\n\n## Purpose\n\nClient API.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Update client API".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["client-api".into()],
            affected_paths: vec!["src/client.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
        root.join("specs/client/requirements.md"),
        "# Requirements\n\nOriginal.\n",
    )
    .unwrap();
    for companion in ["tasks.md", "context.md", "testing.md", "design.md"] {
        fs::write(
            root.join("specs/client").join(companion),
            format!("# {companion}\n\nOriginal.\n"),
        )
        .unwrap();
    }
    fs::write(
        root.join("specs/client/other.spec.md"),
        "# Unrelated canonical spec\n",
    )
    .unwrap();
    let mut delivering = record.clone();
    delivering.state = ChangeState::Implementing;
    assert!(record_covers_project_path(
        root,
        &delivering,
        "specs/client/client-api.spec.md"
    ));
    assert!(record_covers_project_path(
        root,
        &delivering,
        "specs/client/requirements.md"
    ));
    for companion in ["tasks.md", "context.md", "testing.md", "design.md"] {
        assert!(record_covers_project_path(
            root,
            &delivering,
            &format!("specs/client/{companion}")
        ));
    }
    assert!(!record_covers_project_path(
        root,
        &delivering,
        "specs/client/unrelated.md"
    ));
    assert!(!record_covers_project_path(
        root,
        &delivering,
        "specs/client/other.spec.md"
    ));
    let first_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
    fs::write(
        root.join("specs/client/context.md"),
        "# Context\n\nUpdated.\n",
    )
    .unwrap();
    let companion_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
    assert_ne!(first_acceptance, companion_acceptance);
    fs::write(
        root.join("specs/client/unrelated.md"),
        "# Unrelated\n\nUpdated.\n",
    )
    .unwrap();
    let second_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
    assert_eq!(companion_acceptance, second_acceptance);
    fs::write(
        delta_path(root, &record, "client-api"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nRegistry-backed behavior is stable.\n",
    )
    .unwrap();

    let prepared = prepare_pending_delta_application(root, &record)
        .unwrap()
        .files;

    assert!(
        prepared
            .iter()
            .any(|(path, _)| { path == &root.join("specs/client/client-api.spec.md") })
    );
    assert!(
        prepared
            .iter()
            .any(|(path, _)| path == &root.join("specs/client/requirements.md"))
    );
    assert!(
        !prepared
            .iter()
            .any(|(path, _)| path.starts_with(root.join("specs/client-api")))
    );

    let mut effective_record = record;
    effective_record.state = ChangeState::Approved;
    assert!(
        validate_effective_contracts(root, &[effective_record])
            .errors
            .is_empty()
    );
}

// Verifies REQ-change-041.
#[test]
fn canonical_resolution_tolerates_inert_legacy_registry_stub() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "// auth\n").unwrap();
    fs::write(
        root.join(".specsync/registry.toml"),
        "version = 1\n\n[modules]\n",
    )
    .unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    fs::write(
        root.join("specs/auth/requirements.md"),
        "# Requirements\n\nOriginal.\n",
    )
    .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Update auth with inert registry stub".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
        delta_path(root, &record, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nInert stub fallback remains stable.\n",
    )
    .unwrap();

    let prepared = prepare_pending_delta_application(root, &record)
        .unwrap()
        .files;
    assert!(
        prepared
            .iter()
            .any(|(path, _)| path == &root.join("specs/auth/auth.spec.md"))
    );
    let (spec_path, _) = canonical_module_paths(root, "specs", "auth").unwrap();
    assert_eq!(spec_path, root.join("specs/auth/auth.spec.md"));
}

// Verifies REQ-change-041.
#[test]
fn canonical_resolution_fails_closed_on_non_inert_unparsable_registry() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let registry_path = root.join(".specsync/registry.toml");
    fs::write(
        &registry_path,
        "[specs]\nauth = \"specs/auth/auth.spec.md\"\n",
    )
    .unwrap();

    // Pre-fix diagnostic (unchanged for real/non-inert unparsable registries):
    // "failed to parse local registry {path} while resolving `{module}`"
    let error = canonical_module_paths(root, "specs", "auth").unwrap_err();
    assert_eq!(
        error,
        format!(
            "failed to parse local registry {} while resolving `auth`",
            registry_path.display()
        )
    );
}

#[test]
fn semantic_application_rejects_unsafe_registry_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/registry.toml"),
        "[registry]\nname = \"fixture\"\n\n[specs]\nauth = \"../../outside/auth.spec.md\"\n",
    )
    .unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "Update auth".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    fs::write(
        delta_path(root, &record, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable.\n",
    )
    .unwrap();

    let error = prepare_pending_delta_application(root, &record).unwrap_err();

    assert!(error.contains("unsafe registry path"));
    assert!(error.contains("escapes the project root"));

    let mut effective_record = record;
    effective_record.state = ChangeState::Approved;
    let errors = validate_effective_contracts(root, &[effective_record]).errors;
    assert!(
        errors
            .iter()
            .any(|error| error.contains("cannot resolve canonical spec")
                && error.contains("unsafe registry path"))
    );
}

/// The wording an approver actually read, and the wording nobody did.
const APPROVED_DELTA_BODY: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials. Reviewed and approved wording.\n";
const SWAPPED_DELTA_BODY: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nBACKDOOR: this text was never reviewed or approved by anyone.\n";

/// A workflow-v2 change whose `auth` delta has been approved with the wording above.
fn change_with_an_approved_delta(root: &Path) -> ChangeRecord {
    let record = completed_section_only_current_record(root, APPROVED_DELTA_BODY);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap()
}

/// Rewrite the approval ledger the way EVERY approval written before `approved_delta_digests`
/// existed already looks on disk: the key is simply not there.
///
/// `skip_serializing_if = "Option::is_none"` makes this byte-identical to what the old binary
/// wrote, which the assertion in the compatibility test below checks rather than assumes.
fn strip_recorded_delta_digests(root: &Path, record: &ChangeRecord) {
    let mut ledger = load_approvals(root, record).unwrap();
    for approval in &mut ledger.approvals {
        approval.approved_delta_digests = None;
    }
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
}

/// DISCRIMINATOR for #704. Fails on the unfixed binary, which materializes the swapped body
/// into `specs/auth/auth.spec.md` and reports nothing.
///
/// The swap happens exactly where the reproduction puts it: after `approve` recorded a definition
/// approval, before anything applied the delta. Nothing else in the pipeline covers this region —
/// the v2 scope digest hashes intent and boundary, `validate_delta_files` reads filenames, and
/// `project_input_digest` excludes `.specsync/changes/` — so a passing assertion here is evidence
/// of the new binding and of nothing else.
#[test]
fn a_semantic_delta_swapped_after_approval_never_reaches_the_canonical_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    fs::write(delta_path(root, &record, "auth"), SWAPPED_DELTA_BODY).unwrap();
    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("`auth`"),
        "a refusal must name the module whose delta drifted: {error}"
    );
    assert!(
        error.contains("changed after approval"),
        "a refusal must say what went wrong, not just that something did: {error}"
    );
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        !spec.contains("BACKDOOR"),
        "the living spec carries wording nobody approved: {spec}"
    );
    assert!(
        !load_change(root, &record.id).unwrap().canonical_applied,
        "a refused materialization must not record itself as applied"
    );
}

/// Honest label: this is the CONTROL, and it passes on the unfixed binary too — that is the
/// entire point. The check added for #704 must refuse ONLY drift; an honest change must reach
/// the canonical spec exactly as it did before. A discriminator that fires on well-formed work
/// is not a fix, it is an outage, and this is the assertion that would catch that.
///
/// It also pins the positive half of the record: the approval carries a digest keyed by module,
/// so "the check passed" cannot silently mean "there was nothing recorded to check".
#[test]
fn an_approved_delta_that_was_never_touched_still_rewrites_the_canonical_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    let ledger = load_approvals(root, &record).unwrap();
    let recorded = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .and_then(|approval| approval.approved_delta_digests.clone())
        .expect("approve must record the delta bodies it approved");
    assert_eq!(
        recorded.keys().collect::<Vec<_>>(),
        vec!["auth"],
        "the recorded digest must be keyed by module: {recorded:?}"
    );

    let applied = materialize_change_deltas(root, &record.id).unwrap();

    assert!(applied.canonical_applied);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec.contains("Auth tracks credentials. Reviewed and approved wording."),
        "the approved wording must still reach the canonical spec: {spec}"
    );
}

/// COMPATIBILITY. Every one of this repository's archived changes was approved before
/// `approved_delta_digests` existed, so every one of them carries no digest at all.
///
/// Absent evidence is UNKNOWN, never VIOLATED. The delta is swapped here as well, so the test
/// fails the moment someone decides a missing digest should read as tampering: history would
/// then start failing on evidence nobody could have written, which is #672 (unparseable schema
/// read as "tables missing") and #684 (missing config read as a gating warning) all over again.
#[test]
fn an_approval_recorded_before_delta_digests_existed_is_unknown_not_violated() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);
    strip_recorded_delta_digests(root, &record);

    let ledger_bytes =
        fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap();
    assert!(
        !ledger_bytes.contains("approved_delta_digests"),
        "the fixture must be shaped like a pre-#704 ledger, not merely be missing a value: {ledger_bytes}"
    );

    fs::write(delta_path(root, &record, "auth"), SWAPPED_DELTA_BODY).unwrap();
    let applied = materialize_change_deltas(root, &record.id)
        .expect("an approval that made no claim about delta bodies must not fail on that absence");

    assert!(applied.canonical_applied);
}

/// A workflow-v1 change carrying the `auth` delta above, positioned so `--portable-5-0-1` can run.
///
/// The portable projection is workflow-v1-only and refuses without a versioned legacy archive
/// baseline binding, so the fixture writes a real baseline ledger and binds the record to its
/// actual digest rather than to a placeholder. `authority_change_id` names a different change, so
/// `bind_legacy_archive_baseline_authority` sees a present, valid baseline and leaves the binding
/// alone — which is the position an adopter upgrading from 5.x is in.
fn portable_v1_change_with_a_delta(root: &Path) -> ChangeRecord {
    let mut record = completed_section_only_record(root, APPROVED_DELTA_BODY);
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: "CHG-0000-legacy-archive-baseline".into(),
        cutoff_commit: "0".repeat(40),
        entries: Vec::new(),
    };
    let path = root.join(LEGACY_BASELINE_PATH);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    write_json(&path, &baseline).unwrap();
    let (_, digest) = validate_legacy_archive_baseline_bytes(&fs::read(&path).unwrap()).unwrap();
    record.legacy_archive_baseline_digest = Some(digest);
    save_change(root, &record).unwrap();
    record
}

fn recorded_delta_digests(root: &Path, record: &ChangeRecord) -> Option<BTreeMap<String, String>> {
    let ledger = load_approvals(root, record).unwrap();
    effective_definition_approval(root, record, &ledger)
        .unwrap()
        .approved_delta_digests
        .clone()
}

/// DISCRIMINATOR for #719, at the write path the report names.
///
/// `change approve --portable-5-0-1` appends two `definition`-gate approvals, and
/// `effective_definition_approval` reads the LAST one. Recording no delta wording on them
/// therefore did not merely add a silent event: it made the change's effective approval silent,
/// undoing a claim `change approve` had already recorded. On the unfixed binary the assertion
/// below reads `None` where a digest keyed by `auth` had just been written.
///
/// The second half is the other thing carrying the binding forward must not cost. A portable
/// approval exists to be verifiable by SpecSync 5.0.1, so the pair's own digests, its metadata and
/// its resolution have to come out exactly as before — `approved_delta_digests` is an input to
/// none of them, and this pins that rather than assuming it.
#[test]
fn a_portable_definition_approval_carries_the_delta_binding_it_inherits() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_v1_change_with_a_delta(root);
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let approved = recorded_delta_digests(root, &record)
        .expect("the ordinary approve must record the delta bodies it approved");
    assert_eq!(approved.keys().collect::<Vec<_>>(), vec!["auth"]);

    append_portable_definition_approval_v501(root, &record, Some("Scope owner".into()), None)
        .unwrap();

    assert_eq!(
        recorded_delta_digests(root, &record).as_ref(),
        Some(&approved),
        "a portable approval must not leave the change's effective approval claiming less \
         about delta wording than the approval it supersedes"
    );
    let ledger = load_approvals(root, &record).unwrap();
    let portable: Vec<&ApprovalRecord> = ledger
        .approvals
        .iter()
        .filter(|approval| approval.definition_pair.is_some())
        .collect();
    assert_eq!(portable.len(), 2);
    for approval in &portable {
        assert_eq!(
            approval.approved_delta_digests.as_ref(),
            Some(&approved),
            "both members of the pair are definition approvals and both must say what they signed"
        );
    }

    let (current, legacy, _) = portable_definition_digest_pair_v501(root, &record).unwrap();
    assert_eq!(portable[0].digest, current);
    assert_eq!(portable[1].digest, legacy);
    assert_eq!(
        portable[1].note.as_deref(),
        Some("Portable SpecSync 5.0.1 definition projection")
    );
    assert!(
        ensure_definition_approval_valid(root, &record).is_ok(),
        "the 5.0.1 projection must still resolve; the delta binding is not one of its inputs"
    );
}

/// DISCRIMINATOR for #719, at the consequence.
///
/// Honest label: the downgraded ledger here is written directly rather than by
/// `change approve --portable-5-0-1`, and that is deliberate. The portable projection is
/// workflow-v1-only, and a v1 definition digest hashes every delta body through
/// `definition_artifact_snapshot` — so on a v1 change a swapped delta is independently caught by
/// `ensure_definition_approval_valid`, and the portable downgrade costs recorded evidence rather
/// than a refusal. What generalizes, and what the fix has to refuse, is the SHAPE: a later
/// definition approval that records no delta wording on a change whose ledger already recorded
/// some. That shape is exactly what `append_portable_definition_approval_v501` wrote, and under
/// workflow v2 — where the scope digest deliberately hashes intent and boundary only — it is the
/// whole of what stands between a swapped body and the canonical spec.
///
/// Unfixed, this test does not fail on a message; it fails because `specs/auth/auth.spec.md`
/// contains BACKDOOR.
#[test]
fn a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    let mut ledger = load_approvals(root, &record).unwrap();
    let mut withdrawn = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .expect("the fixture approves the definition")
        .clone();
    withdrawn.approved_delta_digests = None;
    withdrawn.timestamp += 1;
    ledger.approvals.push(withdrawn);
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();

    fs::write(delta_path(root, &record, "auth"), SWAPPED_DELTA_BODY).unwrap();
    let error = materialize_change_deltas(root, &record.id)
        .expect_err("a withdrawn delta binding must not read as an approval that predates it");

    assert!(
        error.contains("records no semantic delta wording"),
        "a refusal must say the approval claims less than an earlier one did: {error}"
    );
    assert!(
        error.contains(&format!("specsync change approve {}", record.id)),
        "a refusal must name the command that restores a truthful ledger: {error}"
    );
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        !spec.contains("BACKDOOR"),
        "the living spec carries wording nobody approved: {spec}"
    );
    assert!(
        !load_change(root, &record.id).unwrap().canonical_applied,
        "a refused materialization must not record itself as applied"
    );
}

/// CONTROL, and it passes on the unfixed binary too — that is the entire point of writing it.
///
/// Monotonicity is a property of a LEDGER, so the way to get it wrong is to read "this approval
/// records nothing" as "something was withdrawn" on a ledger where nothing was ever recorded.
/// A pre-#711 change that was approved more than once is precisely that ledger: several
/// definition approvals, not one of them carrying a digest, and the archive is full of them.
/// If the refusal above is ever written as "the latest approval records nothing" instead of
/// "an earlier approval recorded more", this is the test that fails — and it fails as an outage
/// across recorded history, not as a caught bug.
#[test]
fn a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    strip_recorded_delta_digests(root, &record);

    let ledger = load_approvals(root, &record).unwrap();
    assert!(
        ledger
            .approvals
            .iter()
            .filter(|approval| approval.gate == "definition")
            .count()
            > 1,
        "the fixture must carry more than one silent definition approval"
    );
    let ledger_bytes =
        fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap();
    assert!(
        !ledger_bytes.contains("approved_delta_digests"),
        "the fixture must be shaped like a pre-#711 ledger, not merely be missing a value: {ledger_bytes}"
    );

    fs::write(delta_path(root, &record, "auth"), SWAPPED_DELTA_BODY).unwrap();
    let applied = materialize_change_deltas(root, &record.id)
        .expect("a ledger that never recorded delta wording withdrew nothing");

    assert!(applied.canonical_applied);
}

/// The legitimate use of `--portable-5-0-1` is a workflow-v1 change being handed to a 5.0.1
/// verifier, and nothing about it requires an ordinary approval first. Refusing the portable
/// approve whenever a digest already existed would have been the cheaper fix; this is the test
/// that shows the chosen one costs the untouched case nothing.
#[test]
fn a_portable_definition_approval_records_delta_wording_with_no_prior_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_v1_change_with_a_delta(root);
    assert!(
        load_approvals(root, &record).unwrap().approvals.is_empty(),
        "the fixture must reach the portable approve with nothing recorded before it"
    );

    append_portable_definition_approval_v501(root, &record, Some("Scope owner".into()), None)
        .unwrap();

    assert_eq!(
        recorded_delta_digests(root, &record)
            .expect("a definition approval records the wording it approved")
            .keys()
            .collect::<Vec<_>>(),
        vec!["auth"]
    );
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
    assert!(
        ensure_approved_delta_bodies_unchanged(root, &record).is_ok(),
        "the bodies the portable approve just read are the bodies on disk"
    );
}

/// The approved body as a checkout with `core.autocrlf=true` materializes it.
///
/// Derived rather than transcribed, and checked against the original both ways, so the fixture
/// cannot silently become "some other text that happens to contain CRLF".
fn approved_delta_body_as_a_crlf_checkout_writes_it() -> String {
    let crlf = APPROVED_DELTA_BODY.replace('\n', "\r\n");
    assert_ne!(
        crlf, APPROVED_DELTA_BODY,
        "the fixture must actually re-encode the line endings"
    );
    assert_eq!(
        crlf.replace("\r\n", "\n"),
        APPROVED_DELTA_BODY,
        "the fixture must differ from the approved body in line endings and in nothing else"
    );
    crlf
}

/// DISCRIMINATOR for #730. On the unfixed binary this refuses with
/// "semantic delta for `auth` changed after approval" for a delta nobody edited.
///
/// The reproduction is a cross-OS handoff and nothing more: the change is approved where the
/// delta is LF, and the delta is then re-encoded to CRLF exactly as Git materializes it in a
/// working tree with `core.autocrlf=true` or `text eol=crlf`. Not one character of wording moves.
///
/// The module already decides this case, twice, and decides it the other way:
/// `markdown_block_matches` folds CRLF before comparing, `apply_markdown_block` re-emits every
/// body in the target file's own style, and `parse_delta` reads through `str::lines()`, which
/// discards the `\r` of a CRLF pair. The materialized canonical spec is therefore byte-identical
/// either way — which is what the last assertion pins. Only the digest disagreed, so the #711
/// gate refused honest work, and the remedy it names (re-approve) re-signs bytes the operator did
/// not choose and diverges again on the next handoff in the other direction.
#[test]
fn a_delta_a_checkout_rewrote_to_crlf_still_reaches_the_canonical_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    fs::write(
        delta_path(root, &record, "auth"),
        approved_delta_body_as_a_crlf_checkout_writes_it(),
    )
    .unwrap();
    let applied = materialize_change_deltas(root, &record.id)
        .expect("a delta whose line endings a checkout rewrote was not edited by anybody");

    assert!(applied.canonical_applied);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec.contains("Auth tracks credentials. Reviewed and approved wording."),
        "the approved wording must reach the canonical spec from a CRLF delta too: {spec}"
    );
    assert!(
        !spec.contains('\r'),
        "the canonical spec must not inherit the delta's line-ending style: {spec:?}"
    );
}

/// Honest label: this is the CONTROL for the discriminator above, and it is the important half.
/// It passes on the unfixed binary too — that is the point. "Normalize everything" would satisfy
/// the discriminator, so this is the assertion that says what the normalization may NOT do.
///
/// A real wording change must still be refused, and arriving in CRLF must not launder it. The
/// swapped body here differs from the approved one in its words AND its line endings, so the
/// digest has to separate the two axes rather than erase both. If someone ever widens
/// `canonical_delta_body` to normalize the body as a whole — folding case, collapsing runs of
/// whitespace, or comparing through `parse_delta` — this test starts letting BACKDOOR through and
/// the #711 binding is gone while its tests still say it is there.
#[test]
fn a_reworded_delta_is_refused_even_when_it_arrives_with_rewritten_line_endings() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    fs::write(
        delta_path(root, &record, "auth"),
        SWAPPED_DELTA_BODY.replace('\n', "\r\n"),
    )
    .unwrap();
    let error = materialize_change_deltas(root, &record.id)
        .expect_err("re-encoding a swapped body must not launder the swap");

    assert!(
        error.contains("`auth`") && error.contains("changed after approval"),
        "a refusal must still name the module and what went wrong: {error}"
    );
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        !spec.contains("BACKDOOR"),
        "the living spec carries wording nobody approved: {spec}"
    );
    assert!(
        !load_change(root, &record.id).unwrap().canonical_applied,
        "a refused materialization must not record itself as applied"
    );
}

/// Honest label: CONTROL, and it passes on the unfixed binary — it is a boundary marker, not
/// evidence of the fix. It pins the exact line #730 said not to cross.
///
/// `markdown_block_matches` compares "ignoring line-ending style AND surrounding blank lines",
/// and it also trims spaces and tabs. The digest deliberately copies only the first half. The
/// tempting simplification is to reuse the applier's `normalize` and be done; each body below is
/// one the applier would call equal to the approved one, and every one of them must still be
/// REFUSED, because the digest answers a different question. The applier asks whether an edit is
/// already applied; the digest asks whether an approver read these bytes. Blank lines and trailing
/// whitespace are wording a reviewer signed, and Git rewrites neither of them on its own — the
/// line-ending axis is the only one with no author behind it.
///
/// If this test ever goes green-by-acceptance, the binding no longer distinguishes an edited
/// delta from an untouched one on any axis the applier happens to fold.
#[test]
fn a_delta_edited_only_in_whitespace_the_applier_would_ignore_is_still_refused() {
    let variants = [
        ("a trailing blank line", format!("{APPROVED_DELTA_BODY}\n")),
        ("a leading blank line", format!("\n{APPROVED_DELTA_BODY}")),
        (
            "trailing spaces on a content line",
            APPROVED_DELTA_BODY.replace("approved wording.\n", "approved wording.   \n"),
        ),
        (
            "a tab indenting a content line",
            APPROVED_DELTA_BODY.replace("Auth tracks", "\tAuth tracks"),
        ),
    ];

    for (description, body) in variants {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = change_with_an_approved_delta(root);
        assert_ne!(
            body, APPROVED_DELTA_BODY,
            "the {description} fixture must actually differ from the approved body"
        );

        fs::write(delta_path(root, &record, "auth"), &body).unwrap();
        let error = match materialize_change_deltas(root, &record.id) {
            Ok(_) => panic!("a delta edited by {description} must not be read as unchanged"),
            Err(error) => error,
        };

        assert!(
            error.contains("changed after approval"),
            "a delta edited by {description} must still be refused: {error}"
        );
    }
}

/// Honest label: CHARACTERIZATION of a decision taken deliberately, and it passes on the unfixed
/// binary too.
///
/// A LONE `\r` is content, and #730 asked for that to be decided rather than omitted. Git's
/// `text`, `eol` and `core.autocrlf` conversions only ever move between LF and CRLF, so no
/// checkout can introduce a classic-Mac terminator; `str::lines()` and `markdown_block_matches`
/// both keep a bare `\r` as ordinary text, and `parser::parse_frontmatter` preserves it for the
/// same reason (#715). So a bare `\r` reaches the canonical spec, which makes it wording — and a
/// body that gained one was edited by a person, not rewritten by a checkout.
///
/// Folding it would be the widening this fix exists to avoid: the digest would stop distinguishing
/// two deltas that materialize different canonical specs.
#[test]
fn a_lone_carriage_return_is_delta_content_and_is_not_folded_away() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    let body = APPROVED_DELTA_BODY.replace("Reviewed and", "Reviewed\rand");
    assert!(
        !body.contains("\r\n"),
        "the fixture must carry a BARE carriage return, not a CRLF pair"
    );

    fs::write(delta_path(root, &record, "auth"), &body).unwrap();
    let error = materialize_change_deltas(root, &record.id)
        .expect_err("a bare carriage return is content, so a body that gained one was edited");

    assert!(
        error.contains("changed after approval"),
        "a refusal must say what went wrong: {error}"
    );
    assert!(
        matches!(
            canonical_delta_body("no carriage return here"),
            Cow::Borrowed(_)
        ),
        "an LF-only body must take the borrowed path and allocate nothing"
    );
    assert_eq!(
        canonical_delta_body("a\r\nb\rc\n").as_ref(),
        "a\nb\rc\n",
        "only CRLF pairs fold; a bare carriage return survives"
    );
}

/// COMPATIBILITY, and the assertion #730 required be MEASURED rather than reasoned about.
///
/// The normalizing digest must be byte-identical to the digest the unnormalized binding recorded
/// for an LF body, or every `approved_delta_digests` written since #711 silently becomes a
/// refusal. The expected value below is the pre-#730 digest — SHA-256 over the framed domain,
/// module and RAW body bytes — so this fails if the normalization ever touches an LF delta.
///
/// The same recomputation was run across all 198 archived `approvals.json` in this repository:
/// 25 recorded module digests, 0 of which move.
#[test]
fn an_lf_delta_hashes_to_exactly_the_digest_the_unnormalized_binding_recorded() {
    const PRE_NORMALIZATION_DIGEST: &str =
        "66d9882e0429aff9d0dc043c78e84b526e021e9678693967e423fdd06f00734a";

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);

    assert_eq!(
        recorded_delta_digests(root, &record)
            .expect("approve records the delta bodies it approved")
            .get("auth")
            .map(String::as_str),
        Some(PRE_NORMALIZATION_DIGEST),
        "normalizing line endings must not move the digest of an LF delta; every digest recorded \
         since #711 was written over raw bytes and must keep verifying"
    );
}

/// The wording a reviewer asked for instead, signed by a second approval.
///
/// Same module, same section, same shape as `APPROVED_DELTA_BODY` — only the sentence moves,
/// which is exactly what correcting a delta after review is.
const CORRECTED_DELTA_BODY: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials and sessions. Corrected during review.\n";

/// Approve, materialize, correct the delta, re-approve — the ordinary review loop, up to the
/// point where the second `check` has to notice that the canonical spec is behind.
fn change_materialized_then_corrected_and_re_approved(root: &Path) -> ChangeRecord {
    let record = change_with_an_approved_delta(root);
    let applied = materialize_change_deltas(root, &record.id).unwrap();
    assert!(
        applied.canonical_applied,
        "the fixture must reach the second check with materialization already recorded"
    );
    fs::write(delta_path(root, &record, "auth"), CORRECTED_DELTA_BODY).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(
        ensure_approved_delta_bodies_unchanged(root, &record).is_ok(),
        "re-approval makes the #711 guard pass by construction; that is why it cannot see this"
    );
    record
}

/// DISCRIMINATOR for #741. On the unfixed binary the second `materialize_change_deltas` returns
/// `Ok` and writes nothing at all, so the canonical spec keeps the FIRST delta's wording forever
/// while `check` exits 0.
///
/// The sequence is the review loop and nothing else: approve, materialize, a reviewer asks for
/// different wording, correct the delta, re-approve, check again. #711's guard passes on that
/// sequence by construction — a new approval signs the new body — and `canonical_applied` then
/// returned before anything could ask the other question, which is whether the canonical spec
/// still matches the delta. The purpose of correcting a delta is to change the canonical spec,
/// so this was the one path where doing what review asks for discarded the result in silence.
#[test]
fn a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_materialized_then_corrected_and_re_approved(root);

    materialize_change_deltas(root, &record.id).unwrap();

    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec.contains("Auth tracks credentials and sessions. Corrected during review."),
        "the corrected wording must reach the canonical spec: {spec}"
    );
    assert!(
        !spec.contains("Auth tracks credentials. Reviewed and approved wording."),
        "the superseded wording must not survive beside the correction: {spec}"
    );
    assert!(
        spec.contains("version: 1.0.1"),
        "correcting a delta must not bump the version a second time; one change bumps one \
         module's version exactly once: {spec}"
    );
    assert_eq!(
        spec.matches(&format!("{}:", record.id)).count(),
        1,
        "correcting a delta must not append a second Change Log row for the same change: {spec}"
    );
}

/// DISCRIMINATOR for #741's widening, and the half that rules out the narrowest repair.
///
/// `bump_spec_version` and `append_changelog` have exactly one caller, inside the materialization
/// the short-circuit skipped, so the flag skipped all THREE outputs and not merely the applied
/// delta. "Re-apply the delta when its digest moved" would leave these two still skipped: neither
/// a `version:` integer nor a Change Log row is derivable from a delta digest.
///
/// The fixture is the #721 shape, built by deleting exactly those two outputs from a spec that
/// has all three — contract text present, no bump, no row. A rebase that takes upstream's
/// frontmatter and Change Log while keeping the merged body produces precisely this, and on the
/// unfixed binary `change check`, `change audit --strict` and `specsync check --strict` all pass
/// over it, because nothing below the flag runs to ask.
#[test]
fn a_materialized_spec_missing_its_version_bump_and_change_log_row_gets_both_back() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);
    let spec_path = root.join("specs/auth/auth.spec.md");
    materialize_change_deltas(root, &record.id).unwrap();

    let materialized = fs::read_to_string(&spec_path).unwrap();
    let row = format!("{}:", record.id);
    assert!(
        materialized.contains("version: 1.0.1") && materialized.contains(&row),
        "the fixture must start from a spec that has all three outputs: {materialized}"
    );
    let rebased = format!(
        "{}\n",
        materialized
            .lines()
            .filter(|line| !line.contains(&row))
            .collect::<Vec<_>>()
            .join("\n")
    )
    .replace("version: 1.0.1", "version: 1.0.0");
    assert!(
        rebased.contains("Auth tracks credentials. Reviewed and approved wording.")
            && rebased.contains("version: 1.0.0")
            && !rebased.contains(&row),
        "the fixture must keep the contract text and lose only the bump and the row: {rebased}"
    );
    fs::write(&spec_path, &rebased).unwrap();

    materialize_change_deltas(root, &record.id).unwrap();

    let repaired = fs::read_to_string(&spec_path).unwrap();
    assert!(
        repaired.contains("version: 1.0.1"),
        "a spec carrying this change's contract text must carry its version bump: {repaired}"
    );
    assert_eq!(
        repaired.matches(&row).count(),
        1,
        "a spec carrying this change's contract text must carry exactly one Change Log row for \
         it: {repaired}"
    );
}

/// Honest label: this is the CONTROL, and it is the more important half of the pair above. It
/// passes on the unfixed binary too — that is the entire point.
///
/// "Always re-materialize" would satisfy both discriminators and would be a disaster: every
/// `check` would rewrite the canonical specs, bump their versions again and append another
/// Change Log row, which is the exact reason the short-circuit exists. A re-approval whose delta
/// body is byte-identical has nothing outstanding, so it must still short-circuit and leave all
/// three outputs alone — the spec byte for byte, the version at one bump, the Change Log at one
/// row.
///
/// If this test ever goes green only because the assertions were loosened, the fix has become the
/// outage it was supposed to avoid.
#[test]
fn re_approving_a_byte_identical_delta_leaves_the_canonical_spec_byte_for_byte_alone() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = change_with_an_approved_delta(root);
    let spec_path = root.join("specs/auth/auth.spec.md");
    let requirements_path = root.join("specs/auth/requirements.md");
    materialize_change_deltas(root, &record.id).unwrap();

    let spec_after_first = fs::read_to_string(&spec_path).unwrap();
    let requirements_after_first = fs::read_to_string(&requirements_path).unwrap();
    let row = format!("{}:", record.id);
    assert!(
        spec_after_first.contains("version: 1.0.1") && spec_after_first.matches(&row).count() == 1,
        "the first materialization must produce exactly one bump and one row: {spec_after_first}"
    );
    assert_eq!(
        fs::read_to_string(delta_path(root, &record, "auth")).unwrap(),
        APPROVED_DELTA_BODY,
        "the control re-approves the SAME bytes; nothing about the delta may move"
    );

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();

    assert_eq!(
        fs::read_to_string(&spec_path).unwrap(),
        spec_after_first,
        "a re-approval that changes nothing must not rewrite the canonical spec"
    );
    assert_eq!(
        fs::read_to_string(&requirements_path).unwrap(),
        requirements_after_first,
        "a re-approval that changes nothing must not rewrite the canonical requirements"
    );
}

/// DISCRIMINATOR for #741 on the operation that makes re-materialization hard.
///
/// `apply_markdown_block` refuses to remove a block that is not there, and it is right to: on a
/// first run that means the delta names something that never existed. But after materialization
/// the block is gone BECAUSE this change removed it, so re-applying the same delta would refuse
/// its own work. A fix that re-materializes without separating those two readings turns every
/// corrected `## REMOVED` delta into a hard error instead of a silent skip, which is a different
/// defect rather than none.
///
/// On the unfixed binary the second check short-circuits and the corrected section never lands.
#[test]
fn a_corrected_delta_re_materializes_over_a_block_its_own_earlier_run_removed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record =
        completed_section_only_current_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    let requirements_path = root.join("specs/auth/requirements.md");
    fs::write(
        &requirements_path,
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL retire this.\n",
    )
    .unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    assert!(
        !fs::read_to_string(&requirements_path)
            .unwrap()
            .contains("REQ-auth-001"),
        "the fixture must actually remove the requirement on the first run"
    );

    fs::write(
        delta_path(root, &record, "auth"),
        format!("## REMOVED\n\n### REQUIREMENT REQ-auth-001\n\n{CORRECTED_DELTA_BODY}"),
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    materialize_change_deltas(root, &record.id).expect(
        "re-materializing must not refuse the removal its own earlier run already performed",
    );

    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec.contains("Auth tracks credentials and sessions. Corrected during review."),
        "the corrected section must reach the canonical spec: {spec}"
    );
    assert!(
        !fs::read_to_string(&requirements_path)
            .unwrap()
            .contains("REQ-auth-001"),
        "the removal must stay removed across the re-materialization"
    );
}
