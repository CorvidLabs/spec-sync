use super::*;
use std::sync::{Arc, Barrier, mpsc};
use tempfile::TempDir;

const SUBSTANTIVE_PLAN: &str = "# Plan\n\n## Approach\n\nDo the work.\n\n## Out of scope\n\nLeave unrelated work alone.\n\n## Steps\n\nLand the change.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n";

fn prose_or_plan<'a>(artifact: &ArtifactKind, prose: &'a str) -> &'a str {
    if matches!(artifact, ArtifactKind::Plan) {
        SUBSTANTIVE_PLAN
    } else {
        prose
    }
}

// Verifies REQ-change-021.
#[test]
fn append_changelog_preserves_version_date_changes_schema() {
    let spec = "---\nmodule: canary\nversion: 3\n---\n\n## Change Log\n\n| Version | Date | Changes |\n|---------|------|---------|\n| 2 | 2026-07-13 | Previous |\n";

    let updated = append_changelog(spec, "CHG-0003", "Correct the change log");

    assert!(updated.contains(&format!(
        "| 3 | {} | CHG-0003: Correct the change log |",
        today()
    )));
}

#[test]
fn append_changelog_populates_date_author_change_schema() {
    let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Author | Change |\n|------|--------|--------|\n";

    let updated = append_changelog(spec, "CHG-0002", "Document behavior");

    assert!(updated.contains(&format!(
        "| {} | SpecSync | CHG-0002: Document behavior |",
        today()
    )));
}

#[test]
fn append_changelog_keeps_default_two_column_schema() {
    let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n";

    let updated = append_changelog(spec, "CHG-0002", "Document behavior");

    assert!(updated.contains(&format!("| {} | CHG-0002: Document behavior |", today())));
}

#[test]
fn append_changelog_does_not_treat_similar_headers_as_change_columns() {
    let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Changer | Change |\n|------|---------|--------|\n";

    let updated = append_changelog(spec, "CHG-0002", "Document behavior");

    assert!(updated.contains(&format!("| {} |  | CHG-0002: Document behavior |", today())));
}

#[test]
fn effective_contract_workspaces_are_unique() {
    const WORKERS: usize = 32;
    let barrier = Arc::new(Barrier::new(WORKERS));
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..WORKERS {
            let barrier = Arc::clone(&barrier);
            let sender = sender.clone();
            scope.spawn(move || {
                barrier.wait();
                sender
                    .send(create_effective_contract_workspace().unwrap())
                    .unwrap();
            });
        }
    });
    drop(sender);
    let paths: BTreeSet<PathBuf> = receiver.into_iter().collect();
    assert_eq!(paths.len(), WORKERS);
    for path in paths {
        fs::remove_dir(path).unwrap();
    }
}

/// Lifecycle tests need SDD on. Fresh `init` still writes it off via `write_default_policy`.
fn write_lifecycle_test_policy(root: &Path) {
    let path = root.join(POLICY_PATH);
    if path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut policy = SddPolicy::default();
    policy.enabled = true;
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["true".into()];
    write_json(&path, &policy).unwrap();
}

fn ensure_test_verification_policy(root: &Path) {
    if !root.join(POLICY_PATH).exists() {
        write_lifecycle_test_policy(root);
        return;
    }
    let mut policy = load_policy(root).unwrap();
    if policy.verification_commands.is_empty() {
        policy.verification_commands = vec!["true".into()];
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
    }
}

fn persist_legacy_test_record(root: &Path, record: &mut ChangeRecord) {
    record.workflow_version = 1;
    record.workflow_origin_version = None;
    save_change(root, record).unwrap();
    let baseline = root.join(WORKFLOW_V2_BASELINE_PATH);
    if baseline.exists() {
        fs::remove_file(baseline).unwrap();
    }
}

fn completed_record(root: &Path) -> ChangeRecord {
    completed_record_with_workflow(root, true)
}

fn completed_current_record(root: &Path) -> ChangeRecord {
    completed_record_with_workflow(root, false)
}

/// Make the `auth` spec claim `src/auth.rs`, as a real project would.
///
/// `completed_record` declares spec `auth` and path `src/auth.rs`, so any test
/// that approves it needs the spec to own that path — otherwise the fixture
/// describes a change that could never finalize. Applied per-test rather than
/// in the shared fixture, since tests asserting exact path-coverage sets are
/// sensitive to extra files existing.
fn ensure_auth_spec_owns_its_source(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
}

/// A description no change in `root` has already taken.
///
/// While IDs carried a `CHG-NNNN-` prefix, two fixture changes could share one description
/// and still land on distinct IDs. The ID is now the description slugified, so a fixture that
/// builds two changes has to describe them differently — which is what a human would always
/// have had to do. That six shared-fixture tests relied on the collision being papered over is
/// the corpus's own evidence for refusing duplicates rather than suffixing them.
fn distinct_fixture_description(root: &Path, base: &str) -> String {
    let mut candidate = base.to_string();
    let mut suffix = 1;
    while find_change_dir(root, &slugify(&candidate)).is_ok() {
        suffix += 1;
        candidate = format!("{base} {suffix}");
    }
    candidate
}

/// Re-identify a freshly created change under a historical `CHG-NNNN-` ID.
///
/// `create_change` no longer mints ordinals, but the sequence-ledger machinery exists purely
/// to read a corpus that is full of them, so its tests must still be able to build an
/// ordinal-bearing record. They build one the way the archive has one: a directory name and a
/// `state.json` that carry the ID. This is a rename, not a lifecycle verb — nothing in
/// production can produce an ordinal any more.
fn reidentify_as_ordinal(root: &Path, record: &ChangeRecord, id: &str) -> ChangeRecord {
    let mut renamed = record.clone();
    let dir = change_dir(root, id);
    fs::rename(change_dir(root, &record.id), &dir).unwrap();
    renamed.slug = id
        .strip_prefix("CHG-")
        .and_then(|rest| rest.split_once('-'))
        .map(|(_, slug)| slug.to_string())
        .unwrap_or_else(|| id.to_string());
    renamed.id = id.to_string();
    // Artifact and delta front matter names the change, so the rename has to reach it too.
    let mut stack = vec![dir.clone()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md")
                && let Ok(body) = fs::read_to_string(&path)
                && body.contains(&record.id)
            {
                fs::write(&path, body.replace(&record.id, id)).unwrap();
            }
        }
    }
    save_change(root, &renamed).unwrap();
    renamed
}

fn completed_record_with_workflow(root: &Path, legacy: bool) -> ChangeRecord {
    ensure_test_verification_policy(root);
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: distinct_fixture_description(root, "add passkeys"),
            kind: ChangeKind::Feature,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Users can authenticate with a passkey".into()];
    record
        .answers
        .insert("public_contract".into(), "yes".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    if legacy {
        persist_legacy_test_record(root, &mut record);
    } else {
        save_change(root, &record).unwrap();
    }
    write_change_markdown(root, &record).unwrap();
    record
}

fn completed_no_spec_record(root: &Path) -> ChangeRecord {
    completed_no_spec_record_with_workflow(root, true)
}

fn completed_no_spec_current_record(root: &Path) -> ChangeRecord {
    completed_no_spec_record_with_workflow(root, false)
}

fn completed_no_spec_record_with_workflow(root: &Path, legacy: bool) -> ChangeRecord {
    ensure_test_verification_policy(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/change")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    fs::write(
            root.join("specs/change/change.spec.md"),
            "---\nmodule: change\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Change\n\n## Purpose\n\nLifecycle fixture.\n\n## Public API\n\nNone.\n\n## Invariants\n\nVerification is deterministic.\n\n## Behavioral Examples\n\nChecks pass.\n\n## Error Cases\n\nInvalid evidence fails.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: distinct_fixture_description(root, "harden verification"),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["change".into()],
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("No public contract change".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Verification is fresh".into()];
    record.answers.insert("public_contract".into(), "no".into());
    record
        .answers
        .insert("architecture_risk".into(), "no".into());
    if legacy {
        persist_legacy_test_record(root, &mut record);
    } else {
        save_change(root, &record).unwrap();
    }
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

fn completed_section_only_record(root: &Path, delta: &str) -> ChangeRecord {
    completed_section_only_record_with_workflow(root, delta, true)
}

fn completed_section_only_current_record(root: &Path, delta: &str) -> ChangeRecord {
    completed_section_only_record_with_workflow(root, delta, false)
}

fn completed_section_only_record_with_workflow(
    root: &Path,
    delta: &str,
    legacy: bool,
) -> ChangeRecord {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nRetained for compatibility.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let record = if legacy {
        completed_record(root)
    } else {
        completed_current_record(root)
    };
    for artifact in &record.selected_artifacts {
        let content = if *artifact == ArtifactKind::Tasks {
            "# Tasks\n\n- [x] Complete the documentation change.\n"
        } else {
            prose_or_plan(artifact, "# Complete\n\nReviewed content.\n")
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    fs::write(delta_path(root, &record, "auth"), delta).unwrap();
    record
}

fn accept_completed_record(root: &Path, mut record: ChangeRecord) -> ChangeRecord {
    record =
        approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap()
}

fn current_workflow_record(root: &Path, mut record: ChangeRecord) -> ChangeRecord {
    let baseline_path = root.join(WORKFLOW_V2_BASELINE_PATH);
    if !baseline_path.exists() {
        write_json(
            &baseline_path,
            &WorkflowV2Baseline {
                schema_version: 1,
                domain: "specsync.workflow-v2-baseline.v1".into(),
                cutoff_commit: workflow_v2_baseline_cutoff(root),
            },
        )
        .unwrap();
    }
    record.workflow_version = 2;
    record.workflow_origin_version = Some(2);
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record
}

#[test]
fn new_change_serializes_immutable_workflow_origin() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "record workflow origin".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ci/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Lifecycle metadata only".into()),
        },
    )
    .unwrap();

    assert_eq!(record.workflow_version, 2);
    assert_eq!(record.workflow_origin_version, Some(2));
    let state = fs::read_to_string(change_dir(root, &record.id).join("state.json")).unwrap();
    assert!(state.contains("\"workflow_origin_version\": 2"));
}

#[test]
fn workflow_v2_without_its_origin_anchor_is_rejected() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.workflow_origin_version = None;
    save_change(root, &record).unwrap();

    let error = load_change(root, &record.id).unwrap_err();
    assert!(error.contains("missing its immutable workflow_origin_version anchor"));
}

#[test]
fn pre_anchor_workflow_v2_history_accepts_one_way_origin_backfill() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.workflow_origin_version = None;
    save_change(root, &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(
        root,
        &["commit", "-m", "record pre-anchor workflow-v2 state"],
    );

    record.workflow_origin_version = Some(2);
    save_change(root, &record).unwrap();

    assert_eq!(
        load_change(root, &record.id)
            .unwrap()
            .workflow_origin_version,
        Some(2)
    );
}

#[test]
fn workflow_v2_adoption_keeps_explicitly_anchored_workflow_v1_records_readable() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let record = create_change(
        root,
        CreateChangeRequest {
            description: "preserve anchored legacy change".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Upgrade compatibility fixture".into()),
        },
    )
    .unwrap();
    assert_eq!(record.workflow_version, 1);
    assert_eq!(record.workflow_origin_version, Some(1));
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "trusted workflow-v1 cutoff"]);
    let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    policy.version = 2;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    ensure_workflow_v2_baseline(root).unwrap();
    let baseline = read_workflow_v2_baseline(root).unwrap().unwrap().0;
    assert_eq!(baseline.cutoff_commit.as_deref(), Some(cutoff.as_str()));
    assert_eq!(load_change(root, &record.id).unwrap(), record);
}

#[test]
fn change_adopt_moves_only_new_changes_to_v2_without_rewriting_v1_policy() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let legacy = create_change(
        root,
        CreateChangeRequest {
            description: "preserve legacy workflow evidence".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/legacy/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Upgrade compatibility fixture".into()),
        },
    )
    .unwrap();
    assert_eq!(legacy.workflow_version, 1);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "trusted workflow-v1 cutoff"]);
    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let policy_before = fs::read(root.join(POLICY_PATH)).unwrap();

    adopt(root, false, None).unwrap();

    assert_eq!(fs::read(root.join(POLICY_PATH)).unwrap(), policy_before);
    let baseline = read_workflow_v2_baseline(root).unwrap().unwrap().0;
    assert_eq!(baseline.cutoff_commit.as_deref(), Some(cutoff.as_str()));
    assert_eq!(load_change(root, &legacy.id).unwrap(), legacy);
    let current = create_change(
        root,
        CreateChangeRequest {
            description: "use the single current workflow".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/current/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Workflow migration regression".into()),
        },
    )
    .unwrap();
    assert_eq!(current.workflow_version, 2);
    assert_eq!(current.workflow_origin_version, Some(2));
}

#[test]
fn change_adopt_flips_only_enabled_on_a_disabled_v1_policy() {
    // Invariant 20 promises a v1 policy stays byte-identical only while it is already
    // enabled. A v1 policy an author switched off is the one case adoption rewrites, and
    // it rewrites exactly one field.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    policy.enabled = false;
    policy.meaningful_paths.push("ops/".into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "disabled workflow-v1 policy"]);
    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);

    adopt(root, false, None).unwrap();

    let after = load_policy(root).unwrap();
    assert!(
        after.enabled,
        "adopt is the on-switch for a disabled v1 policy too"
    );
    assert_eq!(
        SddPolicy {
            enabled: false,
            ..after
        },
        policy,
        "adoption must rewrite `enabled` and nothing else"
    );
}

#[test]
fn change_adopt_rejects_uncommitted_workflow_v1_records_without_writes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "trusted base"]);
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let legacy = create_change(
        root,
        CreateChangeRequest {
            description: "leave workflow-v1 evidence uncommitted".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/uncommitted/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Upgrade compatibility fixture".into()),
        },
    )
    .unwrap();
    let policy_before = fs::read(root.join(POLICY_PATH)).unwrap();

    let error = adopt(root, false, None).unwrap_err();

    assert!(error.contains(&legacy.id));
    assert!(error.contains("absent from trusted cutoff"));
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert_eq!(fs::read(root.join(POLICY_PATH)).unwrap(), policy_before);
}

#[test]
fn change_adopt_rejects_branch_only_workflow_v1_records_without_writes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "trusted base"]);
    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let trusted_cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let legacy = create_change(
        root,
        CreateChangeRequest {
            description: "leave workflow-v1 evidence on a branch".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/branch-only/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Upgrade compatibility fixture".into()),
        },
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "branch-only workflow-v1 change"]);
    let policy_before = fs::read(root.join(POLICY_PATH)).unwrap();

    let error = adopt(root, false, None).unwrap_err();

    assert!(error.contains(&legacy.id));
    assert!(error.contains(&trusted_cutoff));
    assert!(error.contains("absent from trusted cutoff"));
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert_eq!(fs::read(root.join(POLICY_PATH)).unwrap(), policy_before);
}

#[test]
fn change_adopt_rolls_back_when_comparison_ref_moves_during_publication() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let legacy = create_change(
        root,
        CreateChangeRequest {
            description: "anchor workflow-v1 evidence before adoption".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/legacy/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Adoption race fixture".into()),
        },
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "trusted workflow-v1 record"]);
    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let trusted_head = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let policy_before = fs::read(root.join(POLICY_PATH)).unwrap();
    let hook_root = root.to_path_buf();
    inject_transaction_after_journal_hook(move || {
        quiet_git(
            &hook_root,
            &["update-ref", "refs/remotes/origin/main", "HEAD^"],
        );
    });

    let error = adopt(root, false, None).unwrap_err();

    assert!(error.contains("comparison reference changed"));
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(!root.join(TRANSACTION_PATH).exists());
    assert_eq!(fs::read(root.join(POLICY_PATH)).unwrap(), policy_before);
    assert_eq!(load_change(root, &legacy.id).unwrap(), legacy);

    quiet_git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    adopt(root, false, None).unwrap();
    let baseline = read_workflow_v2_baseline(root).unwrap().unwrap().0;
    assert_eq!(
        baseline.cutoff_commit.as_deref(),
        Some(trusted_head.as_str())
    );
}

#[test]
fn change_adopt_rejects_workflow_v1_records_without_git_history() {
    for initialize_git in [false, true] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        if initialize_git {
            quiet_git(root, &["init", "-b", "main"]);
        }
        write_lifecycle_test_policy(root);
        let mut policy = load_policy(root).unwrap();
        policy.version = 1;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let legacy = create_change(
            root,
            CreateChangeRequest {
                description: "preserve workflow-v1 evidence without history".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/no-history/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Upgrade compatibility fixture".into()),
            },
        )
        .unwrap();
        let policy_before = fs::read(root.join(POLICY_PATH)).unwrap();

        let error = adopt(root, false, None).unwrap_err();

        assert!(error.contains(&legacy.id));
        assert!(error.contains("no trusted Git cutoff"));
        assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
        assert!(!root.join(".specsync/adoption-report.json").exists());
        assert_eq!(fs::read(root.join(POLICY_PATH)).unwrap(), policy_before);
    }
}

#[test]
fn change_adopt_rolls_back_injected_publication_failure_and_retries_cleanly() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    inject_transaction_write_failure(1);

    let error = adopt(root, false, None).unwrap_err();

    assert!(error.contains("injected atomic publication failure"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(TRANSACTION_PATH).exists());

    adopt(root, false, None).unwrap();
    assert!(root.join(POLICY_PATH).is_file());
    assert!(root.join(".specsync/adoption-report.json").is_file());
    assert!(root.join(WORKFLOW_V2_BASELINE_PATH).is_file());
    assert!(!root.join(TRANSACTION_PATH).exists());
}

#[test]
fn change_adopt_recovers_interrupted_publication_before_idempotent_retry() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let original_policy = fs::read_to_string(root.join(POLICY_PATH)).unwrap();
    let entries = vec![
        TransactionEntry {
            path: POLICY_PATH.into(),
            original: Some(original_policy.clone()),
        },
        TransactionEntry {
            path: ".specsync/adoption-report.json".into(),
            original: None,
        },
        TransactionEntry {
            path: WORKFLOW_V2_BASELINE_PATH.into(),
            original: None,
        },
    ];
    let journal = TransactionJournal {
        schema_version: 1,
        entry_count: entries.len(),
        entries_digest: transaction_entries_digest(&entries).unwrap(),
        entries,
    };
    write_json(&root.join(TRANSACTION_PATH), &journal).unwrap();
    write_json(&root.join(POLICY_PATH), &default_policy(root, Vec::new())).unwrap();
    fs::write(
        root.join(".specsync/adoption-report.json"),
        "{\"partial\":true}\n",
    )
    .unwrap();
    write_json(
        &root.join(WORKFLOW_V2_BASELINE_PATH),
        &WorkflowV2Baseline {
            schema_version: 1,
            domain: "specsync.workflow-v2-baseline.v1".into(),
            cutoff_commit: None,
        },
    )
    .unwrap();

    adopt(root, false, None).unwrap();

    assert_eq!(
        fs::read_to_string(root.join(POLICY_PATH)).unwrap(),
        original_policy
    );
    assert!(!root.join(TRANSACTION_PATH).exists());
    assert!(root.join(".specsync/adoption-report.json").is_file());
    assert!(root.join(WORKFLOW_V2_BASELINE_PATH).is_file());
    adopt(root, false, None).unwrap();
    assert_eq!(
        fs::read_to_string(root.join(POLICY_PATH)).unwrap(),
        original_policy
    );
}

#[cfg(unix)]
#[test]
fn change_adopt_rejects_symlinked_import_destination_without_writes() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
    fs::create_dir_all(root.join("openspec/changes/add-passkeys")).unwrap();
    fs::write(
        root.join("openspec/specs/auth/spec.md"),
        "# Authentication\n\nCanonical contract.\n",
    )
    .unwrap();
    fs::write(
        root.join("openspec/changes/add-passkeys/proposal.md"),
        "# Add passkeys\n\nActive proposal.\n",
    )
    .unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    symlink(external.path(), root.join(".specsync/imports")).unwrap();

    let error = adopt(root, false, Some("openspec")).unwrap_err();

    assert!(error.contains("symlinked transaction target"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(TRANSACTION_PATH).exists());
    assert_eq!(external.path().read_dir().unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn change_adopt_rejects_symlinked_metadata_root_before_lock_write() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let root = temp.path();
    symlink(external.path(), root.join(".specsync")).unwrap();

    let error = adopt(root, false, None).unwrap_err();

    assert!(error.contains("symlinked lifecycle lock path"));
    assert_eq!(external.path().read_dir().unwrap().count(), 0);
    assert!(!external.path().join("change.lock").exists());
    assert!(!external.path().join("sdd.json").exists());
    assert!(!external.path().join("adoption-report.json").exists());
    assert!(!external.path().join("workflow-v2-baseline.json").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn change_adopt_rejects_non_utf8_import_target_before_transaction() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let canonical = root.join("openspec/specs/auth");
    fs::create_dir_all(&canonical).unwrap();
    let invalid_name = OsString::from_vec(b"contract-\xff.md".to_vec());
    fs::write(canonical.join(invalid_name), "# Contract\n").unwrap();

    let error = adopt(root, false, Some("openspec")).unwrap_err();

    assert!(error.contains("not valid UTF-8"));
    assert!(error.contains("cannot be journaled losslessly"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(TRANSACTION_PATH).exists());
    assert!(!root.join(".specsync/imports").exists());
}

#[cfg(unix)]
#[test]
fn non_utf8_transaction_target_is_rejected_before_filesystem_lookup() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let target = temp
        .path()
        .join(OsString::from_vec(b"transaction-\xff.json".to_vec()));

    let error = validate_prepared_transaction_target(temp.path(), &target).unwrap_err();

    assert!(error.contains("not valid UTF-8"));
    assert!(error.contains("cannot be journaled losslessly"));
    assert!(!temp.path().join(TRANSACTION_PATH).exists());
}

#[cfg(unix)]
#[test]
fn change_adopt_rejects_backslash_import_target_before_transaction() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let canonical = root.join("openspec/specs/auth");
    fs::create_dir_all(&canonical).unwrap();
    fs::write(canonical.join("contract\\part.md"), "# Contract\n").unwrap();

    let error = adopt(root, false, Some("openspec")).unwrap_err();

    assert!(error.contains("Unix filename component with `\\`"));
    assert!(error.contains("cannot be journaled losslessly"));
    assert!(!root.join(POLICY_PATH).exists());
    assert!(!root.join(".specsync/adoption-report.json").exists());
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());
    assert!(!root.join(TRANSACTION_PATH).exists());
    assert!(!root.join(".specsync/imports").exists());
}

#[test]
fn workflow_v2_baseline_rewrite_then_restore_remains_invalid() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);

    let record = current_workflow_record(root, completed_no_spec_record(root));
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "introduce workflow-v2 baseline"]);
    let introduction = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let baseline_path = root.join(WORKFLOW_V2_BASELINE_PATH);
    let original = fs::read(&baseline_path).unwrap();
    let mut rewritten: WorkflowV2Baseline = serde_json::from_slice(&original).unwrap();
    rewritten.cutoff_commit = Some(introduction);
    write_json(&baseline_path, &rewritten).unwrap();
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "rewrite workflow-v2 baseline"]);
    fs::write(&baseline_path, original).unwrap();
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "restore workflow-v2 baseline"]);

    let error = load_change(root, &record.id).unwrap_err();
    assert!(error.contains("workflow-v2 baseline changed after its introduction"));
}

#[test]
fn deleted_workflow_v2_baseline_cannot_reenable_workflow_v1_creation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let first = create_change(
        root,
        CreateChangeRequest {
            description: "introduce the current workflow".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/current/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Workflow baseline fixture".into()),
        },
    )
    .unwrap();
    assert_eq!(first.workflow_version, 2);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "introduce workflow-v2 baseline"]);
    fs::remove_file(root.join(WORKFLOW_V2_BASELINE_PATH)).unwrap();

    for commit_deletion in [false, true] {
        if commit_deletion {
            quiet_git(root, &["add", "--update"]);
            quiet_git(root, &["commit", "-m", "delete workflow-v2 baseline"]);
        }
        let error = create_change(
            root,
            CreateChangeRequest {
                description: "must not fall back to workflow v1".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/fallback/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Workflow deletion regression".into()),
            },
        )
        .unwrap_err();
        assert!(error.contains("committed workflow-v2 baseline was deleted"));
        assert!(
            !root
                .join(CHANGES_PATH)
                .join("CHG-0002-must-not-fall-back-to-workflow-v1")
                .exists()
        );
    }
}

#[test]
fn merged_second_parent_baseline_cannot_reenable_workflow_v1_creation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.version = 1;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "workflow-v1 base"]);
    let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    quiet_git(root, &["switch", "-c", "baseline-parent"]);
    write_json(
        &root.join(WORKFLOW_V2_BASELINE_PATH),
        &WorkflowV2Baseline {
            schema_version: 1,
            domain: "specsync.workflow-v2-baseline.v1".into(),
            cutoff_commit: Some(cutoff),
        },
    )
    .unwrap();
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "introduce workflow-v2 baseline"]);
    quiet_git(root, &["switch", "main"]);
    quiet_git(
        root,
        &[
            "merge",
            "--no-ff",
            "-s",
            "ours",
            "baseline-parent",
            "-m",
            "retain v1 tree while merging baseline parent",
        ],
    );
    assert!(!root.join(WORKFLOW_V2_BASELINE_PATH).exists());

    let error = create_change(
        root,
        CreateChangeRequest {
            description: "must not ignore a merged baseline parent".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/merge-topology/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Every-parent baseline regression".into()),
        },
    )
    .unwrap_err();

    assert!(error.contains("committed workflow-v2 baseline was deleted"));
    assert!(
        !root
            .join(CHANGES_PATH)
            .join("CHG-0001-must-not-ignore-a-merged-baseline-parent")
            .exists()
    );
}

#[test]
fn workflow_version_downgrade_then_revert_remains_invalid() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    let state_path = change_dir(root, &record.id).join("state.json");
    let workflow_v2 = fs::read(&state_path).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "create workflow-v2 change"]);

    let mut downgraded: serde_json::Value = serde_json::from_slice(&workflow_v2).unwrap();
    let object = downgraded.as_object_mut().unwrap();
    object.remove("workflow_version");
    object.remove("workflow_origin_version");
    fs::write(
        &state_path,
        format!("{}\n", serde_json::to_string_pretty(&downgraded).unwrap()),
    )
    .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "attempt workflow downgrade"]);
    fs::write(&state_path, workflow_v2).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "hide workflow downgrade"]);

    let error = load_change(root, &record.id).unwrap_err();
    assert!(error.contains("workflow-version history changed immutable identity"));
}

#[test]
fn workflow_version_history_follows_cross_date_rearchive_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "seed"]);

    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.state = ChangeState::Accepted;
    save_change(root, &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "record accepted workflow identity"]);

    let first_archive = root
        .join(ARCHIVE_PATH)
        .join(format!("2026-07-14-{}", record.id));
    fs::create_dir_all(first_archive.parent().unwrap()).unwrap();
    fs::rename(change_dir(root, &record.id), &first_archive).unwrap();
    record.state = ChangeState::Archived;
    write_json(&first_archive.join("state.json"), &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "archive on first date"]);

    fs::rename(&first_archive, change_dir(root, &record.id)).unwrap();
    record.state = ChangeState::Verifying;
    save_change(root, &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "reopen archived record"]);

    let second_archive = root
        .join(ARCHIVE_PATH)
        .join(format!("2026-07-15-{}", record.id));
    fs::rename(change_dir(root, &record.id), &second_archive).unwrap();
    record.state = ChangeState::Archived;
    write_json(&second_archive.join("state.json"), &record).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "rearchive on second date"]);

    let state_paths = workflow_version_state_paths(root, &record).unwrap();
    assert!(state_paths.iter().any(|path| path.contains("2026-07-14-")));
    assert!(state_paths.iter().any(|path| path.contains("2026-07-15-")));
    let loaded = load_change(root, &record.id).unwrap();
    assert_eq!(loaded.workflow_version, 2);
    assert_eq!(loaded.workflow_origin_version, Some(2));
}

// Verifies REQ-change-043, REQ-change-044, and REQ-change-046.
#[test]
fn one_approval_review_and_finalize_archive_on_the_same_branch() {
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
    let approved = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert_eq!(approved.state, ChangeState::Approved);

    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);

    let review = record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    assert_eq!(
        review.implementation_commit,
        git_output(root, &["rev-parse", "HEAD"]).unwrap()
    );
    let destination = finalize_change(root, &record.id).unwrap();
    assert!(destination.is_dir());
    assert!(!change_dir(root, &record.id).exists());

    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    let approvals = load_approvals(root, &archived).unwrap();
    assert_eq!(
        approvals
            .approvals
            .iter()
            .filter(|approval| approval.gate == "definition")
            .count(),
        1
    );
    // Same-PR finalization records a terminal finalization approval for reopen recovery.
    assert!(
        latest_terminal_approval(&approvals).is_some_and(|a| a.gate == "finalization"),
        "finalization must leave a terminal closing approval"
    );
    assert!(
        !approvals
            .approvals
            .iter()
            .any(|approval| approval.gate == "acceptance"),
        "same-PR finalization must not write a legacy acceptance gate"
    );
    let verification = load_verification(root, &archived).unwrap();
    assert!(validate_finalization_evidence(root, &archived, &verification).is_ok());
    let mut finalization = load_finalization(root, &archived).unwrap();
    finalization.closing_digest = "0".repeat(64);
    write_json(&destination.join("finalization.json"), &finalization).unwrap();
    assert!(validate_archived_integrity(root, &archived).is_err());
}

#[test]
fn interrupted_same_pr_finalization_resumes_without_another_approval() {
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
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();

    let error = archive_change_with_same_pr_finalize_failure(root, &record.id).unwrap_err();
    assert!(error.contains("source restored"), "{error}");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );

    let destination = finalize_change(root, &record.id).unwrap();
    assert!(destination.is_dir());
    let archived = load_change(root, &record.id).unwrap();
    let approvals = load_approvals(root, &archived).unwrap();
    assert_eq!(
        approvals
            .approvals
            .iter()
            .filter(|approval| approval.gate == "definition")
            .count(),
        1
    );
    assert!(
        latest_terminal_approval(&approvals).is_some_and(|a| a.gate == "finalization"),
        "finalization must leave a terminal closing approval"
    );
}

#[test]
fn post_move_same_pr_finalization_resumes_across_dates_without_another_approval() {
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
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();

    let source = change_dir(root, &record.id);
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("1900-01-01-{}", record.id));
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::rename(&source, &destination).unwrap();
    let state_path = destination.join("state.json");
    let markdown_path = destination.join("change.md");
    let original_state = fs::read_to_string(&state_path).unwrap();
    let original_markdown = fs::read_to_string(&markdown_path).unwrap();
    write_json(
        &root.join(TRANSACTION_PATH),
        &[
            TransactionEntry {
                path: portable_project_path(root, &state_path),
                original: Some(original_state),
            },
            TransactionEntry {
                path: portable_project_path(root, &markdown_path),
                original: Some(original_markdown),
            },
        ],
    )
    .unwrap();
    let mut interrupted = load_change(root, &record.id).unwrap();
    interrupted.state = ChangeState::Archived;
    fs::write(&state_path, json_content(&interrupted).unwrap()).unwrap();

    let resumed = finalize_change(root, &record.id).unwrap();
    assert_eq!(resumed, destination);
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Archived
    );
}

#[test]
fn workflow_v2_archive_survives_squash_merge_in_fresh_clone() {
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
    // Keep workflow-v2 baseline and lifecycle JSON byte-identical across platforms.
    // Windows CI defaults can enable core.autocrlf and rewrite committed digests on clone.
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
    fs::write(
        root.join("src/lib.rs"),
        "// Committed before workflow-v2 adoption.\n",
    )
    .unwrap();
    git(&root, &["add", "src/lib.rs"]);
    git(&root, &["commit", "-m", "pre-adoption implementation"]);

    let record = current_workflow_record(&root, completed_no_spec_record(&root));
    approve_definition(&root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(&root, Some(&record.id)).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Implement approved change"]);
    check_change(&root, Some(&record.id)).unwrap();
    record_scoped_review_with_verdict(
        &root,
        &record.id,
        "Independent reviewer".into(),
        ScopedReviewVerdict::Block,
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Record blocking scoped review"]);
    record_scoped_review(&root, &record.id, "Independent reviewer".into()).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "Record passing scoped review"]);
    let implementation_commit = git_output(&root, &["rev-parse", "HEAD"]).unwrap();
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
    // Apply LF identity at clone time. Setting core.autocrlf after checkout is too late:
    // Windows CI with a system autocrlf=true would already rewrite lifecycle JSON digests.
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
    // Clones do not inherit the source repo's local identity. CI runners often have no
    // global user.name/email, so set them explicitly before any fresh-tree commits.
    git(&fresh, &["config", "user.email", "test@example.com"]);
    git(&fresh, &["config", "user.name", "Test"]);
    git(&fresh, &["config", "core.autocrlf", "false"]);
    git(&fresh, &["config", "core.eol", "lf"]);
    assert!(
        git_output(
            &fresh,
            &[
                "rev-parse",
                "--verify",
                &format!("{implementation_commit}^{{commit}}"),
            ],
        )
        .is_none()
    );
    let archived = load_change(&fresh, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    validate_archived_integrity(&fresh, &archived).unwrap();

    let archived_dir = find_change_dir(&fresh, &record.id).unwrap();
    fs::write(
        archived_dir.join("context.md"),
        "# Context\n\nAttempted later archive rewrite.\n",
    )
    .unwrap();
    let mut rewritten_state = load_change(&fresh, &record.id).unwrap();
    rewritten_state.updated_at += 1;
    fs::write(
        archived_dir.join("state.json"),
        json_content(&rewritten_state).unwrap(),
    )
    .unwrap();
    git(&fresh, &["add", "."]);
    git(&fresh, &["commit", "-m", "Attempt later archive rewrite"]);
    assert!(!archived_finalization_tree_is_recorded(&fresh, &rewritten_state).unwrap());
}

#[test]
fn scoped_review_allows_the_scope_approver_and_records_pass_and_block_verdicts() {
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
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();

    let same_actor =
        record_scoped_review(root, &record.id, "scope OWNER".into()).expect("solo review");
    assert_eq!(same_actor.reviewer, "scope OWNER");
    assert_eq!(same_actor.verdict, ScopedReviewVerdict::Pass);
    let error =
        record_scoped_review(root, &record.id, "Independent\u{200b} reviewer".into()).unwrap_err();
    assert!(error.contains("stable ASCII identity"), "{error}");
    let blocked = record_scoped_review_with_verdict(
        root,
        &record.id,
        "Independent reviewer".into(),
        ScopedReviewVerdict::Block,
    )
    .unwrap();
    assert_eq!(blocked.verdict, ScopedReviewVerdict::Block);
    let reviewed_implementation = blocked.implementation_commit.clone();
    let error = finalize_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("independent scoped review is stale"),
        "{error}"
    );
    let passed = record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    assert_eq!(passed.verdict, ScopedReviewVerdict::Pass);
    assert_eq!(passed.implementation_commit, reviewed_implementation);
    let attempts: ScopedReviewAttemptLedger = serde_json::from_str(
        &fs::read_to_string(scoped_review_attempts_path(root, &record)).unwrap(),
    )
    .unwrap();
    assert_eq!(attempts.reviews.len(), 3);
    assert_eq!(attempts.reviews[0].reviewer, "scope OWNER");
    assert_eq!(attempts.reviews[0].verdict, ScopedReviewVerdict::Pass);
    assert_eq!(attempts.reviews[1].verdict, ScopedReviewVerdict::Block);
    assert_eq!(attempts.reviews[2].verdict, ScopedReviewVerdict::Pass);
}

fn review_ledger_fixture(reviewer: &str) -> ScopedReviewAttemptLedger {
    ScopedReviewAttemptLedger {
        schema_version: 1,
        reviews: vec![ScopedReviewRecord {
            schema_version: 1,
            change_id: "CHG-0001-round-trip".into(),
            reviewer: reviewer.into(),
            provenance: ScopedReviewProvenanceV1 {
                schema_version: 1,
                provider: ScopedReviewProvenanceProvider::GithubActionsCheck,
                required_check: "SpecSync scoped review".into(),
            },
            verdict: ScopedReviewVerdict::Pass,
            implementation_commit: "a".repeat(40),
            contract_digest: "b".repeat(64),
            execution_digest: None,
            workspace_digest: "c".repeat(64),
            timestamp: 1_787_000_000,
        }],
    }
}

/// A change has exactly two homes, and evidence moves between them in BOTH directions.
///
/// `finalize` carries the ledger active -> archive; `reopen` carries the same bytes back
/// archive -> active. Only the first direction was admitted, so a reopened change could never be
/// finalized again (#540) — and the refusal surfaced at the next finalize rather than at the
/// reopen, because it comes from a walk over committed history rather than from the command
/// performing the move.
///
/// #540 shipped with no test at all. Reverting its fix left the entire suite green, so drill 049
/// was its only protection — and that drill passes on a binary with the guard deleted outright.
#[test]
fn scoped_review_evidence_may_move_between_a_change_s_two_homes_in_either_direction() {
    let active = (
        ".specsync/changes/CHG-0001-round-trip/review-attempts.json".to_string(),
        review_ledger_fixture("Independent reviewer"),
    );
    let archived = (
        ".specsync/archive/changes/2026-08-19-CHG-0001-round-trip/review-attempts.json".to_string(),
        review_ledger_fixture("Independent reviewer"),
    );

    // finalize: active -> archive. Admitted before #540 and still admitted.
    validate_scoped_review_history_transition(Some(&active), Some(&archived), false)
        .expect("finalize moves evidence into the archive");

    // reopen: archive -> active. This is the direction #540 restored.
    validate_scoped_review_history_transition(Some(&archived), Some(&active), false)
        .expect("reopen moves the same evidence back to the active workspace");

    // An unchanged path is unaffected either way.
    validate_scoped_review_history_transition(Some(&active), Some(&active), false)
        .expect("an unchanged path is not a move");
}

/// The allowance admits the two canonical homes, not arbitrary relocation.
///
/// This refusal is the reason the guard exists, and before this test nothing asserted it:
/// `grep -rn 'moved evidence outside finalization' src/` returned exactly one hit, in the
/// product. Widening #540's fix to "any move is fine" would have passed every other check,
/// including drill 049.
#[test]
fn scoped_review_evidence_moved_to_a_third_location_is_refused() {
    let active = (
        ".specsync/changes/CHG-0001-round-trip/review-attempts.json".to_string(),
        review_ledger_fixture("Independent reviewer"),
    );
    let elsewhere = (
        "docs/attic/review-attempts.json".to_string(),
        review_ledger_fixture("Independent reviewer"),
    );

    let error = validate_scoped_review_history_transition(Some(&active), Some(&elsewhere), false)
        .expect_err("a move outside the two canonical homes must be refused");
    assert!(
        error.contains("moved evidence outside finalization"),
        "{error}"
    );

    // And back the other way: a third location is not a valid origin either.
    let error = validate_scoped_review_history_transition(Some(&elsewhere), Some(&active), false)
        .expect_err("a move from outside the two canonical homes must be refused");
    assert!(
        error.contains("moved evidence outside finalization"),
        "{error}"
    );
}

/// Deleting committed evidence is refused regardless of where it lived.
#[test]
fn scoped_review_evidence_may_not_be_deleted() {
    let active = (
        ".specsync/changes/CHG-0001-round-trip/review-attempts.json".to_string(),
        review_ledger_fixture("Independent reviewer"),
    );
    let error = validate_scoped_review_history_transition(Some(&active), None, false)
        .expect_err("committed evidence may not vanish");
    assert!(error.contains("deleted committed evidence"), "{error}");
}

/// Evidence written by a later 6.x must still be READABLE by this one.
///
/// These structs carried `#[serde(deny_unknown_fields)]`, so an older 6.x binary rejected any
/// file a newer 6.x had extended. That made every evidence shape unextendable for the whole of
/// 6's life — the mechanism by which "add a field in 6.4" becomes "we need 7.0".
///
/// Each case below deserializes a payload carrying a field this binary does not know.
#[test]
fn evidence_written_by_a_later_six_still_parses() {
    let review = serde_json::json!({
        "schema_version": 2,
        "change_id": "CHG-0001-forward",
        "reviewer": "Peer",
        "provenance": {
            "schema_version": 1,
            "provider": "github_actions_check",
            "required_check": "SpecSync scoped review",
            "future_provenance_field": "ignored by this binary"
        },
        "verdict": "pass",
        "implementation_commit": "a".repeat(40),
        "contract_digest": "b".repeat(64),
        "workspace_digest": "c".repeat(64),
        "timestamp": 1_787_000_000u64,
        "future_review_field": {"nested": true}
    });
    // Read the way production does — `from_slice` over raw bytes (src/change.rs:5300, :5394),
    // not `from_value` over an already-parsed tree. An external reviewer pointed out that the
    // first draft tested only the latter and therefore overclaimed the valve for the files that
    // actually matter.
    let parsed: ScopedReviewRecord =
        serde_json::from_slice(serde_json::to_vec(&review).unwrap().as_slice())
            .expect("a newer review record must still parse from committed bytes");
    assert_eq!(parsed.reviewer, "Peer");

    let finalization = serde_json::json!({
        "schema_version": 1,
        "change_id": "CHG-0001-forward",
        "implementation_commit": "a".repeat(40),
        "implementation_tree": "b".repeat(40),
        "contract_digest": "c".repeat(64),
        "workspace_digest": "d".repeat(64),
        "closing_digest": "e".repeat(64),
        "review_digest": "f".repeat(64),
        "finalization_digest": "0".repeat(64),
        "timestamp": 1_787_000_000u64,
        "future_finalization_field": ["anything"]
    });
    let parsed: FinalizationRecord =
        serde_json::from_slice(serde_json::to_vec(&finalization).unwrap().as_slice())
            .expect("a newer finalization record must still parse from committed bytes");
    assert_eq!(parsed.change_id, "CHG-0001-forward");

    let ledger = serde_json::json!({
        "schema_version": 1,
        "corrections": [],
        "future_ledger_field": "x"
    });
    let parsed: CorrectionLedger =
        serde_json::from_slice(serde_json::to_vec(&ledger).unwrap().as_slice())
            .expect("a newer correction ledger must still parse from committed bytes");
    assert!(parsed.corrections.is_empty());
}

/// Regenerable caches keep rejecting unknown shapes — discarding and rebuilding costs nothing.
///
/// The distinction is the point: tolerance is for EVIDENCE, which cannot be recomputed. A cache
/// that cannot be understood should be thrown away, not tolerated.
#[test]
fn regenerable_caches_still_reject_what_they_cannot_understand() {
    // The first draft of this test named the map `entries`. `HashCache` calls it `hashes` and
    // does not default it, so the parse failed with "missing field `hashes`" no matter what
    // `deny_unknown_fields` did — it passed with the attribute stripped from both structs, and
    // guarded nothing. The payload below is otherwise complete, so the ONLY thing that can
    // reject it is the unknown field, and the assertion says so by name.
    let value = serde_json::json!({
        "format_version": 1,
        "hashes": {},
        "future_cache_field": true
    });
    let error = serde_json::from_value::<crate::hash_cache::HashCache>(value)
        .expect_err("a cache is regenerable, so an unrecognised shape must be discarded");
    let error = error.to_string();
    assert!(
        error.contains("future_cache_field"),
        "the rejection must be caused by the unknown field, not by a malformed payload: {error}"
    );

    // Control: the same payload without the unknown field parses. Without this, a future typo
    // in a required field name would silently make the assertion above unreachable again.
    let known = serde_json::json!({"format_version": 1, "hashes": {}});
    assert!(
        serde_json::from_value::<crate::hash_cache::HashCache>(known).is_ok(),
        "the payload must be valid apart from the unknown field"
    );
}

/// The other direction: a NEW binary reading a policy an OLD binary wrote.
///
/// `deny_unknown_fields` is the old-reads-new door. This is the new-reads-old one: not one of
/// `SddPolicy`'s fields was optional on deserialize, which worked only because SpecSync always
/// writes all of them. The day 6.x adds a ninth, every `sdd.json` written before it becomes
/// unreadable by the binary that added it — the same one-way door, walked from the other side.
///
/// The defaults are the safe ones, so a policy that loses a field enforces more, not less.
#[test]
fn a_policy_written_before_a_field_existed_still_loads_and_fails_closed() {
    // Stands in for a 6.0-era file read by a later 6.x: every field this binary knows is
    // present except the two whose absence would matter most.
    let older = serde_json::json!({
        "version": 2,
        "meaningful_paths": ["src/"],
        "ignored_paths": [],
        "verification_commands": [],
        "custom_artifacts": {}
    });
    let policy: SddPolicy = serde_json::from_slice(serde_json::to_vec(&older).unwrap().as_slice())
        .expect("a policy missing a field this binary added must still load");
    assert_eq!(policy.meaningful_paths, vec!["src/".to_string()]);
    assert!(
        policy.enabled,
        "an absent `enabled` must not read as disabled"
    );
    assert!(
        policy.require_change_for_meaningful_files,
        "an absent requirement must not read as no requirement"
    );
}

/// Tolerance is not enough for the two baselines, and this pins the limit rather than hiding it.
///
/// `read_workflow_v2_baseline` and `validate_legacy_archive_baseline_bytes` re-serialize what
/// they parsed and require the result to equal the bytes on disk. A field added by a newer 6.x
/// survives `from_slice` and is then dropped by the re-serialization, so the comparison fails.
/// The byte gate is deliberate — these files anchor history and must not drift — but it means
/// the forward-compatibility valve buys them nothing, and a reader of the tolerant structs
/// above should not conclude otherwise.
#[test]
fn a_baseline_is_still_frozen_by_its_canonical_byte_gate() {
    let canonical = WorkflowV2Baseline {
        schema_version: 1,
        domain: "specsync.workflow-v2-baseline.v1".into(),
        cutoff_commit: Some("d".repeat(40)),
    };

    // The type is tolerant: a newer baseline parses.
    let extended = serde_json::json!({
        "schema_version": 1,
        "domain": "specsync.workflow-v2-baseline.v1",
        "cutoff_commit": "d".repeat(40),
        "future_baseline_field": 7
    });
    let bytes = serde_json::to_vec_pretty(&extended).unwrap();
    let parsed: WorkflowV2Baseline = serde_json::from_slice(bytes.as_slice())
        .expect("the type itself tolerates the unknown field");
    assert_eq!(parsed, canonical, "the unknown field is dropped on parse");

    // The file-level reader is not: dropping the field breaks the byte round-trip.
    let round_tripped = json_content(&parsed).unwrap();
    assert_ne!(
        String::from_utf8(bytes).unwrap(),
        round_tripped,
        "a newer baseline cannot survive the canonical-bytes gate; if this ever passes, the gate \
         moved and the two baselines became genuinely extensible"
    );
}

/// An unknown workflow version means a NEWER writer, not corruption.
///
/// It previously reported "invalid change state … unsupported workflow version", which is
/// indistinguishable from a damaged file — so the operator's correct action, upgrading, was the
/// one thing the message did not say.
#[test]
fn an_unknown_workflow_version_says_upgrade_rather_than_invalid() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_lifecycle_test_policy(root);
    let mut record = completed_no_spec_record(root);
    record.workflow_version = 9;
    save_change(root, &record).unwrap();

    let error = load_change(root, &record.id).expect_err("an unknown workflow version must fail");
    assert!(
        error.contains("written by a newer SpecSync"),
        "must name the cause: {error}"
    );
    assert!(
        error.contains("upgrade specsync"),
        "must name the remedy: {error}"
    );
    assert!(
        !error.contains("invalid change state"),
        "must not read as corruption: {error}"
    );
}

#[test]
fn scoped_review_attempt_history_rejects_erasing_a_committed_block() {
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
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    check_change(root, Some(&record.id)).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Materialize checked implementation"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review_with_verdict(
        root,
        &record.id,
        "Independent reviewer".into(),
        ScopedReviewVerdict::Block,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Record blocking scoped review"]);

    let passed = record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let rewritten = ScopedReviewAttemptLedger {
        schema_version: 1,
        reviews: vec![passed.clone()],
    };
    fs::write(
        scoped_review_attempts_path(root, &record),
        serde_json::to_vec_pretty(&rewritten).unwrap(),
    )
    .unwrap();
    fs::write(
        scoped_review_path(root, &record),
        serde_json::to_vec_pretty(&passed).unwrap(),
    )
    .unwrap();

    let error = load_scoped_review(root, &record).unwrap_err();
    assert!(
        error.contains("removed or rewrote committed evidence"),
        "{error}"
    );

    git(&["add", "."]);
    git(&["commit", "-m", "Attempt to erase blocking scoped review"]);
    let error = load_scoped_review(root, &record).unwrap_err();
    assert!(
        error.contains("removed or rewrote committed evidence"),
        "{error}"
    );
}

#[test]
fn finalize_requires_current_independent_review() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = current_workflow_record(root, completed_no_spec_record(root));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_change(root, Some(&record.id)).unwrap();

    let error = finalize_change(root, &record.id).unwrap_err();

    assert!(error.contains("independent scoped review evidence is missing"));
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn successor_evidence_fields_are_byte_compatible_when_absent() {
    let temp = TempDir::new().unwrap();
    let record = completed_no_spec_record(temp.path());
    let mut legacy_record = serde_json::to_value(&record).unwrap();
    legacy_record.as_object_mut().unwrap().remove("supersedes");
    legacy_record
        .as_object_mut()
        .unwrap()
        .remove("acceptance_owner_corrections");
    let decoded: ChangeRecord = serde_json::from_value(legacy_record.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), legacy_record);

    let verification = VerificationRecord {
        timestamp: 1,
        commit: None,
        contract_digest: "contract".into(),
        execution_digest: None,
        workspace_digest: "workspace".into(),
        acceptance_input_digest: None,
        acceptance_manifest: None,
        semantic_succession: None,
        passed: true,
        commands: Vec::new(),
        requirement_ids: Vec::new(),
    };
    let mut legacy_verification = serde_json::to_value(&verification).unwrap();
    let object = legacy_verification.as_object_mut().unwrap();
    object.remove("acceptance_manifest");
    object.remove("semantic_succession");
    let decoded: VerificationRecord = serde_json::from_value(legacy_verification.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), legacy_verification);
}

#[test]
fn acceptance_manifest_validation_rejects_topology_and_digest_aliases() {
    let payload = sha256_hex(b"content");
    let entry = AcceptanceInputEntryV1 {
        path: "src/lib.rs".into(),
        kind: AcceptanceInputKind::File,
        mode: 0o100644,
        entry_digest: acceptance_entry_digest(
            "src/lib.rs",
            &AcceptanceInputKind::File,
            0o100644,
            &payload,
        ),
        payload_digest: payload,
        owners: vec!["change".into()],
    };
    let manifest = AcceptanceManifestV1 {
        schema_version: 1,
        entries: vec![entry.clone()],
    };
    assert!(validate_acceptance_manifest(&manifest).is_ok());

    let mut duplicate = manifest.clone();
    duplicate.entries.push(entry.clone());
    assert!(validate_acceptance_manifest(&duplicate).is_err());
    let mut wrong_mode = manifest.clone();
    wrong_mode.entries[0].mode = 0o100755;
    assert!(validate_acceptance_manifest(&wrong_mode).is_err());
    let mut missing = manifest;
    missing.entries[0].kind = AcceptanceInputKind::Missing;
    missing.entries[0].mode = 0;
    missing.entries[0].entry_digest = acceptance_entry_digest(
        "src/lib.rs",
        &AcceptanceInputKind::Missing,
        0,
        &missing.entries[0].payload_digest,
    );
    assert!(validate_acceptance_manifest(&missing).is_err());
}

#[test]
fn portable_symlink_targets_reject_host_specific_or_ambiguous_forms() {
    for target in [
        "../shared/file",
        "shared/file",
        "./file",
        "dir/./file",
        "dir//file",
        "dir/",
    ] {
        assert!(
            validate_portable_symlink_target(target).is_ok(),
            "{target:?}"
        );
    }
    for target in [
        "",
        "/etc/passwd",
        "C:/secret",
        "C:secret",
        "dir\\file",
        "\\\\server\\share",
        "\\\\?\\C:\\secret",
        "line\nfeed",
    ] {
        assert!(
            validate_portable_symlink_target(target).is_err(),
            "{target:?}"
        );
    }
}

#[test]
fn portable_project_paths_reject_host_specific_or_ambiguous_forms() {
    assert_eq!(
        strict_portable_relative_path("src/change.rs").unwrap(),
        "src/change.rs"
    );
    for path in [
        "",
        "/etc/passwd",
        "C:/secret",
        "C:secret",
        "dir\\file",
        "\\\\server\\share",
        "\\\\?\\C:\\secret",
        "./file",
        "../file",
        "dir/../file",
        "dir//file",
        "dir/",
        "line\nfeed",
    ] {
        assert!(strict_portable_relative_path(path).is_err(), "{path:?}");
    }
}

#[test]
fn clean_tracked_files_use_canonical_index_bytes_but_dirty_files_do_not() {
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
    git(&["config", "core.autocrlf", "true"]);
    let path = root.join("tracked.txt");
    fs::write(&path, b"canonical\n").unwrap();
    git(&["add", "tracked.txt"]);
    git(&["commit", "-m", "track canonical bytes"]);
    fs::remove_file(&path).unwrap();
    git(&["checkout", "HEAD", "--", "tracked.txt"]);

    let candidates = BTreeSet::from(["tracked.txt".to_string()]);
    let evidence = git_evidence(root, &candidates).unwrap();
    assert_eq!(
        evidence.entry("tracked.txt").unwrap().payload,
        b"canonical\n"
    );

    fs::write(&path, b"dirty\r\n").unwrap();
    let evidence = git_evidence(root, &candidates).unwrap();
    assert_eq!(evidence.entry("tracked.txt").unwrap().payload, b"dirty\r\n");
}

#[test]
fn repository_detection_distinguishes_plain_and_broken_git_directories() {
    let plain = TempDir::new().unwrap();
    assert!(!git_repository_present(plain.path()).unwrap());

    let broken = TempDir::new().unwrap();
    fs::write(
        broken.path().join(".git"),
        "gitdir: missing-worktree-link\n",
    )
    .unwrap();
    let error = git_repository_present(broken.path()).unwrap_err();
    assert!(error.contains("Git repository detection failed"), "{error}");

    let corrupt = TempDir::new().unwrap();
    quiet_git(corrupt.path(), &["init", "-b", "main"]);
    fs::write(corrupt.path().join(".git/config"), "[broken\n").unwrap();
    let error = git_repository_present(corrupt.path()).unwrap_err();
    assert!(error.contains("Git repository detection failed"), "{error}");

    let bare = TempDir::new().unwrap();
    quiet_git(bare.path(), &["init", "--bare"]);
    let error = git_repository_present(bare.path()).unwrap_err();
    assert!(error.contains("not inside a work tree"), "{error}");
}

#[test]
fn literal_pathspecs_do_not_expand_metacharacter_or_colon_candidates() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    #[cfg(not(windows))]
    fs::write(root.join("literal*.txt"), "selected\n").unwrap();
    fs::write(root.join("literalX.txt"), "unrelated\n").unwrap();
    #[cfg(not(windows))]
    fs::write(root.join(":colon"), "colon\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track literal names"]);
    #[cfg(not(windows))]
    fs::write(root.join("literal*.txt"), "dirty selected\n").unwrap();
    fs::write(root.join("literalX.txt"), "dirty unrelated\n").unwrap();

    let candidates = BTreeSet::from(["literal*.txt".into(), ":colon".into()]);
    let evidence = git_evidence(root, &candidates).unwrap();
    #[cfg(not(windows))]
    assert_eq!(
        evidence.entry("literal*.txt").unwrap().payload,
        b"dirty selected\n"
    );
    #[cfg(windows)]
    assert_eq!(
        evidence.entry("literal*.txt").unwrap().kind,
        AcceptanceInputKind::Missing
    );
    #[cfg(not(windows))]
    assert_eq!(evidence.entry(":colon").unwrap().payload, b"colon\n");
    #[cfg(windows)]
    assert_eq!(
        evidence.entry(":colon").unwrap().kind,
        AcceptanceInputKind::Missing
    );
    assert!(!evidence.entries.contains_key("literalX.txt"));

    let paths = git_scoped_project_paths(
        root,
        &BTreeSet::from(["literal*.txt".into(), ":colon".into()]),
    )
    .unwrap()
    .unwrap();
    #[cfg(not(windows))]
    assert_eq!(paths, vec![":colon", "literal*.txt"]);
    #[cfg(windows)]
    // Windows cannot materialize either candidate; the dirty literalX.txt
    // control proves that Git did not expand the `*` candidate as a pathspec.
    assert!(paths.is_empty());
}

#[test]
fn narrow_git_evidence_ignores_a_large_unrelated_index() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let object = run_git_required(
        root,
        &["hash-object", "-w", "--stdin"],
        Some(b"payload\n".to_vec()),
        128,
    )
    .unwrap();
    let object = std::str::from_utf8(&object).unwrap().trim();
    fs::write(root.join("governed.txt"), b"payload\n").unwrap();
    let mut index = format!("100644 {object}\tgoverned.txt\n");
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

    let evidence = git_evidence(root, &BTreeSet::from(["governed.txt".to_string()])).unwrap();
    assert_eq!(evidence.entries.len(), 1);
    assert_eq!(
        evidence.entry("governed.txt").unwrap().payload,
        b"payload\n"
    );
}

#[test]
fn read_scope_reuses_git_evidence_for_repeated_candidates() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("candidate.txt"), "candidate\n").unwrap();
    quiet_git(root, &["add", "candidate.txt"]);
    quiet_git(root, &["commit", "-m", "candidate"]);
    let candidates = BTreeSet::from(["candidate.txt".to_string()]);

    let _scope = begin_change_read_scope(root);
    reset_test_git_process_count();
    let first = git_evidence(root, &candidates).unwrap();
    let first_queries = test_git_process_count();
    let second = git_evidence(root, &candidates).unwrap();

    assert_eq!(second, first);
    assert!(first_queries > 0);
    assert_eq!(test_git_process_count(), first_queries);
}

#[test]
fn bounded_index_fingerprint_rejects_symlink_and_oversized_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let target = root.join("target-index");
    fs::write(&target, b"index").unwrap();
    let link = root.join("linked-index");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let error = fingerprint_git_index_paths(vec![link]).unwrap_err();
        assert!(error.contains("not a regular file"), "{error}");
    }
    let oversized = root.join("oversized-index");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&oversized)
        .unwrap();
    file.set_len(MAX_GIT_INDEX_BYTES as u64 + 1).unwrap();
    let error = fingerprint_git_index_paths(vec![oversized]).unwrap_err();
    assert!(error.contains("exceed"), "{error}");

    let directory = root.join("directory-index");
    fs::create_dir(&directory).unwrap();
    let error = fingerprint_git_index_paths(vec![directory]).unwrap_err();
    assert!(error.contains("not a regular file"), "{error}");
}

#[cfg(unix)]
#[test]
fn correction_ledger_symlink_is_rejected_before_external_payload_parse() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let external = root.join("external-corrections.json");
    fs::write(&external, "{\"schema_version\":1,\"corrections\":[]}").unwrap();
    let ledger = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    std::os::unix::fs::symlink(&external, &ledger).unwrap();
    let error = load_correction_ledger(root, &record).unwrap_err();
    assert!(error.contains("regular file"), "{error}");
}

#[cfg(unix)]
#[test]
fn definition_delta_inventory_rejects_non_utf8_names() {
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let deltas = change_dir(root, &record.id).join("deltas");
    fs::create_dir_all(&deltas).unwrap();
    let created = fs::write(
        deltas.join(std::ffi::OsString::from_vec(vec![0xff])),
        "delta",
    );
    // Some Unix filesystems (notably the default macOS filesystem) reject
    // non-UTF-8 names before SpecSync can observe them. In that case the
    // invariant is already enforced by the platform, so there is no
    // repository entry available for this regression to exercise.
    if created.is_err() {
        return;
    }
    created.unwrap();
    let error = definition_digest(root, &record).unwrap_err();
    assert!(
        error.contains("portable UTF-8"),
        "non-UTF-8 definition artifact names must be rejected"
    );
}

#[test]
fn checkout_autocrlf_resolution_honors_local_global_and_injected_values() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "core.autocrlf", "input"]);

    let isolated_command = || {
        let mut command = rooted_git_command(root);
        command.env_remove("GIT_CONFIG_COUNT");
        for index in 0..8 {
            command.env_remove(format!("GIT_CONFIG_KEY_{index}"));
            command.env_remove(format!("GIT_CONFIG_VALUE_{index}"));
        }
        command
    };
    let mut local = isolated_command();
    assert_eq!(
        checkout_autocrlf_from_command(&mut local).unwrap(),
        Some("input".into())
    );

    quiet_git(root, &["config", "--unset", "core.autocrlf"]);
    let global_path = root.join("controlled-global-config");
    fs::write(&global_path, "[core]\n\tautocrlf = yes\n").unwrap();
    let mut global = isolated_command();
    global.env("GIT_CONFIG_NOSYSTEM", "1");
    global.env("GIT_CONFIG_GLOBAL", &global_path);
    assert_eq!(
        checkout_autocrlf_from_command(&mut global).unwrap(),
        Some("true".into())
    );

    let mut injected = isolated_command();
    injected.env("GIT_CONFIG_NOSYSTEM", "1");
    injected.env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    );
    injected.env("GIT_CONFIG_COUNT", "1");
    injected.env("GIT_CONFIG_KEY_0", "core.autocrlf");
    injected.env("GIT_CONFIG_VALUE_0", "off");
    assert_eq!(
        checkout_autocrlf_from_command(&mut injected).unwrap(),
        Some("false".into())
    );
}

/// The four core keys are read in ONE `git config --get-regexp`, not four `--get` calls.
///
/// Each case below is one git behaviour where `--get-regexp` could plausibly differ from
/// `--get`. Every expectation was confirmed against git 2.50.1 before being written here, not
/// derived from the docs.
#[test]
fn one_config_read_matches_four_for_every_case_git_distinguishes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);

    // Nothing matching set: rc=1 with empty stdout AND stderr, which must read as "unset"
    // rather than as a failure. git init writes core.filemode on some platforms, so unset
    // every key first and assert only that the absent ones do not appear.
    for key in [
        "core.autocrlf",
        "core.eol",
        "core.symlinks",
        "core.filemode",
    ] {
        let _ = Command::new("git")
            .args(["config", "--unset-all", key])
            .current_dir(root)
            .output();
    }
    let empty = effective_checkout_overrides_uncached(root).unwrap();
    assert!(
        !empty.iter().any(|entry| entry.starts_with("core.autocrlf")),
        "an unset key must not appear: {empty:?}"
    );

    // Multi-valued key: git lists every occurrence and `--get` returns the LAST, so the
    // snapshot must keep the last too.
    quiet_git(root, &["config", "core.autocrlf", "true"]);
    quiet_git(root, &["config", "--add", "core.autocrlf", "input"]);
    let multi = effective_checkout_overrides_uncached(root).unwrap();
    assert!(
        multi.contains(&"core.autocrlf=input".to_string()),
        "last value must win for a multi-valued key: {multi:?}"
    );

    // Valueless key: the record carries no newline, which is the empty value `--get` reports
    // at rc=0, and the empty value normalizes to true for a boolean.
    quiet_git(root, &["config", "--unset-all", "core.autocrlf"]);
    fs::write(
        root.join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n\tsymlinks\n",
    )
    .unwrap();
    let valueless = effective_checkout_overrides_uncached(root).unwrap();
    assert!(
        valueless.contains(&"core.symlinks=true".to_string()),
        "a valueless key must normalize like the empty value: {valueless:?}"
    );

    // Mixed-case section and surrounding whitespace: git emits the key lowercased and the
    // value is trimmed, so both normalize.
    fs::write(
        root.join(".git/config"),
        "[CORE]\n\trepositoryformatversion = 0\n\tFileMode =  FALSE \n",
    )
    .unwrap();
    let mixed = effective_checkout_overrides_uncached(root).unwrap();
    assert!(
        mixed.contains(&"core.filemode=false".to_string()),
        "mixed case and padding must normalize: {mixed:?}"
    );

    // eol passes its value through rather than mapping to a boolean.
    fs::write(
        root.join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n\teol = NATIVE\n",
    )
    .unwrap();
    let eol = effective_checkout_overrides_uncached(root).unwrap();
    assert!(
        eol.contains(&"core.eol=native".to_string()),
        "eol must pass through lowercased: {eol:?}"
    );
}

/// The batched read must survive the ordinary global-plus-local config layout.
///
/// Four `git config --get` calls each returned about six bytes, so the 128-byte stdout cap they
/// shared was never near the limit. One `--get-regexp` returns EVERY occurrence of ALL four keys
/// across EVERY scope into that same cap: four keys in two scopes is 144 bytes, which tripped
/// the deterministic-bounds guard and turned a routine read into a hard error — breaking every
/// git-evidence capture on a machine whose `~/.gitconfig` and repo-local config both set them.
///
/// The dev box this was written on has only `core.filemode` set (20 bytes), and the equivalence
/// test above uses a single scope, so neither could see it. This one sets all four keys twice.
#[test]
fn the_batched_config_read_survives_two_config_scopes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);

    // An included file plus local overrides: every key resolves twice.
    fs::write(
        root.join("team.gitconfig"),
        "[core]\n\tautocrlf = input\n\teol = lf\n\tsymlinks = true\n\tfilemode = false\n",
    )
    .unwrap();
    quiet_git(root, &["config", "include.path", "../team.gitconfig"]);
    quiet_git(root, &["config", "core.autocrlf", "false"]);
    quiet_git(root, &["config", "core.eol", "crlf"]);
    quiet_git(root, &["config", "core.symlinks", "false"]);
    quiet_git(root, &["config", "core.filemode", "true"]);

    let overrides = effective_checkout_overrides_uncached(root)
        .expect("two config scopes must not overflow the read bound");

    // Assert the PROPERTY, not a guess about which scope wins. Whichever value
    // `git config --get` resolves is the one the batched read must derive — that is the whole
    // contract. An earlier version of this test hardcoded "local overrides the include", which
    // git does not do here, so the test failed while the code was right.
    for key in [
        "core.autocrlf",
        "core.eol",
        "core.symlinks",
        "core.filemode",
    ] {
        let raw = Command::new("git")
            .args(["config", "--get", key])
            .current_dir(root)
            .output()
            .expect("git config --get");
        let expected = String::from_utf8(raw.stdout)
            .expect("utf8")
            .trim()
            .to_ascii_lowercase();
        assert!(
            overrides.contains(&format!("{key}={expected}")),
            "batched read disagrees with `git config --get {key}` (= {expected}): {overrides:?}"
        );
    }
}

/// A malformed config must still fail loudly (git exits 128 with stderr), never read as unset.
///
/// This is the vacuity control for the batching change: collapsing "no matching key" and
/// "config is broken" into one empty result would make every assertion above pass while turning
/// a broken repository into a silently default one.
///
/// It asserts the PROPERTY, not the wording. Batching changed the message from naming one key
/// to naming the group, and pinning the new text would make this discriminate on an
/// implementation detail. For a behaviour-preserving refactor the right bar is equivalence:
/// this passes before and after, and the evidence for the change is the spawn count, not a
/// test that only the new code can satisfy.
#[test]
fn a_malformed_config_fails_loudly_rather_than_reading_as_unset() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    fs::write(
        root.join(".git/config"),
        "[core\n\trepositoryformatversion = 0\n",
    )
    .unwrap();

    let error = effective_checkout_overrides_uncached(root)
        .expect_err("a malformed config must not read as unset");
    assert!(
        error.contains("failed to inspect effective Git"),
        "a broken config must name the inspection failure: {error}"
    );
}

#[test]
fn checkout_override_allowlist_normalizes_all_supported_settings() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "core.autocrlf", "YES"]);
    quiet_git(root, &["config", "core.eol", "CRLF"]);
    quiet_git(root, &["config", "core.symlinks", "off"]);
    quiet_git(root, &["config", "core.filemode", "1"]);
    assert_eq!(
        effective_checkout_overrides(root).unwrap(),
        vec![
            "core.autocrlf=true",
            "core.eol=crlf",
            "core.symlinks=false",
            "core.filemode=true",
        ]
    );
}

#[test]
fn clean_materialized_symlink_uses_canonical_git_target() {
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
    let blob_source = root.join("blob-source");
    fs::write(&blob_source, b"../shared/tool").unwrap();
    let output = Command::new("git")
        .args(["hash-object", "-w", "blob-source"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let object = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let materialized = root.join("link");
    fs::write(&materialized, b"host-working-copy-bytes").unwrap();
    let worktree = GitWorktreeState {
        modified: BTreeSet::new(),
        sparse_absent: BTreeSet::new(),
    };
    assert_eq!(
        capture_git_candidate(
            root,
            "link",
            Some(0o120000),
            Some(&object),
            &worktree,
            None,
            GitCaptureOptions::default(),
        )
        .unwrap()
        .payload,
        b"../shared/tool"
    );
}

#[test]
fn portable_symlink_payload_preserves_valid_relative_target_bytes() {
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
    let blob_source = root.join("blob-source");
    let materialized = root.join("link");
    let worktree = GitWorktreeState {
        modified: BTreeSet::new(),
        sparse_absent: BTreeSet::new(),
    };
    for target in [
        "./file",
        "dir/./file",
        "dir//file",
        "dir/",
        "../shared/file",
    ] {
        fs::write(&blob_source, target.as_bytes()).unwrap();
        let output = Command::new("git")
            .args(["hash-object", "-w", "blob-source"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success());
        let object = String::from_utf8(output.stdout).unwrap().trim().to_string();
        fs::write(&materialized, b"host-working-copy-bytes").unwrap();
        assert_eq!(
            capture_git_candidate(
                root,
                "link",
                Some(0o120000),
                Some(&object),
                &worktree,
                None,
                GitCaptureOptions::default(),
            )
            .unwrap(),
            GitCapturedEntry {
                kind: AcceptanceInputKind::Symlink,
                mode: 0o120000,
                object: Some(object),
                payload: target.as_bytes().to_vec(),
                object_identity: false,
            }
        );
    }
}

#[test]
fn deleted_tracked_symlink_is_missing_across_all_digest_surfaces() {
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
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let blob_source = root.join("symlink-target");
    fs::write(&blob_source, b"../shared/tool").unwrap();
    let output = Command::new("git")
        .args(["hash-object", "-w", "symlink-target"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let object = String::from_utf8(output.stdout).unwrap().trim().to_string();
    fs::remove_file(blob_source).unwrap();
    git(&["add", POLICY_PATH]);
    let cache_info = format!("120000,{object},tracked-link");
    git(&["update-index", "--add", "--cacheinfo", &cache_info]);
    git(&["commit", "-m", "track symlink"]);

    let modified = git_worktree_state(root).unwrap().unwrap().modified;
    assert!(modified.contains("tracked-link"));
    assert!(project_input_digest(root).is_ok());

    let mut record = completed_no_spec_record(root);
    record.affected_specs.clear();
    record.affected_paths = vec!["tracked-link".into()];
    record.state = ChangeState::Implementing;
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    assert_eq!(manifest.entries.len(), 1);
    let entry = &manifest.entries[0];
    assert_eq!(entry.path, "tracked-link");
    assert_eq!(entry.kind, AcceptanceInputKind::Missing);
    assert_eq!(entry.mode, 0);
    assert_eq!(entry.payload_digest, sha256_hex(b""));

    let mut expected = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    expected.entry("tracked-link", b"missing", 0, b"");
    assert_eq!(
        acceptance_input_digest(root, &record, &[]).unwrap(),
        expected.finish()
    );
}

#[test]
fn modified_tracked_symlink_uses_current_file_topology() {
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
            "git {} failed",
            args.join(" ")
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    let blob_source = root.join("symlink-target");
    fs::write(&blob_source, b"../shared/tool").unwrap();
    let output = Command::new("git")
        .args(["hash-object", "-w", "symlink-target"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let object = String::from_utf8(output.stdout).unwrap().trim().to_string();
    fs::remove_file(blob_source).unwrap();
    git(&["add", POLICY_PATH]);
    let cache_info = format!("120000,{object},tracked-link");
    git(&["update-index", "--add", "--cacheinfo", &cache_info]);
    git(&["commit", "-m", "track symlink"]);
    fs::write(root.join("tracked-link"), b"regular replacement\n").unwrap();

    let state = git_worktree_state(root).unwrap().unwrap();
    assert!(state.modified.contains("tracked-link"));
    assert!(project_input_digest(root).is_ok());
    let mut record = completed_no_spec_record(root);
    record.affected_specs.clear();
    record.affected_paths = vec!["tracked-link".into()];
    record.state = ChangeState::Implementing;
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    assert_eq!(manifest.entries.len(), 1);
    let entry = &manifest.entries[0];
    assert_eq!(entry.kind, AcceptanceInputKind::File);
    assert_eq!(entry.mode, 0o100644);
    assert_eq!(entry.payload_digest, sha256_hex(b"regular replacement\n"));

    let mut expected = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    expected.entry("tracked-link", b"file", 0o100644, b"regular replacement\n");
    assert_eq!(
        acceptance_input_digest(root, &record, &[]).unwrap(),
        expected.finish()
    );
}

#[test]
fn hidden_and_ambiguous_git_index_states_fail_closed() {
    let assume = TempDir::new().unwrap();
    let root = assume.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "base"]);
    quiet_git(root, &["update-index", "--assume-unchanged", "tracked.txt"]);
    fs::write(root.join("tracked.txt"), "hidden edit\n").unwrap();
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("assume-unchanged"), "{error}");

    let unmerged = TempDir::new().unwrap();
    let root = unmerged.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "base"]);
    quiet_git(root, &["switch", "-c", "other"]);
    fs::write(root.join("tracked.txt"), "other\n").unwrap();
    quiet_git(root, &["commit", "-am", "other"]);
    quiet_git(root, &["switch", "main"]);
    fs::write(root.join("tracked.txt"), "main\n").unwrap();
    quiet_git(root, &["commit", "-am", "main"]);
    let status = Command::new("git")
        .args(["merge", "other"])
        .current_dir(root)
        .status()
        .unwrap();
    assert!(!status.success());
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("unresolved Git index stages"), "{error}");
}

#[test]
fn sparse_absent_files_use_index_bytes_but_materialized_paths_fail_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    write_lifecycle_test_policy(root);
    fs::write(root.join("sparse.txt"), b"canonical sparse\n").unwrap();
    quiet_git(root, &["add", POLICY_PATH, "sparse.txt"]);
    quiet_git(root, &["commit", "-m", "track sparse file"]);
    quiet_git(root, &["update-index", "--skip-worktree", "sparse.txt"]);
    fs::remove_file(root.join("sparse.txt")).unwrap();

    let state = git_worktree_state(root).unwrap().unwrap();
    assert!(state.sparse_absent.contains("sparse.txt"));
    assert!(project_input_digest(root).is_ok());
    let mut record = completed_no_spec_record(root);
    record.affected_specs.clear();
    record.affected_paths = vec!["sparse.txt".into()];
    record.state = ChangeState::Implementing;
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].kind, AcceptanceInputKind::File);
    assert_eq!(
        manifest.entries[0].payload_digest,
        sha256_hex(b"canonical sparse\n")
    );
    let mut expected = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    expected.entry("sparse.txt", b"file", 0o100644, b"canonical sparse\n");
    assert_eq!(
        acceptance_input_digest(root, &record, &[]).unwrap(),
        expected.finish()
    );

    fs::write(root.join("sparse.txt"), b"materialized\n").unwrap();
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("materialized skip-worktree"), "{error}");
}

#[test]
fn custom_content_attributes_fail_before_index_substitution() {
    for (attribute, expected) in [
        ("filter=demo", "filter"),
        ("working-tree-encoding=UTF-8", "working-tree-encoding"),
    ] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        quiet_git(root, &["init", "-b", "main"]);
        quiet_git(root, &["config", "user.email", "test@example.com"]);
        quiet_git(root, &["config", "user.name", "Test"]);
        fs::write(root.join(".gitattributes"), format!("odd* {attribute}\n")).unwrap();
        fs::write(root.join("odd name-ß.txt"), "content\n").unwrap();
        quiet_git(root, &["add", ".gitattributes", "odd name-ß.txt"]);
        quiet_git(root, &["commit", "-m", "track attributed path"]);
        let error = project_input_digest(root).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}

fn configure_official_lfs_driver(root: &Path) {
    quiet_git(root, &["config", "filter.lfs.required", "false"]);
    quiet_git(root, &["config", "filter.lfs.clean", "git-lfs clean -- %f"]);
    quiet_git(
        root,
        &["config", "filter.lfs.smudge", "git-lfs smudge -- %f"],
    );
}

#[test]
fn git_lfs_clean_file_is_named_by_its_object_id() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    configure_official_lfs_driver(root);
    fs::write(root.join(".gitattributes"), "*.bin filter=lfs -text\n").unwrap();
    let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nsize 1\n";
    fs::write(root.join("pack.bin"), pointer).unwrap();
    quiet_git(root, &["add", ".gitattributes", "pack.bin"]);
    quiet_git(root, &["commit", "-m", "track lfs pointer"]);

    let candidates = BTreeSet::from(["pack.bin".to_string()]);
    let inspected = inspect_git_candidates(root, &candidates, false, true).unwrap();
    let entry = inspected.entries.get("pack.bin").unwrap();
    let object = entry.object.clone().unwrap();
    assert!(entry.object_identity);
    assert_eq!(entry.payload, object.as_bytes());
    assert_ne!(entry.payload.as_slice(), pointer);

    let clean = project_input_digest(root).unwrap();
    fs::write(root.join("pack.bin"), object.as_bytes()).unwrap();
    let same_bytes = project_input_digest(root).unwrap();
    assert_ne!(clean, same_bytes);
    fs::write(root.join("pack.bin"), b"smudged working tree\n").unwrap();
    let dirty = project_input_digest(root).unwrap();
    assert_ne!(clean, dirty);
}

#[test]
fn clean_object_id_does_not_match_a_file_of_those_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("note.txt"), b"hello\n").unwrap();
    quiet_git(root, &["add", "note.txt"]);
    quiet_git(root, &["commit", "-m", "track note"]);
    let clean = project_input_digest(root).unwrap();
    let object = quiet_git(root, &["rev-parse", "HEAD:note.txt"]);
    fs::write(root.join("note.txt"), object.trim().as_bytes()).unwrap();
    assert_ne!(clean, project_input_digest(root).unwrap());
}

#[test]
fn filter_lfs_without_the_git_lfs_driver_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    // Store the pointer before the attribute exists so a host Git LFS driver
    // cannot rewrite it. `git diff-files` compares those bytes directly.
    let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nsize 1\n";
    fs::write(root.join("pack.bin"), pointer).unwrap();
    quiet_git(root, &["add", "pack.bin"]);
    quiet_git(root, &["commit", "-m", "track pointer"]);
    quiet_git(root, &["config", "filter.lfs.required", "false"]);
    quiet_git(root, &["config", "filter.lfs.clean", "cat"]);
    quiet_git(root, &["config", "filter.lfs.smudge", "cat"]);
    fs::write(root.join(".gitattributes"), "*.bin filter=lfs -text\n").unwrap();
    quiet_git(root, &["add", ".gitattributes"]);
    quiet_git(root, &["commit", "-m", "attribute"]);
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("filter.lfs"), "{error}");
}

#[test]
fn filter_lfs_rejects_a_blob_that_is_not_a_pointer() {
    assert!(!is_git_lfs_pointer(b"not a pointer\n"));
    assert!(!is_git_lfs_pointer(
        b"version https://git-lfs.github.com/spec/v1\n"
    ));
    assert!(is_git_lfs_pointer(
        b"version https://git-lfs.github.com/spec/v1\noid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nsize 1\n"
    ));

    // `git diff-files` runs `git-lfs clean` when that driver is installed, and
    // the clean output of a non-pointer is a different blob, so the path looks
    // dirty and the pointer gate never runs. Call the gate directly.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let blob = b"not a pointer\n";
    fs::write(root.join("pack.bin"), blob).unwrap();
    let object = quiet_git(root, &["hash-object", "-w", "--no-filters", "pack.bin"]);
    let worktree = GitWorktreeState {
        modified: BTreeSet::new(),
        sparse_absent: BTreeSet::new(),
    };
    let prefetched = BTreeMap::from([(object.clone(), blob.to_vec())]);
    let error = capture_git_candidate(
        root,
        "pack.bin",
        Some(0o100644),
        Some(&object),
        &worktree,
        Some(&prefetched),
        GitCaptureOptions {
            identify_clean_blobs: true,
            lfs: true,
        },
    )
    .unwrap_err();
    assert!(error.contains("not a Git LFS pointer"), "{error}");
    assert!(error.contains("pack.bin"), "{error}");
}

#[test]
fn attribute_output_requires_the_exact_requested_cartesian_set() {
    let paths = ["tracked.txt".to_string()];
    let refs = paths.iter().collect::<Vec<_>>();
    let valid = b"tracked.txt\0filter\0unspecified\0tracked.txt\0working-tree-encoding\0unset\0tracked.txt\0ident\0unspecified\0";
    validate_git_attribute_output(&refs, valid).unwrap();

    for invalid in [
            &valid[..valid.len() - 1],
            b"tracked.txt\0filter\0unspecified\0tracked.txt\0ident\0unspecified\0".as_slice(),
            b"other.txt\0filter\0unspecified\0tracked.txt\0working-tree-encoding\0unset\0tracked.txt\0ident\0unspecified\0".as_slice(),
            b"tracked.txt\0filter\0unspecified\0tracked.txt\0filter\0unset\0tracked.txt\0ident\0unspecified\0".as_slice(),
            b"tracked.txt\0unknown\0unspecified\0tracked.txt\0working-tree-encoding\0unset\0tracked.txt\0ident\0unspecified\0".as_slice(),
            b"tracked.txt\0filter\0unspecified\0\0working-tree-encoding\0unset\0tracked.txt\0ident\0unspecified\0".as_slice(),
        ] {
            assert!(validate_git_attribute_output(&refs, invalid).is_err());
        }
}

#[test]
fn large_attribute_inventory_completes_and_rejects_late_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    fs::write(root.join(".gitattributes"), "late-* filter=demo\n").unwrap();
    let mut paths = (0..4_000)
        .map(|index| format!("ordinary-{index:05}-{}.txt", "x".repeat(48)))
        .collect::<BTreeSet<_>>();
    paths.insert("late-attributed-path.txt".into());
    let error = validate_canonical_git_attributes(root, &paths).unwrap_err();
    assert!(
        error.contains("filter") && error.contains("late-attributed-path"),
        "{error}"
    );
}

#[test]
fn excluded_attributes_and_flags_do_not_block_definition_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let record = completed_no_spec_record(root);
    fs::write(root.join(".gitattributes"), "unrelated.txt filter=demo\n").unwrap();
    fs::write(root.join("unrelated.txt"), "unrelated\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(
        root,
        &["commit", "-m", "track definition and unrelated input"],
    );
    quiet_git(
        root,
        &["update-index", "--assume-unchanged", "unrelated.txt"],
    );
    assert!(definition_digest(root, &record).is_ok());
}

#[test]
fn scoped_acceptance_discovery_ignores_unrelated_transform_and_visibility_flags() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let mut record = completed_no_spec_record(root);
    record.affected_specs.clear();
    record.affected_paths = vec!["governed.txt".into()];
    record.state = ChangeState::Implementing;
    fs::write(root.join("governed.txt"), "governed\n").unwrap();
    fs::write(root.join("unrelated.txt"), "unrelated\n").unwrap();
    fs::write(root.join(".gitattributes"), "unrelated.txt filter=demo\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track scoped evidence"]);
    quiet_git(
        root,
        &["update-index", "--assume-unchanged", "unrelated.txt"],
    );
    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].path, "governed.txt");

    fs::write(root.join(".gitattributes"), "governed.txt filter=demo\n").unwrap();
    let error = acceptance_manifest(root, &record, &[]).unwrap_err();
    assert!(error.contains("filter"), "{error}");
}

#[test]
fn fsmonitor_valid_and_ident_conversion_fail_closed() {
    let monitored = TempDir::new().unwrap();
    let root = monitored.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "base"]);
    quiet_git(root, &["config", "core.fsmonitor", "true"]);
    quiet_git(root, &["update-index", "--fsmonitor-valid", "tracked.txt"]);
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("fsmonitor"), "{error}");

    let identified = TempDir::new().unwrap();
    let root = identified.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join(".gitattributes"), "tracked.txt ident\n").unwrap();
    fs::write(root.join("tracked.txt"), "$Id$\n").unwrap();
    quiet_git(root, &["add", ".gitattributes", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "track ident input"]);
    fs::remove_file(root.join("tracked.txt")).unwrap();
    quiet_git(root, &["checkout", "HEAD", "--", "tracked.txt"]);
    assert!(
        fs::read_to_string(root.join("tracked.txt"))
            .unwrap()
            .contains("$Id:")
    );
    let error = project_input_digest(root).unwrap_err();
    assert!(error.contains("ident"), "{error}");
}

#[test]
fn index_generation_mutation_retries_then_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "base"]);
    let candidates = BTreeSet::from(["tracked.txt".to_string()]);
    let error = git_evidence_with_hook(root, &candidates, |attempt, root| {
        let path = format!("mutation-{attempt}.txt");
        fs::write(root.join(&path), format!("{attempt}\n")).unwrap();
        quiet_git(root, &["add", &path]);
    })
    .unwrap_err();
    assert!(error.contains("index changed"), "{error}");
}

#[test]
fn candidate_worktree_mutation_retries_then_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["commit", "-m", "base"]);
    let candidates = BTreeSet::from(["tracked.txt".to_string()]);
    let error = git_evidence_with_hook(root, &candidates, |attempt, root| {
        fs::write(root.join("tracked.txt"), format!("mutation-{attempt}\n")).unwrap();
    })
    .unwrap_err();
    assert!(error.contains("candidate state changed"), "{error}");
}

// `finalize` supplies archived-change paths as extra candidates. A candidate
// naming a directory expands under `:(top,literal)` to every tracked file
// beneath it, and the scope guard rejected the first expansion because the
// candidate set holds only the directory. The effect was that finalize failed
// in any repository that had ever archived a change — every project past its
// first — while passing on the fresh fixtures the suite is built on.
#[test]
fn a_directory_candidate_admits_the_files_it_expands_to() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let archived = ".specsync/archive/changes/2026-01-01-CHG-0001-old";
    fs::create_dir_all(root.join(archived)).unwrap();
    fs::write(root.join(archived).join("approvals.json"), "{}\n").unwrap();
    fs::write(root.join(archived).join("state.json"), "{}\n").unwrap();
    fs::write(root.join("tracked.txt"), "tracked\n").unwrap();
    quiet_git(root, &["add", "-A"]);

    let mut extra = BTreeSet::new();
    extra.insert(archived.to_string());

    let result = stable_discovered_evidence(root, None, &extra, false, false);
    assert!(
        result.is_ok(),
        "directory candidate rejected its own expansion: {}",
        result.unwrap_err()
    );
}

#[test]
fn discovered_git_inventory_mutation_retries_then_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    fs::write(root.join("tracked.txt"), "tracked\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    let error = stable_discovered_evidence_with_hook(
        root,
        None,
        &BTreeSet::new(),
        false,
        false,
        |attempt, root| {
            let path = format!("appeared-{attempt}.txt");
            fs::write(root.join(&path), "new\n").unwrap();
            if attempt == 1 {
                quiet_git(root, &["add", &path]);
            }
        },
    )
    .unwrap_err();
    assert!(
        error.contains("inventory") || error.contains("evidence"),
        "{error}"
    );
}

#[test]
fn discovered_non_git_inventory_mutation_retries_then_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("existing.txt"), "existing\n").unwrap();
    let error = stable_discovered_evidence_with_hook(
        root,
        None,
        &BTreeSet::new(),
        false,
        false,
        |attempt, root| {
            fs::write(root.join(format!("appeared-{attempt}.txt")), "new\n").unwrap();
        },
    )
    .unwrap_err();
    assert!(
        error.contains("inventory") || error.contains("evidence"),
        "{error}"
    );
}

#[test]
fn repository_worktree_link_mutation_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    fs::write(root.join("tracked.txt"), "tracked\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    let error = stable_discovered_evidence_with_hook(
        root,
        None,
        &BTreeSet::new(),
        false,
        false,
        |_attempt, root| {
            if !root.join(".git-original").exists() {
                fs::rename(root.join(".git"), root.join(".git-original")).unwrap();
                fs::write(root.join(".git"), "gitdir: missing-link\n").unwrap();
            }
        },
    )
    .unwrap_err();
    assert!(
        error.contains("repository") || error.contains("Git"),
        "{error}"
    );
}

#[test]
fn unrelated_unmerged_stage_does_not_block_scoped_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("governed.txt"), "governed\n").unwrap();
    fs::write(root.join("conflict.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "base"]);
    quiet_git(root, &["switch", "-c", "other"]);
    fs::write(root.join("conflict.txt"), "other\n").unwrap();
    quiet_git(root, &["commit", "-am", "other"]);
    quiet_git(root, &["switch", "main"]);
    fs::write(root.join("conflict.txt"), "main\n").unwrap();
    quiet_git(root, &["commit", "-am", "main"]);
    assert!(
        !Command::new("git")
            .args(["merge", "other"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    let governed = BTreeSet::from(["governed.txt".to_string()]);
    assert!(git_evidence(root, &governed).is_ok());
    let conflict = BTreeSet::from(["conflict.txt".to_string()]);
    let error = git_evidence(root, &conflict).unwrap_err();
    assert!(error.contains("unresolved Git index stages"), "{error}");
}

#[test]
fn every_false_fsmonitor_spelling_is_inactive() {
    for value in ["false", "FALSE", "no", "No", "off", "OFF", "0"] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        quiet_git(root, &["init", "-b", "main"]);
        quiet_git(root, &["config", "user.email", "test@example.com"]);
        quiet_git(root, &["config", "user.name", "Test"]);
        fs::write(root.join("tracked.txt"), "base\n").unwrap();
        quiet_git(root, &["add", "tracked.txt"]);
        quiet_git(root, &["commit", "-m", "base"]);
        quiet_git(root, &["config", "core.fsmonitor", value]);
        let candidates = BTreeSet::from(["tracked.txt".to_string()]);
        assert!(git_evidence(root, &candidates).is_ok(), "{value}");
    }
}

#[test]
fn split_index_fingerprint_uses_only_the_effective_dependency() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    fs::write(root.join("tracked.txt"), "base\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    quiet_git(root, &["update-index", "--split-index"]);
    let before = git_index_fingerprint(root).unwrap();
    let git_dir = root.join(".git");
    fs::write(git_dir.join("sharedindex.unrelated"), b"not an index").unwrap();
    assert_eq!(git_index_fingerprint(root).unwrap(), before);
    fs::write(root.join("tracked.txt"), "next\n").unwrap();
    quiet_git(root, &["add", "tracked.txt"]);
    assert_ne!(git_index_fingerprint(root).unwrap(), before);
}

#[test]
fn alternate_index_path_mutation_changes_the_bounded_fingerprint() {
    let temp = TempDir::new().unwrap();
    let alternate = temp.path().join("alternate.index");
    let shared = temp.path().join("shared.index");
    fs::write(&alternate, b"DIRC alternate generation one").unwrap();
    fs::write(&shared, b"shared generation").unwrap();
    let paths = vec![alternate.clone(), shared.clone()];
    let before = fingerprint_git_index_paths(paths.clone()).unwrap();
    fs::write(&alternate, b"DIRC alternate generation two").unwrap();
    let after = fingerprint_git_index_paths(paths).unwrap();
    assert_ne!(before, after);
}

#[test]
fn post_detection_git_failure_is_fatal_with_bounded_diagnostics() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    assert!(git_repository_present(root).unwrap());
    let error = run_git_required(
        root,
        &["ls-files", "--definitely-invalid-specsync-option"],
        None,
        1024,
    )
    .unwrap_err();
    assert!(error.contains("failed"), "{error}");
    assert!(error.len() <= MAX_GIT_DIAGNOSTIC_BYTES + 1024);
}

#[test]
fn first_baseline_authority_binding_requires_the_ledger() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record.affected_paths = vec![LEGACY_BASELINE_PATH.to_string()];
    let error = bind_legacy_archive_baseline_authority(root, &mut record).unwrap_err();
    assert!(error.contains("cannot bind a missing ledger"), "{error}");
    assert!(record.legacy_archive_baseline_digest.is_none());
}

// Verifies REQ-change-014.
#[test]
fn baseline_authority_manifest_signs_exact_volatile_ledger_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record.state = ChangeState::Implementing;
    record.affected_paths.push(LEGACY_BASELINE_PATH.to_string());
    fs::create_dir_all(root.join(".specsync/archive")).unwrap();
    fs::write(root.join(LEGACY_BASELINE_PATH), b"{\"schema_version\":1}\n").unwrap();
    fs::write(root.join(".specsync/archive/unrelated.txt"), b"ignored\n").unwrap();

    let manifest = acceptance_manifest(root, &record, &[]).unwrap();
    let ledger = manifest
        .entries
        .iter()
        .find(|entry| entry.path == LEGACY_BASELINE_PATH)
        .expect("the protected baseline ledger must be signed");
    assert_eq!(ledger.owners, vec![EXACT_DELIVERY_OWNER]);
    assert!(
        manifest
            .entries
            .iter()
            .all(|entry| entry.path != ".specsync/archive/unrelated.txt")
    );
}

#[cfg(unix)]
#[test]
fn clean_executable_definition_artifact_is_regular_evidence() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let context = change_dir(root, &record.id).join("context.md");
    let mut permissions = fs::metadata(&context).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&context, permissions).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(
        root,
        &["commit", "-m", "track executable definition artifact"],
    );
    assert!(definition_digest(root, &record).is_ok());
}

#[test]
fn capped_git_runner_drains_kills_and_reaps_large_output() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let payload = vec![b'x'; 2 * 1024 * 1024];
    let object =
        run_git_required(root, &["hash-object", "-w", "--stdin"], Some(payload), 256).unwrap();
    let object = std::str::from_utf8(&object).unwrap().trim();
    let error = run_git_bounded(root, &["cat-file", "blob", object], None, 1024).unwrap_err();
    assert!(
        error.contains("output exceeds deterministic bounds"),
        "{error}"
    );
    assert!(run_git_required(root, &["rev-parse", "--git-dir"], None, 1024).is_ok());
}

#[test]
fn batched_git_blob_reads_preserve_binary_payloads_and_order() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    let payloads = [
        b"first\nline\0tail".to_vec(),
        Vec::new(),
        b"third\r\nline\n".to_vec(),
    ];
    let objects: Vec<_> = payloads
        .iter()
        .map(|payload| {
            run_git_required(
                root,
                &["hash-object", "-w", "--stdin"],
                Some(payload.clone()),
                128,
            )
            .unwrap()
        })
        .map(|object| String::from_utf8(object).unwrap().trim().to_string())
        .collect();
    let object_refs: Vec<_> = objects.iter().map(String::as_str).collect();

    assert_eq!(git_blob_bytes_batch(root, &object_refs).unwrap(), payloads);
}

#[cfg(unix)]
#[test]
fn bounded_git_runner_times_out_a_silent_process() {
    let mut command = Command::new("sh");
    let started = Instant::now();
    let error = run_git_command_bounded_with_deadline(
        &mut command,
        &["-c", "exec sleep 30"],
        None,
        1024,
        Duration::from_millis(30),
    )
    .unwrap_err();
    assert!(error.contains("wall-clock deadline"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn timed_out_git_runner_reaps_child_and_joins_blocked_writer() {
    assert_timed_out_git_runner_cleanup(false);
}

#[cfg(unix)]
#[test]
fn timed_out_git_runner_reaps_child_before_child_can_make_progress() {
    assert_timed_out_git_runner_cleanup(true);
}

#[cfg(unix)]
fn assert_timed_out_git_runner_cleanup(stop_child: bool) {
    let pid = std::rc::Rc::new(Cell::new(None));
    let observed_pid = std::rc::Rc::clone(&pid);
    TEST_GIT_STDIN_WRITER_JOINED.with(|joined| joined.set(false));
    TEST_GIT_CHILD_SPAWNED.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |pid| {
            observed_pid.set(Some(pid));
            if stop_child {
                // Stop the child before starting the runner's deadline. No child-side
                // readiness write or stdin read can be needed for cleanup to succeed.
                // SAFETY: pid belongs to the live child just spawned by this thread.
                assert_eq!(unsafe { libc::kill(pid as libc::pid_t, libc::SIGSTOP) }, 0);
                let mut status = 0;
                loop {
                    // SAFETY: status is writable and this thread owns the child.
                    let waited =
                        unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WUNTRACED) };
                    if waited == -1
                        && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted
                    {
                        continue;
                    }
                    assert_eq!(waited, pid as libc::pid_t);
                    break;
                }
                assert!(libc::WIFSTOPPED(status));
            }
        }));
    });
    let mut command = Command::new("sleep");
    let result = run_git_command_bounded_with_deadline(
        &mut command,
        &["30"],
        Some(vec![b'x'; 4 * 1024 * 1024]),
        1024,
        Duration::from_millis(50),
    );
    let pid = pid.get().expect("parent must observe the spawned child");
    // waitpid identifies our child without probing a PID that could have been reused.
    // SAFETY: no status pointer is requested; WNOHANG never waits for a live child.
    let remaining =
        unsafe { libc::waitpid(pid as libc::pid_t, std::ptr::null_mut(), libc::WNOHANG) };
    let wait_error = std::io::Error::last_os_error();
    if remaining == 0 {
        // A deliberately broken runner still owns this live, unreaped child.
        // SAFETY: the unreaped child PID remains reserved until this wait completes.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
            libc::waitpid(pid as libc::pid_t, std::ptr::null_mut(), 0);
        }
    }
    let error = result.unwrap_err();
    assert!(error.contains("wall-clock deadline"), "{error}");
    assert_eq!(remaining, -1, "timed-out child {pid} was not reaped");
    assert_eq!(wait_error.raw_os_error(), Some(libc::ECHILD));
    TEST_GIT_STDIN_WRITER_JOINED.with(|joined| {
        assert!(joined.get(), "blocked stdin writer was not joined");
    });
}

#[cfg(unix)]
#[test]
fn non_git_walk_skips_volatile_trees_but_fails_on_relevant_walk_errors() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("target/hidden")).unwrap();
    fs::write(root.join("target/hidden/file"), b"ignored").unwrap();
    fs::set_permissions(
        root.join("target/hidden"),
        fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    assert!(strict_walk_project_paths(root).is_ok());
    fs::set_permissions(
        root.join("target/hidden"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();

    fs::create_dir_all(root.join("relevant/hidden")).unwrap();
    fs::write(root.join("relevant/hidden/file"), b"governed").unwrap();
    fs::set_permissions(
        root.join("relevant/hidden"),
        fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    let result = strict_walk_project_paths(root);
    fs::set_permissions(
        root.join("relevant/hidden"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert!(result.is_err());
}

#[test]
fn oversized_git_evidence_inventory_fails_before_subprocess_work() {
    let candidates = (0..=MAX_GIT_EVIDENCE_PATHS)
        .map(|index| format!("path-{index}"))
        .collect();
    let error = git_evidence(Path::new("."), &candidates).unwrap_err();
    assert!(error.contains("inventory exceeds"), "{error}");
}

#[test]
fn sparse_archive_entries_and_dirty_symlink_topology_are_preserved() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let archive = ".specsync/archive/changes/2026-07-15-CHG-0001-sparse";
    fs::create_dir_all(root.join(archive)).unwrap();
    fs::write(root.join(archive).join("sparse.txt"), "canonical sparse\n").unwrap();
    let target = root.join("target");
    fs::write(&target, "../shared/tool").unwrap();
    let output = Command::new("git")
        .args(["hash-object", "-w", "target"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let object = String::from_utf8(output.stdout).unwrap().trim().to_string();
    fs::remove_file(target).unwrap();
    quiet_git(root, &["add", &format!("{archive}/sparse.txt")]);
    let cache_info = format!("120000,{object},{archive}/link");
    quiet_git(root, &["update-index", "--add", "--cacheinfo", &cache_info]);
    quiet_git(root, &["commit", "-m", "track archive topology"]);
    quiet_git(
        root,
        &[
            "update-index",
            "--skip-worktree",
            &format!("{archive}/sparse.txt"),
        ],
    );
    fs::remove_file(root.join(archive).join("sparse.txt")).unwrap();
    fs::write(root.join(archive).join("link"), "regular replacement\n").unwrap();

    let snapshot = archive_workspace_snapshot(root, &root.join(archive), archive).unwrap();
    assert_eq!(snapshot.get("sparse.txt").unwrap().1, b"canonical sparse\n");
    assert_eq!(snapshot.get("link").unwrap().0, 0o100644);
    assert_eq!(snapshot.get("link").unwrap().1, b"regular replacement\n");
}

#[test]
fn shared_archive_evidence_is_partitioned_by_exact_baseline_subtree() {
    let first = ".specsync/archive/changes/2026-07-15-CHG-0001-first/state.json";
    let second = ".specsync/archive/changes/2026-07-15-CHG-0002-second/state.json";
    let evidence = GitEvidence {
        modes: BTreeMap::new(),
        entries: BTreeMap::from([
            (
                first.to_string(),
                GitCapturedEntry {
                    kind: AcceptanceInputKind::File,
                    mode: 0o100644,
                    object: None,
                    payload: b"first".to_vec(),
                    object_identity: false,
                },
            ),
            (
                second.to_string(),
                GitCapturedEntry {
                    kind: AcceptanceInputKind::File,
                    mode: 0o100755,
                    object: None,
                    payload: b"second".to_vec(),
                    object_identity: false,
                },
            ),
        ]),
    };
    let paths = vec![first.to_string(), second.to_string()];

    let snapshot = archive_snapshot_from_evidence(
        &paths,
        &evidence,
        ".specsync/archive/changes/2026-07-15-CHG-0001-first",
    )
    .unwrap();

    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot["state.json"], (0o100644, b"first".to_vec()));
}

#[cfg(unix)]
#[test]
fn definition_artifact_symlinks_fail_before_referent_reads() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let context = change_dir(root, &record.id).join("context.md");
    let external = temp.path().join("external-large.md");
    fs::write(
        &external,
        vec![b'x'; MAX_CHANGE_ARTIFACT_BYTES as usize + 1],
    )
    .unwrap();
    fs::remove_file(&context).unwrap();
    symlink(&external, &context).unwrap();
    let error = definition_digest(root, &record).unwrap_err();
    assert!(
        error.contains("not a regular file"),
        "symlinked definition artifacts must be rejected"
    );

    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track symlinked definition"]);
    let error = definition_digest(root, &record).unwrap_err();
    assert!(
        error.contains("not a regular file"),
        "tracked symlinked definition artifacts must be rejected"
    );
    fs::remove_file(&context).unwrap();
    symlink("another-external-target", &context).unwrap();
    let error = definition_digest(root, &record).unwrap_err();
    assert!(
        error.contains("not a regular file"),
        "retargeted definition artifact symlinks must be rejected"
    );
}

#[test]
fn attribute_subprocess_failure_is_reaped_and_reported() {
    let temp = TempDir::new().unwrap();
    let paths = BTreeSet::from(["tracked.txt".to_string()]);
    let error = validate_canonical_git_attributes(temp.path(), &paths).unwrap_err();
    assert!(
        error.contains("failed to inspect Git content attributes"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn conversion_attributes_do_not_block_canonical_symlink_payloads() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join(".gitattributes"), "link filter=demo ident\n").unwrap();
    fs::write(root.join("target"), "target\n").unwrap();
    symlink("target", root.join("link")).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track attributed symlink"]);
    assert!(project_input_digest(root).is_ok());
}

#[test]
fn missing_bound_baseline_cannot_retain_stale_definition_digest() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record.legacy_archive_baseline_digest = Some("a".repeat(64));
    let error = bind_legacy_archive_baseline_authority(root, &mut record).unwrap_err();
    assert!(
        error.contains("stale binding cannot be retained"),
        "{error}"
    );
}

#[test]
fn root_source_files_are_not_misclassified_as_delivery_metadata() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("main.py"), "print('ok')\n").unwrap();
    fs::write(root.join("main.go"), "package main\n").unwrap();
    assert!(path_is_production_source(root, "main.py"));
    assert!(path_is_production_source(root, "main.go"));
    assert!(!path_is_recognized_delivery_metadata("main.py"));
    assert!(!path_is_recognized_delivery_metadata("main.go"));
    for path in [
        "pyproject.toml",
        "go.mod",
        "package.json",
        "pnpm-lock.yaml",
        "action.yml",
    ] {
        assert!(path_is_recognized_delivery_metadata(path), "{path}");
    }
}

#[test]
fn invalid_supersedes_mutation_is_transactional() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let predecessor = current_workflow_record(
        root,
        completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nPredecessor.\n",
        ),
    );
    let mut successor = create_change(
        root,
        CreateChangeRequest {
            description: "Successor".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    let state_path = change_dir(root, &successor.id).join("state.json");
    let markdown_path = change_dir(root, &successor.id).join("change.md");
    let before_state = fs::read(&state_path).unwrap();
    let before_markdown = fs::read(&markdown_path).unwrap();
    let oversized = format!("src/{}.rs", "x".repeat(MAX_ACCEPTANCE_PATH_BYTES + 1));
    assert!(
        add_supersedes_obligation(
            root,
            &successor.id,
            &predecessor.id,
            &oversized,
            "auth",
            &sha256_hex(b"entry"),
        )
        .is_err()
    );
    assert_eq!(fs::read(state_path).unwrap(), before_state);
    assert_eq!(fs::read(markdown_path).unwrap(), before_markdown);
}

#[test]
fn mapped_tests_remain_exact_only() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nTests remain governed.\n",
    );
    let candidates = BTreeSet::from(["tests/auth.rs".to_string()]);
    let evidence = git_evidence(root, &candidates).unwrap();
    assert_eq!(
        acceptance_input_owners(
            root,
            &record,
            "tests/auth.rs",
            &[],
            &evidence,
            UnownedProductionSource::Reject,
        )
        .unwrap(),
        vec![EXACT_TEST_OWNER.to_string()]
    );
}

#[test]
fn signed_directory_deletion_remains_a_missing_manifest_entry() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record.affected_paths = vec!["src/".into()];
    record.state = ChangeState::Accepted;
    let payload = sha256_hex(b"old");
    let signed = AcceptanceManifestV1 {
        schema_version: 1,
        entries: vec![AcceptanceInputEntryV1 {
            path: "src/deleted.rs".into(),
            kind: AcceptanceInputKind::File,
            mode: 0o100644,
            entry_digest: acceptance_entry_digest(
                "src/deleted.rs",
                &AcceptanceInputKind::File,
                0o100644,
                &payload,
            ),
            payload_digest: payload,
            owners: vec!["change".into()],
        }],
    };
    let current = acceptance_manifest_with_signed_owners(root, &record, &[], &signed).unwrap();
    let deleted = current
        .entries
        .iter()
        .find(|entry| entry.path == "src/deleted.rs")
        .unwrap();
    assert_eq!(deleted.kind, AcceptanceInputKind::Missing);
    assert_eq!(deleted.payload_digest, sha256_hex(b""));
}

#[test]
fn duplicate_active_and_archived_locations_fail_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_no_spec_record(root);
    let archived = root
        .join(ARCHIVE_PATH)
        .join(format!("2026-07-14-{}", record.id));
    fs::create_dir_all(&archived).unwrap();
    let mut archived_record = record.clone();
    archived_record.state = ChangeState::Archived;
    write_json(&archived.join("state.json"), &archived_record).unwrap();
    assert!(
        find_change_dir(root, &record.id)
            .unwrap_err()
            .contains("ambiguous")
    );
    assert!(
        list_all_changes_checked(root)
            .unwrap_err()
            .contains("multiple")
    );
    assert_eq!(
        terminal_evidence_summary(root, &archived_record).validity,
        TerminalEvidenceValidity::CorruptHistory
    );
}

#[test]
fn legacy_archive_tombstones_without_lifecycle_state_are_skipped() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(ARCHIVE_PATH).join("legacy/deltas")).unwrap();
    fs::write(
        root.join(ARCHIVE_PATH).join("legacy/deltas/auth.md"),
        "## REMOVED\n### REQUIREMENT REQ-auth-001\nRetired.\n",
    )
    .unwrap();
    assert!(list_all_changes_checked(root).unwrap().is_empty());
}

/// A real package that lost its lifecycle files is corruption, whatever it is called.
///
/// The old discriminator was `name.contains("-CHG-")`. An UNDATED package — `CHG-0001-foo` —
/// does not contain that substring, so this exact case was already being swallowed before any
/// identity redesign: `list_all_changes_uncached` and `located_change_sequences` skipped a
/// damaged archive instead of refusing it. The name never carried the meaning the check
/// attributed to it.
/// A description that slugifies to a Windows device name must not become that directory.
///
/// `slugify("NUL")` produced `"nul"`. Harmless while the directory was `CHG-0091-nul`, and
/// fatal the moment the slug is the whole component: Windows cannot create or open a directory
/// called `nul`, and it matches device names case-insensitively so lowercasing is no escape.
#[test]
fn a_description_that_slugifies_to_a_reserved_device_is_not_left_as_one() {
    for reserved in ["NUL", "con", "AUX", "prn", "COM1", "lpt9"] {
        let slug = slugify(reserved);
        assert!(
            !crate::commands::is_reserved_module_name(&slug),
            "`{reserved}` slugified to the reserved name `{slug}`"
        );
        assert!(
            slug.starts_with(&reserved.to_ascii_lowercase()),
            "the transform must stay recognisable: {reserved} -> {slug}"
        );
    }
    // `change` and `specs` are reserved for a different reason — they collide with the
    // workspace layout — and the empty fallback used to be exactly `change`.
    assert!(!crate::commands::is_reserved_module_name(&slugify("!!!")));
    assert!(!crate::commands::is_reserved_module_name(&slugify(
        "change"
    )));
}

/// The cap bounds BYTES of output, not characters of input.
///
/// `take(80)` counted input characters, so runs of punctuation collapsed into single hyphens
/// and a "capped" slug could finish well under 80 — 43 of this repository's 159 archived slugs
/// land at exactly 80 for that reason. Bounding the output is what makes the cap actually
/// bound the path component.
#[test]
fn the_slug_cap_bounds_the_directory_component_not_the_input() {
    let long = "a".repeat(400);
    assert!(
        slugify(&long).len() <= MAX_SLUG_BYTES,
        "a long single word must still fit the component budget"
    );

    // Punctuation-heavy input: many input characters, few output bytes. The old cap counted
    // the former and so under-filled the slug.
    let spaced = (0..200)
        .map(|i| format!("w{i}"))
        .collect::<Vec<_>>()
        .join("   ---   ");
    let slug = slugify(&spaced);
    assert!(slug.len() <= MAX_SLUG_BYTES, "{} bytes", slug.len());
    assert!(
        slug.len() > 80,
        "the cap must actually be larger than the old one here: {} bytes",
        slug.len()
    );
}

/// Truncation stops at a word boundary rather than mid-word.
///
/// 52 of this repository's 159 archived slugs end mid-word (`...preserved-audited-guara`).
/// Trimming back to the last separator costs a handful of characters and is the whole
/// readability fix — the cap itself buys nothing else, since slug uniqueness across those 159
/// saturates at 50 bytes.
///
/// The exact expected value is asserted rather than a property, because a property like "every
/// segment is a whole word" is satisfied by accident at the old 80-character cap: a shorter cut
/// lands somewhere else and may happen to be clean. Pinning the string discriminates.
#[test]
fn a_truncated_slug_does_not_end_mid_word() {
    let words = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike \
                 november oscar papa quebec romeo sierra tango uniform victor whiskey";
    let slug = slugify(words);
    // The raw 120-byte cut would end `...sierra-ta`. The trim gives back the whole word.
    assert_eq!(
        slug,
        "alpha-bravo-charlie-delta-echo-foxtrot-golf-hotel-india-juliet-kilo-lima-mike-november-oscar-papa-quebec-romeo-sierra"
    );
    assert!(slug.len() <= MAX_SLUG_BYTES, "{} bytes", slug.len());
    let source: Vec<&str> = words.split_whitespace().collect();
    for segment in slug.split('-') {
        assert!(
            source.contains(&segment),
            "`{segment}` is a fragment, not a whole word: {slug}"
        );
    }
}
