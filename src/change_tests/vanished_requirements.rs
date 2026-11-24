// #1081 (draft, red first): a change never deletes a canonical requirement that no approved
// `## REMOVED` delta names. In CorvidLabs/starling PR #43 a setup script truncated
// `requirements.md` to the change's one added block, and every check passed.

const VANISHED_BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\n### REQ-auth-002\n\nThe system SHALL expire a session after one day.\n";

/// The starling shape: the change adds REQ-auth-003, is approved and pushed as the default
/// branch's state, and then the branch's `requirements.md` is truncated to the header and the
/// added block. `change check` must refuse before writing, naming REQ-auth-001 and REQ-auth-002.
#[test]
fn a_branch_that_lost_requirements_no_removed_delta_names_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(
        root,
        "## ADDED\n\n### REQUIREMENT REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n",
    );
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "approved"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    // The setup script truncates the file before appending the new block.
    let truncated = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n";
    fs::write(requirements_path(root), truncated).unwrap();
    git(&["commit", "-q", "-am", "truncate and append"]);

    let error = materialize_change_deltas(root, &record.id)
        .expect_err("a branch that lost REQ-auth-001 and REQ-auth-002 must be refused");

    assert!(
        error.contains("REQ-auth-001") && error.contains("REQ-auth-002"),
        "{error}"
    );
    assert!(error.contains("REMOVED"), "{error}");
    assert!(error.contains("specs/auth/requirements.md"), "{error}");
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), truncated);
    // #847: the refused check leaves a refused attempt behind.
    let (_, history) = load_verification_attempt_ledger(root, &record).unwrap();
    assert!(
        history.attempts.last().is_some_and(|attempt| !attempt.passed),
        "{:?}",
        history.attempts
    );
    // The health check reports the same refusal (CHANGE-17.b).
    let report = audit_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|line| line.starts_with(&format!("{}:", record.id)) && line.contains("REQ-auth-001")),
        "{:?}",
        report.errors
    );
}

const ADDED_THIRD: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n";

/// A repository whose `origin/main` holds `VANISHED_BASE_REQUIREMENTS` (plus `extra` files) with
/// the change approved on it, checked out on branch `change`.
fn vanished_repository(root: &Path, delta: &str, extra: &[(&str, &str)]) -> ChangeRecord {
    vanished_repository_with_base(root, VANISHED_BASE_REQUIREMENTS, delta, extra)
}

/// As `vanished_repository`, with `base` as the canonical requirements.
fn vanished_repository_with_base(
    root: &Path,
    base: &str,
    delta: &str,
    extra: &[(&str, &str)],
) -> ChangeRecord {
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), base).unwrap();
    for (path, text) in extra {
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        fs::write(root.join(path), text).unwrap();
    }
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "approved"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    record
}

fn drop_block(text: &str, id: &str) -> String {
    let start = text.find(&format!("### {id}\n")).unwrap();
    let end = text[start + 4..]
        .find("\n### ")
        .map_or(text.len(), |offset| start + 4 + offset + 1);
    format!("{}{}", &text[..start], &text[end..])
}

/// REQ-change-169: an approved `## REMOVED` item of the change accounts for the requirement it
/// removes, so the check that applies it passes.
#[test]
fn a_requirement_an_approved_removed_item_names_is_not_lost() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-002\n", &[]);
    materialize_change_deltas(root, &record.id).unwrap();
    assert!(!fs::read_to_string(requirements_path(root)).unwrap().contains("REQ-auth-002"));
    assert_eq!(ensure_no_lost_requirements(root, &load_change(root, &record.id).unwrap()), Ok(None));
}

/// REQ-change-169: a requirement another change removed on the default branch is not this
/// branch's loss once the branch has merged it in; cherry-picking the removal alone is refused.
#[test]
fn a_removal_merged_in_from_main_is_not_lost_but_a_cherry_pick_is() {
    for merge in [true, false] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = vanished_repository(root, ADDED_THIRD, &[]);
        let git = git_in(root);
        git(&["checkout", "-q", "main"]);
        let removed = drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001");
        fs::write(requirements_path(root), &removed).unwrap();
        git(&["commit", "-q", "-am", "main removes REQ-auth-001"]);
        git(&["push", "-q", "origin", "main"]);
        git(&["checkout", "-q", "change"]);
        if merge {
            git(&["merge", "-q", "--no-edit", "origin/main"]);
        } else {
            fs::write(requirements_path(root), &removed).unwrap();
            git(&["commit", "-q", "-am", "the same removal, cherry-picked"]);
        }
        let outcome = materialize_change_deltas(root, &record.id);
        if merge {
            outcome.unwrap();
        } else {
            let error = outcome.unwrap_err();
            assert!(error.contains("REQ-auth-001"), "{error}");
            // The default branch removed it too, so the way out is to merge it (Leif's rule 3; the
            // tester's L2b at 660066e1), not to restore the block or name it in a REMOVED delta.
            assert!(
                error.contains("origin/main has removed REQ-auth-001 too")
                    && error.contains("merge origin/main into this branch (`git merge origin/main`)")
                    && !error.contains("Restore each block"),
                "{error}"
            );
        }
    }
}

#[test]
fn a_merge_hint_quotes_the_default_ref_in_both_diagnostics() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    let remote_ref = "refs/remotes/origin/review's-default";
    git(&["checkout", "-q", "main"]);
    let removed = drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001");
    fs::write(requirements_path(root), &removed).unwrap();
    git(&["commit", "-q", "-am", "default branch removes REQ-auth-001"]);
    git(&["update-ref", remote_ref, "HEAD"]);
    git(&["symbolic-ref", "refs/remotes/origin/HEAD", remote_ref]);
    git(&["checkout", "-q", "change"]);

    // Inspect the rendered command only. The ref's apostrophe is literal text.
    let expected = "`git merge 'origin/review'\\''s-default'`";
    fs::write(requirements_path(root), &removed).unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("one lost requirement");
    assert!(error.contains(expected) && !error.contains("For REQ-auth-002"), "{error}");

    let removed_both = drop_block(&removed, "REQ-auth-002");
    fs::write(requirements_path(root), removed_both).unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("two lost requirements");
    assert!(error.contains(expected) && error.contains("For REQ-auth-002"), "{error}");
}

/// REQ-change-169: a block moved to another requirements file under its id is not lost; a
/// renumbered one is lost under its old id, and the refusal says ids are permanent.
#[test]
fn a_moved_requirement_passes_and_a_renumbered_one_is_refused_with_the_hint() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    let without = drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-002");
    fs::write(requirements_path(root), &without).unwrap();
    fs::create_dir_all(root.join("specs/session")).unwrap();
    fs::write(
        root.join("specs/session/requirements.md"),
        "# Requirements\n\n### REQ-auth-002\n\nThe system SHALL expire a session after one day.\n",
    )
    .unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "move REQ-auth-002"]);
    materialize_change_deltas(root, &record.id).unwrap();

    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let renumbered = VANISHED_BASE_REQUIREMENTS.replace("### REQ-auth-002", "### REQ-auth-009");
    fs::write(requirements_path(root), renumbered).unwrap();
    git_in(root)(&["commit", "-q", "-am", "renumber"]);
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(error.contains("REQ-auth-002"), "{error}");
    assert!(error.contains("renumbered to REQ-auth-009"), "{error}");
    assert!(error.contains("permanent"), "{error}");
}

/// REQ-change-169: a deleted requirements file loses every id it held, and a legacy
/// `<module>.req.md` is read like `requirements.md`.
#[test]
fn a_deleted_file_and_a_truncated_legacy_req_file_are_refused() {
    let billing = "# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once a month.\n";
    for (path, how) in [
        ("specs/billing/requirements.md", "delete"),
        ("specs/billing/billing.req.md", "truncate"),
    ] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = vanished_repository(root, ADDED_THIRD, &[(path, billing)]);
        if how == "delete" {
            fs::remove_file(root.join(path)).unwrap();
        } else {
            fs::write(root.join(path), "# Requirements\n").unwrap();
        }
        git_in(root)(&["commit", "-q", "-am", how]);
        let error = materialize_change_deltas(root, &record.id).unwrap_err();
        assert!(error.contains("REQ-billing-001") && error.contains(path), "{how}: {error}");
    }
}

/// REQ-change-169: with no default branch the change's `base_commit` is the reference; with
/// neither, the check is skipped with a note; with a default branch whose merge-base text Git
/// cannot read, it refuses, naming what could not be read.
#[test]
fn the_reference_falls_back_skips_and_fails_closed() {
    // No default branch: `base_commit`.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    git(&["remote", "remove", "origin"]);
    let base = String::from_utf8(
        Command::new("git").args(["rev-parse", "HEAD"]).current_dir(root).output().unwrap().stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    record.base_commit = Some(base);
    save_change(root, &record).unwrap();
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "truncate"]);
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(error.contains("REQ-auth-001"), "{error}");

    // Neither: skipped, with a note.
    record.base_commit = None;
    save_change(root, &record).unwrap();
    let note = ensure_no_lost_requirements(root, &record).unwrap();
    assert!(note.is_some_and(|note| note.contains("not checked")));

    // A default branch whose merge-base text cannot be read: refused.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git_in(root)(&["commit", "-q", "-am", "truncate"]);
    let blob = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "origin/main:specs/auth/requirements.md"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    fs::remove_file(root.join(".git/objects").join(&blob[..2]).join(&blob[2..])).unwrap();
    let error = ensure_no_lost_requirements(root, &record).unwrap_err();
    // It names what could not be read and the way out (Leif's rule 3; ss-lead's review (5)).
    assert!(error.contains("cannot read"), "{error}");
    assert!(
        error.contains("repair the repository or fetch the default branch (`git fetch origin`)")
            && error.contains(&format!("specsync change check {}", record.id))
            && !error.contains("--unshallow"),
        "{error}"
    );
}

/// REQ-change-169: an approved `## REMOVED` item of another active change on the branch accounts
/// for the requirement it names, so one change's legitimate removal never refuses another.
#[test]
fn another_active_change_s_approved_removal_accounts_for_the_loss() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let first = completed_section_only_current_record(root, ADDED_THIRD);
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    let first = approve_definition(root, &first.id, Some("Scope owner".into()), None).unwrap();
    let second =
        completed_section_only_current_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    assert_ne!(first.id, second.id);
    let second = approve_definition(root, &second.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "two approved changes"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();
    git(&["commit", "-q", "-am", "the second change's removal"]);

    assert_eq!(ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap()), Ok(None));
    assert_eq!(ensure_no_lost_requirements(root, &load_change(root, &second.id).unwrap()), Ok(None));
}

#[test]
fn a_sibling_removal_must_survive_the_selected_changes_commit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let first = vanished_repository(root, ADDED_THIRD, &[]);
    let sibling = completed_section_only_current_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    let sibling = approve_definition(root, &sibling.id, Some("Scope owner".into()), None).unwrap();
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();

    // The selected workspace will be staged by its own lifecycle command.
    assert_eq!(ensure_no_lost_requirements(root, &sibling), Ok(None));
    let refused = || {
        let error = ensure_no_lost_requirements(root, &first)
            .expect_err("the sibling removal authority is missing from the index");
        assert!(error.contains("REQ-auth-001"), "{error}");
    };
    refused();
    let package = format!("{CHANGES_PATH}/{}", sibling.id);
    let git = git_in(root);
    git(&["add", "--", &format!("{package}/state.json"), &format!("{package}/approvals.json")]);
    refused();
    git(&["add", "--", &package]);
    let approvals = format!("{package}/approvals.json");
    let approved = fs::read(root.join(&approvals)).unwrap();
    fs::write(root.join(&approvals), "{}").unwrap();
    git(&["add", "--", &approvals]);
    fs::write(root.join(&approvals), approved).unwrap();
    refused();
    git(&["add", "--", &approvals]);
    fs::write(root.join(&package).join("notes.local"), "Private scratch notes.").unwrap();
    assert_eq!(ensure_no_lost_requirements(root, &first), Ok(None));

    git(&["commit", "-qam", "record the removal and its authority"]);
    let fresh = TempDir::new().unwrap();
    git(&["clone", "--quiet", "--no-local", ".", fresh.path().to_str().unwrap()]);
    let first = load_change(fresh.path(), &first.id).unwrap();
    assert_eq!(ensure_no_lost_requirements(fresh.path(), &first), Ok(None));
}

/// Two approved changes on `origin/main`, one adding REQ-auth-003 and one removing REQ-auth-001,
/// checked out on branch `change`: `(adding, removing)`.
fn two_change_repository(root: &Path) -> (ChangeRecord, ChangeRecord) {
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let first = completed_section_only_current_record(root, ADDED_THIRD);
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    let first = approve_definition(root, &first.id, Some("Scope owner".into()), None).unwrap();
    let second =
        completed_section_only_current_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    assert_ne!(first.id, second.id);
    let second = approve_definition(root, &second.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "two approved changes"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    (first, second)
}

/// Moves `record`'s workspace into the archive in `state`, as finalize leaves an archived change,
/// and commits it.
fn move_into_archive(root: &Path, record: &ChangeRecord, state: ChangeState) {
    move_into_archive_uncommitted(root, record, state);
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "archive the removing change"]);
}

/// As `move_into_archive`, leaving the move uncommitted, as finalize leaves it until the archive
/// tip is committed.
fn move_into_archive_uncommitted(root: &Path, record: &ChangeRecord, state: ChangeState) {
    let archive = root.join(ARCHIVE_PATH).join(format!("2026-11-20-{}", record.id));
    fs::create_dir_all(archive.parent().unwrap()).unwrap();
    fs::rename(change_dir(root, &record.id), &archive).unwrap();
    let mut moved = record.clone();
    moved.state = state;
    write_json(&archive.join("state.json"), &moved).unwrap();
}

/// REQ-change-169: a change archived on the branch after the reference accounts for the
/// requirements its archived `## REMOVED` items name. Codex on #1108 (P2): the authority is a real
/// archived record whose definition approval binds that delta.
#[test]
fn a_change_archived_on_the_branch_accounts_for_its_removal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (first, second) = two_change_repository(root);
    let reference = vanished_rev(root, "HEAD");
    move_into_archive(root, &second, ChangeState::Archived);
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();
    git_in(root)(&["commit", "-q", "-am", "the archived change's removal"]);

    let removed = removed_requirement_ids_on_branch(root, &reference, Some(&first.id)).unwrap();

    assert!(removed.contains("REQ-auth-001"), "{removed:?}");
    assert_eq!(ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap()), Ok(None));
}

/// Codex on #1108 (P2): a delta file under the archive path is not an archived change, and a
/// retired change is not an accepted one, so neither accounts for a removal (ss-lead: one
/// mechanism for both, in the active roster and in the archive).
#[test]
fn a_bare_archived_delta_or_a_retired_change_accounts_for_no_removal() {
    // A delta alone under the archive path, with no archived record around it.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let reference = vanished_rev(root, "HEAD");
    let package = root.join(ARCHIVE_PATH).join("2026-11-19-retire-session-one/deltas");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("auth.md"), "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n").unwrap();
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "a bare delta under the archive path"]);
    let removed = removed_requirement_ids_on_branch(root, &reference, Some(&record.id)).unwrap();
    assert!(!removed.contains("REQ-auth-001"), "{removed:?}");
    let error = ensure_no_lost_requirements(root, &record).unwrap_err();
    assert!(error.contains("REQ-auth-001"), "{error}");

    // A retired change, still in the active roster and moved into the archive.
    for archived in [false, true] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let (first, second) = two_change_repository(root);
        if archived {
            move_into_archive(root, &second, ChangeState::Retired);
        } else {
            let mut retired = load_change(root, &second.id).unwrap();
            retired.state = ChangeState::Retired;
            save_change(root, &retired).unwrap();
        }
        fs::write(
            requirements_path(root),
            drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
        )
        .unwrap();
        git_in(root)(&["add", "--all"]);
        git_in(root)(&["commit", "-q", "-m", "truncate beside a retired change"]);

        let error = ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap())
            .expect_err("a retired change's REMOVED item accounts for nothing");

        assert!(error.contains("REQ-auth-001"), "archived {archived}: {error}");
    }
}

/// `revision` resolved in `root`.
fn vanished_rev(root: &Path, revision: &str) -> String {
    String::from_utf8(
        Command::new("git")
            .args(["rev-parse", revision])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

/// ss-lead's review of b8018cc0 (1): with `specs_dir = "."` the check never reads a change
/// workspace, `.git` or an `exclude_dirs` directory as canonical requirements, in the working tree
/// or at the reference. Here the truncated file's ids survive only in those places, and the
/// change's own `requirements.md` artifact held an id of its own at the reference.
#[test]
fn a_root_specs_project_reads_no_workspace_git_or_excluded_directory_as_canonical() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    let artifact = change_dir(root, &record.id).join("requirements.md");
    git(&["checkout", "-q", "main"]);
    fs::write(&artifact, "# Requirements\n\n### REQ-auth-009\n\nA draft note.\n").unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \".\"\nexclude_dirs = [\"vendored\"]\n",
    )
    .unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "a root-specs project"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    let kept = "# Requirements\n\n### REQ-auth-001\n\nKept here.\n\n### REQ-auth-002\n\nKept here.\n";
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    fs::write(&artifact, kept).unwrap();
    fs::create_dir_all(root.join("vendored")).unwrap();
    fs::write(root.join("vendored/requirements.md"), kept).unwrap();
    fs::create_dir_all(root.join(".git/stray")).unwrap();
    fs::write(root.join(".git/stray/requirements.md"), kept).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "truncate"]);

    let scope = requirement_scope(root);
    assert_eq!(scope.directories, vec![".".to_string()]);
    let scanned: Vec<String> = working_tree_requirement_blocks(root, &scope, &excluded_requirement_directories(root))
        .files
        .into_iter()
        .map(|file| file.chain[0].clone())
        .collect();
    assert!(scanned.contains(&"specs/auth/requirements.md".to_string()), "{scanned:?}");
    assert!(
        scanned
            .iter()
            .all(|path| !path.starts_with(".specsync/") && !path.starts_with(".git/") && !path.starts_with("vendored/")),
        "{scanned:?}"
    );
    let error = ensure_no_lost_requirements(root, &load_change(root, &record.id).unwrap()).unwrap_err();
    assert!(
        error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
    assert!(!error.contains("REQ-auth-009"), "{error}");
}

/// ss-lead's review of b8018cc0 (3): a change workspace that cannot be read accounts for no
/// removal, and never turns this refusal into its own error (#1104's pattern): the lost ids are
/// still named.
#[test]
fn an_unreadable_change_never_hides_the_lost_requirements() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let broken = root.join(CHANGES_PATH).join("broken-change");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("state.json"), "{ not json").unwrap();
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "truncate beside a broken workspace"]);

    let error = ensure_no_lost_requirements(root, &record).unwrap_err();

    assert!(
        error.contains("lost canonical requirements") && error.contains("REQ-auth-001"),
        "{error}"
    );
}

/// ss-lead's review of b8018cc0 (4): a person reads the list, so lost ids are in natural order.
#[test]
fn lost_ids_are_listed_in_natural_order() {
    assert_eq!(natural_order("REQ-a-2", "REQ-a-10"), std::cmp::Ordering::Less);
    assert_eq!(natural_order("REQ-a-10", "REQ-a-9"), std::cmp::Ordering::Greater);
    assert_eq!(natural_order("REQ-a-002", "REQ-a-10"), std::cmp::Ordering::Less);
    assert_eq!(natural_order("REQ-a-1", "REQ-b-1"), std::cmp::Ordering::Less);
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let base = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-1\n\nOne.\n\n### REQ-auth-2\n\nTwo.\n\n### REQ-auth-10\n\nTen.\n";
    let record = vanished_repository_with_base(root, base, ADDED_THIRD, &[]);
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git_in(root)(&["commit", "-q", "-am", "truncate"]);

    let error = ensure_no_lost_requirements(root, &record).unwrap_err();

    assert!(
        error.contains("REQ-auth-1, REQ-auth-2, REQ-auth-10 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// A truncated branch `change` in `source` whose change records the approval commit as its base,
/// then a depth-1 clone of that branch at `clone`.
fn shallow_clone_of_a_truncated_branch(source: &Path, clone: &Path, main_moves_on: bool) -> ChangeRecord {
    fs::create_dir_all(source).unwrap();
    let mut record = vanished_repository(source, ADDED_THIRD, &[]);
    record.base_commit = Some(vanished_rev(source, "HEAD"));
    save_change(source, &record).unwrap();
    fs::write(requirements_path(source), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    let git = git_in(source);
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "truncate"]);
    if main_moves_on {
        git(&["checkout", "-q", "main"]);
        fs::write(source.join("NOTES.md"), "Main moves on.\n").unwrap();
        git(&["add", "--all"]);
        git(&["commit", "-q", "-m", "main moves on"]);
        git(&["checkout", "-q", "change"]);
    }
    let url = format!("file://{}", source.display());
    let output = Command::new("git")
        .args(["clone", "-q", "--depth", "1", "--branch", "change", url.as_str(), clone.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    record
}

/// ss-lead's review of b8018cc0 (5), as CI checks out: a depth-1 clone with no remote-tracking
/// ref has no default branch, and lacks the change's base commit, so the check is skipped with
/// the note rather than refused.
#[test]
fn a_depth_one_clone_with_no_origin_ref_is_skipped_with_the_note() {
    let temp = TempDir::new().unwrap();
    let (source, clone) = (temp.path().join("source"), temp.path().join("clone"));
    let record = shallow_clone_of_a_truncated_branch(&source, &clone, false);
    let refs = String::from_utf8(
        Command::new("git")
            .args(["for-each-ref", "--format=%(refname)", "refs/remotes"])
            .current_dir(&clone)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    for name in refs.lines().filter(|line| !line.is_empty()) {
        git_in(&clone)(&["update-ref", "-d", name]);
    }

    let note = ensure_no_lost_requirements(&clone, &load_change(&clone, &record.id).unwrap()).unwrap();

    assert!(note.is_some_and(|note| note.contains("not checked")));
}

/// ss-lead's review of b8018cc0 (5), as an agent's shallow clone often is: `origin/main` is there
/// but the merge-base is not, so the check cannot tell; it refuses, naming the unshallowing fetch.
#[test]
fn a_shallow_clone_without_the_merge_base_is_refused_naming_the_unshallow_fetch() {
    let temp = TempDir::new().unwrap();
    let (source, clone) = (temp.path().join("source"), temp.path().join("clone"));
    let record = shallow_clone_of_a_truncated_branch(&source, &clone, true);
    git_in(&clone)(&["fetch", "-q", "--depth", "1", "origin", "main:refs/remotes/origin/main"]);

    let error = ensure_no_lost_requirements(&clone, &load_change(&clone, &record.id).unwrap())
        .unwrap_err();

    assert!(
        error.contains("cannot tell whether this branch lost canonical requirements")
            && error.contains("`git fetch --unshallow origin`")
            && error.contains(&format!("specsync change check {}", record.id)),
        "{error}"
    );
}


/// The tester's L2b at 660066e1, exactly: main removes REQ-auth-002 and pushes; the feature branch
/// fetches and cherry-picks that commit (`-x`, so it is always a new commit; a cherry-pick made in
/// the same second as the original, onto the same parent, is byte-identical to it and so is main's
/// own commit, which is why the tester's first run at b8018cc0 passed); only then is the change
/// created, adding REQ-auth-003. The loss is refused, naming the merge as the way out, and merging
/// the default branch clears it.
#[test]
fn a_change_created_after_cherry_picking_main_s_removal_is_told_to_merge_main() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(requirements_path(root).parent().unwrap()).unwrap();
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "base"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "feat"]);
    git(&["checkout", "-q", "main"]);
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-002"),
    )
    .unwrap();
    git(&["commit", "-q", "-am", "main drops REQ-auth-002"]);
    let removal = vanished_rev(root, "HEAD");
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "feat"]);
    git(&["fetch", "-q", "origin"]);
    git(&["cherry-pick", "-x", removal.as_str()]);
    assert_ne!(vanished_rev(root, "HEAD"), removal, "the cherry-pick is a new commit");
    let record = completed_section_only_current_record(root, ADDED_THIRD);
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "the change, created after the cherry-pick"]);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("REQ-auth-002")
            && error.contains("origin/main has removed REQ-auth-002 too")
            && error.contains("`git merge origin/main`"),
        "{error}"
    );
    git(&["merge", "-q", "--no-edit", "origin/main"]);
    materialize_change_deltas(root, &record.id).unwrap();
}

/// Adds `files` on `main`, pushes, and brings branch `change` up to it.
fn on_main_and_change(root: &Path, files: &[(&str, &str)]) {
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    for (path, text) in files {
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        fs::write(root.join(path), text).unwrap();
    }
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "main adds files"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
}

#[cfg(unix)]
fn tracked_requirement_link(root: &Path) -> ChangeRecord {
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    on_main_and_change(
        root,
        &[
            ("specs/billing/notes.md", "Billing"),
            ("content/billing.md", "### REQ-billing-001\n\nBilling runs monthly.\n"),
        ],
    );
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    std::os::unix::fs::symlink("../../content/billing.md", root.join("specs/billing/requirements.md"))
        .unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "Register the billing requirements link"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    record
}

#[cfg(unix)]
#[test]
fn a_tracked_requirements_link_preserves_and_reports_ids_from_its_target() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = tracked_requirement_link(root);
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));

    // Only the target changed, outside the canonical specs directory.
    fs::write(root.join("content/billing.md"), "# Billing\n").unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("the linked requirement was lost");
    assert!(error.contains("REQ-billing-001 (specs/billing/requirements.md)"), "{error}");
    assert!(error.contains(":content/billing.md") && !error.contains("git merge"), "{error}");

    // A tracked canonical link cannot make bytes in an untracked target durable.
    let git = git_in(root);
    let canonical = root.join("specs/billing/requirements.md");
    fs::remove_file(&canonical).unwrap();
    std::os::unix::fs::symlink("../../content/next.md", &canonical).unwrap();
    fs::write(root.join("content/next.md"), "### REQ-billing-001\n\nBilling runs monthly.\n").unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("an untracked target cannot keep the id");
    assert!(error.contains("Git does not track") && error.contains("git add content/next.md"), "{error}");
    git(&["add", "content/next.md"]);
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));
}

#[cfg(unix)]
#[test]
fn a_linked_requirement_removed_on_the_default_tip_names_the_merge() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = tracked_requirement_link(root);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    fs::write(root.join("content/billing.md"), "# Billing\n").unwrap();
    git(&["commit", "-q", "-am", "Retire the billing requirement on main"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    fs::write(root.join("content/billing.md"), "# Billing\n").unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("merge the default branch first");
    assert!(error.contains("origin/main has removed REQ-billing-001 too"), "{error}");
}

#[cfg(unix)]
#[test]
fn a_requirements_file_can_use_two_tracked_links_inside_the_project() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = tracked_requirement_link(root);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    fs::rename(root.join("content/billing.md"), root.join("content/current.md")).unwrap();
    std::os::unix::fs::symlink("current.md", root.join("content/billing.md")).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "Use a second link for the billing requirements"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));

    fs::write(root.join("content/current.md"), "# Billing\n").unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("the linked requirement was lost");
    assert!(error.contains("REQ-billing-001 (specs/billing/requirements.md)"), "{error}");
    assert!(error.contains(":content/current.md") && !error.contains("git merge"), "{error}");
}

#[cfg(unix)]
#[test]
fn a_dangling_reference_requirement_link_names_the_unreadable_file() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = tracked_requirement_link(root);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    fs::remove_file(root.join("content/billing.md")).unwrap();
    git(&["commit", "-q", "-am", "Leave a dangling requirements link"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    let error = ensure_no_lost_requirements(root, &record).expect_err("a reference link cannot be read");
    assert!(error.contains("cannot read specs/billing/requirements.md") && error.contains("content/billing.md"), "{error}");
}

#[cfg(unix)]
#[test]
fn a_cyclic_reference_requirement_link_fails_closed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = tracked_requirement_link(root);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    let target = root.join("content/billing.md");
    fs::remove_file(&target).unwrap();
    std::os::unix::fs::symlink("../specs/billing/requirements.md", &target).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "Record a cyclic requirements link"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    let error = ensure_no_lost_requirements(root, &record).expect_err("a reference cycle cannot be read");
    assert!(error.contains("cannot read specs/billing/requirements.md") && error.contains("cyclic"), "{error}");
}

/// Codex on #1108 (P1): a spec registered at the project root has its requirements beside it, in
/// the root's own `requirements.md` and `<module>.req.md` ("beside a registered spec"), and losing
/// them is refused. ss-lead's guard: that reads the root directory's own files, not the whole
/// tree, so a stray requirements file elsewhere is still not reported.
#[test]
fn a_root_level_registered_spec_s_requirements_are_read_and_the_rest_of_the_tree_is_not() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    on_main_and_change(
        root,
        &[
            (
                ".specsync/registry.toml",
                "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nroot = \"root.spec.md\"\n",
            ),
            ("requirements.md", "# Requirements\n\n### REQ-root-001\n\nThe system SHALL start.\n"),
            ("legacy.req.md", "# Requirements\n\n### REQ-legacy-001\n\nThe system SHALL stop.\n"),
            ("docs/notes/requirements.md", "# Requirements\n\n### REQ-stray-001\n\nA note.\n"),
        ],
    );
    for path in ["requirements.md", "legacy.req.md", "docs/notes/requirements.md"] {
        fs::write(root.join(path), "# Requirements\n").unwrap();
    }
    git_in(root)(&["commit", "-q", "-am", "truncate the root's requirements and a stray file"]);

    let error = ensure_no_lost_requirements(root, &record)
        .expect_err("the root-level spec's requirements were lost");

    assert!(
        error.contains("REQ-legacy-001 (legacy.req.md); REQ-root-001 (requirements.md)"),
        "{error}"
    );
    assert!(!error.contains("REQ-stray-001"), "{error}");
}

/// Codex on #1108 (P1): a finalization that stopped after acceptance and is retried with its
/// workspace still active asks again whether the branch lost a requirement, beside the task check
/// it already repeats; and the audit asks it of an accepted change ("for each active change").
#[test]
fn an_accepted_change_resuming_finalization_is_refused_and_the_audit_reports_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(
        root,
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
        &[],
    );
    check_change(root, Some(&record.id)).unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "materialize"]);
    check_change(root, Some(&record.id)).unwrap();
    record_scoped_review(root, &record.id, "Scoped reviewer".into()).unwrap();
    let accepted =
        accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    assert_eq!(accepted.state, ChangeState::Accepted);
    let workspace = change_dir(root, &record.id);
    let (attempts_path, before) = load_verification_attempt_ledger(root, &accepted).unwrap();
    let verification = fs::read(workspace.join("verification.json")).unwrap();
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();

    let error = finalize_change(root, &record.id).expect_err("the resume must refuse the loss");

    assert!(
        error.contains("lost canonical requirements") && error.contains("REQ-auth-001"),
        "{error}"
    );
    assert!(change_dir(root, &record.id).is_dir(), "nothing was archived");
    let (_, after) = load_verification_attempt_ledger(root, &accepted).unwrap();
    assert_eq!(after.attempts.len(), before.attempts.len() + 1);
    let refusal = after.attempts.last().unwrap();
    assert!(!refusal.passed);
    assert_eq!(refusal.refusal.as_deref(), Some(error.as_str()));
    assert_eq!(load_change(root, &record.id).unwrap(), accepted);
    assert_eq!(fs::read(workspace.join("verification.json")).unwrap(), verification);
    let attempts = fs::read(&attempts_path).unwrap();
    let report = audit_project(root);
    assert!(
        report
            .errors
            .iter()
            .any(|line| line.starts_with(&format!("{}:", record.id)) && line.contains("REQ-auth-001")),
        "{:?}",
        report.errors
    );
    assert_eq!(fs::read(&attempts_path).unwrap(), attempts);

    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    record_ship_lost_requirements_refusal(root, &record.id, &error).unwrap();
    assert_eq!(fs::read(attempts_path).unwrap(), attempts);
    let archive = finalize_change(root, &record.id).expect("restoring the requirement allows archival");
    assert_eq!(fs::read(archive.join("verification-attempts.json")).unwrap(), attempts);
}

/// A registry with backslash separators can map an external companion in each snapshot.
#[test]
fn registry_companions_use_normalized_paths_in_the_worktree_and_reference() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let original = "modules/billing/requirements.md";
    let moved = "packages/billing/billing.req.md";
    let registry = ".specsync/registry.toml";
    let mapping = |path: &str| format!("[registry]\nname = \"vanished\"\n\n[specs]\nbilling = '{path}'\n");
    let text = "# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once a month.\n";
    let record = vanished_repository(
        root,
        ADDED_THIRD,
        &[(registry, &mapping(r"modules\billing\billing.spec.md")), (original, text)],
    );
    let git = git_in(root);
    fs::write(root.join(registry), mapping(r"packages\billing\billing.spec.md")).unwrap();
    fs::create_dir_all(root.join("packages/billing")).unwrap();
    fs::rename(root.join(original), root.join(moved)).unwrap();
    git(&["add", "--all"]);
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));

    fs::remove_file(root.join(moved)).unwrap();
    git(&["add", "--all"]);
    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-billing-001 was lost");
    assert!(error.contains("REQ-billing-001 (modules/billing/requirements.md)"), "{error}");
}

/// A requirements file canonical at the reference is still read after the branch drops its
/// registry mapping or moves the specs directory away from it.
#[test]
fn requirements_canonical_only_at_the_reference_are_still_read() {
    // The branch drops the registry mapping and the file beside its spec.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    on_main_and_change(
        root,
        &[
            (
                ".specsync/registry.toml",
                "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nbilling = \"modules/billing/billing.spec.md\"\n",
            ),
            (
                "modules/billing/requirements.md",
                "# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once a month.\n",
            ),
        ],
    );
    fs::write(
        root.join(".specsync/registry.toml"),
        "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\n",
    )
    .unwrap();
    fs::remove_file(root.join("modules/billing/requirements.md")).unwrap();
    git_in(root)(&["commit", "-q", "-am", "drop the billing mapping and its requirements"]);
    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-billing-001 was lost");
    assert!(error.contains("REQ-billing-001 (modules/billing/requirements.md)"), "{error}");

    // The branch moves the specs directory and deletes the old one's requirements.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(root.join(".specsync/config.toml"), "specs_dir = \"docs/specs\"\n").unwrap();
    fs::remove_file(requirements_path(root)).unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "a new specs directory, the old requirements gone"]);
    let error = ensure_no_lost_requirements(root, &record).expect_err("the old specs were lost");
    assert!(
        error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// Codex on #1108 (P2): Git's default `core.quotePath` quotes a non-ASCII path in a plain name
/// listing; the check reads NUL-separated names, so a module directory named in other letters is
/// read like any other.
#[test]
fn a_non_ascii_module_directory_is_read() {
    let path = "specs/modülé/requirements.md";
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(
        root,
        ADDED_THIRD,
        &[(path, "# Requirements\n\n### REQ-modulo-001\n\nThe system SHALL keep accents.\n")],
    );
    git_in(root)(&["config", "core.quotePath", "true"]);
    fs::write(root.join(path), "# Requirements\n").unwrap();
    git_in(root)(&["commit", "-q", "-am", "truncate the accented module"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-modulo-001 was lost");

    assert!(error.contains(&format!("REQ-modulo-001 ({path})")), "{error}");
    // The way out prints the block from the reference by that same path, quoted as one word.
    assert!(error.contains(&format!(":{path}'`")), "{error}");
}

#[test]
fn a_specs_directory_with_leading_whitespace_preserves_and_reports_requirement_ids() {
    let path = " specs/requirements.md";
    let requirements = "# Requirements\n\n### REQ-space-001\n\nThe system SHALL keep the path.\n";
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    on_main_and_change(
        root,
        &[(".specsync/config.toml", "specs_dir = \" specs\"\n"), (path, requirements)],
    );

    fs::write(root.join(path), requirements.replace("keep the path", "preserve the path")).unwrap();
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));

    fs::remove_file(root.join(path)).unwrap();
    let error = ensure_no_lost_requirements(root, &record).expect_err("the requirement was lost");
    assert!(error.contains(&format!("REQ-space-001 ({path})")), "{error}");
    let command = error.split('`').find(|part| part.starts_with("git show ")).unwrap();
    let restore = Command::new("sh").args(["-c", command]).current_dir(root).output().unwrap();
    assert!(restore.status.success(), "{command}: {}", String::from_utf8_lossy(&restore.stderr));
    assert_eq!(restore.stdout, requirements.as_bytes());
}

#[test]
fn a_reference_requirement_after_an_indented_fence_marker_is_reported_lost() {
    let base = format!("    ```\n\n{VANISHED_BASE_REQUIREMENTS}");
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository_with_base(root, &base, ADDED_THIRD, &[]);
    fs::write(requirements_path(root), drop_block(&base, "REQ-auth-001")).unwrap();

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-auth-001 was lost");
    assert!(error.contains("REQ-auth-001 (specs/auth/requirements.md)"), "{error}");
    let command = error.split('`').find(|part| part.starts_with("git show ")).unwrap();
    let restored = Command::new("sh").args(["-c", command]).current_dir(root).output().unwrap();
    assert!(restored.status.success(), "{command}: {}", String::from_utf8_lossy(&restored.stderr));
    assert_eq!(restored.stdout, base.as_bytes());
}

#[test]
fn default_tip_keeps_a_requirement_after_an_indented_fence_marker() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    fs::write(requirements_path(root), format!("    ```\n\n{VANISHED_BASE_REQUIREMENTS}")).unwrap();
    git(&["commit", "-q", "-am", "add an indented code example without removing any requirement"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-auth-001 was lost");
    assert!(error.contains("REQ-auth-001 (specs/auth/requirements.md)"), "{error}");
    assert!(error.contains("Restore each block"), "{error}");
    assert!(!error.contains("origin/main has removed REQ-auth-001 too"), "{error}");
}

#[test]
fn requirements_are_read_in_a_project_subdirectory_with_leading_whitespace() {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    let git = git_in(repository);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let root = repository.join(" app");
    fs::create_dir(&root).unwrap();
    let mut record = completed_section_only_current_record(&root, ADDED_THIRD);
    fs::write(requirements_path(&root), VANISHED_BASE_REQUIREMENTS).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "seed the project subdirectory"]);
    record.base_commit = Some(vanished_rev(repository, "HEAD"));
    fs::write(
        requirements_path(&root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();

    let error = ensure_no_lost_requirements(&root, &record).expect_err("REQ-auth-001 was lost");
    assert!(error.contains("REQ-auth-001 (specs/auth/requirements.md)"), "{error}");
    assert!(error.contains(": app/specs/auth/requirements.md"), "{error}");
}

/// Codex on #1108 (P2): the working tree is read to the same depth as Git's listing, so a
/// requirements file twenty directories below the specs directory that still holds its id is
/// not reported lost.
#[test]
fn a_requirements_file_deep_below_the_specs_directory_is_read() {
    let deep = format!(
        "specs/{}/requirements.md",
        (0..20).map(|level| format!("d{level}")).collect::<Vec<_>>().join("/")
    );
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(
        root,
        ADDED_THIRD,
        &[(deep.as_str(), "# Requirements\n\n### REQ-deep-001\n\nKept.\n")],
    );
    fs::write(root.join(&deep), "# Requirements\n\n### REQ-deep-001\n\nKept, and reworded.\n").unwrap();
    git_in(root)(&["commit", "-q", "-am", "reword the deep requirement"]);

    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));
}

/// corvid-agent on #1108 (blocking), inside the approved words: an `origin/HEAD` that does not
/// resolve to a commit is not a default branch. It is set and broken here with `git symbolic-ref`,
/// so the test does not depend on what a given Git's clone writes. With `origin/main` present the
/// check compares with it, and the merge way out names it.
#[test]
fn a_dangling_origin_head_is_not_the_default_branch() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    let removed = drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001");
    git(&["checkout", "-q", "main"]);
    fs::write(requirements_path(root), &removed).unwrap();
    git(&["commit", "-q", "-am", "main removes REQ-auth-001"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    fs::write(requirements_path(root), &removed).unwrap();
    git(&["commit", "-q", "-am", "the same removal, by hand"]);
    // The default branch was renamed and pruned: origin/HEAD still names it.
    git(&["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/trunk"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-auth-001 was lost");

    assert!(
        error.contains("origin/main has removed REQ-auth-001 too")
            && error.contains("`git merge origin/main`")
            && !error.contains("trunk"),
        "{error}"
    );
}

/// corvid-agent on #1108 (blocking): with a dangling `origin/HEAD` and neither `origin/main` nor
/// `origin/master`, there is no default branch, so the change's base commit is the reference;
/// without one the check is skipped with the note, never refused for a merge-base it cannot read.
#[test]
fn a_dangling_origin_head_alone_falls_back_to_the_base_commit_or_the_note() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    record.base_commit = Some(vanished_rev(root, "HEAD"));
    save_change(root, &record).unwrap();
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "truncate"]);
    git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    git(&["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-auth-001 was lost");
    assert!(error.contains("lost canonical requirements") && error.contains("REQ-auth-001"), "{error}");

    record.base_commit = None;
    save_change(root, &record).unwrap();
    let note = ensure_no_lost_requirements(root, &record).unwrap();
    assert!(note.is_some_and(|note| note.contains("not checked")));
}

/// Finalize stages its own archive. Another change can rely on that archive once the
/// removal authority is in the index and will survive a checkout of the resulting commit.
#[test]
fn an_interrupted_archive_authorizes_its_own_finalization_and_then_a_staged_sibling() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (first, second) = two_change_repository(root);
    let reference = vanished_rev(root, "HEAD");
    move_into_archive_uncommitted(root, &second, ChangeState::Archived);
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();

    let own = removed_requirement_ids_on_branch(root, &reference, Some(&second.id)).unwrap();
    assert!(own.contains("REQ-auth-001"), "{own:?}");
    let unselected = removed_requirement_ids_on_branch(root, &reference, None).unwrap();
    assert!(!unselected.contains("REQ-auth-001"), "{unselected:?}");
    let refused = || {
        let error = ensure_no_lost_requirements(root, &first)
            .expect_err("the sibling archive is missing removal authority in Git");
        assert!(error.contains("REQ-auth-001"), "{error}");
    };
    refused();
    let archive = format!("{ARCHIVE_PATH}/2026-11-20-{}", second.id);
    let git = git_in(root);
    git(&["add", "--", &format!("{archive}/state.json"), &format!("{archive}/approvals.json")]);
    refused();
    git(&["add", "--", &archive]);
    assert_eq!(ensure_no_lost_requirements(root, &first), Ok(None));

    let state = root.join(&archive).join("state.json");
    let indexed = fs::read(&state).unwrap();
    let mut changed = indexed.clone();
    changed.push(b'\n');
    fs::write(&state, changed).unwrap();
    refused();
    fs::write(&state, indexed).unwrap();
    assert_eq!(ensure_no_lost_requirements(root, &first), Ok(None));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        git(&["config", "core.filemode", "true"]);
        let original = fs::metadata(&state).unwrap().permissions();
        fs::set_permissions(&state, fs::Permissions::from_mode(original.mode() | 0o111)).unwrap();
        refused();
        fs::set_permissions(&state, original).unwrap();
        assert_eq!(ensure_no_lost_requirements(root, &first), Ok(None));
    }

    git(&["commit", "-qam", "record the archived sibling and its removal"]);
    let fresh = TempDir::new().unwrap();
    git(&["clone", "--quiet", "--no-local", ".", fresh.path().to_str().unwrap()]);
    let first = load_change(fresh.path(), &first.id).unwrap();
    assert_eq!(ensure_no_lost_requirements(fresh.path(), &first), Ok(None));
}

/// Legacy approvals bind the selected artifacts at their logical change paths even after a move.
#[test]
fn an_archived_legacy_sibling_needs_its_selected_artifacts_in_the_index() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "legacy approval"]);
    let reference = vanished_rev(root, "HEAD");
    move_into_archive_uncommitted(root, &record, ChangeState::Archived);
    let archive = format!("{ARCHIVE_PATH}/2026-11-20-{}", record.id);
    git(&["add", "--", &archive]);
    let removals = || removed_requirement_ids_on_branch(root, &reference, None).unwrap();
    assert!(removals().contains("REQ-auth-001"));
    let selected = format!("{archive}/{}", record.selected_artifacts[0].file_name());
    git(&["restore", "--staged", "--", &selected]);
    assert!(!removals().contains("REQ-auth-001"));
    git(&["add", "--", &selected]);
    assert!(removals().contains("REQ-auth-001"));
}

/// Codex on #1108, second round (P1): a branch that adds a formerly canonical directory to
/// `exclude_dirs` while deleting its requirements is still refused. A file that was canonical at
/// the reference is judged by the exclusions at the reference, not by ones the branch added.
#[test]
fn a_directory_the_branch_newly_excludes_still_counts_at_the_reference() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(root.join(".specsync/config.toml"), "exclude_dirs = [\"auth\"]\n").unwrap();
    fs::remove_file(requirements_path(root)).unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "exclude auth and delete its requirements"]);

    let error = ensure_no_lost_requirements(root, &record)
        .expect_err("excluding the directory does not remove its requirements");

    assert!(
        error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// corvid-agent on #1108 (blocking), the part that changes the approved sentence and waits for
/// Leif: a clone of one branch (`git clone --branch change`, which Git 2.47 on Linux points
/// `origin/HEAD` at) compares the branch with its own pushed tip and finds nothing lost. Set here
/// with `git symbolic-ref`, so the test does not depend on the Git version: an `origin/HEAD` naming
/// the branch HEAD is on is not the default branch, and the pushed loss is refused against
/// `origin/main`.
#[test]
fn an_origin_head_naming_the_branch_itself_is_not_the_default_branch() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    let main = vanished_rev(root, "origin/main");
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git(&["commit", "-q", "-am", "truncate"]);
    git(&["push", "-q", "origin", "change"]);
    git(&["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/change"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("the pushed loss is refused");

    assert!(
        error.contains(&format!("compared with {}", &main[..12]))
            && error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// Leif's 'A' (2026-11-20), its side effect pinned: on `main` itself, `origin/HEAD` and
/// `origin/main` both name the branch HEAD is on, so neither is the default branch and the check
/// compares from the change's base commit; a loss already pushed to `origin/main` is refused.
#[test]
fn work_directly_on_main_compares_from_the_base_commit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = vanished_repository(root, ADDED_THIRD, &[]);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    let base = vanished_rev(root, "HEAD");
    record.base_commit = Some(base.clone());
    save_change(root, &record).unwrap();
    fs::write(requirements_path(root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "truncate on main"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("the pushed loss is refused");

    assert!(
        error.contains(&format!("compared with {}", &base[..12]))
            && error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// Codex on #1108, second round (P2): a registry mapping outside the project
/// (`../shared/other.spec.md`) adds no directory, in the working tree or at the reference, as
/// `canonical_module_paths` refuses it: requirements outside the project never mask a loss inside
/// it, and Git is never handed a pathspec outside the repository.
#[test]
fn a_registry_path_outside_the_project_adds_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    let shared = temp.path().join("shared");
    fs::create_dir_all(&shared).unwrap();
    fs::write(shared.join("requirements.md"), VANISHED_BASE_REQUIREMENTS).unwrap();
    let record = vanished_repository(&root, ADDED_THIRD, &[]);
    on_main_and_change(
        &root,
        &[(
            ".specsync/registry.toml",
            "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nother = \"../shared/other.spec.md\"\n",
        )],
    );
    fs::write(requirements_path(&root), "---\nspec: auth.spec.md\n---\n\n# Requirements\n").unwrap();
    git_in(&root)(&["commit", "-q", "-am", "truncate"]);

    let error = ensure_no_lost_requirements(&root, &record).expect_err("the loss is refused");

    assert!(
        error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// Codex on #1108, second round (P2): "beside a registered spec" is the spec's own directory, for
/// a spec registered outside the specs directory as for one at the root. Its subtree is not
/// canonical: a file further down neither masks a loss nor is reported lost.
#[test]
fn a_registered_spec_s_companions_are_its_own_directory_only() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    on_main_and_change(
        root,
        &[
            (
                ".specsync/registry.toml",
                "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nbilling = \"modules/billing/billing.spec.md\"\n",
            ),
            (
                "modules/billing/requirements.md",
                "# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once a month.\n",
            ),
            (
                "modules/billing/docs/requirements.md",
                "# Requirements\n\n### REQ-billing-doc-001\n\nA note.\n",
            ),
        ],
    );
    fs::remove_file(root.join("modules/billing/docs/requirements.md")).unwrap();
    fs::create_dir_all(root.join("modules/billing/notes")).unwrap();
    fs::write(
        root.join("modules/billing/notes/requirements.md"),
        "# Requirements\n\n### REQ-auth-001\n\nA copy in a note.\n",
    )
    .unwrap();
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "a note below billing, and a loss in auth"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-auth-001 was lost");

    assert!(error.contains("REQ-auth-001 (specs/auth/requirements.md)"), "{error}");
    assert!(!error.contains("REQ-billing-doc-001"), "{error}");
}

/// Codex on #1108, second round (P2): the restore command quotes the whole `<commit>:<path>`, so
/// it prints the lost block when it is run as printed, whatever the path holds (Leif's rule 3).
#[test]
fn the_printed_restore_command_runs_for_a_path_with_a_space() {
    let path = "specs/my module/requirements.md";
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(
        root,
        ADDED_THIRD,
        &[(path, "# Requirements\n\n### REQ-space-001\n\nThe system SHALL keep spaces.\n")],
    );
    fs::write(root.join(path), "# Requirements\n").unwrap();
    git_in(root)(&["commit", "-q", "-am", "truncate the spaced module"]);

    let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-space-001 was lost");
    let command = error
        .split('`')
        .find(|part| part.starts_with("git show ") && part.contains("my module"))
        .unwrap_or_else(|| panic!("no restore command for {path}: {error}"));
    let output = Command::new("sh").args(["-c", command]).current_dir(root).output().unwrap();

    assert!(output.status.success(), "{command}: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("### REQ-space-001"), "{command}");
}

/// Codex on #1108, second round (P2): "for each active change" includes a draft, so the audit
/// reports a loss beside a change that is still a draft. The draft's own `## REMOVED` item still
/// accounts for nothing.
#[test]
fn the_audit_reports_a_loss_beside_a_draft() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let draft = completed_section_only_current_record(root, "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n");
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
    assert_eq!(load_change(root, &draft.id).unwrap().state, ChangeState::Draft);
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "a draft change"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git(&["commit", "-q", "-am", "a loss beside the draft"]);

    let report = audit_project(root);

    assert!(
        report
            .errors
            .iter()
            .any(|line| line.starts_with(&format!("{}:", draft.id)) && line.contains("REQ-auth-001")),
        "{:?}",
        report.errors
    );
}

/// Codex on #1108, third round (P1): a specs directory the branch replaced with a link to a
/// directory outside the project is not read, so a copy of an id there never masks the loss of
/// the project's own requirement.
#[cfg(unix)]
#[test]
fn a_specs_directory_that_leads_outside_the_project_is_not_read() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    let record = vanished_repository(&root, ADDED_THIRD, &[]);
    let outside = temp.path().join("outside");
    fs::create_dir_all(outside.join("auth")).unwrap();
    fs::write(outside.join("auth/requirements.md"), VANISHED_BASE_REQUIREMENTS).unwrap();
    fs::rename(root.join("specs"), temp.path().join("specs-moved-away")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("specs")).unwrap();
    git_in(&root)(&["add", "--all"]);
    git_in(&root)(&["commit", "-q", "-m", "the specs directory becomes a link"]);

    let error = ensure_no_lost_requirements(&root, &record)
        .expect_err("a copy outside the project masks nothing");

    assert!(
        error.contains("REQ-auth-001, REQ-auth-002 (specs/auth/requirements.md)"),
        "{error}"
    );
}

/// Codex on #1108, third round (P1): an approval that records no delta digests, as one made
/// before they existed, binds no delta wording, so its change's `## REMOVED` items account for
/// nothing until the change is approved again.
#[test]
fn a_removal_its_approval_does_not_bind_accounts_for_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (first, second) = two_change_repository(root);
    let ledger = change_dir(root, &second.id).join("approvals.json");
    let mut approvals: serde_json::Value = serde_json::from_str(&fs::read_to_string(&ledger).unwrap()).unwrap();
    let definition = approvals["approvals"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|approval| approval["gate"] == "definition")
        .unwrap();
    assert!(definition.as_object_mut().unwrap().remove("approved_delta_digests").is_some());
    fs::write(&ledger, serde_json::to_string_pretty(&approvals).unwrap()).unwrap();
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "an approval with no delta digests, and a loss"]);

    let error = ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap())
        .expect_err("an unbound removal accounts for nothing");

    assert!(error.contains("REQ-auth-001"), "{error}");
}

/// Codex on #1108, third round (P2, the other half of the second round's P1): a branch that
/// newly excludes a formerly canonical directory and only edits its requirements loses nothing.
/// The file was canonical at the reference, so its current ids still count.
#[test]
fn a_directory_the_branch_newly_excludes_keeps_its_edited_requirements() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(root.join(".specsync/config.toml"), "exclude_dirs = [\"auth\"]\n").unwrap();
    fs::write(
        requirements_path(root),
        VANISHED_BASE_REQUIREMENTS.replace("after one day", "after a day"),
    )
    .unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "exclude auth and reword one of its requirements"]);

    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));
}

/// The tester's excl2 on #1108 at 4cdc28dd: a branch adds a module's directory to
/// `exclude_dirs`, in either spelling, and deletes its requirements. The default branch still
/// holds them, so the refusal names the restore, never a merge that changes nothing, and the
/// restore command it prints, run as printed, prints the lost block.
#[test]
fn a_newly_excluded_directory_s_loss_names_a_restore_that_runs() {
    for excluded in ["gadget", "specs/gadget"] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = vanished_repository(
            root,
            ADDED_THIRD,
            &[("specs/gadget/requirements.md", "# Requirements\n\n### REQ-gadget-001\n\nThe gadget SHALL start.\n")],
        );
        fs::write(root.join(".specsync/config.toml"), format!("exclude_dirs = [\"{excluded}\"]\n")).unwrap();
        fs::remove_file(root.join("specs/gadget/requirements.md")).unwrap();
        git_in(root)(&["add", "--all"]);
        git_in(root)(&["commit", "-q", "-m", "exclude gadget and delete its requirements"]);

        let error = ensure_no_lost_requirements(root, &record).expect_err("REQ-gadget-001 was lost");

        assert!(error.contains("REQ-gadget-001 (specs/gadget/requirements.md)"), "{excluded}: {error}");
        assert!(!error.contains("git merge"), "{excluded}: {error}");
        let command = error
            .split('`')
            .find(|part| part.starts_with("git show ") && part.contains("specs/gadget"))
            .unwrap_or_else(|| panic!("{excluded}: no restore command: {error}"));
        let output = Command::new("sh").args(["-c", command]).current_dir(root).output().unwrap();
        assert!(output.status.success(), "{excluded}: {command}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("### REQ-gadget-001"), "{excluded}");
    }
}

/// Codex on #1108, fourth round (P1): the registry read at the reference is judged as it was
/// there, not against today's tree. A branch that drops a mapping and makes that spec's
/// directory a link outside the project is still compared with the old companion's files.
#[cfg(unix)]
#[test]
fn a_reference_registry_path_is_judged_as_a_path_not_by_today_s_tree() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    let record = vanished_repository(&root, ADDED_THIRD, &[]);
    on_main_and_change(
        &root,
        &[
            (
                ".specsync/registry.toml",
                "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nbilling = \"modules/billing/billing.spec.md\"\n",
            ),
            (
                "modules/billing/requirements.md",
                "# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once a month.\n",
            ),
        ],
    );
    fs::write(
        root.join(".specsync/registry.toml"),
        "[registry]\nname = \"vanished\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\n",
    )
    .unwrap();
    let outside = temp.path().join("outside-billing");
    fs::create_dir_all(&outside).unwrap();
    fs::remove_dir_all(root.join("modules/billing")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("modules/billing")).unwrap();
    git_in(&root)(&["add", "--all"]);
    git_in(&root)(&["commit", "-q", "-m", "drop the mapping; billing becomes a link outside"]);

    let error = ensure_no_lost_requirements(&root, &record).expect_err("REQ-billing-001 was lost");

    assert!(error.contains("REQ-billing-001 (modules/billing/requirements.md)"), "{error}");
}

/// Codex on #1108, fourth round (P1): "a change the branch archived after the reference commit".
/// A change archived before the reference, whose package the branch only renames, accounts for
/// none of the branch's losses.
#[test]
fn an_archive_the_branch_only_renames_accounts_for_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (first, second) = two_change_repository(root);
    let git = git_in(root);
    git(&["checkout", "-q", "main"]);
    move_into_archive(root, &second, ChangeState::Archived);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "--ff-only", "main"]);
    let archive = root.join(ARCHIVE_PATH);
    fs::rename(
        archive.join(format!("2026-11-20-{}", second.id)),
        archive.join(format!("2026-11-21-{}", second.id)),
    )
    .unwrap();
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-q", "-m", "rename the old archive and drop REQ-auth-001"]);

    let error = ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap())
        .expect_err("an old archive authorizes nothing new");

    assert!(error.contains("REQ-auth-001"), "{error}");
}

/// Codex on #1108, fourth round (P1): an id copied into a requirements file Git does not track
/// does not count as present when the tracked files have lost it, since `change check --commit`
/// commits no such file. The refusal names `git add` for that file; run as printed, it makes the
/// copy tracked and the check passes.
#[test]
fn an_untracked_copy_of_a_lost_requirement_does_not_count() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();
    git_in(root)(&["commit", "-q", "-am", "move REQ-auth-001 out"]);
    fs::create_dir_all(root.join("specs/notes")).unwrap();
    fs::write(
        root.join("specs/notes/requirements.md"),
        "# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n",
    )
    .unwrap();

    let error = ensure_no_lost_requirements(root, &record)
        .expect_err("an untracked copy keeps nothing in the committed tree");

    assert!(error.contains("REQ-auth-001 (specs/auth/requirements.md)"), "{error}");
    let command = error
        .split('`')
        .find(|part| part.starts_with("git add "))
        .unwrap_or_else(|| panic!("no git add in the refusal: {error}"));
    assert!(command.contains("specs/notes/requirements.md"), "{error}");
    let output = Command::new("sh").args(["-c", command]).current_dir(root).output().unwrap();
    assert!(output.status.success(), "{command}");
    assert_eq!(ensure_no_lost_requirements(root, &record), Ok(None));
}

/// Codex on #1134, second round (REQ-change-177): an error the directory listing yields, or one
/// from reading an entry's kind, is kept with its path for the cannot-tell refusal, not dropped.
/// A failing filesystem cannot be made in a test, so the scan's entry step is driven directly.
#[test]
fn a_listing_or_entry_error_is_kept_with_its_path() {
    let directory = std::path::Path::new("specs/auth");
    let listing = listed_entry(directory, Err(std::io::Error::other("input/output error")));
    assert!(
        matches!(&listing, Err((path, error)) if path == directory && error.to_string() == "input/output error"),
        "{listing:?}"
    );
    let file = directory.join("requirements.md");
    let entry = listed_entry(
        directory,
        Ok((file.clone(), Err(std::io::Error::other("input/output error")))),
    );
    assert!(matches!(&entry, Err((path, _)) if *path == file), "{entry:?}");
}

#[test]
fn ship_status_omits_the_lost_requirement_refusal_for_a_retired_change() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mut record = vanished_repository(root, ADDED_THIRD, &[]);
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-002"),
    )
    .unwrap();

    let active = lost_requirements_refusal(root, &record).unwrap();
    assert!(active.contains("REQ-auth-002"), "{active}");
    record.state = ChangeState::Retired;
    assert_eq!(lost_requirements_refusal(root, &record), None);
}
