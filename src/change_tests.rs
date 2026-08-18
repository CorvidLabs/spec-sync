//! Unit tests for [`crate::change`].
//!
//! Split out of `change.rs` by #589: that file was 29,983 lines — 23% of the
//! codebase — and its size is not cosmetic. The defect this release has spent
//! its correctness campaign on is a fix landing at the site named in the bug
//! report while a parallel implementation survives, which has now happened
//! seven times. You cannot sweep for a sibling in a file you cannot hold in
//! your head.
//!
//! This is a PURE MOVE. No test was added, removed, renamed or edited; the
//! `#[test]` count is identical before and after. `#[path]` keeps the module
//! inline for the compiler, so `use super::*` still reaches every private item
//! exactly as before.
//!
//! The 24 `#[cfg(test)]` helpers and fault-injection hooks stay in `change.rs`:
//! production code paths reference them, so they are not test code that merely
//! lives near production code.

use super::*;
use std::sync::{Arc, Barrier, mpsc};
use tempfile::TempDir;

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

fn ensure_test_verification_policy(root: &Path) {
    if !root.join(POLICY_PATH).exists() {
        write_default_policy(root, vec!["true".into()]).unwrap();
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

fn completed_record_with_workflow(root: &Path, legacy: bool) -> ChangeRecord {
    ensure_test_verification_policy(root);
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "add passkeys".into(),
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
            description: "harden verification".into(),
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
            "# Complete\n\nReviewed.\n"
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
            "# Complete\n\nReviewed content.\n"
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
fn change_adopt_rejects_uncommitted_workflow_v1_records_without_writes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    quiet_git(root, &["add", "seed.txt"]);
    quiet_git(root, &["commit", "-m", "trusted base"]);
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
        write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
fn scoped_review_requires_an_independent_passing_verdict() {
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

    let error = record_scoped_review(root, &record.id, "scope OWNER".into()).unwrap_err();
    assert!(
        error.contains("someone other than the scope approver"),
        "{error}"
    );
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
    assert_eq!(attempts.reviews.len(), 2);
    assert_eq!(attempts.reviews[0].verdict, ScopedReviewVerdict::Block);
    assert_eq!(attempts.reviews[1].verdict, ScopedReviewVerdict::Pass);
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
        capture_git_candidate(root, "link", Some(0o120000), Some(&object), &worktree, None,)
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
            capture_git_candidate(root, "link", Some(0o120000), Some(&object), &worktree, None,)
                .unwrap(),
            GitCapturedEntry {
                kind: AcceptanceInputKind::Symlink,
                mode: 0o120000,
                object: Some(object),
                payload: target.as_bytes().to_vec(),
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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

    let result = stable_discovered_evidence(root, None, &extra, false);
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
    let temp = TempDir::new().unwrap();
    let pid_path = temp.path().join("child.pid");
    let mut command = Command::new("sh");
    command.env("SPECSYNC_TEST_PID_PATH", &pid_path);
    let error = run_git_command_bounded_with_deadline(
        &mut command,
        &["-c", "echo $$ > \"$SPECSYNC_TEST_PID_PATH\"; exec sleep 30"],
        Some(vec![b'x'; 4 * 1024 * 1024]),
        1024,
        Duration::from_millis(50),
    )
    .unwrap_err();
    assert!(error.contains("wall-clock deadline"), "{error}");

    let pid = fs::read_to_string(&pid_path).unwrap();
    let status = Command::new("kill")
        .args(["-0", pid.trim()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!status.success(), "timed-out child {pid} was not reaped");
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
                },
            ),
            (
                second.to_string(),
                GitCapturedEntry {
                    kind: AcceptanceInputKind::File,
                    mode: 0o100755,
                    object: None,
                    payload: b"second".to_vec(),
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
    let successor = create_change(
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

    let inspected = inspect_git_candidates(root, &candidates, false).unwrap();

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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
fn change_ids_are_sequential_and_readable() {
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
    assert_eq!(first.id, "CHG-0001-add-passkeys");
    assert_eq!(second.id, "CHG-0002-fix-login");
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
    let mut first = create_change(
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
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
    assert!(change_id_sorts_after(
        "CHG-10000-first-five-digit",
        "CHG-9999-last-four-digit"
    ));
    assert!(change_id_sorts_after(
        "CHG-9999-second-collision",
        "CHG-9999-first-collision"
    ));
    assert!(!change_id_sorts_after(
        "CHG-09999-noncanonical-width",
        "CHG-9999-last-four-digit"
    ));
    assert!(!change_id_sorts_after(
        "CHG-10000-first-five-digit",
        "not-a-change-id"
    ));
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(&root, Vec::new()).unwrap();
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
    git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    assert!(ensure_closing_approval_valid(&root, &record).is_err());
    let error = reopen_change(
        &root,
        &record.id,
        "Reviewer".into(),
        "The verification commit is off history".into(),
    )
    .unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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
            "complete\n",
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
                "# Complete\n\nReviewed.\n"
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
        "../tests/fixtures/chg-0068-adopted-scope.json"
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
    write_default_policy(root, Vec::new()).unwrap();
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
    write_default_policy(root, Vec::new()).unwrap();
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

#[test]
fn validate_artifacts_rejects_hash_todo_body() {
    let temp = TempDir::new().unwrap();
    let record = completed_no_spec_record(temp.path());
    let dir = change_dir(temp.path(), &record.id);
    for artifact in &record.selected_artifacts {
        fs::write(
            dir.join(artifact.file_name()),
            format!(
                "---\nchange: {}\nartifact: {}\n---\n\n# TODO\n",
                record.id,
                artifact.file_name().trim_end_matches(".md")
            ),
        )
        .unwrap();
    }
    let error = validate_artifacts(temp.path(), &record).unwrap_err();
    assert!(
        error.contains("artifact is incomplete"),
        "expected incomplete artifact, got {error}"
    );
}

/// Build a repo whose committed ledger is `committed` and whose working tree
/// ledger is `working`, which is the divergence #533 commits backwards.
fn ledger_divergence_fixture(root: &Path, committed: u64, working: u64) {
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
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let write = |sequence: u64| {
        fs::write(
            root.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };
    write(committed);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "committed high-water mark"]);
    write(working);
}

/// Build a repo with an `origin` whose main carries `origin_seq`, and a local
/// branch whose ledger is `local_seq`. `diverged` decides which shape:
///
/// - `false` — the branch is honestly BEHIND: it was cut from origin at
///   `local_seq` and simply has not caught up. Nothing is wrong with it.
/// - `true` — the branch was cut from origin's tip and then REWROTE the ledger
///   downwards, which is the #533 regression.
///
/// On disk these two are indistinguishable: both show a ledger below origin with
/// no higher workspaces present. Only the history separates them, which is why
/// the gate compares against the merge-base and not against origin.
fn remote_divergence_fixture(root: &Path, origin_seq: u64, local_seq: u64, diverged: bool) {
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
    let origin = root.join("origin.git");
    let work = root.join("work");
    fs::create_dir_all(&work).unwrap();
    git(root, &["init", "--bare", "origin.git"]);
    git(root, &["init", "-b", "main", "work"]);
    git(&work, &["config", "user.email", "test@example.com"]);
    git(&work, &["config", "user.name", "Test"]);
    git(
        &work,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    fs::create_dir_all(work.join(".specsync")).unwrap();
    let write = |dir: &Path, sequence: u64| {
        fs::write(
            dir.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };

    // Commit the branch point at `local_seq`.
    write(&work, local_seq);
    fs::write(work.join("README.md"), "base\n").unwrap();
    git(&work, &["add", "."]);
    git(&work, &["commit", "-m", "branch point"]);

    if diverged {
        // Origin advances FIRST, so the branch point is origin's tip; the local
        // commit below then lowers the ledger relative to where it diverged.
        write(&work, origin_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "origin high-water"]);
        git(&work, &["push", "origin", "main"]);
        write(&work, local_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "hand-regressed ledger"]);
    } else {
        // Push the branch point, then advance origin on a throwaway branch so
        // local's HEAD stays an ancestor of origin/main — honestly behind.
        git(&work, &["push", "origin", "main"]);
        git(&work, &["checkout", "-q", "-b", "ahead"]);
        write(&work, origin_seq);
        git(&work, &["add", "."]);
        git(&work, &["commit", "-m", "origin advanced"]);
        git(&work, &["push", "origin", "ahead:main"]);
        git(&work, &["checkout", "-q", "main"]);
        write(&work, local_seq);
    }
    git(&work, &["fetch", "-q", "origin"]);
}

/// A branch that is merely BEHIND the default branch must not be refused (#533
/// regression).
///
/// The first attempt at the read-side gate compared the ledger against
/// `origin/main` directly. Every unrebased branch trips that: its ledger is
/// older than origin's and perfectly consistent with its own history. `change
/// new` returned exit 1 and told the user to `git checkout origin/HEAD --` a
/// file that was not corrupt.
#[test]
fn a_branch_merely_behind_the_default_branch_is_not_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    remote_divergence_fixture(root, 4, 2, false);
    let work = root.join("work");

    let result = validate_change_sequences(&work);
    assert!(
        result.is_ok(),
        "a branch that is behind origin has an older ledger by definition; \
refusing it punishes the ordinary state of every unrebased branch: {result:?}"
    );
}

/// The vacuity control for the test above: a branch that actually rewrote the
/// ledger downwards must still be refused, so "stop comparing against the
/// remote" cannot be satisfied by removing the gate.
#[test]
fn a_branch_that_lowered_the_ledger_after_diverging_is_still_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    remote_divergence_fixture(root, 4, 2, true);
    let work = root.join("work");

    let error = validate_change_sequences(&work).expect_err(
        "a ledger rewritten below the mark this branch diverged from is the #533 regression",
    );
    assert!(
        error.contains("CHG-0004"),
        "the error must name the mark that was lost, not merely say something is wrong: {error}"
    );
}

/// A branch that RAISED the ledger and then rewrote it downwards is caught,
/// even though it never fell below the mark it diverged at.
///
/// This is the case that decided the oracle. A merge-base comparison acquits
/// it: the branch diverged at 2, raised to 9, rewrote to 3, and 3 is still
/// above the divergence point. Only the branch's own recorded history — which
/// contains the 9 — convicts. The ledger is not a distance from the default
/// branch; it is a high-water mark this branch is accountable for.
#[test]
fn a_branch_that_raised_then_rewrote_the_ledger_is_refused() {
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
    let write = |sequence: u64| {
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(SEQUENCE_PATH),
            serde_json::to_string_pretty(&ChangeSequenceLedger {
                schema_version: 1,
                sequence,
                id: format!("CHG-{sequence:04}-fixture"),
                acknowledged_collisions: Vec::new(),
            })
            .unwrap(),
        )
        .unwrap();
    };

    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write(2);
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "branch point at 2"]);
    write(9);
    git(&["add", "."]);
    git(&["commit", "-m", "raised to 9"]);
    write(3);
    git(&["add", "."]);
    git(&["commit", "-m", "rewrote down to 3"]);

    let error = validate_change_sequences(root).expect_err(
        "a ledger below the highest mark this branch itself recorded is the #533 regression, \
even when it is still above where the branch diverged",
    );
    assert!(
        error.contains("CHG-0009"),
        "the error must name the mark that was lost: {error}"
    );
}

/// A ledger that went stale while the branch sat must not be committed backwards
/// (#533).
///
/// `change new` writes the ledger into the working tree only. Nothing commits it
/// until a later lifecycle step runs `git add -A`, so a value written days
/// earlier — correct when written — is staged over a higher mark the branch has
/// since caught up to. The allocation-time floor cannot help: the value did not
/// start wrong, it went stale.
#[test]
fn a_stale_sequence_ledger_is_raised_to_the_committed_mark_before_staging() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 3, 1);

    let raised = floor_sequence_ledger_to_committed(root).unwrap();
    assert_eq!(
        raised,
        Some((1, 3)),
        "the caller must be told what was raised and from where, so it can disclose it"
    );
    let now = load_change_sequence_ledger(root).unwrap().unwrap();
    assert_eq!(now.sequence, 3, "the high-water mark must not regress");
    assert_eq!(now.id, "CHG-0003-fixture");
}

/// The control that keeps the fix from becoming "always overwrite the ledger".
///
/// A working tree ahead of the committed mark is the ordinary case — that is
/// exactly what `change new` produces — and raising it would destroy the claim
/// the author just made. Without this, a fix that unconditionally restored the
/// committed value would pass the test above.
#[test]
fn a_sequence_ledger_ahead_of_the_committed_mark_is_left_alone() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 3, 7);

    assert_eq!(
        floor_sequence_ledger_to_committed(root).unwrap(),
        None,
        "nothing was raised, so nothing may be reported"
    );
    let now = load_change_sequence_ledger(root).unwrap().unwrap();
    assert_eq!(now.sequence, 7, "the author's newer claim must survive");
    assert_eq!(now.id, "CHG-0007-fixture");
}

/// Equal marks are not a divergence and must not be reported as one.
#[test]
fn a_sequence_ledger_equal_to_the_committed_mark_is_not_reported() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    ledger_divergence_fixture(root, 5, 5);
    assert_eq!(floor_sequence_ledger_to_committed(root).unwrap(), None);
    assert_eq!(
        load_change_sequence_ledger(root).unwrap().unwrap().sequence,
        5
    );
}

#[test]
fn maximum_observed_sequence_floors_on_remote_ledger() {
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
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let ledger = ChangeSequenceLedger {
        schema_version: 1,
        sequence: 42,
        id: "CHG-0042-remote-high-water".into(),
        acknowledged_collisions: Vec::new(),
    };
    fs::write(
        root.join(SEQUENCE_PATH),
        serde_json::to_string_pretty(&ledger).unwrap(),
    )
    .unwrap();
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base with remote ledger"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    // Local working tree has no active changes and a stale/low ledger deleted.
    fs::remove_file(root.join(SEQUENCE_PATH)).unwrap();
    assert_eq!(maximum_observed_sequence(root).unwrap(), 42);
    assert_eq!(next_change_id(root, "feature").unwrap(), "CHG-0043-feature");
}

#[test]
fn sequence_base_env_raises_high_water() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(CHANGES_PATH)).unwrap();
    // SAFETY: single-threaded test; env is restored before the test ends.
    unsafe {
        std::env::set_var("SPECSYNC_SEQUENCE_BASE", "200");
    }
    let maximum = maximum_observed_sequence(root).unwrap();
    unsafe {
        std::env::remove_var("SPECSYNC_SEQUENCE_BASE");
    }
    assert_eq!(maximum, 199);
    assert_eq!(
        {
            unsafe {
                std::env::set_var("SPECSYNC_SEQUENCE_BASE", "200");
            }
            let id = next_change_id(root, "agent-b").unwrap();
            unsafe {
                std::env::remove_var("SPECSYNC_SEQUENCE_BASE");
            }
            id
        },
        "CHG-0200-agent-b"
    );
}

#[test]
fn definition_digest_caps_canonical_bytes_not_larger_checkout_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    fs::write(root.join(".gitattributes"), "*.md text eol=crlf\n").unwrap();
    let record = completed_no_spec_record(root);
    let context = change_dir(root, &record.id).join("context.md");
    fs::write(
        &context,
        b"x\n".repeat(MAX_CHANGE_ARTIFACT_BYTES as usize / 2),
    )
    .unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track definition"]);
    fs::remove_file(&context).unwrap();
    quiet_git(
        root,
        &[
            "checkout",
            "HEAD",
            "--",
            &format!("{CHANGES_PATH}/{}/context.md", record.id),
        ],
    );

    assert!(fs::metadata(&context).unwrap().len() > MAX_CHANGE_ARTIFACT_BYTES);
    assert!(definition_digest(root, &record).is_ok());
}

#[test]
fn definition_digest_rejects_oversized_canonical_sparse_bytes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let record = completed_no_spec_record(root);
    let context_relative = format!("{CHANGES_PATH}/{}/context.md", record.id);
    let context = root.join(&context_relative);
    fs::write(&context, vec![b'x'; MAX_CHANGE_ARTIFACT_BYTES as usize + 1]).unwrap();
    quiet_git(root, &["add", "."]);
    quiet_git(root, &["commit", "-m", "track oversized definition"]);
    quiet_git(
        root,
        &["update-index", "--skip-worktree", &context_relative],
    );
    fs::remove_file(context).unwrap();

    let error = definition_digest(root, &record).unwrap_err();
    assert!(error.contains("exceeds") && error.contains("byte limit"));
}

#[test]
fn non_git_policy_disables_only_changed_path_coverage() {
    let temp = TempDir::new().unwrap();
    write_default_policy(temp.path(), Vec::new()).unwrap();
    assert!(
        !load_policy(temp.path())
            .unwrap()
            .require_change_for_meaningful_files
    );
    let report = check_project(temp.path());
    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.contains("changed paths"))
    );
}

#[test]
fn committed_policy_cannot_be_disabled_or_deleted_locally() {
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
    write_default_policy(root, Vec::new()).unwrap();
    fs::write(root.join(".specsync/version"), SDD_VERSION).unwrap();
    git(&["add", ".specsync/sdd.json", ".specsync/version"]);
    git(&["commit", "-m", "enable sdd"]);

    let mut policy = load_policy(root).unwrap();
    policy.enabled = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("changed without")),
        "unexpected report: {report:?}"
    );

    policy.enabled = true;
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );

    policy.require_change_for_meaningful_files = true;
    policy.meaningful_paths.clear();
    policy.ignored_paths.push(POLICY_PATH.into());
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );

    fs::remove_file(root.join(POLICY_PATH)).unwrap();
    fs::remove_file(root.join(".specsync/version")).unwrap();
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| error.contains("changed without"))
    );
}

#[test]
fn clean_initial_commit_needs_no_changed_path_coverage() {
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
    fs::write(root.join("README.md"), "clean\n").unwrap();
    git(&["add", "README.md"]);
    git(&["commit", "-m", "initial"]);
    assert!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn clean_feature_branch_still_requires_changed_path_coverage() {
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
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"]
    );
}

fn minimal_archived_record(id: &str, affected_paths: Vec<String>) -> ChangeRecord {
    ChangeRecord {
        schema_version: 1,
        workflow_version: 2,
        workflow_origin_version: Some(2),
        id: id.into(),
        slug: "archive".into(),
        title: "Archive fixture".into(),
        description: "fixture".into(),
        kind: ChangeKind::Feature,
        state: ChangeState::Archived,
        canonical_applied: true,
        correction_count: 0,
        base_commit: None,
        created_at: 1,
        updated_at: 1,
        affected_specs: Vec::new(),
        affected_paths,
        no_spec_change: true,
        no_spec_change_rationale: Some("fixture".into()),
        acceptance_criteria: vec!["fixture".into()],
        selected_artifacts: Vec::new(),
        dependencies: Vec::new(),
        supersedes: Vec::new(),
        acceptance_owner_corrections: Vec::new(),
        legacy_archive_baseline_digest: None,
        answers: BTreeMap::new(),
    }
}

#[test]
fn same_pr_archived_change_covers_delivery_paths_with_zero_actives() {
    // Mirrors product Lifecycle gate after `change ship` on the same PR:
    // product paths remain in the base...HEAD delivery, actives are empty,
    // but the archive package is on the tip and must still cover.
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
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "product tip"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"]
    );

    let archived = minimal_archived_record("CHG-0001-same-pr-archive", vec!["src/".into()]);
    let archive_dir = root
        .join(ARCHIVE_PATH)
        .join("2026-08-07-CHG-0001-same-pr-archive");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::write(
        archive_dir.join("state.json"),
        serde_json::to_string_pretty(&archived).unwrap(),
    )
    .unwrap();
    git(&["add", ARCHIVE_PATH]);
    git(&["commit", "-m", "archive tip"]);

    let uncovered = uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap();
    assert!(
        uncovered.is_empty(),
        "delivery archive must cover product paths with zero actives; got {uncovered:?}"
    );
}

#[test]
fn historical_archive_not_in_delivery_does_not_cover_unrelated_paths() {
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
    // Historical archive already on main — not part of this feature delivery.
    let historical = minimal_archived_record("CHG-0000-old", vec!["src/".into()]);
    let archive_dir = root.join(ARCHIVE_PATH).join("2026-01-01-CHG-0000-old");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::write(
        archive_dir.join("state.json"),
        serde_json::to_string_pretty(&historical).unwrap(),
    )
    .unwrap();
    git(&["add", "README.md", ARCHIVE_PATH]);
    git(&["commit", "-m", "base with old archive"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    git(&["switch", "-c", "feature"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "uncovered feature"]);

    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs"],
        "archives outside the delivery must not cover new product paths"
    );
}

#[test]
fn approved_change_does_not_cover_delivery_paths_until_started() {
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
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "feature"]);
    let mut record = completed_record(root);
    record.affected_paths = vec!["src/".into(), SEQUENCE_PATH.into(), POLICY_PATH.into()];
    record.state = ChangeState::Approved;
    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[record.clone()]).unwrap(),
        vec![
            SEQUENCE_PATH.to_string(),
            POLICY_PATH.to_string(),
            "src/lib.rs".into(),
        ]
    );
    record.state = ChangeState::Implementing;
    assert!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[record])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn local_coverage_unions_staged_unstaged_and_untracked_paths() {
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
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::write(root.join("src/lib.rs"), "unstaged\n").unwrap();
    fs::write(root.join("src/staged.rs"), "staged\n").unwrap();
    git(&["add", "src/staged.rs"]);
    fs::write(root.join("src/untracked.rs"), "untracked\n").unwrap();
    assert_eq!(
        uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
        vec!["src/lib.rs", "src/staged.rs", "src/untracked.rs"]
    );
}

#[test]
fn accepted_changes_require_matching_closing_evidence() {
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
    let verification = VerificationRecord {
        timestamp: now(),
        commit: None,
        contract_digest: definition_digest(root, &record).unwrap(),
        execution_digest: None,
        workspace_digest: "workspace".into(),
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
    assert!(ensure_closing_approval_valid(root, &record).is_err());
    append_approval(
        root,
        &record,
        "acceptance",
        Some("Reviewer".into()),
        closing_digest(&record, &verification),
        None,
    )
    .unwrap();
    let error = ensure_closing_approval_valid(root, &record).unwrap_err();
    assert!(error.contains("missing current delivery-input evidence"));
    let mut verification = verification;
    verification.acceptance_input_digest =
        Some(acceptance_input_digest(root, &record, &[]).unwrap());
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
        Some("Reapprove additive delivery-input evidence".into()),
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_ok());
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.approvals.last_mut().unwrap().digest = "tampered".into();
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
    .unwrap();
    assert!(ensure_closing_approval_valid(root, &record).is_err());
}

#[test]
fn working_tree_changes_invalidate_verification() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(
        error.contains("verification project-input digest is stale"),
        "{error}"
    );
}

#[test]
fn stale_accepted_change_reopens_with_audited_evidence_and_reaccepts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let prior_verification = load_verification(root, &record).unwrap();
    let prior_ledger = load_approvals(root, &record).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let stale_report = check_project(root);
    assert!(stale_report.errors.iter().any(|error| {
        error.contains("accepted change verification is stale for current delivery inputs")
    }));
    assert_eq!(
        summarize_change(root, &record).next_action,
        format!(
            "run `specsync change reopen {} --actor <name> --reason <reason>`",
            record.id
        )
    );

    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Review fixes changed scoped delivery inputs".into(),
    )
    .unwrap();
    record = reopened.change;
    assert_eq!(record.state, ChangeState::Verifying);
    assert!(record.canonical_applied);
    assert_eq!(reopened.audit.superseded_approval.actor, "Closer");
    assert_eq!(
        reopened.audit.prior_verification.contract_digest,
        prior_verification.contract_digest
    );
    let reopened_ledger = load_approvals(root, &record).unwrap();
    assert_eq!(
        reopened_ledger.approvals.len(),
        prior_ledger.approvals.len()
    );
    assert_eq!(reopened_ledger.reopenings.len(), 1);
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains("verification evidence is stale for the current commit or contract")
    }));

    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(check_project(root).errors.is_empty());
    let final_ledger = load_approvals(root, &record).unwrap();
    assert_eq!(
        final_ledger.approvals.len(),
        prior_ledger.approvals.len() + 1
    );
    assert_eq!(final_ledger.reopenings.len(), 1);
    assert_eq!(
        final_ledger.reopenings[0]
            .prior_verification
            .contract_digest,
        prior_verification.contract_digest
    );
}

// Verifies reopen recovery when verification.json no longer matches the closing
// approval (tip re-verify drift) but attempt history still authenticates it.
#[test]
fn reopen_binds_historical_verification_when_tip_no_longer_matches_closing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let accepted_verification = load_verification(root, &record).unwrap();
    let closing = latest_terminal_approval(&load_approvals(root, &record).unwrap())
        .cloned()
        .expect("closing approval");
    assert_eq!(
        closing.digest,
        closing_digest(&record, &accepted_verification)
    );

    // Drift the tip: rewrite verification.json to a fresh pre-accept shape that no
    // longer carries the acceptance digest the closing approval signed.
    let mut drifted = accepted_verification.clone();
    drifted.acceptance_input_digest = None;
    drifted.acceptance_manifest = None;
    drifted.semantic_succession = None;
    drifted.timestamp = now();
    record_verification_attempt(root, &record, &drifted).unwrap();
    assert_ne!(
        closing.digest,
        closing_digest(&record, &load_verification(root, &record).unwrap())
    );

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();

    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "Tip verification drifted after accept; delivery inputs also changed".into(),
    )
    .expect("reopen must bind the historical acceptance-bound verification");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(
        reopened.audit.prior_verification.acceptance_input_digest,
        accepted_verification.acceptance_input_digest
    );

    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(check_project(root).errors.is_empty());
    let closing_after = latest_terminal_approval(&load_approvals(root, &record).unwrap())
        .cloned()
        .expect("new closing approval");
    assert_eq!(
        closing_after.digest,
        closing_digest(&record, &load_verification(root, &record).unwrap())
    );
}

// Verifies workflow-v2 same-PR finalize after reopen writes a terminal finalization
// approval that a later reopen can supersede.
#[test]
fn workflow_v2_reopen_after_finalization_accept_then_finalize_archives() {
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

    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    assert!(record.workflow_version >= 2);
    record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    git(&["add", "."]);
    git(&["commit", "-m", "Implement approved change"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();

    // Accept via finalization gate without archiving yet (interrupted finalize shape).
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);
    let ledger = load_approvals(root, &record).unwrap();
    assert!(
        latest_terminal_approval(&ledger).is_some_and(|a| a.gate == "finalization"),
        "finalization must leave a terminal ledger approval"
    );

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let reopened = reopen_change(
        root,
        &record.id,
        "Release reviewer".into(),
        "delivery input changed after finalization accept".into(),
    )
    .expect("v2 reopen must supersede finalization closing approval");
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert_eq!(reopened.audit.superseded_approval.gate, "finalization");

    git(&["add", "."]);
    git(&["commit", "-m", "fix after reopen"]);
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let path = finalize_change(root, &record.id).expect("finalize after reopen must archive");
    assert!(path.join("finalization.json").exists());
    assert!(!change_dir(root, &record.id).exists());
}

#[test]
fn accepted_change_can_refresh_stale_definition_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Accepted);

    // Mutate a selected artifact so the definition digest drifts while accepted.
    let context = change_dir(root, &record.id).join("context.md");
    let mut body = fs::read_to_string(&context).unwrap();
    body.push_str("\n\nRefreshed context while accepted.\n");
    fs::write(&context, body).unwrap();
    assert!(ensure_definition_approval_valid(root, &record).is_err());

    record = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect("accepted records must re-approve a stale definition");
    assert_eq!(record.state, ChangeState::Accepted);
    assert!(ensure_definition_approval_valid(root, &record).is_ok());
}

#[test]
fn legacy_workflow_finalize_refuses_and_names_accept_archive() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    assert_eq!(record.workflow_version, 1);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let error = finalize_change(root, &record.id).unwrap_err();
    assert!(
        error.contains("legacy workflow") && error.contains("change accept"),
        "{error}"
    );
}

// Verifies REQ-change-034.
#[test]
fn stale_accepted_change_error_names_uncovered_input_and_reopen_remediation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: delivery input `src/lib.rs` (owner `change`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen {}` to re-verify the accepted change",
        record.id, record.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
    assert!(
        check_project(root)
            .errors
            .iter()
            .any(|error| *error == expected)
    );
}

// Verifies REQ-change-034.
#[test]
fn stale_accepted_change_error_names_covering_successor_with_stale_evidence() {
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
    write_default_policy(root, Vec::new()).unwrap();
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
    assert!(check_project(root).errors.is_empty());

    fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: delivery input `specs/auth/auth.spec.md` (owner `auth`) changed after acceptance; covering successor change(s) `{}` have stale delivery-input evidence of their own; verify and accept a covering successor, or run `specsync change reopen {}` to re-verify the accepted change",
        predecessor.id, successor.id, predecessor.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
}

// Verifies REQ-change-034.
#[test]
fn stale_accepted_change_error_names_exact_only_input_and_audited_reopen() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, vec!["true".into()]).unwrap();
    fs::write(root.join("README.md"), "Initial review instructions.\n").unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "update review instructions".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Documentation-only review guidance".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Reviewers can follow the release workflow".into()];
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
            "# Complete\n\nReviewed.\n"
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(root.join("README.md"), "Final review instructions.\n").unwrap();
    let expected = format!(
        "{}: accepted change verification is stale for current delivery inputs: exact-only delivery input `README.md` changed after acceptance and requires an audited reopen; run `specsync change reopen {}` to re-verify the accepted change",
        record.id, record.id
    );
    let stale_report = check_project(root);
    assert!(
        stale_report.errors.iter().any(|error| *error == expected),
        "{:?}",
        stale_report.errors
    );
}

// Verifies REQ-change-033.
// Canonical ownership is knowable the moment a path is declared, but was
// enforced only when building the acceptance manifest at finalize. A change
// declaring a path owned by a module it does not name therefore passed
// `approve` and every `check`, was reviewed, and only then failed — into a
// state with no exit, since `correct-owner` is scoped to already-applied
// changes and `reopen` takes only Accepted/Archived.
//
// Rejecting at approve costs two seconds instead of several verification
// passes and a reviewer's signature, and keeps the unrecoverable state
// unreachable.
#[test]
fn approve_rejects_a_declared_path_owned_by_an_undeclared_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();

    // `legacy` owns src/lib.rs but not src/orphan.rs. The change declares
    // `legacy` and touches both — the shape CHG-0081 was stuck in, where a
    // change edits a file belonging to a module it never declared.
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn owned() {}\n").unwrap();
    fs::write(root.join("src/orphan.rs"), "pub fn unowned() {}\n").unwrap();

    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    record.affected_paths = vec!["src/lib.rs".into(), "src/orphan.rs".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();

    let error = approve_definition(root, &record.id, Some("Reviewer".into()), None)
        .expect_err("approve must reject a path no declared module owns");
    assert!(
        error.contains("src/orphan.rs") && error.to_lowercase().contains("own"),
        "error should name the unowned path and the ownership problem: {error}"
    );
    assert!(
        !error.contains("src/lib.rs"),
        "the owned path must not be reported as a problem: {error}"
    );
    assert!(
        error.contains("--spec") || error.contains("files:"),
        "error should say how to resolve it, not just that it failed: {error}"
    );
}

// A change that reaches Verifying without ever closing has no reopen event,
// because nothing ever closed it. Owner correction demanded one, so a change
// whose acceptance inputs touch a file owned by an undeclared module could
// neither finalize (ownership unresolved) nor be corrected (no reopen) nor be
// reopened (reopen takes only Accepted/Archived). Every exit was closed.
//
// The reopen proves definition match against closing verification. A
// never-closed change substitutes a live definition approval so the guided
// path stays reachable (weaker provenance, not audit-equivalent).
#[test]
fn never_closed_verifying_change_corrects_an_owner_without_a_reopen() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();

    // Deliberately no accept and no reopen: the change sits at Verifying,
    // exactly where a reviewed change waits for finalize. `check` materializes
    // the delta before verifying, so a change that reached this point has its
    // canonical application recorded — which is what the real stranded changes
    // look like, and what distinguishes this from a draft.
    record = load_change(root, &record.id).unwrap();
    record.canonical_applied = true;
    save_change(root, &record).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);

    let corrected = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "The definition omitted the canonical owner of an affected path".into(),
    )
    .expect("a never-closed verifying change has no reopen to reference");
    assert_eq!(corrected.acceptance_owner_corrections.len(), 1);
}

#[test]
fn reopened_change_adds_exact_canonical_owner_without_replaying_delivery() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Legacy\n\n## Purpose\n\nLegacy owner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
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
        "Release reviewer".into(),
        "The accepted source changed during release review".into(),
    )
    .unwrap()
    .change;
    record = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "The historical definition omitted the current canonical owner".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 1);
    assert_eq!(record.acceptance_owner_corrections[0].sequence, 1);
    let state_path = change_dir(root, &record.id).join("state.json");
    let corrected_state = fs::read(&state_path).unwrap();
    let duplicate = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Release reviewer".into(),
        "Duplicate owner".into(),
    )
    .unwrap_err();
    assert!(duplicate.contains("duplicate acceptance owner correction"));
    assert_eq!(fs::read(&state_path).unwrap(), corrected_state);

    let mut tampered = record.clone();
    tampered.acceptance_owner_corrections[0].sequence = 2;
    save_change(root, &tampered).unwrap();
    let error = validate_definition(root, &tampered).unwrap_err();
    assert!(error.contains("sequence is not contiguous"), "{error}");
    save_change(root, &record).unwrap();

    let mut broadened = record.clone();
    broadened
        .dependencies
        .push("CHG-9999-unapproved-scope".into());
    save_change(root, &broadened).unwrap();
    let error = ensure_reopened_definition_unchanged(root, &broadened).unwrap_err();
    assert!(error.contains("modified definition"), "{error}");
    save_change(root, &record).unwrap();

    let portable = TempDir::new().unwrap();
    let portable_root = portable.path();
    let portable_dir = change_dir(portable_root, &record.id);
    fs::create_dir_all(portable_dir.join("deltas")).unwrap();
    save_change(portable_root, &record).unwrap();
    for artifact in &record.selected_artifacts {
        fs::copy(
            change_dir(root, &record.id).join(artifact.file_name()),
            portable_dir.join(artifact.file_name()),
        )
        .unwrap();
    }
    assert_eq!(
        definition_digest(root, &record).unwrap(),
        definition_digest(portable_root, &record).unwrap()
    );
    assert!(
        ensure_definition_approval_valid(root, &record)
            .unwrap_err()
            .contains("definition approval is stale")
    );

    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let verification = load_verification(root, &record).unwrap();
    let source = verification
        .acceptance_manifest
        .unwrap()
        .entries
        .into_iter()
        .find(|entry| entry.path == "src/lib.rs")
        .unwrap();
    assert_eq!(source.owners, vec!["change", "legacy"]);
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn ready() -> bool { false }\n"
    );
}

// Verifies REQ-change-039.
#[test]
fn batch_owner_corrections_append_transactionally_or_not_at_all() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, vec!["true".into()]).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() {}\n").unwrap();
    fs::write(root.join("src/b.rs"), "pub fn b() {}\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    let owned = |module: &str| {
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/a.rs\n  - src/b.rs\n  - src/lib.rs\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
        )
    };
    fs::write(root.join("specs/legacy/legacy.spec.md"), owned("legacy")).unwrap();
    fs::write(root.join("specs/current/current.spec.md"), owned("current")).unwrap();
    let mut record = create_change(
        root,
        CreateChangeRequest {
            description: "batch owner correction".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["legacy".into()],
            affected_paths: vec!["src/a.rs".into(), "src/b.rs".into(), "src/lib.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("ownership evidence only".into()),
        },
    )
    .unwrap();
    record.acceptance_criteria = vec!["Owners can be batch-corrected".into()];
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
            "# Complete\n\nReviewed.\n"
        };
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            content,
        )
        .unwrap();
    }
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
        "Release reviewer".into(),
        "Sources changed during release review".into(),
    )
    .unwrap()
    .change;
    let state_path = change_dir(root, &record.id).join("state.json");
    let before = fs::read(&state_path).unwrap();

    let failed = add_acceptance_owner_corrections(
        root,
        &record.id,
        vec![
            ("src/a.rs".into(), "current".into()),
            ("outside/x.rs".into(), "current".into()),
        ],
        "Release reviewer".into(),
        "Batch repair".into(),
    )
    .unwrap_err();
    assert!(
        failed.contains("batch entry 2 failed")
            || failed.contains("outside the original affected path scope"),
        "{failed}"
    );
    assert_eq!(fs::read(&state_path).unwrap(), before);

    record = add_acceptance_owner_corrections(
        root,
        &record.id,
        vec![
            ("src/a.rs".into(), "current".into()),
            ("src/b.rs".into(), "current".into()),
        ],
        "Release reviewer".into(),
        "Batch repair".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 2);
    assert_eq!(record.acceptance_owner_corrections[0].sequence, 1);
    assert_eq!(record.acceptance_owner_corrections[1].sequence, 2);

    fs::write(
            root.join("specs/legacy/legacy.spec.md"),
            "---\nmodule: legacy\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# Legacy\n\n## Purpose\n\nEmpty.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
    let missing = missing_acceptance_owner_paths(root, &record.id, "current").unwrap();
    assert_eq!(missing, vec!["src/lib.rs"]);
    record = add_acceptance_owner_corrections(
        root,
        &record.id,
        missing
            .into_iter()
            .map(|path| (path, "current".into()))
            .collect(),
        "Release reviewer".into(),
        "Discover remaining".into(),
    )
    .unwrap();
    assert_eq!(record.acceptance_owner_corrections.len(), 3);
    assert!(
        missing_acceptance_owner_paths(root, &record.id, "current")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn owner_batch_validation_queries_canonical_module_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    quiet_git(root, &["init", "-b", "main"]);
    quiet_git(root, &["config", "user.email", "test@example.com"]);
    quiet_git(root, &["config", "user.name", "Test"]);
    let paths = (0..20)
        .map(|index| format!("src/owner_{index}.rs"))
        .collect::<Vec<_>>();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    for path in &paths {
        fs::write(root.join(path), "pub fn owned() {}\n").unwrap();
    }
    let files = paths
        .iter()
        .map(|path| format!("  - {path}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
            root.join("specs/current/current.spec.md"),
            format!(
                "---\nmodule: current\nversion: 1\nstatus: stable\nfiles:\n{files}\n---\n\n# Current\n\n## Purpose\n\nOwner fixture.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
            ),
        )
        .unwrap();
    let mut record = completed_no_spec_record(root);
    record.affected_specs = vec!["legacy".into()];
    record.affected_paths = paths.clone();
    record.acceptance_owner_corrections = paths
        .iter()
        .enumerate()
        .map(|(index, path)| AcceptanceOwnerCorrection {
            schema_version: 1,
            sequence: index as u64 + 1,
            path: path.clone(),
            module: "current".into(),
            actor: "Reviewer".into(),
            reason: "Repair historical ownership".into(),
            timestamp: 1,
        })
        .collect();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "owner fixture"]);

    reset_test_canonical_module_query_count();
    reset_test_git_process_count();
    validate_acceptance_owner_corrections_current(root, &record).unwrap();
    let git_queries = test_git_process_count();

    assert_eq!(test_canonical_module_query_count(), 1);
    assert!(
        git_queries <= 40,
        "one module should require one bounded evidence capture, observed {git_queries} Git queries"
    );
}

// Verifies REQ-change-033.
#[test]
fn owner_correction_rejects_invalid_requests_without_mutation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    let state_path = change_dir(root, &record.id).join("state.json");
    let before = fs::read(&state_path).unwrap();

    let wrong_state = add_acceptance_owner_correction(
        root,
        &record.id,
        "src/lib.rs".into(),
        "change".into(),
        "Reviewer".into(),
        "Missing owner".into(),
    )
    .unwrap_err();
    assert!(wrong_state.contains("cannot correct an acceptance input owner"));
    assert_eq!(fs::read(&state_path).unwrap(), before);

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    record = reopen_change(root, &record.id, "Reviewer".into(), "Source changed".into())
        .unwrap()
        .change;
    let reopened = fs::read(&state_path).unwrap();
    let error = add_acceptance_owner_correction(
        root,
        &record.id,
        "outside/file.rs".into(),
        "change".into(),
        "Reviewer".into(),
        "Missing owner".into(),
    )
    .unwrap_err();
    assert!(
        error.contains("outside the original affected path scope"),
        "{error}"
    );
    assert_eq!(fs::read(&state_path).unwrap(), reopened);
}

#[test]
fn broad_successor_without_explicit_obligations_cannot_suppress_stale_predecessor() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut predecessor = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nOriginal governed behavior.\n",
    );
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
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    assert!(check_project(root).errors.is_empty());

    fs::write(
        root.join("src/auth-extra.rs"),
        "// Existing product surface.\n",
    )
    .unwrap();
    let spec_path = root.join("specs/auth/auth.spec.md");
    let expanded = fs::read_to_string(&spec_path).unwrap().replace(
        "  - src/auth.rs\n",
        "  - src/auth.rs\n  - src/auth-extra.rs\n",
    );
    fs::write(&spec_path, expanded).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    let mut successor = create_change(
        root,
        CreateChangeRequest {
            description: "Expand the governed auth surface".into(),
            kind: ChangeKind::BugFix,
            affected_specs: vec!["auth".into()],
            affected_paths: vec!["src/auth.rs".into(), "src/auth-extra.rs".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: false,
            rationale: None,
        },
    )
    .unwrap();
    successor.acceptance_criteria = vec!["Both auth files remain governed".into()];
    successor
        .answers
        .insert("public_contract".into(), "yes".into());
    successor
        .answers
        .insert("architecture_risk".into(), "no".into());
    save_change(root, &successor).unwrap();
    write_change_markdown(root, &successor).unwrap();
    for artifact in &successor.selected_artifacts {
        fs::write(
            change_dir(root, &successor.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Govern the expanded surface.\n"
            } else {
                "# Complete\n\nReviewed successor evidence.\n"
            },
        )
        .unwrap();
    }
    fs::write(
        delta_path(root, &successor, "auth"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nBoth existing auth files remain governed.\n",
    )
    .unwrap();
    successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    successor = start_implementation(root, &successor.id).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    let mut policy = load_policy(root).unwrap();
    policy.verification_commands =
        vec!["cargo metadata --manifest-path definitely-missing/Cargo.toml".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    assert!(verify_change(root, &successor.id).is_err());
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    policy.verification_commands = vec!["true".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    verify_change(root, &successor.id).unwrap();
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));
    successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
    assert_eq!(successor.state, ChangeState::Accepted);
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));
}

#[test]
fn reopen_rejects_current_evidence_and_requires_explicit_audit_fields() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

    let error = reopen_change(root, &record.id, "Reviewer".into(), "Not stale".into()).unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    let error = reopen_change(root, &record.id, " ".into(), "Review fix".into()).unwrap_err();
    assert!(error.contains("non-empty human actor"), "{error}");
    let error = reopen_change(root, &record.id, "Reviewer".into(), " ".into()).unwrap_err();
    assert!(error.contains("non-empty reason"), "{error}");
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
}

/// A reopen that is correctly refused must not consume the dated archive package. The
/// un-archive is a move performed before the preconditions are known to hold, so every
/// failure after it has to put the package back; otherwise the refusal leaves an active
/// orphan whose state.json still says `archived` and no verb can recover it.
#[test]
fn refused_reopen_restores_the_archived_package() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept definition"]);
    let archived = archive_change(root, &record.id).unwrap();
    assert!(archived.is_dir());

    // Delivery inputs are still current, so this reopen is refused on purpose.
    let error = reopen_change(root, &record.id, "Reopener".into(), "probe".into()).unwrap_err();
    assert!(error.contains("delivery inputs are current"), "{error}");
    assert!(error.contains("archive restored"), "{error}");
    assert!(
        archived.is_dir(),
        "the refused reopen consumed the archive package at {}",
        archived.display()
    );
    assert!(
        !change_dir(root, &record.id).exists(),
        "the refused reopen left an active orphan"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Archived
    );

    // The refusal stays the same diagnostic instead of degrading into an orphan collision.
    let retry = reopen_change(root, &record.id, "Reopener".into(), "retry".into()).unwrap_err();
    assert!(retry.contains("delivery inputs are current"), "{retry}");
    assert!(
        !retry.contains("an active change directory already exists"),
        "{retry}"
    );

    // Control: the un-archive is still reachable when the reopen can succeed, so the
    // restore above cannot be passing by never un-archiving at all.
    git(&["add", "."]);
    git(&["commit", "-m", "archive tip"]);
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "drift input"]);
    let reopened = reopen_change(root, &record.id, "Reopener".into(), "drifted".into()).unwrap();
    assert_eq!(reopened.change.state, ChangeState::Verifying);
    assert!(change_dir(root, &record.id).is_dir());
    assert!(!archived.exists());
}

#[test]
fn reaccept_rejects_definition_changes_after_canonical_application() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
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
        "Release reviewer".into(),
        "Review fixes changed scoped delivery inputs".into(),
    )
    .unwrap()
    .change;
    fs::write(
        change_dir(root, &record.id).join("testing.md"),
        "# Testing\n\nThe modified definition must not be silently ignored.\n",
    )
    .unwrap();
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    assert_eq!(record.state, ChangeState::Verifying);
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains("modified definition of an already-applied change") })
    );
    verify_change(root, &record.id).unwrap();

    let error = accept_change(root, &record.id, Some("Closer".into()), None).unwrap_err();
    assert!(
        error.contains("perform further spec changes in a new change workspace"),
        "{error}"
    );
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Verifying
    );
}

// Verifies REQ-change-032.
#[test]
fn accepted_metadata_correction_preserves_original_evidence_and_adds_artifacts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let record = accept_completed_record(root, completed_no_spec_record(root));
    let original_answers = record.answers.clone();
    let original_artifacts = record.selected_artifacts.clone();
    let original_approvals = serde_json::to_value(load_approvals(root, &record).unwrap()).unwrap();
    let prior_verification = load_verification(root, &record).unwrap();

    let result = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted change affects persisted lifecycle architecture".into(),
    )
    .unwrap();

    assert_eq!(result.change.state, ChangeState::Verifying);
    assert!(result.change.canonical_applied);
    assert_eq!(result.change.correction_count, 1);
    assert_eq!(result.change.answers, original_answers);
    assert_eq!(result.change.selected_artifacts, original_artifacts);
    assert_eq!(result.correction.original_value, "no");
    assert_eq!(result.correction.prior_effective_value, "no");
    assert_eq!(result.correction.corrected_value, "yes");
    assert_eq!(
        result.correction.prior_verification.contract_digest,
        prior_verification.contract_digest
    );
    assert_eq!(
        result.correction.added_artifacts,
        vec![
            ArtifactKind::Research,
            ArtifactKind::Design,
            ArtifactKind::Plan,
        ]
    );
    assert_eq!(
        result
            .effective_definition
            .answers
            .get("architecture_risk")
            .map(String::as_str),
        Some("yes")
    );
    for artifact in ["research.md", "design.md", "plan.md"] {
        assert!(result.summary.next_action.contains(artifact));
    }
    assert!(!result.summary.approval_valid);
    assert_eq!(
        serde_json::to_value(load_approvals(root, &result.change).unwrap()).unwrap(),
        original_approvals
    );
    for artifact in &result.correction.added_artifacts {
        let content =
            fs::read_to_string(change_dir(root, &record.id).join(artifact.file_name())).unwrap();
        assert!(content.contains("<!-- TODO"));
    }
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Release reviewer".into(),
        "A second correction cannot bypass reacceptance".into(),
    )
    .unwrap_err();
    assert!(error.contains("expected accepted"), "{error}");
}

#[test]
fn metadata_correction_rejects_noops_unsupported_fields_and_missing_audit_inputs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let record = accept_completed_record(root, completed_no_spec_record(root));

    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "no".into(),
        "Reviewer".into(),
        "No change".into(),
    )
    .unwrap_err();
    assert!(error.contains("already `no`"), "{error}");
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        " ".into(),
        "Missing actor".into(),
    )
    .unwrap_err();
    assert!(error.contains("non-empty human actor"), "{error}");
    let error = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        " ".into(),
    )
    .unwrap_err();
    assert!(error.contains("non-empty reason"), "{error}");
    assert!(CorrectionField::parse("acceptance_criteria").is_err());
    assert!(!change_dir(root, &record.id).join(CORRECTIONS_FILE).exists());
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Accepted
    );
}

#[test]
fn correction_values_preserve_supported_boolean_aliases() {
    for value in ["yes", "y", "true", "1", " YES "] {
        assert_eq!(canonical_correction_value(value).unwrap(), "yes");
    }
    for value in ["no", "n", "false", "0", " NO "] {
        assert_eq!(canonical_correction_value(value).unwrap(), "no");
    }
    assert!(canonical_correction_value("maybe").is_err());
}

#[test]
fn corrected_acceptance_requires_fresh_gates_and_never_replays_canonical_deltas() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nCorrected metadata never replays this canonical section.\n";
    let mut record = completed_section_only_record(root, delta);
    record.answers.insert("public_contract".into(), "no".into());
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    record = accept_completed_record(root, record);
    let canonical_path = root.join("specs/auth/auth.spec.md");
    let canonical_after_first_accept = fs::read_to_string(&canonical_path).unwrap();

    let first = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Release reviewer".into(),
        "The accepted semantic delta changed the public contract".into(),
    )
    .unwrap();
    assert!(first.correction.added_artifacts.is_empty());
    assert_eq!(
        first.summary.next_action,
        format!("run `specsync change approve {} --actor <name>`", record.id)
    );
    assert!(ensure_definition_approval_valid(root, &first.change).is_err());
    record =
        approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
    assert_eq!(
        fs::read_to_string(&canonical_path).unwrap(),
        canonical_after_first_accept
    );
    assert_eq!(correction_history(root, &record).unwrap().len(), 1);

    let second = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "no".into(),
        "Release reviewer".into(),
        "A later audit restored the original classification".into(),
    )
    .unwrap();
    assert_eq!(second.correction.sequence, 2);
    record =
        approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();

    assert_eq!(
        record.answers.get("public_contract").map(String::as_str),
        Some("no")
    );
    assert_eq!(record.correction_count, 2);
    assert_eq!(correction_history(root, &record).unwrap().len(), 2);
    assert_eq!(
        effective_change_definition(root, &record)
            .unwrap()
            .answers
            .get("public_contract")
            .map(String::as_str),
        Some("no")
    );
    assert_eq!(
        fs::read_to_string(canonical_path).unwrap(),
        canonical_after_first_accept
    );
}

// Verifies REQ-change-032.
#[test]
fn trusted_history_rejects_correction_rollback_and_divergent_same_count() {
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
    git(&["config", "core.autocrlf", "false"]);
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nAccepted passkey changes retain lifecycle evidence.\n",
    );
    record.answers.insert("public_contract".into(), "no".into());
    save_change(root, &record).unwrap();
    write_change_markdown(root, &record).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    let first = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted implementation has architectural risk".into(),
    )
    .unwrap();
    for artifact in &first.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept first correction"]);
    let first_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let second = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "no".into(),
        "Reviewer".into(),
        "A follow-up audit removed the architectural risk".into(),
    )
    .unwrap();
    assert_eq!(second.correction.sequence, 2);
    assert!(effective_change_definition(root, &second.change).is_ok());
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept second correction"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let second_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    let workspace = format!("{CHANGES_PATH}/{}", record.id);

    git(&[
        "restore",
        "--source",
        first_commit.as_str(),
        "--",
        &workspace,
    ]);
    let rolled_back = load_change(root, &record.id).unwrap();
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
    git(&[
        "restore",
        "--source",
        second_commit.as_str(),
        "--",
        &workspace,
    ]);

    git(&["switch", "-c", "divergent", first_commit.as_str()]);
    let stale = load_change(root, &record.id).unwrap();
    let error = effective_change_definition(root, &stale).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
    git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    let divergent = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Reviewer".into(),
        "The public contract classification was corrected independently".into(),
    )
    .unwrap();
    git(&[
        "update-ref",
        "refs/remotes/origin/main",
        second_commit.as_str(),
    ]);
    let error = effective_change_definition(root, &divergent.change).unwrap_err();
    assert!(
        error.contains("correction history divergence detected"),
        "expected correction history divergence"
    );
}

// Verifies REQ-change-032.
#[test]
fn full_history_finds_a_corrected_anchor_hidden_by_a_treesame_merge_result() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);
    let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    git(&["switch", "-c", "corrected-side-branch"]);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(&[
        "restore",
        "--source",
        original.as_str(),
        "--",
        workspace.as_str(),
    ]);
    git(&["add", "."]);
    git(&["commit", "-m", "roll back corrected workspace"]);
    git(&["switch", "main"]);
    git(&[
        "merge",
        "--no-ff",
        "corrected-side-branch",
        "-m",
        "merge side-branch history",
    ]);

    let rolled_back = load_change(root, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn trusted_history_ignores_a_dangling_remote_default_symbolic_ref() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let record = accept_completed_record(root, completed_no_spec_record(root));
    git(&["add", "."]);
    git(&["commit", "-m", "accept definition"]);
    git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/missing",
    ]);

    assert!(effective_change_definition(root, &record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn historical_git_paths_are_nul_safe_in_a_non_ascii_project_directory() {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    let fixture = if cfg!(windows) {
        "fixtures/naïve quoted"
    } else {
        "fixtures/naïve \"quoted\""
    };
    let root = repository.join(fixture);
    fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repository)
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
    write_default_policy(&root, Vec::new()).unwrap();
    let mut record = completed_section_only_record(
        &root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nHistorical paths remain byte-delimited.\n",
    );
    record.answers.insert("public_contract".into(), "no".into());
    save_change(&root, &record).unwrap();
    write_change_markdown(&root, &record).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(&root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    record = correct_interview_metadata(
        &root,
        &record.id,
        CorrectionField::PublicContract,
        "yes".into(),
        "Reviewer".into(),
        "The accepted semantic delta changes a public contract".into(),
    )
    .unwrap()
    .change;
    record = approve_definition(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(&root, &record.id).unwrap();
    record = accept_change(&root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);

    let commit = git_output(&root, &["rev-parse", "HEAD"]).unwrap();
    let directory =
        git_repo_relative_path(&root, &format!("{CHANGES_PATH}/{}", record.id)).unwrap();
    assert_eq!(
        historical_change_directories(&root, &commit, &record.id).unwrap(),
        vec![directory.clone()]
    );
    assert!(
        closing_authenticated_correction_anchor(&root, &commit, &directory, &record.id)
            .unwrap()
            .is_some()
    );
    assert!(
        git_entry_at_commit(&root, &commit, &format!("{directory}/state.json"))
            .unwrap()
            .is_some()
    );
    assert!(effective_change_definition(&root, &record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn archived_change_uses_prior_active_correction_anchor() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept corrected definition"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let archive = archive_change(root, &record.id).unwrap();
    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    assert!(effective_change_definition(root, &archived).is_ok());

    let mut ledger = load_correction_ledger(root, &archived).unwrap();
    ledger.corrections.clear();
    write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
    let mut rolled_back = archived;
    rolled_back.correction_count = 0;
    write_json(&archive.join("state.json"), &rolled_back).unwrap();
    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn archived_only_corrected_snapshot_remains_a_trusted_anchor() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);

    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    let archive = archive_change(root, &record.id).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record only archived corrected snapshot"]);
    assert!(
        git_output(
            root,
            &[
                "log",
                "--format=%H",
                "--",
                &format!("{CHANGES_PATH}/{}/corrections.json", record.id),
            ],
        )
        .is_none()
    );

    let archived = load_change(root, &record.id).unwrap();
    assert_eq!(archived.state, ChangeState::Archived);
    let mut ledger = load_correction_ledger(root, &archived).unwrap();
    ledger.corrections.clear();
    write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
    let mut rolled_back = archived;
    rolled_back.correction_count = 0;
    write_json(&archive.join("state.json"), &rolled_back).unwrap();

    let error = effective_change_definition(root, &rolled_back).unwrap_err();
    assert!(
        error.contains("correction history rollback detected"),
        "{error}"
    );
}

// Verifies REQ-change-032.
#[test]
fn shallow_history_with_corrections_fails_closed() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    let head = git_output(root, &["rev-parse", "HEAD"]).unwrap();
    fs::write(root.join(".git/shallow"), format!("{head}\n")).unwrap();
    let error = effective_change_definition(root, &correction.change).unwrap_err();
    assert!(
        error.contains("shallow Git checkout"),
        "expected shallow Git checkout rejection"
    );
}

// Verifies REQ-change-032.
#[test]
fn shallow_rollback_tip_cannot_hide_a_corrected_acceptance() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let clone = temp.path().join("clone");
    fs::create_dir_all(&source).unwrap();
    let git = |root: &Path, args: &[&str]| {
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
    git(&source, &["init", "-b", "main"]);
    git(&source, &["config", "user.email", "test@example.com"]);
    git(&source, &["config", "user.name", "Test"]);
    fs::write(source.join("README.md"), "# Fixture\n").unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "base"]);
    write_default_policy(&source, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(&source);
    record = accept_completed_record(&source, record);
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "accept original definition"]);
    let original = git_output(&source, &["rev-parse", "HEAD"]).unwrap();

    let correction = correct_interview_metadata(
        &source,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(&source, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(&source, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(&source, &record.id).unwrap();
    record = accept_change(&source, &record.id, Some("Reviewer".into()), None).unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "accept corrected definition"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(
        &source,
        &["restore", "--source", original.as_str(), "--", &workspace],
    );
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "rollback lifecycle workspace"]);
    let source_url = format!("file://{}", source.display());
    git(
        temp.path(),
        &[
            "clone",
            "--depth",
            "1",
            source_url.as_str(),
            clone.to_str().unwrap(),
        ],
    );
    let rolled_back = load_change(&clone, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    let error = effective_change_definition(&clone, &rolled_back).unwrap_err();
    assert!(error.contains("incomplete shallow Git checkout"), "{error}");

    let new_record = create_change(
        &clone,
        CreateChangeRequest {
            description: "Add a local shallow-checkout note".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Local documentation only".into()),
        },
    )
    .unwrap();
    assert!(effective_change_definition(&clone, &new_record).is_ok());
}

// Verifies REQ-change-032.
#[test]
fn accepted_snapshot_with_a_stale_contract_is_not_an_anchor() {
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base definition"]);
    record = accept_completed_record(root, record);
    git(&["add", "."]);
    git(&["commit", "-m", "accept original definition"]);
    let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

    let correction = correct_interview_metadata(
        root,
        &record.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Reviewer".into(),
        "The accepted verification path has architectural risk".into(),
    )
    .unwrap();
    for artifact in &correction.correction.added_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Review the corrected classification.\n"
            } else {
                "# Complete\n\nThe corrected classification was reviewed.\n"
            },
        )
        .unwrap();
    }
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    verify_change(root, &record.id).unwrap();
    record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
    fs::write(
        change_dir(root, &record.id).join("testing.md"),
        "# Complete\n\nChanged after the accepted contract was approved.\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "record stale accepted contract snapshot"]);

    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    git(&["restore", "--source", original.as_str(), "--", &workspace]);
    let rolled_back = load_change(root, &record.id).unwrap();
    assert_eq!(rolled_back.correction_count, 0);
    assert!(effective_change_definition(root, &rolled_back).is_ok());
}

#[test]
fn correction_ledgers_fail_closed_and_hash_portably() {
    let first = TempDir::new().unwrap();
    let root = first.path();
    write_default_policy(root, Vec::new()).unwrap();
    let accepted = accept_completed_record(root, completed_no_spec_record(root));
    let result = correct_interview_metadata(
        root,
        &accepted.id,
        CorrectionField::ArchitectureRisk,
        "yes".into(),
        "Release reviewer".into(),
        "Architecture classification was stale".into(),
    )
    .unwrap();
    let record = result.change;
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let valid_ledger = fs::read_to_string(&ledger_path).unwrap();

    fs::remove_file(&ledger_path).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("does not match state.json correction_count"),
        "expected correction_count mismatch"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut tampered: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    tampered["corrections"][0]["sequence"] = serde_json::json!(2);
    write_json(&ledger_path, &tampered).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("sequence is not contiguous"),
        "expected non-contiguous correction sequence"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut unsupported: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    unsupported["corrections"][0]["field"] = serde_json::json!("acceptance_criteria");
    write_json(&ledger_path, &unsupported).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("invalid correction ledger"),
        "expected invalid correction ledger"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let mut tampered_definition: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
    tampered_definition["corrections"][0]["superseded_definition_approval"]["digest"] =
        serde_json::json!("forged-definition-digest");
    write_json(&ledger_path, &tampered_definition).unwrap();
    let error = effective_change_definition(root, &record).unwrap_err();
    assert!(
        error.contains("invalid prior gate evidence"),
        "expected invalid prior gate evidence"
    );
    fs::write(&ledger_path, &valid_ledger).unwrap();

    let second = TempDir::new().unwrap();
    let second_root = second.path();
    let second_dir = change_dir(second_root, &record.id);
    fs::create_dir_all(second_dir.join("deltas")).unwrap();
    save_change(second_root, &record).unwrap();
    fs::write(second_dir.join(CORRECTIONS_FILE), valid_ledger).unwrap();
    let effective = effective_change_definition(root, &record).unwrap();
    for artifact in &effective.selected_artifacts {
        let content = fs::read(change_dir(root, &record.id).join(artifact.file_name())).unwrap();
        fs::write(second_dir.join(artifact.file_name()), content).unwrap();
    }
    assert_eq!(
        definition_digest(root, &record).unwrap(),
        definition_digest(second_root, &record).unwrap()
    );
}

// Verifies REQ-change-056.
#[test]
fn text_correction_ledger_health_hides_invalid_ledger_detail() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let record = completed_no_spec_record(root);
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);

    assert!(effective_change_definition(root, &record).is_ok());

    fs::write(&ledger_path, "{ malformed correction ledger\n").unwrap();

    assert!(effective_change_definition(root, &record).is_err());
}

// Verifies REQ-change-057.
#[test]
fn mutation_rechecks_correction_ledger_after_lock_acquisition() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let record = completed_no_spec_record(root);
    let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
    let state_path = change_dir(root, &record.id).join("state.json");
    let change_path = change_dir(root, &record.id).join("change.md");

    let project_lock = acquire_project_lock(root).unwrap();
    let mutation_root = root.to_path_buf();
    let mutation_id = record.id.clone();
    let (started_sender, started_receiver) = std::sync::mpsc::channel();
    let mutation = std::thread::spawn(move || {
        started_sender.send(()).unwrap();
        answer_question(
            &mutation_root,
            &mutation_id,
            "acceptance_criteria",
            "This mutation must not persist",
        )
    });

    started_receiver.recv().unwrap();
    fs::write(&ledger_path, "{ malformed correction ledger\n").unwrap();
    let state_before = fs::read(&state_path).unwrap();
    let change_before = fs::read(&change_path).unwrap();
    drop(project_lock);

    let error = mutation.join().unwrap().unwrap_err();
    assert_eq!(error, INVALID_CORRECTION_LEDGER_TEXT);
    assert_eq!(fs::read(&state_path).unwrap(), state_before);
    assert_eq!(fs::read(&change_path).unwrap(), change_before);
}

#[test]
fn acceptance_rechecks_late_dependency_state() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let dependency = create_change(
        root,
        CreateChangeRequest {
            description: "unfinished prerequisite".into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ops/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Ordering fixture".into()),
        },
    )
    .unwrap();
    record = load_change(root, &record.id).unwrap();
    record.dependencies.push(dependency.id.clone());
    save_change(root, &record).unwrap();
    append_approval(
        root,
        &record,
        "definition",
        Some("Reviewer".into()),
        definition_digest(root, &record).unwrap(),
        Some("Approved late ordering change".into()),
    )
    .unwrap();
    let mut evidence = load_verification(root, &record).unwrap();
    evidence.contract_digest = definition_digest(root, &record).unwrap();
    evidence.execution_digest = Some(execution_digest(root, &record).unwrap());
    evidence.workspace_digest = project_input_digest(root).unwrap();
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &evidence,
    )
    .unwrap();
    let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
    assert!(error.contains("must be accepted"), "{error}");
}

#[test]
fn failed_evidence_keeps_local_check_red() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_default_policy(root, Vec::new()).unwrap();
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    let mut evidence = verify_change(root, &record.id).unwrap();
    evidence.passed = false;
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &evidence,
    )
    .unwrap();
    let report = check_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("latest verification evidence failed"))
    );
}

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
            "# Complete\n\nReviewed content.\n"
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
    assert!(spec.contains(&format!("| 2026-01-01 | Initial |\n| {} |", today())));
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

#[test]
fn direct_recursive_verification_command_is_rejected_before_execution() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands = vec!["specsync check --strict".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let error = verify_change(root, &record.id).unwrap_err();

    assert!(error.contains("recursive lifecycle verification command"));
    assert!(error.contains("specsync check --strict"));
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Implementing
    );
    assert!(
        !change_dir(root, &record.id)
            .join("verification.json")
            .exists()
    );
    assert!(
        !change_dir(root, &record.id)
            .join("verification-attempts.json")
            .exists()
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

    let error = verify_change(root, &record.id).unwrap_err();

    assert!(error.contains("recursive lifecycle verification command"));
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Implementing
    );
    assert!(
        !change_dir(root, &record.id)
            .join("verification.json")
            .exists()
    );
    assert!(
        !change_dir(root, &record.id)
            .join("verification-attempts.json")
            .exists()
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
    assert!(answered.affected_paths.contains(&"src/lib.rs".into()));
    assert!(answered.affected_paths.contains(&SEQUENCE_PATH.into()));
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
    assert!(answered.affected_paths.contains(&SEQUENCE_PATH.into()));
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
fn failed_native_verification_is_retryable_with_append_only_history() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    policy.verification_commands =
        vec!["cargo metadata --manifest-path definitely-missing/Cargo.toml".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
    let mut record = completed_section_only_record(
        root,
        "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
    );
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();

    let first_error = verify_change(root, &record.id).unwrap_err();
    // The failure names the exact command and where its evidence lives, so an
    // author does not have to open verification.json to learn which step failed.
    assert!(first_error.contains("cargo metadata --manifest-path definitely-missing/Cargo.toml"));
    assert!(first_error.contains("verification.json"));
    assert_eq!(
        load_change(root, &record.id).unwrap().state,
        ChangeState::Verifying
    );

    policy.verification_commands = vec!["true".into()];
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
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
                "# Complete\n\nReviewed.\n"
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
    for artifact in &record.selected_artifacts {
        fs::write(
            change_dir(root, &record.id).join(artifact.file_name()),
            "# Complete\n\nReviewed.\n",
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
            "# Complete\n\nReviewed.\n",
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
            "# Complete\n\nReady for approval.\n"
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
                "# Complete\n\nReviewed.\n"
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

    let mut first = completed_no_spec_record(root);
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
    let mut record = current_workflow_record(root, completed_no_spec_record(root));
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".specsync".into()];
    save_change(root, &record).unwrap();
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
    let mut predecessor = completed_no_spec_current_record(root);
    predecessor.state = ChangeState::Implementing;
    predecessor.affected_paths = vec![SEQUENCE_PATH.into()];
    save_change(root, &predecessor).unwrap();
    let before = acceptance_input_digest(root, &predecessor, &[]).unwrap();

    let mut successor = create_change(
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
    write_default_policy(root, Vec::new()).unwrap();
    let mut legacy_policy = load_policy(root).unwrap();
    legacy_policy.version = 1;
    write_json(&root.join(POLICY_PATH), &legacy_policy).unwrap();

    let mut predecessor = completed_no_spec_record(root);
    git(&["add", "."]);
    git(&["commit", "-m", "base predecessor"]);
    predecessor = approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
    predecessor = start_implementation(root, &predecessor.id).unwrap();
    verify_change(root, &predecessor.id).unwrap();
    predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let mut intermediate = completed_no_spec_record(root);
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
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: 2,
            id: intermediate.id.clone(),
            acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
        },
    )
    .unwrap();

    let mut owner = completed_no_spec_record(root);
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
    let mut record = completed_no_spec_record(root);
    record.state = ChangeState::Implementing;
    record.affected_paths = vec![".specsync".into()];
    save_change(root, &record).unwrap();
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
    let mut record = completed_no_spec_record(root);
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
    let prepared = prepare_delta_application(root, &record).unwrap();
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

    let prepared = prepare_delta_application(root, &record).unwrap();

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

    let prepared = prepare_delta_application(root, &record).unwrap();
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

    let error = prepare_delta_application(root, &record).unwrap_err();

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
    assert!(error.contains("historical semantic delta is empty"));

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
            "# Complete\n\nReviewed.\n",
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
    let mut record = completed_no_spec_record(root);
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
            "run `specsync change review {id} --reviewer <independent-reviewer>` after the PR's scoped review passes"
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
fn persisted_scoped_review_rejects_scope_approver_as_reviewer() {
    let (temp, id, _) = verification_history_fixture();
    let root = temp.path();
    let verification_paths = verification_persistence_paths(&id);
    let verification_path_refs: Vec<&str> = verification_paths.iter().map(String::as_str).collect();
    commit_paths(root, &verification_path_refs, "persist verification");
    let mut review = record_scoped_review(root, &id, "Independent Reviewer".into()).unwrap();
    review.reviewer = "reviewer".into();
    let attempts = ScopedReviewAttemptLedger {
        schema_version: 1,
        reviews: vec![review.clone()],
    };
    let record = load_change(root, &id).unwrap();
    fs::write(
        scoped_review_path(root, &record),
        json_content(&review).unwrap(),
    )
    .unwrap();
    fs::write(
        scoped_review_attempts_path(root, &record),
        json_content(&attempts).unwrap(),
    )
    .unwrap();

    let error = load_scoped_review(root, &record).unwrap_err();
    assert!(error.contains("also the scope approver"), "{error}");
    let error =
        accept_change_with_gate(root, &id, None, None, "finalization", true, true).unwrap_err();
    assert!(error.contains("also the scope approver"), "{error}");
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
    write_default_policy(root, Vec::new()).unwrap();
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
