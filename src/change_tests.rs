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

// Honest label: DISCRIMINATOR for #716, the other end of the same mistake and the destructive
// one. A trailing space on the CLOSING delimiter sent the scan past the real end of the block and
// stopped it at the first horizontal rule in the body, deleting the prose above it. What survived
// here is a bare `TODO`, so a written design was refused as "artifact is incomplete" and the
// author had no way to see why.
//
// Discriminates: on the unfixed binary this asserts `false` on a `true` — the artifact is refused.
#[test]
fn a_padded_closing_delimiter_does_not_delete_the_prose_above_a_body_horizontal_rule() {
    let content = "---\nchange: CHG-1\nartifact: design\n---  \n\n\
# Design\n\nThe retry budget is per-host, not per-request.\n\n---\n\nTODO\n";

    assert!(
        !artifact_content_is_incomplete(content),
        "written prose above a body horizontal rule must survive a padded closing delimiter"
    );
}

// Honest label: CHARACTERIZATION. It passes on the unfixed binary too — that is why it is here.
//
// #696 replaced this module's own `strip_yaml_frontmatter`, which had no BOM trim, so a
// BOM-prefixed artifact kept its frontmatter, the YAML counted as prose, and an artifact whose
// body was empty or only TODO passed the completeness gate. The replacement fixed that without
// claiming it and without a test. Undisclosed correct behaviour is still undisclosed; this is the
// record of it (#716).
#[test]
fn a_bom_prefixed_artifact_with_no_written_body_is_incomplete() {
    assert!(artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n"
    ));
    assert!(artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n\nTODO\n"
    ));
    // ...and a BOM must not make a WRITTEN artifact read as incomplete either.
    assert!(!artifact_content_is_incomplete(
        "\u{feff}---\nchange: CHG-1\nartifact: design\n---\n\nThe retry budget is per-host.\n"
    ));
}

// Honest label: CHARACTERIZATION of a KNOWN RESIDUAL, and it passes on the unfixed binary. It
// asserts a WRONG verdict on purpose, because the alternative is worse.
//
// `----` is a legal Markdown thematic break. Treating it as an opening delimiter would make the
// stripper scan forward to the next rule and return a body cut at it, which is the failure the
// canonical reader exists to prevent (#697, #699, #705) — so an artifact that is nothing but
// frontmatter opened with `----` still reads as complete here. Deriving the gate from the
// generated scaffold instead does not close this either: a file with a mangled opener no longer
// equals the scaffold, so it would read as written for the same reason.
//
// If this test ever fails, the hole closed — check that it closed for a defensible reason and
// delete the test, do not "restore" it.
#[test]
fn a_four_dash_opener_still_hides_an_empty_artifact_from_the_gate() {
    assert!(!artifact_content_is_incomplete(
        "----\nchange: CHG-1\nartifact: design\n---\n"
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
    write_lifecycle_test_policy(temp.path());
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
    write_lifecycle_test_policy(root);
    let mut policy = load_policy(root).unwrap();
    policy.require_change_for_meaningful_files = true;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
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
    // The ledger used to appear in the working tree because `change new` wrote it. Nothing
    // writes it now, so the fixture puts it there: the subject of this test is a *changed*
    // protected path that an approved change scopes, and it needs one to be changed.
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(SEQUENCE_PATH), "{}\n").unwrap();
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    let mut policy = SddPolicy::default();
    policy.require_change_for_meaningful_files = false;
    write_json(&root.join(POLICY_PATH), &policy).unwrap();
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

    // A failing verification must not clear the stale-predecessor finding. The
    // failure has to sit inside the successor's own scope — `change check` is
    // scoped, so drift in a module it neither declares nor maps is ignored.
    // `auth_extra` maps `src/auth-extra.rs`, one of the successor's declared
    // paths, so the scoped spec↔code pass is what fails.
    let drifted_spec = root.join("specs/auth_extra/auth_extra.spec.md");
    fs::create_dir_all(drifted_spec.parent().unwrap()).unwrap();
    fs::write(
        &drifted_spec,
        PHANTOM_AUTH_SPEC
            .replace("module: auth", "module: auth_extra")
            .replace("src/auth.rs", "src/auth-extra.rs")
            .replace("# Auth", "# Auth Extra"),
    )
    .unwrap();
    assert!(verify_change(root, &successor.id).is_err());
    assert!(check_project(root).errors.iter().any(|error| {
        error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
    }));

    fs::remove_file(&drifted_spec).unwrap();
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(&root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(&source);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    write_lifecycle_test_policy(root);
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
    // `HandoffSignals` carries no "verification committed" signal: currency is a
    // content question, and `change check` without `--commit` leaves the evidence
    // untracked. The reason may only claim what the classifier checked.
    assert!(!awaiting_review.reason.contains("verification is committed"));
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
