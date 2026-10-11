
/// Vacuity control: an ordinary description is unchanged by all of the above.
///
/// Must produce the same slug on this binary and the one before it, so the change cannot be
/// satisfied by mangling every slug.
#[test]
fn an_ordinary_description_slugifies_exactly_as_before() {
    assert_eq!(
        slugify("Identity must come from state.json, never from the shape of a name"),
        "identity-must-come-from-state-json-never-from-the-shape-of-a-name"
    );
    assert_eq!(slugify("Add reversal"), "add-reversal");
}

/// An identity with no ordinal is a legal change ID.
///
/// `validate_change_id` opened with `id.starts_with("CHG-")`, which hard-rejected every shape
/// without an ordinal. It gates `find_change_dir` and `validate_loaded_change`, so it gated the
/// whole system on a prefix anyone can type.
#[test]
fn a_change_id_without_an_ordinal_is_accepted() {
    for id in [
        "retire-the-auth-module",
        "a-slug-only-identity",
        "CHG-0158-and-the-old-shape-too",
    ] {
        assert!(
            validate_change_id(id).is_ok(),
            "`{id}` must be a legal change ID"
        );
    }
}

/// The safety half is unchanged, and gained the bounds it never had.
///
/// The prefix test was never evidence of well-formedness. These are the properties that
/// actually matter for a string used as a directory component — and two of them were
/// unenforced, survivable only because every ID was generated from a capped slug.
#[test]
fn an_unsafe_or_unbounded_change_id_is_still_refused() {
    let cases: [(&str, &str); 8] = [
        ("../escape", "path traversal"),
        ("a/b", "forward slash"),
        ("a\\b", "backslash"),
        (".", "current directory"),
        ("..", "parent directory"),
        ("", "empty"),
        ("nul", "Windows reserved device"),
        ("con", "Windows reserved device"),
    ];
    for (id, why) in cases {
        assert!(
            validate_change_id(id).is_err(),
            "`{}` must be refused ({why})",
            id.escape_default()
        );
    }
    assert!(
        validate_change_id(&"a".repeat(MAX_CHANGE_ID_BYTES + 1)).is_err(),
        "an ID longer than a path component must be refused"
    );
    assert!(
        validate_change_id(&"a".repeat(MAX_CHANGE_ID_BYTES)).is_ok(),
        "an ID exactly at the limit must be accepted"
    );
    assert!(
        validate_change_id("has\u{7f}control").is_err(),
        "control characters must be refused"
    );
}

/// Vacuity control: every ID this repository has ever minted is still legal.
///
/// Without this, "reject everything" would satisfy the refusal test above.
#[test]
fn every_historical_identity_shape_remains_legal() {
    for id in [
        "CHG-0001-bootstrap-and-ship-the-verified-specsync-5-0-full-sdd-lifecycle",
        "CHG-0091-add-change-ship-status-for-local-ship-readiness-and-merge-before-finalize-warning",
        "CHG-10000-large-sequence",
    ] {
        assert!(validate_change_id(id).is_ok(), "`{id}` must remain legal");
    }
}

#[test]
fn an_undated_package_stripped_of_its_lifecycle_files_is_still_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let package = root.join(ARCHIVE_PATH).join("CHG-0001-foo");
    fs::create_dir_all(package.join("deltas")).unwrap();
    fs::write(
        package.join("deltas/auth.md"),
        "## REMOVED\n### REQUIREMENT REQ-auth-001\nRetired.\n",
    )
    .unwrap();
    // One artifact git could track. The package is damaged, not absent.
    fs::write(package.join("plan.md"), "# plan\n").unwrap();

    let error =
        list_all_changes_checked(root).expect_err("a damaged package must be refused, not skipped");
    assert!(error.contains("failed to read archived state"), "{error}");
    let error =
        located_change_sequences(root).expect_err("a damaged package must be refused, not skipped");
    assert!(
        error.contains("failed to read archived change state"),
        "{error}"
    );
}

/// The identity shape must not decide it either. Same package, name with no ordinal at all.
#[test]
fn a_slug_named_package_stripped_of_its_lifecycle_files_is_still_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let package = root.join(ARCHIVE_PATH).join("retire-the-auth-module");
    fs::create_dir_all(package.join("deltas")).unwrap();
    fs::write(package.join("deltas/auth.md"), "## REMOVED\n").unwrap();
    fs::write(package.join("context.md"), "# context\n").unwrap();

    assert!(
        list_all_changes_checked(root).is_err(),
        "a damaged package must be refused whatever it is named"
    );
}

/// Vacuity control: a genuine legacy tombstone is still skipped, so this is not simply
/// "refuse everything". It must behave identically on both binaries.
#[test]
fn a_deltas_only_legacy_tombstone_is_still_skipped_whatever_it_is_named() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // Only names WITHOUT a lifecycle ordinal. A dated or undated `CHG-NNNN` name is a real
    // package by signal 3 and must stay refused — see `an_undated_package_...` above.
    for name in ["legacy", "retired-auth", "pre-lifecycle-removals"] {
        let package = root.join(ARCHIVE_PATH).join(name);
        fs::create_dir_all(package.join("deltas")).unwrap();
        fs::write(package.join("deltas/auth.md"), "## REMOVED\n").unwrap();
    }
    // None carries an ordinal and none holds a file outside `deltas/`, so all three are
    // tombstones and all three are skipped — on this binary and on the one before it.
    assert!(list_all_changes_checked(root).unwrap().is_empty());
}

#[test]
fn archive_husk_of_empty_directories_is_skipped_by_enumeration() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // Exactly what a checkout of a pre-archive commit strands: the dated
    // package's tracked files are gone, the untrackable directory remains.
    fs::create_dir_all(
        root.join(ARCHIVE_PATH)
            .join("2026-08-18-CHG-0001-husk/deltas"),
    )
    .unwrap();
    assert!(list_all_changes_checked(root).unwrap().is_empty());
    assert!(located_change_sequences(root).unwrap().is_empty());
}

#[test]
fn archive_directory_with_files_but_no_state_is_still_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let package = root.join(ARCHIVE_PATH).join("2026-08-18-CHG-0001-broken");
    fs::create_dir_all(&package).unwrap();
    // A file git *could* track means the checkout theory does not apply: this
    // package is damaged, not absent, and skipping it would hide corruption.
    fs::write(package.join("change.md"), "# truncated\n").unwrap();
    let error = list_all_changes_checked(root).unwrap_err();
    assert!(error.contains("failed to read archived state"), "{error}");
    let error = located_change_sequences(root).unwrap_err();
    assert!(
        error.contains("failed to read archived change state"),
        "{error}"
    );
}

#[test]
fn archive_husk_nested_below_an_empty_directory_is_still_a_husk() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(
        root.join(ARCHIVE_PATH)
            .join("2026-08-18-CHG-0001-husk/deltas/nested"),
    )
    .unwrap();
    assert!(list_all_changes_checked(root).unwrap().is_empty());
}

/// An active directory that holds a file but no `state.json` is damaged, not absent.
///
/// An empty `deltas/` left by checkout is a husk and must stay invisible, including
/// to `change new`. Reporting that husk as corrupt made the first command fail.
#[test]
fn active_directory_with_files_and_no_state_is_still_open() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let healthy = create_change(
        root,
        CreateChangeRequest {
            description: "Healthy sibling".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/healthy/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("husk fixture".into()),
        },
    )
    .unwrap()
    .id;

    let damaged = root.join(CHANGES_PATH).join("broken-notes");
    fs::create_dir_all(damaged.join("deltas")).unwrap();
    fs::write(damaged.join("deltas").join("note.md"), "note\n").unwrap();
    fs::create_dir_all(root.join(CHANGES_PATH).join("left-husk").join("deltas")).unwrap();

    let roster = list_changes(root).unwrap();
    assert_eq!(
        roster
            .records
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>(),
        vec![healthy.as_str()],
        "a damaged neighbour must not erase the healthy change"
    );
    assert_eq!(roster.unreadable.len(), 1);
    assert_eq!(roster.unreadable[0].id, "broken-notes");
    assert!(
        roster.unreadable[0]
            .reason
            .contains("state file is missing"),
        "{}",
        roster.unreadable[0].reason
    );
    assert!(
        roster.unreadable[0].reason.contains("state.json"),
        "{}",
        roster.unreadable[0].reason
    );
    let missing = load_change(root, "broken-notes").unwrap_err();
    assert!(
        missing.contains("Restore state.json or remove the stray directory"),
        "{missing}"
    );

    let checked = list_changes_checked(root).unwrap_err();
    assert!(checked.contains("state file is missing"), "{checked}");

    let sequences = located_change_sequences(root).unwrap_err();
    assert!(
        sequences.contains("failed to read active change state"),
        "{sequences}"
    );
    assert!(sequences.contains("broken-notes"), "{sequences}");

    fs::remove_dir_all(&damaged).unwrap();
    let roster = list_changes(root).unwrap();
    assert!(
        !roster.is_degraded(),
        "a husk must not be unreadable: {:?}",
        roster.unreadable
    );
    assert_eq!(roster.records.len(), 1);
    assert!(located_change_sequences(root).is_ok());
    assert!(validate_change_sequences(root).is_ok());

    let linked = root.join(CHANGES_PATH).join("linked-notes");
    fs::create_dir_all(&linked).unwrap();
    std::os::unix::fs::symlink("missing-target", linked.join("pointer")).unwrap();
    let linked_roster = list_changes(root).unwrap();
    assert!(
        linked_roster
            .unreadable
            .iter()
            .any(|entry| entry.id == "linked-notes"),
        "a symlink is a tracked file, not a husk: {:?}",
        linked_roster.unreadable
    );
}

#[test]
fn archived_package_keeps_directories_that_hold_files_and_drops_the_rest() {
    let temp = TempDir::new().unwrap();
    let package = temp.path().join("2026-08-18-CHG-0001-demo");
    fs::create_dir_all(package.join("deltas")).unwrap();
    fs::create_dir_all(package.join("evidence/nested")).unwrap();
    fs::write(package.join("state.json"), "{}").unwrap();
    fs::write(package.join("evidence/nested/proof.md"), "kept\n").unwrap();
    prune_empty_package_directories(&package);
    assert!(!package.join("deltas").exists());
    assert!(package.join("evidence/nested/proof.md").is_file());
    assert!(package.join("state.json").is_file());
    assert!(package.is_dir());
}

#[test]
fn legacy_archive_baseline_bytes_are_canonical_sorted_and_definition_digestible() {
    let entry = |id: &str, path: &str| LegacyArchiveBaselineEntryV1 {
        id: id.into(),
        archive_path: path.into(),
        introduction_commit: "1111111111111111111111111111111111111111".into(),
        subtree_digest: "a".repeat(64),
    };
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: "CHG-0042-authority".into(),
        cutoff_commit: "2222222222222222222222222222222222222222".into(),
        entries: vec![
            entry(
                "CHG-0001-first",
                ".specsync/archive/changes/2026-07-11-CHG-0001-first",
            ),
            entry(
                "CHG-0002-second",
                ".specsync/archive/changes/2026-07-11-CHG-0002-second",
            ),
        ],
    };
    let bytes = json_content(&baseline).unwrap();
    let (_, digest) = validate_legacy_archive_baseline_bytes(bytes.as_bytes()).unwrap();
    validate_sha256_digest(&digest, "baseline digest").unwrap();

    let compact = serde_json::to_vec(&baseline).unwrap();
    assert!(
        validate_legacy_archive_baseline_bytes(&compact)
            .unwrap_err()
            .contains("canonical persisted JSON")
    );

    let mut unsorted = baseline.clone();
    unsorted.entries.reverse();
    let unsorted = json_content(&unsorted).unwrap();
    assert!(
        validate_legacy_archive_baseline_bytes(unsorted.as_bytes())
            .unwrap_err()
            .contains("strictly sorted")
    );

    let mut duplicate = baseline;
    duplicate.entries[1].id = duplicate.entries[0].id.clone();
    let duplicate = json_content(&duplicate).unwrap();
    assert!(
        validate_legacy_archive_baseline_bytes(duplicate.as_bytes())
            .unwrap_err()
            .contains("must each be unique")
    );
}

#[test]
fn definition_approval_binds_the_exact_legacy_baseline_bytes() {
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
    fs::write(root.join("seed"), "seed\n").unwrap();
    git(&["add", "seed"]);
    git(&["commit", "-m", "baseline cutoff"]);
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

    let approved = approve_definition(root, &authority.id, Some("Reviewer".into()), None).unwrap();

    assert_eq!(
        approved.legacy_archive_baseline_digest.as_deref(),
        Some(expected.as_str())
    );
    assert!(ensure_definition_approval_valid(root, &approved).is_ok());

    let mut changed = baseline;
    fs::write(root.join("later"), "later\n").unwrap();
    git(&["add", "later"]);
    git(&["commit", "-m", "descendant cutoff"]);
    changed.cutoff_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    write_json(&root.join(LEGACY_BASELINE_PATH), &changed).unwrap();
    let changed_bytes = fs::read(root.join(LEGACY_BASELINE_PATH)).unwrap();
    let (_, changed_digest) = validate_legacy_archive_baseline_bytes(&changed_bytes).unwrap();
    assert_ne!(
        approved.legacy_archive_baseline_digest.as_deref(),
        Some(changed_digest.as_str())
    );
    let error = bind_legacy_archive_baseline_authority(root, &mut authority).unwrap_err();
    assert!(error.contains("must equal the authority definition base commit"));
}

#[test]
fn legacy_baseline_cutoff_accepts_only_the_exact_definition_base_in_current_history() {
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
    fs::write(root.join("history"), "root\n").unwrap();
    git(&["add", "history"]);
    git(&["commit", "-m", "root"]);
    let ancestor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    fs::write(root.join("history"), "base\n").unwrap();
    git(&["commit", "-am", "definition base"]);
    let base = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let mut authority = completed_no_spec_record(root);
    authority.base_commit = Some(base.clone());

    assert!(validate_legacy_baseline_authority_cutoff(root, &authority, &base).is_ok());
    let earlier =
        validate_legacy_baseline_authority_cutoff(root, &authority, &ancestor).unwrap_err();
    assert!(earlier.contains("must equal the authority definition base commit"));

    fs::write(root.join("history"), "descendant\n").unwrap();
    git(&["commit", "-am", "descendant"]);
    let descendant = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let later =
        validate_legacy_baseline_authority_cutoff(root, &authority, &descendant).unwrap_err();
    assert!(later.contains("must equal the authority definition base commit"));

    git(&["switch", "-c", "divergent", &ancestor]);
    fs::write(root.join("divergent"), "divergent\n").unwrap();
    git(&["add", "divergent"]);
    git(&["commit", "-m", "divergent"]);
    let divergent = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    git(&["switch", "main"]);
    authority.base_commit = Some(divergent.clone());
    let unrelated =
        validate_legacy_baseline_authority_cutoff(root, &authority, &divergent).unwrap_err();
    assert!(unrelated.contains("not an ancestor of current authority history"));
}

#[test]
fn legacy_baseline_rejects_post_cutoff_archive_additions() {
    let baseline = LegacyArchiveBaselineV1 {
        schema_version: 1,
        domain: "specsync.legacy-archive-baseline.v1".into(),
        authority_change_id: "CHG-0043-authority".into(),
        cutoff_commit: "2222222222222222222222222222222222222222".into(),
        entries: vec![LegacyArchiveBaselineEntryV1 {
            id: "CHG-0001-pre-cutoff".into(),
            archive_path: ".specsync/archive/changes/2026-07-11-CHG-0001-pre-cutoff".into(),
            introduction_commit: "1111111111111111111111111111111111111111".into(),
            subtree_digest: "a".repeat(64),
        }],
    };

    assert!(legacy_baseline_entry(&baseline, "CHG-0001-pre-cutoff").is_ok());
    let error = legacy_baseline_entry(&baseline, "CHG-0044-post-cutoff").unwrap_err();
    assert!(error.contains("not enumerated by the baseline"));
}

#[test]
fn legacy_archive_subtree_digest_binds_path_mode_kind_and_payload() {
    let snapshot = BTreeMap::from([
        ("approvals.json".into(), (0o100644, b"approval".to_vec())),
        ("tool".into(), (0o100755, b"binary".to_vec())),
        ("link".into(), (0o120000, b"tool".to_vec())),
    ]);
    let expected = legacy_archive_subtree_digest(&snapshot).unwrap();
    for changed in [
        BTreeMap::from([
            ("approvals.json".into(), (0o100644, b"tampered".to_vec())),
            ("tool".into(), (0o100755, b"binary".to_vec())),
            ("link".into(), (0o120000, b"tool".to_vec())),
        ]),
        BTreeMap::from([
            ("approvals.json".into(), (0o100644, b"approval".to_vec())),
            ("tool".into(), (0o100644, b"binary".to_vec())),
            ("link".into(), (0o120000, b"tool".to_vec())),
        ]),
        BTreeMap::from([
            ("renamed.json".into(), (0o100644, b"approval".to_vec())),
            ("tool".into(), (0o100755, b"binary".to_vec())),
            ("link".into(), (0o120000, b"tool".to_vec())),
        ]),
    ] {
        assert_ne!(legacy_archive_subtree_digest(&changed).unwrap(), expected);
    }
    let escaped = BTreeMap::from([("link".into(), (0o120000, b"/tool".to_vec()))]);
    assert!(legacy_archive_subtree_digest(&escaped).is_err());
    let gitlink = BTreeMap::from([("nested".into(), (0o160000, vec![0; 20]))]);
    assert!(legacy_archive_subtree_digest(&gitlink).is_err());
}

#[test]
fn archive_snapshot_ignores_large_unrelated_index_inventory() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let subtree = ".specsync/archive/changes/2026-07-15-CHG-0043-evidence";
    let workspace = root.join(subtree);
    fs::create_dir_all(&workspace).unwrap();
    fs::write(workspace.join("approvals.json"), "archive\n").unwrap();
    let object = run_git_required(
        root,
        &["hash-object", "-w", "--stdin"],
        Some(b"unrelated\n".to_vec()),
        128,
    )
    .unwrap();
    let object = std::str::from_utf8(&object).unwrap().trim();
    let mut index = String::new();
    for number in 0..5_000 {
        index.push_str(&format!("100644 {object}\tunrelated-{number:05}.txt\n"));
    }
    run_git_required(
        root,
        &["update-index", "--index-info"],
        Some(index.into_bytes()),
        128,
    )
    .unwrap();

    let snapshot = archive_workspace_snapshot(root, &workspace, subtree).unwrap();
    assert_eq!(
        snapshot,
        BTreeMap::from([(
            "approvals.json".to_string(),
            (0o100644, b"archive\n".to_vec())
        )])
    );
}

#[test]
fn git_candidate_inspection_deduplicates_identical_overlapping_pathspec_entries() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let parent = ".specsync/changes";
    fs::create_dir_all(root.join(parent)).unwrap();
    let mut candidates = BTreeSet::from([parent.to_string()]);
    for number in 0..(GIT_ATTRIBUTE_BATCH_PATHS + 1) {
        let path = format!("{parent}/evidence-{number:04}.json");
        fs::write(root.join(&path), "{}\n").unwrap();
        candidates.insert(path);
    }
    quiet_git(root, &["add", parent]);

    let inspected = inspect_git_candidates(root, &candidates, false, false).unwrap();

    assert_eq!(inspected.modes.len(), GIT_ATTRIBUTE_BATCH_PATHS + 1);
    assert_eq!(inspected.objects.len(), GIT_ATTRIBUTE_BATCH_PATHS + 1);
}

#[test]
fn git_stage_zero_accumulator_rejects_conflicting_mode() {
    let path = ".specsync/changes/evidence.json".to_string();
    let object = "a".repeat(40);
    let mut entries = BTreeMap::new();
    record_git_stage_zero_entry(&mut entries, path.clone(), 0o100644, object.clone()).unwrap();

    let error = record_git_stage_zero_entry(&mut entries, path.clone(), 0o100755, object.clone())
        .unwrap_err();

    assert_eq!(
        error,
        format!("conflicting duplicate Git index stage-zero entry for `{path}`")
    );
    assert_eq!(entries, BTreeMap::from([(path, (0o100644, object))]));
}

#[test]
fn git_stage_zero_accumulator_rejects_conflicting_object() {
    let path = ".specsync/changes/evidence.json".to_string();
    let original_object = "a".repeat(40);
    let mut entries = BTreeMap::new();
    record_git_stage_zero_entry(
        &mut entries,
        path.clone(),
        0o100644,
        original_object.clone(),
    )
    .unwrap();

    let error = record_git_stage_zero_entry(&mut entries, path.clone(), 0o100644, "b".repeat(40))
        .unwrap_err();

    assert_eq!(
        error,
        format!("conflicting duplicate Git index stage-zero entry for `{path}`")
    );
    assert_eq!(
        entries,
        BTreeMap::from([(path, (0o100644, original_object))])
    );
}

#[test]
fn dated_lifecycle_archive_missing_state_fails_global_enumeration() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let archived = root
        .join(ARCHIVE_PATH)
        .join("2026-07-15-CHG-0001-missing-state");
    fs::create_dir_all(archived.join("deltas")).unwrap();
    fs::write(
        archived.join("deltas/change.md"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nTampered.\n",
    )
    .unwrap();
    fs::write(archived.join("approvals.json"), "{}\n").unwrap();

    let all_error = list_all_changes_checked(root).unwrap_err();
    assert!(
        all_error.contains("failed to read archived state"),
        "{all_error}"
    );
    let sequence_error = located_change_sequences(root).unwrap_err();
    assert!(
        sequence_error.contains("failed to read archived change state"),
        "{sequence_error}"
    );
}

#[test]
fn status_and_check_share_exact_and_stale_terminal_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let exact = summarize_change(root, &record).terminal_evidence.unwrap();
    assert_eq!(exact.validity, TerminalEvidenceValidity::Exact);
    assert!(exact.reason.is_none());
    assert_eq!(check_project(root).terminal_evidence[0].evidence, exact);

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let stale = summarize_change(root, &record).terminal_evidence.unwrap();
    assert_eq!(stale.validity, TerminalEvidenceValidity::Stale);
    assert!(
        stale
            .reason
            .as_deref()
            .is_some_and(|reason| !reason.is_empty())
    );
    assert_eq!(check_project(root).terminal_evidence[0].evidence, stale);
}

#[test]
fn read_scope_memoizes_repeated_summary_git_lookups() {
    let (temp, id, _) = verification_history_fixture();
    let root = temp.path();
    let record = load_change(root, &id).unwrap();
    let _scope = begin_change_read_scope(root);

    reset_test_git_process_count();
    let first = summarize_change(root, &record);
    let first_queries = test_git_process_count();
    let second = summarize_change(root, &record);

    assert_eq!(
        serde_json::to_value(&second).unwrap(),
        serde_json::to_value(&first).unwrap()
    );
    assert!(first_queries > 0);
    assert_eq!(test_git_process_count(), first_queries);
}

#[test]
fn strict_check_reports_standalone_unprovable_archived_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let source = change_dir(root, &record.id);
    let archived_dir = root
        .join(ARCHIVE_PATH)
        .join(format!("2026-07-14-{}", record.id));
    fs::create_dir_all(archived_dir.parent().unwrap()).unwrap();
    fs::rename(&source, &archived_dir).unwrap();
    record.state = ChangeState::Archived;
    write_json(&archived_dir.join("state.json"), &record).unwrap();

    let report = check_project(root);
    assert!(report.errors.iter().any(|error| {
        error.contains(&record.id) && error.contains("archived change historical integrity")
    }));
    assert_eq!(report.terminal_evidence.len(), 1);
    assert_eq!(
        report.terminal_evidence[0].evidence.validity,
        TerminalEvidenceValidity::CorruptHistory
    );
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!(
            "restore the archive for {} from trusted Git history, then run `specsync change check`",
            record.id
        )
    );
}

#[test]
fn legacy_archived_unowned_production_source_reconstructs_with_exact_delivery_owner() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    // Production source no canonical spec owns — the adoption-era shape from #397
    // (spec-less repos whose legacy inputs can never gain a canonical owner).
    fs::write(
        root.join("src/unowned.rs"),
        "pub fn adopted() -> bool { true }\n",
    )
    .unwrap();
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        definition_digest(root, &record).unwrap(),
        None,
    )
    .unwrap();
    record.state = ChangeState::Accepted;
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    let verification = VerificationRecord {
        timestamp: now(),
        commit: None,
        contract_digest: definition_digest(root, &record).unwrap(),
        execution_digest: None,
        workspace_digest: project_input_digest(root).unwrap(),
        acceptance_input_digest: Some(acceptance_input_digest(root, &record, &[]).unwrap()),
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
        &record,
        "acceptance",
        Some("Reviewer".into()),
        closing_digest(&record, &verification),
        None,
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
    git(&["commit", "-m", "record accepted legacy evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    // The strict current path still rejects the unowned production source.
    let error = acceptance_manifest(root, &record, &[]).unwrap_err();
    assert!(
        error.contains("production source without deterministic canonical ownership"),
        "{error}"
    );

    // Legacy reconstruction assigns the exact delivery owner instead, so archival and
    // archived-integrity validation succeed without any per-repo repair.
    archive_change(root, &record.id).unwrap();
    let record = load_change(root, &record.id).unwrap();
    let manifest = resolved_acceptance_manifest(root, &record).unwrap();
    let entry = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "src/unowned.rs")
        .expect("unowned source must remain an acceptance input");
    assert_eq!(entry.owners, vec![EXACT_DELIVERY_OWNER.to_string()]);
    assert!(
        manifest
            .entries
            .iter()
            .find(|entry| entry.path == "src/lib.rs")
            .is_some_and(|entry| entry.owners == ["change".to_string()])
    );
}

/// Builds an accepted change with one reacceptance reopening in its approvals ledger, then
/// strips the 5.1 digest fields from that reopening to reproduce a 5.0.1-era ledger.
fn reopening_ledger_without_digest_fields(root: &Path) -> ChangeRecord {
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(
        root,
        &record.id,
        "Reviewer".into(),
        "The governed source drifted after acceptance".into(),
    )
    .unwrap()
    .change;
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&approvals_path).unwrap()).unwrap();
    for reopening in ledger["reopenings"].as_array_mut().unwrap() {
        reopening
            .as_object_mut()
            .unwrap()
            .remove("stale_acceptance_input_digest");
        reopening
            .as_object_mut()
            .unwrap()
            .remove("current_acceptance_input_digest");
    }
    fs::write(
        &approvals_path,
        format!("{}\n", serde_json::to_string_pretty(&ledger).unwrap()),
    )
    .unwrap();
    record
}

#[test]
fn backfill_repairs_5_0_1_reopening_idempotently() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = reopening_ledger_without_digest_fields(root);
    let approvals_path = change_dir(root, &record.id).join("approvals.json");

    // The un-migrated ledger fails to parse, and the failure carries the migrate hint.
    let error = load_approvals(root, &record).unwrap_err();
    assert!(
        error.contains("missing field `stale_acceptance_input_digest`"),
        "{error}"
    );
    assert!(error.contains("specsync migrate 5.0"), "{error}");

    // Dry run reports the repair without writing.
    let before = fs::read(&approvals_path).unwrap();
    let report = backfill_reopen_digests(root, true).unwrap();
    assert_eq!(report.repaired, vec![record.id.clone()]);
    assert!(report.failed.is_empty());
    assert_eq!(fs::read(&approvals_path).unwrap(), before);

    // The write restores exactly the recorded evidence.
    let report = backfill_reopen_digests(root, false).unwrap();
    assert_eq!(report.repaired, vec![record.id.clone()]);
    assert!(report.is_clean());
    let ledger = load_approvals(root, &record).unwrap();
    let reopening = ledger.reopenings.last().unwrap();
    assert_eq!(
        reopening.stale_acceptance_input_digest,
        reopening
            .prior_verification
            .acceptance_input_digest
            .clone()
            .unwrap()
    );
    let superseding = load_verification(root, &record).unwrap();
    assert_eq!(
        reopening.current_acceptance_input_digest,
        superseding.acceptance_input_digest.unwrap()
    );
    assert_ne!(
        reopening.stale_acceptance_input_digest,
        reopening.current_acceptance_input_digest
    );

    // A second run is a no-op.
    let before = fs::read(&approvals_path).unwrap();
    let report = backfill_reopen_digests(root, false).unwrap();
    assert!(report.repaired.is_empty());
    assert_eq!(report.unchanged, vec![record.id.clone()]);
    assert_eq!(fs::read(&approvals_path).unwrap(), before);
}

#[test]
fn backfill_leaves_unrepairable_reopening_untouched() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = reopening_ledger_without_digest_fields(root);
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&approvals_path).unwrap()).unwrap();
    ledger["reopenings"][0]
        .as_object_mut()
        .unwrap()
        .get_mut("prior_verification")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("acceptance_input_digest");
    fs::write(
        &approvals_path,
        format!("{}\n", serde_json::to_string_pretty(&ledger).unwrap()),
    )
    .unwrap();
    let before = fs::read(&approvals_path).unwrap();
    let report = backfill_reopen_digests(root, false).unwrap();
    assert!(report.repaired.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert!(
        report.failed[0]
            .1
            .contains("missing its embedded prior verification"),
        "{:?}",
        report.failed
    );
    assert_eq!(fs::read(&approvals_path).unwrap(), before);
}

#[test]
fn normal_merge_does_not_create_a_duplicate_accepted_transition() {
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
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["switch", "-c", "feature"]);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept feature"]);
    git(&["switch", "main"]);
    git(&["merge", "--no-ff", "feature", "-m", "merge feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let (anchor, _, accepted) = authenticated_accepted_transition(root, &record).unwrap();
    assert_eq!(accepted.id, record.id);
    assert_ne!(anchor, git_output(root, &["rev-parse", "HEAD"]).unwrap());
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
}

#[test]
fn archive_post_move_failure_restores_exact_source_bytes_without_residue() {
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
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept change"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let source = change_dir(root, &record.id);
    let original_state = fs::read(source.join("state.json")).unwrap();
    let original_markdown = fs::read(source.join("change.md")).unwrap();
    let error = archive_change_with_finalize_failure(root, &record.id, true).unwrap_err();
    assert!(error.contains("source restored"), "{error}");
    assert_eq!(fs::read(source.join("state.json")).unwrap(), original_state);
    assert_eq!(
        fs::read(source.join("change.md")).unwrap(),
        original_markdown
    );
    assert!(!source.join("accepted-state.json").exists());
    assert!(
        !root
            .join(ARCHIVE_PATH)
            .join(format!("{}-{}", today(), record.id))
            .exists()
    );
}

#[test]
fn authenticated_archive_ignores_later_input_drift_but_rejects_snapshot_tampering() {
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
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept change"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let archived_dir = archive_change(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "archive change"]);

    let snapshot = fs::read(archived_dir.join("accepted-state.json")).unwrap();
    fs::remove_file(archived_dir.join("accepted-state.json")).unwrap();
    let missing_snapshot =
        validate_archived_integrity(root, &load_change(root, &record.id).unwrap()).unwrap_err();
    assert!(missing_snapshot.contains("missing its authenticated accepted-state snapshot"));
    fs::write(archived_dir.join("accepted-state.json"), &snapshot).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "evolve archived input"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let report = check_project(root);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(
        report.terminal_evidence[0].evidence.validity,
        TerminalEvidenceValidity::AuthenticatedHistory
    );

    fs::write(archived_dir.join("accepted-state.json"), b"{}\n").unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains(&record.id) && error.contains("historical integrity") })
    );
    assert_eq!(
        report.terminal_evidence[0].evidence.validity,
        TerminalEvidenceValidity::CorruptHistory
    );
}

#[test]
fn gitlink_manifest_hashes_the_exact_index_object_id() {
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
    fs::write(root.join("seed"), "seed\n").unwrap();
    git(&["add", "seed"]);
    git(&["commit", "-m", "seed"]);
    let object = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{object},vendor/library"),
    ]);
    let mut record = completed_no_spec_record(root);
    record.state = ChangeState::Accepted;
    record.affected_paths = vec!["vendor/library".into()];
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    let entry = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "vendor/library")
        .unwrap();
    assert_eq!(entry.kind, AcceptanceInputKind::Gitlink);
    assert_eq!(entry.mode, 0o160000);
    assert_eq!(entry.payload_digest, sha256_hex(object.as_bytes()));
}

#[test]
fn persisted_supersedes_cycle_fails_before_predecessor_manifest_use() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut first = current_workflow_record(
        root,
        completed_section_only_record(root, "## MODIFIED\n### SPEC SECTION Invariants\n\nFirst.\n"),
    );
    let mut second = create_change(
        root,
        CreateChangeRequest {
            description: "Second successor".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    let obligation = SuccessionObligation {
        path: "src/auth.rs".into(),
        module: "auth".into(),
        predecessor_entry_digest: sha256_hex(b"entry"),
    };
    first.state = ChangeState::Accepted;
    first.supersedes = vec![SupersedesEdge {
        predecessor_id: second.id.clone(),
        obligations: vec![obligation.clone()],
    }];
    save_change(root, &first).unwrap();
    second.supersedes = vec![SupersedesEdge {
        predecessor_id: first.id.clone(),
        obligations: vec![obligation],
    }];
    let error = validate_supersedes_semantics(root, &second).unwrap_err();
    assert!(error.contains("succession cycle"), "{error}");
}

/// Two sorts over one list disagreed at five digits, and one of them fed a digest.
///
/// `validate_supersedes_edges` enforced a NUMERIC strict sort while `approved_scope` sorted the
/// same edges lexicographically and hashed the result into `scope_digest`. Those agree only
/// while every ordinal is four digits:
///
/// ```text
/// numeric:        CHG-9999  <  CHG-10000
/// lexicographic:  CHG-10000 <  CHG-9999
/// ```
///
/// So at `CHG-10000` — a shape the CI harness already exercises — `approved_scope` emitted an
/// order `validate_supersedes_edges` then rejected. Ordering succession by creation time rather
/// than by the name settles it: everything is lexicographic now, and all three agree.
#[test]
fn supersedes_edges_sort_the_same_way_everywhere_at_five_digits() {
    let obligation = |module: &str| SuccessionObligation {
        path: format!("src/{module}.rs"),
        module: module.into(),
        predecessor_entry_digest: sha256_hex(b"entry"),
    };
    // Lexicographic order puts CHG-10000 first; numeric order puts CHG-9999 first.
    let edges = vec![
        SupersedesEdge {
            predecessor_id: "CHG-10000-large-sequence".into(),
            obligations: vec![obligation("auth")],
        },
        SupersedesEdge {
            predecessor_id: "CHG-9999-earlier-ordinal".into(),
            obligations: vec![obligation("billing")],
        },
    ];
    let temp = TempDir::new().unwrap();
    let mut record = completed_no_spec_record(temp.path());
    record.supersedes = edges.clone();
    record.affected_specs = vec!["auth".into(), "billing".into()];

    validate_supersedes_edges(&record)
        .expect("lexicographic order must be accepted by the strict-sort gate");

    // And the reverse order must be refused, so the gate is a real sort check and not a
    // check that happens to accept anything.
    let mut reversed = record.clone();
    reversed.supersedes.reverse();
    assert!(
        validate_supersedes_edges(&reversed).is_err(),
        "the strict sort must still reject an out-of-order list"
    );
}

/// A predecessor created after its successor is refused even when its name sorts first.
///
/// Exercised through `validate_supersedes_semantics`, not through the new helper, so it runs on
/// both binaries and actually discriminates. The old guard compared `succession_change_key`,
/// which sees only the ID: a predecessor named `CHG-0001-...` satisfied it regardless of when
/// either change was created. Ordering by `created_at` is what makes "supersedes" mean
/// "came after" instead of "has a smaller name".
#[test]
fn a_predecessor_created_after_its_successor_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let base = completed_no_spec_record(root);

    // Sorts FIRST by name, created LAST in time.
    let mut predecessor = ChangeRecord {
        id: "CHG-0001-aaa-sorts-first".into(),
        created_at: 2_000,
        state: ChangeState::Accepted,
        ..base.clone()
    };
    predecessor.supersedes.clear();
    save_change(root, &predecessor).unwrap();

    let successor = ChangeRecord {
        id: "CHG-0002-bbb-sorts-second".into(),
        created_at: 1_000,
        supersedes: vec![SupersedesEdge {
            predecessor_id: predecessor.id.clone(),
            obligations: vec![SuccessionObligation {
                path: "src/lib.rs".into(),
                module: "change".into(),
                predecessor_entry_digest: sha256_hex(b"entry"),
            }],
        }],
        ..base.clone()
    };

    let error = validate_supersedes_semantics(root, &successor)
        .expect_err("a predecessor created after its successor must be refused");
    assert!(
        error.contains("created before successor"),
        "must refuse on creation order, not on some later gate: {error}"
    );
}

/// Vacuity control: the ordinary case is accepted past the ordering guard on both binaries.
///
/// Same fixture with the timestamps the right way round. It must NOT fail on the ordering
/// guard — proving the change did not simply make succession refuse everything.
#[test]
fn a_predecessor_created_before_its_successor_passes_the_ordering_guard() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let base = completed_no_spec_record(root);

    let mut predecessor = ChangeRecord {
        id: "CHG-0001-aaa-sorts-first".into(),
        created_at: 1_000,
        state: ChangeState::Accepted,
        ..base.clone()
    };
    predecessor.supersedes.clear();
    save_change(root, &predecessor).unwrap();

    let successor = ChangeRecord {
        id: "CHG-0002-bbb-sorts-second".into(),
        created_at: 2_000,
        supersedes: vec![SupersedesEdge {
            predecessor_id: predecessor.id.clone(),
            obligations: vec![SuccessionObligation {
                path: "src/lib.rs".into(),
                module: "change".into(),
                predecessor_entry_digest: sha256_hex(b"entry"),
            }],
        }],
        ..base.clone()
    };

    // It may still fail further along on manifest evidence — that is a different gate and not
    // what this pins. It must not fail on ORDERING.
    if let Err(error) = validate_supersedes_semantics(root, &successor) {
        assert!(
            !error.contains("created before successor") && !error.contains("must sort before"),
            "the ordinary direction must clear the ordering guard: {error}"
        );
    }
}

/// Equal timestamps must still yield a strict total order, because callers enforce strict sorts.
#[test]
fn changes_created_in_the_same_second_are_still_strictly_ordered() {
    let temp = TempDir::new().unwrap();
    let base = completed_no_spec_record(temp.path());
    let first = ChangeRecord {
        id: "CHG-0001-alpha".into(),
        created_at: 1_000,
        ..base.clone()
    };
    let second = ChangeRecord {
        id: "CHG-0002-beta".into(),
        created_at: 1_000,
        ..base.clone()
    };
    assert!(happens_after(&second, &first));
    assert!(!happens_after(&first, &second));
    assert!(
        !happens_after(&first, &first),
        "the relation is irreflexive"
    );
}

#[test]
fn explicit_semantic_successor_covers_changed_entry_but_rejects_unchanged_entry() {
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
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
    let mut predecessor = completed_section_only_record(root, delta);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let predecessor_manifest = load_verification(root, &predecessor)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let mut successor = completed_section_only_record(root, delta);
    successor.affected_paths.extend(
        predecessor_manifest
            .entries
            .iter()
            .filter(|entry| {
                entry.owners.iter().any(|owner| owner == "auth")
                    && entry.path != "specs/auth/requirements.md"
            })
            .map(|entry| entry.path.clone()),
    );
    successor.affected_paths.sort();
    successor.affected_paths.dedup();
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for entry in predecessor_manifest.entries.iter().filter(|entry| {
        entry.owners.iter().any(|owner| owner == "auth")
            && entry.path != "specs/auth/requirements.md"
    }) {
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
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    successor = start_implementation(root, &successor.id).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
    verify_change(root, &successor.id).unwrap();
    successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept semantic successor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let predecessor_evidence = summarize_change(root, &predecessor)
        .terminal_evidence
        .unwrap();
    assert_eq!(
        predecessor_evidence.validity,
        TerminalEvidenceValidity::SuccessorCovered,
        "{:?}",
        predecessor_evidence.reason
    );
    let archived_successor = archive_change(root, &successor.id).unwrap();
    assert!(archived_successor.is_dir());
    let recursive_evidence = summarize_change(root, &predecessor)
        .terminal_evidence
        .unwrap();
    assert_eq!(
        recursive_evidence.validity,
        TerminalEvidenceValidity::SuccessorCovered,
        "{:?}",
        recursive_evidence.reason
    );
    let post_move = check_project(root);
    assert!(post_move.errors.is_empty(), "{:?}", post_move.errors);
    assert!(post_move.terminal_evidence.iter().any(|result| {
        result.id == successor.id
            && result.evidence.validity == TerminalEvidenceValidity::AuthenticatedHistory
    }));

    let successor_manifest = load_verification(root, &successor)
        .unwrap()
        .acceptance_manifest
        .unwrap();
    let current_source = fs::read(root.join("src/auth.rs")).unwrap();
    let current_spec = fs::read(root.join("specs/auth/auth.spec.md")).unwrap();
    let mut unchanged = completed_section_only_record(root, delta);
    fs::write(root.join("src/auth.rs"), current_source).unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), current_spec).unwrap();
    unchanged.affected_paths.extend(
        successor_manifest
            .entries
            .iter()
            .filter(|entry| {
                entry.owners.iter().any(|owner| owner == "auth")
                    && entry.path != "specs/auth/requirements.md"
            })
            .map(|entry| entry.path.clone()),
    );
    unchanged.affected_paths.sort();
    unchanged.affected_paths.dedup();
    save_change(root, &unchanged).unwrap();
    write_change_markdown(root, &unchanged).unwrap();
    for entry in successor_manifest.entries.iter().filter(|entry| {
        entry.owners.iter().any(|owner| owner == "auth")
            && entry.path != "specs/auth/requirements.md"
    }) {
        unchanged = add_supersedes_obligation(
            root,
            &unchanged.id,
            &successor.id,
            &entry.path,
            "auth",
            &entry.entry_digest,
        )
        .unwrap();
    }
    unchanged = approve_definition(root, &unchanged.id, Some("Reviewer".into()), None).unwrap();
    start_implementation(root, &unchanged.id).unwrap();
    verify_change(root, &unchanged.id).unwrap();
    let error = accept_change(root, &unchanged.id, Some("Closer".into()), None).unwrap_err();
    assert!(
        error.contains("does not change the predecessor entry"),
        "{error}"
    );
    fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
    verify_change(root, &unchanged.id).unwrap();
    let recursive_successor =
        accept_change(root, &unchanged.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept recursive semantic successor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    assert_eq!(recursive_successor.state, ChangeState::Accepted);
    let intermediate_evidence = summarize_change(root, &successor)
        .terminal_evidence
        .unwrap();
    assert_eq!(
        intermediate_evidence.validity,
        TerminalEvidenceValidity::SuccessorCovered,
        "{:?}",
        intermediate_evidence.reason
    );
    let final_evidence = summarize_change(root, &predecessor)
        .terminal_evidence
        .unwrap();
    assert_eq!(
        final_evidence.validity,
        TerminalEvidenceValidity::SuccessorCovered,
        "{:?}",
        final_evidence.reason
    );
    let recursive_report = check_project(root);
    assert!(
        recursive_report.errors.is_empty(),
        "{:?}",
        recursive_report.errors
    );
}

#[test]
fn legacy_reconstruction_deduplicates_identical_transitions_but_rejects_distinct_evidence() {
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
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let delta = "## MODIFIED\n### SPEC SECTION Invariants\n\nLegacy evidence remains governed.\n";
    let mut record = completed_section_only_record(root, delta);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    let signed_legacy_digest = acceptance_input_digest(root, &record, &[]).unwrap();
    let mut verification = load_verification(root, &record).unwrap();
    verification.acceptance_input_digest = Some(signed_legacy_digest.clone());
    verification.acceptance_manifest = None;
    verification.semantic_succession = None;
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &verification,
    )
    .unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger
        .approvals
        .iter_mut()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .unwrap()
        .digest = closing_digest(&record, &verification);
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record legacy acceptance"]);

    let lifecycle_dir = change_dir(root, &record.id);
    let accepted_state = fs::read(lifecycle_dir.join("state.json")).unwrap();
    let accepted_markdown = fs::read(lifecycle_dir.join("change.md")).unwrap();
    let accepted_verification = fs::read(lifecycle_dir.join("verification.json")).unwrap();
    let accepted_approvals = fs::read(lifecycle_dir.join("approvals.json")).unwrap();
    let commit_transition = |message: &str, distinct_evidence: bool| {
        let mut verifying = record.clone();
        verifying.state = ChangeState::Verifying;
        save_change(root, &verifying).unwrap();
        write_change_markdown(root, &verifying).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", &format!("prepare {message}")]);
        fs::write(lifecycle_dir.join("state.json"), &accepted_state).unwrap();
        fs::write(lifecycle_dir.join("change.md"), &accepted_markdown).unwrap();
        if distinct_evidence {
            let mut distinct_verification = verification.clone();
            distinct_verification.timestamp += 1;
            write_json(
                &lifecycle_dir.join("verification.json"),
                &distinct_verification,
            )
            .unwrap();
            let mut distinct_ledger = ledger.clone();
            distinct_ledger
                .approvals
                .iter_mut()
                .rev()
                .find(|approval| approval.gate == "acceptance")
                .unwrap()
                .digest = closing_digest(&record, &distinct_verification);
            write_json(&lifecycle_dir.join("approvals.json"), &distinct_ledger).unwrap();
        } else {
            fs::write(
                lifecycle_dir.join("verification.json"),
                &accepted_verification,
            )
            .unwrap();
            fs::write(lifecycle_dir.join("approvals.json"), &accepted_approvals).unwrap();
        }
        git(&["add", "."]);
        git(&["commit", "-m", message]);
    };

    commit_transition("repeat identical legacy acceptance", false);
    let identical_result =
        reconstruct_legacy_acceptance_manifest(root, &record, &signed_legacy_digest);
    assert!(
        identical_result.is_ok(),
        "identical legacy reconstruction must succeed"
    );

    // Product #511: scratch worktree remove failure must not discard Ok(result).
    FORCE_LEGACY_WORKTREE_REMOVE_FAILURE.set(true);
    let cleanup_failure_result =
        reconstruct_legacy_acceptance_manifest(root, &record, &signed_legacy_digest);
    FORCE_LEGACY_WORKTREE_REMOVE_FAILURE.set(false);
    // Do not Debug-format the Result (CodeQL rust/cleartext-logging / alert #59):
    // success embeds acceptance digests from trusted correction history.
    assert!(
        cleanup_failure_result.is_ok(),
        "successful reconstruction must survive worktree cleanup failure"
    );

    commit_transition("repeat distinct legacy acceptance", true);
    let error =
        reconstruct_legacy_acceptance_manifest(root, &record, &signed_legacy_digest).unwrap_err();
    assert!(
        error.contains("found 2"),
        "distinct legacy reconstructions must remain ambiguous"
    );
}

#[test]
fn change_ids_are_the_description_slugified_and_a_taken_name_is_refused() {
    let temp = TempDir::new().unwrap();
    let first = create_change(
        temp.path(),
        CreateChangeRequest {
            description: "Add passkeys".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec![],
            affected_paths: vec![],
            requested_artifacts: vec![],
            no_spec_change: true,
            rationale: Some("test".into()),
        },
    )
    .unwrap();
    let second = create_change(
        temp.path(),
        CreateChangeRequest {
            description: "Fix login".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec![],
            affected_paths: vec![],
            requested_artifacts: vec![],
            no_spec_change: true,
            rationale: Some("test".into()),
        },
    )
    .unwrap();
    assert_eq!(first.id, "add-passkeys");
    assert_eq!(second.id, "fix-login");
    // The ordinal was what made two identical descriptions produce two IDs. Without it a
    // repeated description is a repeated identity, and the refusal names the existing change
    // rather than minting a near-twin the author would have to tell apart by a trailing digit.
    let taken = create_change(
        temp.path(),
        CreateChangeRequest {
            description: "Add passkeys".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec![],
            affected_paths: vec![],
            requested_artifacts: vec![],
            no_spec_change: true,
            rationale: Some("test".into()),
        },
    )
    .unwrap_err();
    assert!(
        taken.contains("a change named `add-passkeys` already exists"),
        "got: {taken}"
    );
    assert!(
        taken.contains(".specsync/changes/add-passkeys"),
        "got: {taken}"
    );
    // A description with nothing to slugify would otherwise land every non-Latin description
    // in a repository on one shared fallback ID, of which only the first could ever be created.
    let unslugifiable = create_change(
        temp.path(),
        CreateChangeRequest {
            description: "缓存必须在写入前失效".into(),
            kind: ChangeKind::Feature,
            affected_specs: vec![],
            affected_paths: vec![],
            requested_artifacts: vec![],
            no_spec_change: true,
            rationale: Some("test".into()),
        },
    )
    .unwrap_err();
    assert!(
        unslugifiable.contains("no ASCII letters or digits"),
        "got: {unslugifiable}"
    );
}

#[test]
fn concurrent_change_creation_assigns_unique_ids() {
    let temp = TempDir::new().unwrap();
    let root = std::sync::Arc::new(temp.path().to_path_buf());
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let root = std::sync::Arc::clone(&root);
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                create_change(
                    &root,
                    CreateChangeRequest {
                        description: format!("Concurrent change {index}"),
                        kind: ChangeKind::Operations,
                        affected_specs: Vec::new(),
                        affected_paths: vec![format!("ops/{index}/")],
                        requested_artifacts: Vec::new(),
                        no_spec_change: true,
                        rationale: Some("Concurrency fixture".into()),
                    },
                )
                .unwrap()
                .id
            })
        })
        .collect();
    let ids: BTreeSet<String> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(ids.len(), 8);
    let roster = list_changes(&root).unwrap();
    assert!(!roster.is_degraded(), "unreadable: {:?}", roster.unreadable);
    assert_eq!(roster.records.len(), 8);
}

/// One unreadable workspace must not erase its healthy siblings (#443).
///
/// The old roster was `list_changes_checked().unwrap_or_default()`: the first bad
/// `state.json` aborted enumeration, and the resulting `Err` became an empty vec.
/// Both halves are asserted here — the healthy record survives (enumeration no
/// longer aborts) and the bad one is named (the failure is no longer discarded).
#[test]
fn unreadable_workspace_is_reported_beside_its_healthy_siblings() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let healthy = create_change(
        root,
        CreateChangeRequest {
            description: "Healthy sibling".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/healthy/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("443 fixture".into()),
        },
    )
    .unwrap()
    .id;
    let broken = create_change(
        root,
        CreateChangeRequest {
            description: "Broken sibling".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/broken/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("443 fixture".into()),
        },
    )
    .unwrap()
    .id;

    // Control: both are readable before the corruption, so a later empty roster
    // cannot be blamed on the fixture never having created them.
    let before = list_changes(root).unwrap();
    assert!(!before.is_degraded());
    assert_eq!(before.records.len(), 2);

    fs::write(change_dir(root, &broken).join("state.json"), "{ not json").unwrap();

    let roster = list_changes(root).unwrap();
    assert_eq!(
        roster
            .records
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        vec![healthy.as_str()],
        "the healthy sibling must survive an unreadable neighbour"
    );
    assert!(roster.is_degraded());
    assert_eq!(roster.unreadable.len(), 1);
    assert_eq!(roster.unreadable[0].id, broken);
    assert!(
        roster.unreadable[0].reason.contains("state.json"),
        "the reason must name the offending file, got: {}",
        roster.unreadable[0].reason
    );

    // Vacuity control: removing the corruption returns a clean, complete roster.
    // Without this, a change that reported every workspace unreadable would pass
    // every assertion above.
    fs::remove_dir_all(change_dir(root, &broken)).unwrap();
    let restored = list_changes(root).unwrap();
    assert!(!restored.is_degraded());
    assert_eq!(restored.records.len(), 1);
}

#[test]
fn sequence_ledger_rejects_unacknowledged_active_and_archived_collisions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let first = create_change(
        root,
        CreateChangeRequest {
            description: "First claim".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/first".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();
    // Ordinals are no longer minted, so a fixture that needs one builds it the way the
    // archive carries one — the machinery under test exists only to read such records.
    let first = reidentify_as_ordinal(root, &first, "CHG-0001-first-claim");
    let mut archived = first.clone();
    archived.id = "CHG-0001-archived-claim".into();
    archived.slug = "archived-claim".into();
    archived.state = ChangeState::Archived;
    let archived_dir = root
        .join(ARCHIVE_PATH)
        .join("2026-07-13-CHG-0001-archived-claim");
    fs::create_dir_all(&archived_dir).unwrap();
    write_json(&archived_dir.join("state.json"), &archived).unwrap();

    let error = validate_change_sequences(root).unwrap_err();
    assert!(error.contains("duplicate numeric change sequence CHG-0001"));
    assert!(error.contains(&first.id));
    assert!(error.contains(&archived.id));
    assert!(error.contains(".specsync/changes"));
    assert!(error.contains(".specsync/archive/changes"));
}

#[test]
fn exact_historical_collision_baseline_preserves_immutable_records() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let first = create_change(
        root,
        CreateChangeRequest {
            description: "First claim".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/first".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();
    let mut first = reidentify_as_ordinal(root, &first, "CHG-0001-first-claim");
    first.state = ChangeState::Accepted;
    save_change(root, &first).unwrap();
    let mut second = first.clone();
    second.id = "CHG-0001-second-claim".into();
    second.slug = "second-claim".into();
    second.state = ChangeState::Archived;
    let archived_dir = root
        .join(ARCHIVE_PATH)
        .join("2026-07-14-CHG-0001-second-claim");
    fs::create_dir_all(&archived_dir).unwrap();
    write_json(&archived_dir.join("state.json"), &second).unwrap();
    let mut ids = vec![first.id.clone(), second.id.clone()];
    ids.sort();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: first.id.clone(),
            acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
        },
    )
    .unwrap();

    assert!(validate_change_sequences(root).is_ok());
    let ledger = load_change_sequence_ledger(root).unwrap().unwrap();
    assert_eq!(ledger.id, first.id);

    fs::remove_dir_all(archived_dir).unwrap();
    let error = validate_change_sequences(root).unwrap_err();
    assert!(error.contains("no longer matches the exact historical ID set"));
    assert!(error.contains(&second.id));
}

#[test]
fn acknowledged_collision_rejects_mutable_active_records() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let first = create_change(
        root,
        CreateChangeRequest {
            description: "First mutable claim".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/first".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("fixture".into()),
        },
    )
    .unwrap();
    let first = reidentify_as_ordinal(root, &first, "CHG-0001-first-mutable-claim");
    let mut second = first.clone();
    second.id = "CHG-0001-second-mutable-claim".into();
    second.slug = "second-mutable-claim".into();
    fs::create_dir_all(change_dir(root, &second.id)).unwrap();
    save_change(root, &second).unwrap();
    let mut ids = vec![first.id.clone(), second.id.clone()];
    ids.sort();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: first.id.clone(),
            acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
        },
    )
    .unwrap();

    let error = validate_change_sequences(root).unwrap_err();
    assert!(error.contains("includes a mutable change"));
    let summary = summarize_change(root, &first);
    assert!(
        summary
            .next_action
            .contains("remove the premature acknowledgement")
            || summary
                .next_action
                .contains("accept or archive every member"),
        "frozen ledger must surface freeze remediation as next_action, got: {}",
        summary.next_action
    );
}

// Verifies REQ-change-035.
#[test]
fn acknowledged_collision_allows_only_valid_audited_reopen_history() {
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
    let mut record = completed_no_spec_record(root);
    record = reidentify_as_ordinal(root, &record, "CHG-0001-harden-verification");
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept first collision member"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let mut duplicate = record.clone();
    duplicate.id = "CHG-0001-second-accepted-claim".into();
    duplicate.slug = "second-accepted-claim".into();
    duplicate.title = "Second accepted claim".into();
    save_change(root, &duplicate).unwrap();
    let mut ids = vec![record.id.clone(), duplicate.id];
    ids.sort();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 1,
            id: record.id.clone(),
            acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
        },
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();

    let reopened = reopen_change(
        root,
        &record.id,
        "Reviewer".into(),
        "Integrated delivery input changed".into(),
    )
    .unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert!(validate_change_sequences(root).is_ok());

    let mut approvals = load_approvals(root, &reopened.change).unwrap();
    approvals
        .reopenings
        .last_mut()
        .unwrap()
        .prior_verification
        .passed = false;
    write_json(
        &change_dir(root, &reopened.change.id).join("approvals.json"),
        &approvals,
    )
    .unwrap();
    let error = validate_change_sequences(root).unwrap_err();
    assert!(error.contains("includes a mutable change"), "{error}");
}

#[test]
fn change_sequences_allow_more_than_four_digits() {
    assert_eq!(change_sequence("CHG-9999-last-four-digit"), Some(9999));
    assert_eq!(change_sequence("CHG-10000-first-five-digit"), Some(10000));
    assert_eq!(change_sequence("CHG-123-too-short"), None);
    assert_eq!(change_sequence("CHG-abcd-malformed"), None);
    assert_eq!(change_sequence("CHG-09999-noncanonical-width"), None);
    assert_eq!(change_sequence("CHG-18446744073709551616-overflow"), None);
    // `change_id_sorts_after` used to be asserted here. It was `#[cfg(test)]`, and its only
    // caller was another `#[cfg(test)]` helper: a fossil of the ordinal successor ordering that
    // CHG-0160 replaced with `happens_after` on `(created_at, id)`. Parsing width still has to
    // fail closed, which the `change_sequence` assertions above cover.
    assert_eq!(
        located_change_ordinal("CHG-9999-last-four-digit").unwrap(),
        Some(9999)
    );
    assert_eq!(located_change_ordinal("a-slug-only-change").unwrap(), None);
    assert_eq!(located_change_ordinal("CHG-abcd-malformed").unwrap(), None);
    assert!(located_change_ordinal("CHG-09999-noncanonical-width").is_err());
    assert!(located_change_ordinal("CHG-123-too-short").is_err());
}

#[test]
fn lifecycle_lock_releases_when_owner_drops() {
    let temp = TempDir::new().unwrap();
    let first = acquire_project_lock(temp.path()).unwrap();
    drop(first);
    let second = acquire_project_lock(temp.path()).unwrap();
    drop(second);
}

#[test]
fn archive_waits_until_delivery_diff_no_longer_needs_coverage() {
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
    git(&["config", "core.autocrlf", "false"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "base"]);
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.require_change_for_meaningful_files = true;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    let error = archive_change(root, &record.id).unwrap_err();
    assert!(error.contains("archive after merge"));
    git(&["add", "."]);
    git(&["commit", "-m", "record accepted lifecycle evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    archive_change(root, &record.id).unwrap();
}

// Verifies REQ-change-018.
#[test]
fn accepted_evidence_survives_integrated_squash_merge_and_archives() {
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
    let verification = load_verification(root, &record).unwrap();

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    assert!(!verification_commit_is_accepted_current(
        root,
        &verification
    ));
    assert!(ensure_closing_approval_valid(root, &record).is_ok());

    git(&["switch", "-c", "followup"]);
    git(&["commit", "--allow-empty", "-m", "followup"]);
    assert!(!accepted_workspace_is_integrated(root, &record));
    assert!(accepted_change_is_recorded_in_current_history(
        root, &record
    ));
    assert!(accepted_change_is_recorded_on_remote_default(root, &record));
    assert!(ensure_closing_approval_valid(root, &record).is_ok());

    git(&["switch", "main"]);
    archive_change(root, &record.id).unwrap();
}

#[test]
fn refreshed_accepted_evidence_squash_merged_while_accepted_archives() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
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

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    // Refresh the accepted evidence on a second branch: the input drift makes the
    // accepted evidence stale, and the audited reopen produces a new verification and
    // closing approval while the change is already recorded as accepted on main.
    git(&["switch", "-c", "refresh"]);
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { true }\n// drifted after acceptance\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "drift input"]);
    reopen_change(
        root,
        &record.id,
        "Reviewer".into(),
        "The governed source drifted after acceptance".into(),
    )
    .unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "reaccept"]);
    let verification = load_verification(root, &record).unwrap();

    git(&["switch", "main"]);
    git(&["merge", "--squash", "refresh"]);
    git(&["commit", "-m", "squash refresh"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    // The re-verification commit stayed on the discarded branch and the first squash
    // commit already records the change as accepted, so no first-acceptance transition
    // anchor carries the current evidence; the recording-anchor fallback must.
    assert!(!verification_commit_is_accepted_current(
        root,
        &verification
    ));
    let archived = archive_change(root, &record.id).unwrap();
    assert!(archived.join("accepted-state.json").exists());
}

#[test]
fn squash_merged_recording_anchor_fails_closed_without_matching_evidence() {
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

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    // Tamper with the current closing-evidence bytes so no in-history accepted record
    // matches them; archival must still fail closed. The note text is not bound by the
    // closing digest, so every earlier guard still passes and the trusted-transition
    // preflight is what must reject the archive.
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    let tampered = fs::read_to_string(&approvals_path).unwrap().replacen(
        "\"note\": null",
        "\"note\": \"tampered\"",
        1,
    );
    assert_ne!(tampered, fs::read_to_string(&approvals_path).unwrap());
    fs::write(&approvals_path, tampered).unwrap();
    let error = archive_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("requires exactly one trusted transition"),
        "{error}"
    );
}

#[test]
fn accepted_evidence_survives_squash_merge_from_nested_project_root() {
    let temp = TempDir::new().unwrap();
    let repo_root = temp.path();
    let root = repo_root.join("packages/app");
    fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repo_root)
                .status()
                .unwrap()
                .success()
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
    write_lifecycle_test_policy(&root);
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["switch", "-c", "feature"]);

    let mut record = completed_no_spec_record(&root);
    record = approve_definition(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(&root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    verify_change(&root, &record.id).unwrap();
    record = accept_change(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept"]);
    let verification = load_verification(&root, &record).unwrap();

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    assert!(!verification_commit_is_accepted_current(
        &root,
        &verification
    ));
    assert!(ensure_closing_approval_valid(&root, &record).is_ok());
    assert!(accepted_change_is_recorded_in_current_history(
        &root, &record
    ));
    // CONTROL (#673): while the evidence is still anchored — the remote default records the
    // squash — reopen must REFUSE even though the verification commit itself is unreachable.
    // This is the vacuity control for the widening below: without it, a fix that made reopen
    // admit everything would pass just as happily.
    let anchored_refusal = reopen_change(
        &root,
        &record.id,
        "Reviewer".into(),
        "The verification commit is off history".into(),
    )
    .expect_err("anchored evidence with current inputs must still refuse reopen");
    assert!(
        anchored_refusal.contains("still anchored in current history"),
        "{anchored_refusal}"
    );

    // Now drop the anchor. Delivery inputs are BYTE-IDENTICAL throughout; the only thing that
    // changed is that no reachable history records the acceptance any more.
    git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    assert!(ensure_closing_approval_valid(&root, &record).is_err());

    // #673: this asserted `unwrap_err()` with "delivery inputs are current" until 6.0. That
    // pinned the dead end the field report hit: `check` refused because the commit was
    // unreachable while `reopen` refused because the inputs were current, both true at once and
    // no verb in between. Amended invariants 15/18 of specs/change/change.spec.md make commit
    // reachability a staleness axis in its own right, so the recovery verb now fires.
    let reopened = reopen_change(
        &root,
        &record.id,
        "Reviewer".into(),
        "The verification commit is off history".into(),
    )
    .expect("an unanchored verification commit must be reopenable");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.stale_acceptance_input_digest,
        reopened.audit.current_acceptance_input_digest,
        "the inputs never drifted — the anchor is what went stale"
    );
    assert_eq!(
        reopened.audit.stale_evidence_cause,
        Some(ReopenCauseV1::VerificationCommitUnanchored),
        "the ledger must record WHY, because a sibling validator reads digest equality as proof"
    );
}

#[test]
fn squash_merged_acceptance_reopens_after_a_current_canonical_successor() {
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
    git(&["switch", "-c", "feature"]);

    let mut original = completed_no_spec_record(root);
    fs::write(
        root.join("src/auth.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nAuthentication remains governed.\n\n## Behavioral Examples\n\nChecks pass.\n\n## Error Cases\n\nInvalid evidence fails.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    original.affected_specs = vec!["auth".into()];
    original.affected_paths = vec!["src/auth.rs".into()];
    save_change(root, &original).unwrap();
    write_change_markdown(root, &original).unwrap();
    original = approve_definition(root, &original.id, Some("Reviewer".into()), None).unwrap();
    original = start_implementation(root, &original.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement original"]);
    verify_change(root, &original.id).unwrap();
    original = accept_change(root, &original.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept original"]);
    let original_verification = load_verification(root, &original).unwrap();

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash original"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    adopt(root, false, None).unwrap();

    let mut no_spec_successor = completed_no_spec_current_record(root);
    no_spec_successor.affected_specs = original.affected_specs.clone();
    no_spec_successor.affected_paths = original.affected_paths.clone();
    save_change(root, &no_spec_successor).unwrap();
    write_change_markdown(root, &no_spec_successor).unwrap();
    no_spec_successor =
        approve_definition(root, &no_spec_successor.id, Some("Reviewer".into()), None).unwrap();
    no_spec_successor = start_implementation(root, &no_spec_successor.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement no-spec successor"]);
    verify_change(root, &no_spec_successor.id).unwrap();
    no_spec_successor =
        accept_change(root, &no_spec_successor.id, Some("Closer".into()), None).unwrap();
    assert_eq!(no_spec_successor.state, ChangeState::Accepted);
    git(&["add", "."]);
    git(&["commit", "-m", "accept no-spec successor"]);

    assert!(!accepted_change_has_current_canonical_successors(
        root, &original
    ));

    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nA later semantic change governs authentication.\n\nAcceptance Criteria\n- Authentication remains governed by the successor.\n";
    let mut successor = completed_section_only_current_record(root, delta);
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    successor = start_implementation(root, &successor.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement semantic successor"]);
    verify_change(root, &successor.id).unwrap();
    successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept semantic successor"]);

    assert!(!verification_commit_is_accepted_current(
        root,
        &original_verification
    ));
    assert!(!accepted_workspace_is_integrated(root, &original));
    assert!(accepted_change_is_recorded_in_current_history(
        root, &original
    ));
    assert!(accepted_change_has_current_canonical_successors(
        root, &original
    ));
    assert_ne!(
        acceptance_input_digest(root, &original, &[]).unwrap(),
        original_verification
            .acceptance_input_digest
            .clone()
            .unwrap()
    );
    assert!(ensure_closing_approval_valid(root, &original).is_err());
    assert_eq!(
        summarize_change(root, &original).next_action,
        format!(
            "run `specsync change reopen {} --actor <name> --reason <reason>`",
            original.id
        )
    );

    let reopened = reopen_change(
        root,
        &original.id,
        "Release reviewer".into(),
        "A later accepted change superseded the original governed source".into(),
    )
    .unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.reason,
        "A later accepted change superseded the original governed source"
    );
    assert_eq!(
        reopened.audit.prior_verification.contract_digest,
        original_verification.contract_digest
    );
    assert_eq!(
        reopened.audit.prior_verification.acceptance_input_digest,
        original_verification.acceptance_input_digest
    );
    assert_eq!(successor.state, ChangeState::Accepted);
}

#[test]
fn squash_fallback_rejects_unintegrated_or_changed_evidence() {
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
    git(&["commit", "--allow-empty", "-m", "move head"]);
    let verification = load_verification(root, &record).unwrap();
    assert!(verification_commit_is_accepted_current(root, &verification));

    git(&["switch", "main"]);
    git(&["merge", "--squash", "feature"]);
    git(&["commit", "-m", "squash feature"]);
    assert!(ensure_closing_approval_valid(root, &record).is_err());
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { changed }\n",
    )
    .unwrap();
    let error = ensure_closing_approval_valid(root, &record).unwrap_err();
    assert!(
        error.contains("delivery inputs")
            || error.contains(
                "changed after acceptance and no accepted or archived successor change covers it"
            ),
        "{error}"
    );
}

#[test]
fn failed_archive_move_leaves_an_accepted_change_retryable() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        definition_digest(root, &record).unwrap(),
        None,
    )
    .unwrap();
    record.state = ChangeState::Accepted;
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    let verification = VerificationRecord {
        timestamp: now(),
        commit: None,
        contract_digest: definition_digest(root, &record).unwrap(),
        execution_digest: None,
        workspace_digest: project_input_digest(root).unwrap(),
        acceptance_input_digest: Some(acceptance_input_digest(root, &record, &[]).unwrap()),
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
        &record,
        "acceptance",
        Some("Reviewer".into()),
        closing_digest(&record, &verification),
        None,
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
    git(&["commit", "-m", "record accepted evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{}", today(), record.id));
    fs::create_dir_all(&destination).unwrap();

    let error = archive_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("archive destination already exists"),
        "{error}"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
    fs::remove_dir_all(destination).unwrap();
    if let Err(error) = archive_change(root, &record.id) {
        panic!("archive retry failed: {error}");
    }
}

#[test]
fn semantic_delta_requires_shall_and_criteria() {
    let valid = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL support passkeys.\n\nAcceptance Criteria\n- Works\n";
    let items = parse_delta(valid).unwrap();
    assert_eq!(items.len(), 1);
    validate_requirement(&items[0].key, &items[0].content).unwrap();
    let invalid = "## ADDED\n### REQUIREMENT REQ-auth-001\nSupport passkeys.\n";
    let item = parse_delta(invalid).unwrap().remove(0);
    assert!(validate_requirement(&item.key, &item.content).is_err());
}

#[test]
fn unknown_delta_operation_heading_is_rejected() {
    let typo = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n\n## REMVOED\n### REQUIREMENT REQ-auth-002\nRetired.\n";
    let error = parse_delta(typo).unwrap_err();
    assert!(error.contains("invalid delta operation heading"));
    assert!(
        error.contains("## Added") && error.contains("## Modified") && error.contains("## Removed"),
        "invalid operation heading must name the allowed values: {error}"
    );
}

#[test]
fn populated_delta_without_operation_headings_is_not_empty() {
    let error = parse_delta("# greeter\n\nAdds greeting format docs.\n").unwrap_err();
    assert!(
        error.contains("no recognized operation headings"),
        "populated unrecognized file must not pretend to be empty: {error}"
    );
    assert!(
        !error.contains("is empty"),
        "populated unrecognized file must not say is empty: {error}"
    );
    assert!(
        error.contains("## Added") && error.contains("## Modified") && error.contains("## Removed"),
        "must name allowed operation headings: {error}"
    );
}

#[test]
fn whitespace_delta_parses_as_empty() {
    assert!(parse_delta("").unwrap().is_empty());
    assert!(parse_delta("   \n\n\t\n").unwrap().is_empty());
}

#[test]
fn recognized_operation_without_items_is_not_empty() {
    let error = parse_delta("## ADDED\n").unwrap_err();
    assert!(
        error.contains("no items under a recognized operation heading"),
        "a heading with no items must not pretend to be an empty file: {error}"
    );
    assert!(
        !error.contains("is empty"),
        "a heading with no items must not say is empty: {error}"
    );
    assert!(
        error.contains("### REQUIREMENT") && error.contains("### SPEC SECTION"),
        "must name the required item forms: {error}"
    );
}

#[test]
fn item_headings_are_accepted_case_insensitively() {
    let requirement = parse_delta(
        "## added\n### requirement REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n",
    )
    .unwrap();
    assert_eq!(requirement.len(), 1);
    assert_eq!(requirement[0].target, DeltaTarget::Requirement);
    assert_eq!(requirement[0].key, "REQ-auth-001");

    let section =
        parse_delta("## Modified\n### spec section Public API\n| `login` | Login |\n").unwrap();
    assert_eq!(section.len(), 1);
    assert_eq!(section[0].target, DeltaTarget::SpecSection);
    assert_eq!(section[0].key, "Public API");
}

#[test]
fn non_item_subheading_inside_an_item_remains_content() {
    let items = parse_delta(
        "## MODIFIED\n### SPEC SECTION Public API\n### Structs & Enums\n| `login` | Login |\n",
    )
    .unwrap();
    assert!(
        items
            .iter()
            .any(|item| item.content.contains("### Structs & Enums")),
        "scaffold subheadings must stay item content, not an error: {items:?}"
    );
}

#[test]
fn lowercase_item_heading_inside_a_body_opens_a_new_item() {
    let items = parse_delta(
        "## MODIFIED\n### SPEC SECTION Public API\nintro\n### requirement REQ-auth-002\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n",
    )
    .unwrap();
    assert_eq!(items.len(), 2, "{items:?}");
    assert_eq!(items[0].target, DeltaTarget::SpecSection);
    assert_eq!(items[0].key, "Public API");
    assert_eq!(items[1].target, DeltaTarget::Requirement);
    assert_eq!(items[1].key, "REQ-auth-002");
}

#[test]
fn live_populated_delta_without_headings_is_not_empty() {
    let temp = TempDir::new().unwrap();
    let record = completed_record(temp.path());
    fs::write(
        delta_path(temp.path(), &record, "auth"),
        "# auth\n\nAdds docs.\n",
    )
    .unwrap();
    let error = validate_delta_files(temp.path(), &record).unwrap_err();
    assert!(
        error.contains("no recognized operation headings"),
        "{error}"
    );
    assert!(!error.contains("is empty"), "{error}");
}

#[test]
fn live_empty_delta_still_reports_empty() {
    let temp = TempDir::new().unwrap();
    let record = completed_record(temp.path());
    fs::write(delta_path(temp.path(), &record, "auth"), "\n").unwrap();
    let error = validate_delta_files(temp.path(), &record).unwrap_err();
    assert!(
        error.contains("semantic delta for `auth` is empty"),
        "{error}"
    );
}

#[test]
fn extra_delta_modules_are_rejected() {
    let temp = TempDir::new().unwrap();
    let record = completed_record(temp.path());
    let valid = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n";
    fs::write(delta_path(temp.path(), &record, "auth"), valid).unwrap();
    fs::write(
        change_dir(temp.path(), &record.id).join("deltas/billing.md"),
        valid.replace("REQ-auth", "REQ-billing"),
    )
    .unwrap();
    let error = validate_delta_files(temp.path(), &record).unwrap_err();
    assert!(error.contains("extra: billing"), "{error}");
}

#[test]
fn spec_versions_preserve_integer_or_semantic_format() {
    assert!(
        bump_spec_version("---\nversion: 4\n---\n")
            .unwrap()
            .contains("version: 5")
    );
    assert!(
        bump_spec_version("---\nversion: 1.2.9\n---\n")
            .unwrap()
            .contains("version: 1.2.10")
    );
    assert!(
        bump_spec_version("---\nversion: 1.2.9 # release\n---\n")
            .unwrap()
            .contains("version: 1.2.10 # release")
    );
    assert!(
        bump_spec_version("---\nversion: \"1.2.9\"\n---\n")
            .unwrap()
            .contains("version: \"1.2.10\"")
    );
    assert!(bump_spec_version("---\nversion: one\n---\n").is_err());
}

#[test]
fn stale_definition_approval_is_rejected() {
    let temp = TempDir::new().unwrap();
    ensure_auth_spec_owns_its_source(temp.path());
    let mut record = completed_record(temp.path());
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(temp.path(), &record.id).join(artifact.file_name()),
            prose_or_plan(artifact, "# Complete\n\nReviewed.\n"),
        )
        .unwrap();
    }
    fs::write(delta_path(temp.path(), &record, "auth"), "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n").unwrap();
    record = approve_definition(temp.path(), &record.id, Some("Reviewer".into()), None).unwrap();
    assert!(ensure_definition_approval_valid(temp.path(), &record).is_ok());
    fs::write(
        change_dir(temp.path(), &record.id).join("context.md"),
        "changed\n",
    )
    .unwrap();
    assert!(ensure_definition_approval_valid(temp.path(), &record).is_err());
    record = approve_definition(
        temp.path(),
        &record.id,
        Some("Reviewer".into()),
        Some("Reapproved updated context".into()),
    )
    .unwrap();
    assert_eq!(record.state, ChangeState::Approved);
    assert!(ensure_definition_approval_valid(temp.path(), &record).is_ok());
}

#[test]
fn current_workflow_scope_approval_survives_execution_detail_changes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let approved_digest = definition_digest(root, &record).unwrap();
    let record = start_implementation(root, &record.id).unwrap();
    let verification = verify_change(root, &record.id).unwrap();
    assert_eq!(verification.contract_digest, approved_digest);
    let approved_execution = verification.execution_digest.clone().unwrap();

    fs::write(
        change_dir(root, &record.id).join("context.md"),
        "# Context\n\nImplementation evidence was refined after approval.\n",
    )
    .unwrap();

    assert_eq!(definition_digest(root, &record).unwrap(), approved_digest);
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
    assert_ne!(execution_digest(root, &record).unwrap(), approved_execution);
    assert!(!verification_is_current(root, &record, &verification));
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!("run `specsync change check {}`", record.id)
    );
}

#[test]
fn current_workflow_scope_changes_require_renewal_with_plain_diff() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let mut expanded = record.clone();
    expanded.affected_specs.clear();
    expanded.affected_paths.clear();
    expanded.acceptance_criteria.clear();
    expanded.affected_paths.push("src/new-surface.rs".into());
    expanded
        .acceptance_criteria
        .push("A newly exposed surface is supported".into());
    save_change(root, &expanded).unwrap();
    write_change_markdown(root, &expanded).unwrap();

    let error = ensure_definition_approval_valid(root, &expanded).unwrap_err();
    assert!(
        error.contains("affected paths added: src/new-surface.rs"),
        "{error}"
    );
    assert!(
        error.contains("affected paths removed:") && error.contains("src/"),
        "{error}"
    );
    assert!(
        error.contains("acceptance criteria added: A newly exposed surface is supported"),
        "{error}"
    );
    assert!(
        error.contains("acceptance criteria removed: Verification is fresh"),
        "{error}"
    );
    assert!(
        error.contains("affected canonical specs removed: change"),
        "{error}"
    );
    let summary = summarize_change(root, &expanded);
    assert!(!summary.approval_valid);
    assert_eq!(summary.scope_expansion.len(), 5);
    assert!(summary.next_action.contains("change approve"));
}

#[test]
fn current_workflow_delta_materialization_preserves_scope_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = current_workflow_record(root, completed_record(root));
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
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL authenticate users with a passkey.\n\nAcceptance Criteria\n- A passkey authenticates the user.\n",
        )
        .unwrap();
    record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let scope = definition_digest(root, &record).unwrap();
    let execution = execution_digest(root, &record).unwrap();

    fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL authenticate users with a passkey.\n\nAcceptance Criteria\n- A registered passkey authenticates the user.\n- A rejected passkey does not create a session.\n",
        )
        .unwrap();

    assert_eq!(definition_digest(root, &record).unwrap(), scope);
    assert_ne!(execution_digest(root, &record).unwrap(), execution);
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn unallowlisted_scope_migration_cannot_rewrite_an_approval_projection() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let scope = approved_scope(root, &record).unwrap();
    let scope_digest = scope_digest_from_approved(&scope).unwrap();
    let source_definition_digest = execution_digest(root, &record).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.approvals.push(ApprovalRecord {
        gate: "definition".into(),
        actor: "Scope owner".into(),
        timestamp: now(),
        digest: source_definition_digest.clone(),
        note: None,
        definition_pair: None,
        approved_scope: Some(scope),
        scope_migration: Some(ScopeApprovalMigrationV1 {
            schema_version: 1,
            source_definition_digest,
            scope_digest,
            changes: vec![NonMaterialScopeChangeV1 {
                path: format!("{CHANGES_PATH}/{}/testing.md", record.id),
                category: NonMaterialScopeChangeCategory::TestEvidence,
                summary: "Refined automated evidence after scope approval.".into(),
            }],
        }),
        approved_delta_digests: None,
    });
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    let error = ensure_definition_approval_valid(root, &record).unwrap_err();
    assert!(
        error.contains("incompatible direct and adopted projections"),
        "{error}"
    );
}

#[test]
fn scope_adoption_fails_closed_when_anchor_is_unavailable_or_replayed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    assert!(
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.id = CHG_0068_ID.into();
    let adopted_scope: ApprovedScopeV1 = serde_json::from_str(include_str!(concat!(
        "../../tests/fixtures/chg-0068-adopted-scope.json"
    )))
    .unwrap();
    assert_eq!(
        scope_digest_from_approved(&adopted_scope).unwrap(),
        CHG_0068_ADOPTED_SCOPE_DIGEST
    );
    let change_root = format!("{CHANGES_PATH}/{CHG_0068_ID}");
    let changes = [
            (
                "state.json",
                NonMaterialScopeChangeCategory::LifecycleMetadata,
                "Recorded workflow-version and implementation-state metadata.",
            ),
            (
                "change.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Regenerated the human-readable change projection from the already-approved intent and scope.",
            ),
            (
                "deltas/change.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved one-workflow, finalization, bounded-validation, and scoped-review contract.",
            ),
            (
                "deltas/cli.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved strict-validator and no-external-merge CLI behavior.",
            ),
            (
                "deltas/cmd_change.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved guided status and finalize command behavior.",
            ),
            (
                "deltas/cmd_check.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved schema and warning-reporting reliability behavior.",
            ),
            (
                "deltas/github.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved same-PR archive and lightweight required-CI behavior.",
            ),
            (
                "deltas/validator.md",
                NonMaterialScopeChangeCategory::CanonicalMaterialization,
                "Materialized the approved non-vacuous schema validation behavior.",
            ),
            (
                "testing.md",
                NonMaterialScopeChangeCategory::TestEvidence,
                "Expanded targeted regression and final-gate evidence for the approved contract.",
            ),
            (
                "tasks.md",
                NonMaterialScopeChangeCategory::Implementation,
                "Recorded implementation progress without changing task scope.",
            ),
        ]
        .into_iter()
        .map(|(path, category, summary)| NonMaterialScopeChangeV1 {
            path: format!("{change_root}/{path}"),
            category,
            summary: summary.into(),
        })
        .collect::<Vec<_>>();
    let ledger = ApprovalLedger {
        approvals: vec![ApprovalRecord {
            gate: "definition".into(),
            actor: "0xLeif".into(),
            timestamp: 1_785_369_606,
            digest: CHG_0068_LEGACY_APPROVAL_DIGEST.into(),
            note: None,
            definition_pair: None,
            approved_scope: None,
            scope_migration: None,
            approved_delta_digests: None,
        }],
        scope_adoptions: vec![ScopeAdoptionV1 {
            schema_version: 1,
            change_id: CHG_0068_ID.into(),
            source_approval_index: 0,
            legacy_approval_digest: CHG_0068_LEGACY_APPROVAL_DIGEST.into(),
            source_preimage_status: ScopeAdoptionSourcePreimageStatus::Unavailable,
            equivalence_claim: ScopeAdoptionEquivalenceClaim::None,
            adopted_scope,
            adopted_scope_digest: CHG_0068_ADOPTED_SCOPE_DIGEST.into(),
            anchor: ScopeAdoptionAnchorV1 {
                base_commit: CHG_0068_ADOPTION_BASE_COMMIT.into(),
                commit: CHG_0068_ADOPTION_ANCHOR_COMMIT.into(),
                approval_index: 0,
                approvals_blob_sha256: CHG_0068_ADOPTION_ANCHOR_BLOB.into(),
            },
            authorization: ScopeAdoptionAuthorizationV1 {
                actor: "0xLeif".into(),
                recorded_at: 1_785_381_022,
                reason: CHG_0068_ADOPTION_REASON.into(),
            },
            changes,
        }],
        reopenings: Vec::new(),
    };
    let error =
        validate_scope_adoption(root, &record, &ledger, 0, &ledger.approvals[0]).unwrap_err();
    assert!(error.contains("anchor is unavailable"), "{error}");
}

#[test]
fn renewed_direct_scope_approval_supersedes_legacy_adoption() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let scope = approved_scope(root, &record).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.scope_adoptions.push(ScopeAdoptionV1 {
        schema_version: 1,
        change_id: record.id.clone(),
        source_approval_index: 0,
        legacy_approval_digest: "a".repeat(64),
        source_preimage_status: ScopeAdoptionSourcePreimageStatus::Unavailable,
        equivalence_claim: ScopeAdoptionEquivalenceClaim::None,
        adopted_scope: scope.clone(),
        adopted_scope_digest: scope_digest_from_approved(&scope).unwrap(),
        anchor: ScopeAdoptionAnchorV1 {
            base_commit: "b".repeat(40),
            commit: "c".repeat(40),
            approval_index: 0,
            approvals_blob_sha256: "d".repeat(64),
        },
        authorization: ScopeAdoptionAuthorizationV1 {
            actor: "Scope owner".into(),
            recorded_at: now(),
            reason: "Test one-time adoption".into(),
        },
        changes: vec![NonMaterialScopeChangeV1 {
            path: format!("{CHANGES_PATH}/{}/context.md", record.id),
            category: NonMaterialScopeChangeCategory::LifecycleMetadata,
            summary: "Test adoption".into(),
        }],
    });
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();

    let renewed = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let ledger = load_approvals(root, &renewed).unwrap();
    assert!(ledger.scope_adoptions.is_empty());
    assert!(
        ledger
            .approvals
            .last()
            .and_then(|approval| approval.approved_scope.as_ref())
            .is_some()
    );
    assert!(ensure_definition_approval_valid(root, &renewed).is_ok());
}

fn portable_definition_record(root: &Path) -> ChangeRecord {
    let mut record = completed_no_spec_record(root);
    persist_legacy_test_record(root, &mut record);
    record.legacy_archive_baseline_digest = Some("a".repeat(64));
    save_change(root, &record).unwrap();
    record
}

#[test]
fn marked_portable_definition_pair_is_atomic_current_and_fail_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    fs::write(&approvals_path, "{\n  \"approvals\": []\n}\n").unwrap();

    append_portable_definition_approval_v501(
        root,
        &record,
        Some("user:0xLeif".into()),
        Some("Approve portable definition".into()),
    )
    .unwrap();
    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&approvals_path).unwrap()).unwrap();
    assert!(persisted.get("reopenings").is_none());
    let ledger = load_approvals(root, &record).unwrap();
    assert_eq!(ledger.approvals.len(), 2);
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
    let effective = effective_definition_approval(root, &record, &ledger).unwrap();
    assert_eq!(
        effective.definition_pair.as_ref().unwrap().role,
        DefinitionApprovalPairRole::Current
    );
    let (current, legacy, prefix) = portable_definition_digest_pair_v501(root, &record).unwrap();
    assert_eq!(effective.digest, current);
    assert_eq!(ledger.approvals[1].digest, legacy);
    assert_ne!(current, legacy);
    assert_eq!(ledger.approvals[0].actor, ledger.approvals[1].actor);
    assert_eq!(ledger.approvals[0].timestamp, ledger.approvals[1].timestamp);

    let assert_invalid = |mut candidate: ApprovalLedger| {
        assert!(
            resolve_definition_approval_event(
                &record,
                &candidate,
                &current,
                Some(&legacy),
                &prefix,
            )
            .is_err()
        );
        candidate.approvals.clear();
    };

    let mut wrong_actor = ledger.clone();
    wrong_actor.approvals[1].actor = "another-reviewer".into();
    assert_invalid(wrong_actor);
    let mut wrong_timestamp = ledger.clone();
    wrong_timestamp.approvals[1].timestamp += 1;
    assert_invalid(wrong_timestamp);
    let mut reversed = ledger.clone();
    reversed.approvals.swap(0, 1);
    assert_invalid(reversed);
    let mut wrong_digest = ledger.clone();
    wrong_digest.approvals[1].digest = "b".repeat(64);
    assert_invalid(wrong_digest);
    let mut same_digest = ledger.clone();
    same_digest.approvals[1].digest = same_digest.approvals[0].digest.clone();
    same_digest.approvals[1]
        .definition_pair
        .as_mut()
        .unwrap()
        .legacy_digest = same_digest.approvals[0].digest.clone();
    assert_invalid(same_digest);
    let mut intervening = ledger.clone();
    intervening.approvals.insert(
        1,
        ApprovalRecord {
            gate: "acceptance".into(),
            actor: "Closer".into(),
            timestamp: intervening.approvals[0].timestamp,
            digest: "c".repeat(64),
            note: None,
            definition_pair: None,
            approved_scope: None,
            scope_migration: None,
            approved_delta_digests: None,
        },
    );
    assert_invalid(intervening);
    for field in [
        "pair",
        "change",
        "correction",
        "prefix",
        "event",
        "projection",
    ] {
        let mut malformed = ledger.clone();
        let metadata = malformed.approvals[1].definition_pair.as_mut().unwrap();
        match field {
            "pair" => metadata.pair_id = "d".repeat(64),
            "change" => metadata.change_id = "CHG-9999-other".into(),
            "correction" => metadata.correction_count = 1,
            "prefix" => metadata.correction_prefix_digest = "e".repeat(64),
            "event" => metadata.event_index = 42,
            "projection" => metadata.projection = "specsync-5.0.0".into(),
            _ => unreachable!(),
        }
        assert_invalid(malformed);
    }
    let mut replayed = ledger.clone();
    replayed.approvals.extend(ledger.approvals.clone());
    assert_invalid(replayed);
    let mut loose = ledger.clone();
    for approval in &mut loose.approvals {
        approval.definition_pair = None;
    }
    assert_invalid(loose);

    let mut followed_by_closing = ledger.clone();
    followed_by_closing.approvals.push(ApprovalRecord {
        gate: "acceptance".into(),
        actor: "Closer".into(),
        timestamp: ledger.approvals[1].timestamp + 1,
        digest: "f".repeat(64),
        note: None,
        definition_pair: None,
        approved_scope: None,
        scope_migration: None,
        approved_delta_digests: None,
    });
    assert!(
        resolve_definition_approval_event(
            &record,
            &followed_by_closing,
            &current,
            Some(&legacy),
            &prefix,
        )
        .is_ok()
    );
    let mut superseded_by_ordinary = ledger.clone();
    superseded_by_ordinary.approvals[0]
        .definition_pair
        .as_mut()
        .unwrap()
        .pair_id = "0".repeat(64);
    superseded_by_ordinary.approvals.push(ApprovalRecord {
        gate: "definition".into(),
        actor: "Later reviewer".into(),
        timestamp: ledger.approvals[1].timestamp + 1,
        digest: current.clone(),
        note: None,
        definition_pair: None,
        approved_scope: None,
        scope_migration: None,
        approved_delta_digests: None,
    });
    assert!(
        resolve_definition_approval_event(
            &record,
            &superseded_by_ordinary,
            &current,
            None,
            &prefix,
        )
        .is_ok()
    );

    fs::write(
        change_dir(root, &record.id).join("context.md"),
        "# Changed definition\n",
    )
    .unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_err());
}

#[test]
fn portable_definition_pair_accepts_historical_exact_task_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
    append_portable_definition_approval_v501(root, &record, Some("Scope owner".into()), None)
        .unwrap();

    let (current, legacy, prefix) =
        portable_definition_digest_pair_v501_with_task_mode(root, &record, false).unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    let event_index = 0;
    let actor = ledger.approvals[0].actor.clone();
    let timestamp = ledger.approvals[0].timestamp;
    let pair_id = definition_approval_pair_id(
        &record,
        event_index,
        &actor,
        timestamp,
        &prefix,
        &current,
        &legacy,
    );
    for (index, approval) in ledger.approvals.iter_mut().enumerate() {
        approval.digest = if index == 0 {
            current.clone()
        } else {
            legacy.clone()
        };
        let metadata = approval.definition_pair.as_mut().unwrap();
        metadata.pair_id = pair_id.clone();
        metadata.current_digest = current.clone();
        metadata.legacy_digest = legacy.clone();
    }
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();

    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn portable_projection_rejects_unsupported_nonempty_v501_fields() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
    assert!(portable_definition_digest_pair_v501(root, &record).is_ok());

    let mut canonical = record.clone();
    canonical.canonical_applied = true;
    assert!(portable_definition_digest_pair_v501(root, &canonical).is_err());
    let mut corrected = record.clone();
    corrected.correction_count = 1;
    assert!(portable_definition_digest_pair_v501(root, &corrected).is_err());
    let mut successor = record;
    successor.supersedes.push(SupersedesEdge {
        predecessor_id: "CHG-0000-predecessor".into(),
        obligations: Vec::new(),
    });
    assert!(portable_definition_digest_pair_v501(root, &successor).is_err());
}

#[test]
fn v501_record_projection_is_frozen_across_new_field_representations() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
    let expected = definition_projection_bytes_v501(&record).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&expected).unwrap();
    for omitted in [
        "canonical_applied",
        "correction_count",
        "supersedes",
        "legacy_archive_baseline_digest",
    ] {
        assert!(value.get(omitted).is_none(), "{omitted}");
    }

    let mut archived = record.clone();
    archived.state = ChangeState::Archived;
    archived.updated_at += 100;
    archived.canonical_applied = true;
    archived.correction_count = 2;
    archived.supersedes.push(SupersedesEdge {
        predecessor_id: "CHG-0000-predecessor".into(),
        obligations: Vec::new(),
    });
    archived.legacy_archive_baseline_digest = Some("b".repeat(64));
    assert_eq!(
        definition_projection_bytes_v501(&archived).unwrap(),
        expected
    );

    let mut explicit_false = record;
    explicit_false.canonical_applied = false;
    explicit_false.correction_count = 0;
    explicit_false.supersedes.clear();
    explicit_false.legacy_archive_baseline_digest = None;
    assert_eq!(
        definition_projection_bytes_v501(&explicit_false).unwrap(),
        expected
    );
}

#[test]
fn portable_projection_rejects_clean_crlf_smudging_before_ledger_mutation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = portable_definition_record(root);
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
    git(&["config", "core.autocrlf", "true"]);
    git(&["add", "."]);
    git(&["commit", "-m", "canonical definition"]);

    let context = change_dir(root, &record.id).join("context.md");
    let lf = fs::read_to_string(&context).unwrap();
    fs::write(&context, lf.replace('\n', "\r\n")).unwrap();
    assert!(
        Command::new("git")
            .args(["diff", "--quiet", "--", context.to_str().unwrap()])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    let before = fs::read(&approvals_path).unwrap();
    let error =
        append_portable_definition_approval_v501(root, &record, Some("Reviewer".into()), None)
            .unwrap_err();
    assert!(error.contains("context.md"), "{error}");
    assert!(error.contains("canonical LF release checkout"), "{error}");
    assert_eq!(fs::read(approvals_path).unwrap(), before);
}

#[test]
fn strict_check_requires_definition_approval_only_in_gated_active_states() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    assert!(
        !check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("definition approval"))
    );

    record.state = ChangeState::Approved;
    save_change(root, &record).unwrap();
    for state in [
        ChangeState::Approved,
        ChangeState::Implementing,
        ChangeState::Verifying,
    ] {
        record.state = state;
        save_change(root, &record).unwrap();
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&record.id) && error.contains("definition approval is missing")
        }));
    }
    record.state = ChangeState::Draft;
    save_change(root, &record).unwrap();
    assert!(
        !check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("definition approval"))
    );
}

#[test]
fn false_canonical_application_preserves_legacy_definition_approvals() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    let state_path = change_dir(root, &record.id).join("state.json");

    let draft_json = fs::read_to_string(&state_path).unwrap();
    assert!(!draft_json.contains("canonical_applied"));
    let stable_digest = definition_digest(root, &record).unwrap();
    let explicit_false_digest = definition_digest_with_explicit_false(root, &record).unwrap();
    assert_ne!(stable_digest, explicit_false_digest);
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        explicit_false_digest,
        Some("Approved with the transitional explicit-false encoding".into()),
    )
    .unwrap();
    record.state = ChangeState::Approved;
    save_change(root, &record).unwrap();

    let approved_json = fs::read_to_string(&state_path).unwrap();
    assert!(!approved_json.contains("canonical_applied"));
    let loaded = load_change(root, &record.id).unwrap();
    assert!(!loaded.canonical_applied);
    assert!(ensure_definition_approval_valid(root, &loaded).is_ok());

    record.canonical_applied = true;
    save_change(root, &record).unwrap();
    let accepted_json = fs::read_to_string(state_path).unwrap();
    assert!(accepted_json.contains("\"canonical_applied\": true"));
}

#[test]
fn acceptance_normalizes_transitional_definition_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    let stable_digest = definition_digest(root, &record).unwrap();
    let transitional_digest = definition_digest_with_explicit_false(root, &record).unwrap();
    append_approval(
        root,
        &record,
        "definition",
        Some("Original reviewer".into()),
        transitional_digest.clone(),
        Some("Approved with transitional evidence".into()),
    )
    .unwrap();
    record.state = ChangeState::Approved;
    save_change(root, &record).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();

    record = accept_change(
        root,
        &record.id,
        Some("Release reviewer".into()),
        Some("Accepted after verification".into()),
    )
    .unwrap();

    let ledger = load_approvals(root, &record).unwrap();
    assert_eq!(ledger.approvals.len(), 3);
    assert_eq!(ledger.approvals[0].digest, transitional_digest);
    assert_eq!(ledger.approvals[1].gate, "definition");
    assert_eq!(ledger.approvals[1].actor, "Release reviewer");
    assert_eq!(ledger.approvals[1].digest, stable_digest);
    assert_eq!(
        ledger.approvals[1].note.as_deref(),
        Some("Normalized compatible definition evidence during explicit acceptance")
    );
    assert_eq!(ledger.approvals[2].gate, "acceptance");
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn reaccept_accepts_transitional_pre_reopen_definition_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    let transitional_digest = definition_digest_with_explicit_false(root, &record).unwrap();
    let mut prior_verification = load_verification(root, &record).unwrap();
    prior_verification.contract_digest = transitional_digest.clone();
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &prior_verification,
    )
    .unwrap();
    let mut ledger = load_approvals(root, &record).unwrap();
    let closing = ledger
        .approvals
        .iter_mut()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .unwrap();
    closing.digest = closing_digest(&record, &prior_verification);
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_ok());

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Delivery input changed after legacy acceptance".into(),
    )
    .unwrap()
    .change;
    assert_eq!(
        load_approvals(root, &record).unwrap().reopenings[0]
            .prior_verification
            .contract_digest,
        transitional_digest
    );

    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(record.canonical_applied);
}

#[test]
fn markdown_blocks_apply_semantically() {
    let source = "# Requirements\n\n### REQ-auth-001\n\nOld\n\n### REQ-auth-002\n\nKeep\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(modified.contains("### REQ-auth-001\n\nNew"));
    assert!(modified.contains("### REQ-auth-002\n\nKeep"));
    let removed = apply_markdown_block(
        &modified,
        "### ",
        "REQ-auth-001",
        "",
        DeltaOperation::Removed,
    )
    .unwrap();
    assert!(!removed.contains("REQ-auth-001"));
}

#[test]
fn markdown_block_stops_at_higher_level_heading() {
    let source = "# Requirements\n\n## Durable requirements\n\n### REQ-auth-001\n\nOld text.\n\n## Public API\n\n| Name |\n|---|\n| `authenticate` |\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New text.",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(modified.contains("### REQ-auth-001\n\nNew text."));
    assert!(modified.contains("## Public API\n\n| Name |\n|---|\n| `authenticate` |"));
    let removed =
        apply_markdown_block(source, "### ", "REQ-auth-001", "", DeltaOperation::Removed).unwrap();
    assert!(!removed.contains("REQ-auth-001"));
    assert!(removed.contains("## Public API\n\n| Name |\n|---|\n| `authenticate` |"));
}

#[test]
fn markdown_block_preserves_crlf_and_unrelated_bytes() {
    let source = "# Requirements\r\n\r\n### REQ-auth-001\r\n\r\nOld.\r\n\r\n## Public API  \r\n\r\nKeep trailing spaces.  \r\n";
    let modified = apply_markdown_block(
        source,
        "### ",
        "REQ-auth-001",
        "New.",
        DeltaOperation::Modified,
    )
    .unwrap();
    assert!(!modified.replace("\r\n", "").contains('\n'));
    assert!(modified.ends_with("## Public API  \r\n\r\nKeep trailing spaces.  \r\n"));
}

#[test]
fn malformed_policy_fails_closed() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".specsync")).unwrap();
    fs::write(temp.path().join(POLICY_PATH), "{ invalid json").unwrap();
    let report = check_project(temp.path());
    assert!(report.enabled);
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].contains("invalid SDD policy"));
}

#[test]
fn malformed_active_change_state_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let dir = root.join(CHANGES_PATH).join("CHG-0001-corrupt");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("state.json"), "{ invalid json").unwrap();

    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("invalid active change state"))
    );
}

#[test]
fn oversized_change_artifacts_are_rejected() {
    let temp = TempDir::new().unwrap();
    let record = completed_no_spec_record(temp.path());
    fs::write(
        change_dir(temp.path(), &record.id).join("context.md"),
        vec![b'x'; MAX_CHANGE_ARTIFACT_BYTES as usize + 1],
    )
    .unwrap();
    let error = validate_artifacts(temp.path(), &record).unwrap_err();
    assert!(error.contains("exceeds") && error.contains("byte limit"));
}

#[test]
fn artifact_content_rejects_hash_todo_headings_and_html_todos() {
    assert!(artifact_content_is_incomplete(""));
    assert!(artifact_content_is_incomplete("   \n  "));
    assert!(artifact_content_is_incomplete("# TODO\n"));
    assert!(artifact_content_is_incomplete("## TODO: fill me\n"));
    assert!(artifact_content_is_incomplete("TODO\n"));
    assert!(artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: context\n---\n\n# TODO\n"
    ));
    assert!(artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: context\n---\n\n# Context\n\n<!-- TODO: fill -->\n"
    ));
    // Checkbox-only TODO body (no real section prose) is incomplete.
    assert!(artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: tasks\n---\n\n- [ ] TODO\n"
    ));
    // A real section heading with only TODO checkbox under it still has a non-placeholder
    // heading line — that is allowed; HTML TODO / pure `# TODO` bodies are the filed gap.
    assert!(!artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: context\n---\n\n# Context\n\nReal prose for the change.\n"
    ));
    assert!(!artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: tasks\n---\n\n# Tasks\n\n- [x] Ship the fix\n"
    ));
}

// `artifact_content_is_incomplete` used its own stripper, `strip_yaml_frontmatter`, which was a
// CONTENT DELETER: it searched the WHOLE document for `\n---\n` before trying `\r\n---\r\n`, so a
// CRLF artifact carrying one LF horizontal rule lost everything above that rule. What survived
// here is a bare `TODO`, so a fully written design was refused as "artifact is incomplete" and
// the author had no way to see why. Deleted in favour of `parser::strip_frontmatter` (#696).
//
// Discriminates: under `strip_yaml_frontmatter` this asserts `false` on a `true`.
#[test]
fn a_crlf_artifact_with_an_lf_body_rule_is_complete_when_its_prose_is_written() {
    let content = "---\r\nchange: CHG-1\r\nartifact: design\r\n---\r\n\r\n\
# Design\r\n\r\nThe retry budget is per-host, not per-request.\r\n\n---\n\r\nTODO\r\n";

    assert!(
        !artifact_content_is_incomplete(content),
        "written prose above a body horizontal rule must not be deleted by the stripper"
    );
}

// The same replacement closes the opposite failure: `strip_yaml_frontmatter` only ever matched a
// closing delimiter followed by a newline, so frontmatter closed at EOF was not stripped at all
// and its own `---` / `change:` lines read as body prose. An artifact with NO content passed the
// completeness gate.
//
// Discriminates: under `strip_yaml_frontmatter` this asserts `true` on a `false`.
#[test]
fn an_artifact_that_is_only_frontmatter_closed_at_eof_is_incomplete() {
    assert!(artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: design\n---"
    ));
}

// Honest label: this is the CONTROL for the two above. The LF shapes the old stripper handled
// correctly must keep their verdicts — this is a swap of implementations, not of policy.
#[test]
fn artifact_completeness_verdicts_are_unchanged_for_lf_artifacts() {
    assert!(artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: design\n---\n\nTODO\n"
    ));
    assert!(!artifact_content_is_incomplete(
        "---\nchange: CHG-1\nartifact: design\n---\n\nReal prose.\n\n---\n\nMore prose.\n"
    ));
}

// Honest label: DISCRIMINATOR for #716. The approval gate this feeds is the whole reason the
// stripper's delimiter rule matters: a trailing space on the OPENING delimiter meant the block
// was never stripped, the `change:` and `artifact:` lines counted as prose, and an artifact with
// nothing written in it was approved as complete.
//
// Discriminates: on the unfixed binary this asserts `true` on a `false` — the artifact passes.
#[test]
fn an_artifact_that_is_only_frontmatter_with_a_padded_opening_delimiter_is_incomplete() {
    assert!(artifact_content_is_incomplete(
        "---  \nchange: CHG-1\nartifact: design\n---\n"
    ));
    assert!(artifact_content_is_incomplete(
        "---  \nchange: CHG-1\nartifact: design\n---\n\nTODO\n"
    ));
}
