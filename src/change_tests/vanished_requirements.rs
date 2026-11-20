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

    let removed = removed_requirement_ids_on_branch(root, &reference).unwrap();

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
    let removed = removed_requirement_ids_on_branch(root, &reference).unwrap();
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
    let scanned: Vec<String> = working_tree_requirement_blocks(root, &scope)
        .into_iter()
        .map(|(path, _)| path)
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
    let record = vanished_repository(root, ADDED_THIRD, &[]);
    let mut accepted = load_change(root, &record.id).unwrap();
    accepted.state = ChangeState::Accepted;
    accepted.canonical_applied = true;
    save_change(root, &accepted).unwrap();
    fs::write(
        requirements_path(root),
        drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001"),
    )
    .unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "accepted, then a loss"]);

    let error = finalize_change(root, &record.id).expect_err("the resume must refuse the loss");

    assert!(
        error.contains("lost canonical requirements") && error.contains("REQ-auth-001"),
        "{error}"
    );
    assert!(change_dir(root, &record.id).is_dir(), "nothing was archived");
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

/// Codex on #1108 (P1): a requirements file canonical at the reference ("in such a file at the
/// reference commit") is read even when the branch dropped its registry mapping or moved the
/// specs directory away from it.
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
    // The way out prints the block from the reference by that same path.
    assert!(error.contains(&format!(":{path}`")), "{error}");
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

/// corvid-agent on #1108 (non-blocking): `change finalize` moves a change into the archive before
/// the archive tip is committed. A second active change checked on that tree still sees the
/// archived change's approved removals, though Git does not track its record yet.
#[test]
fn a_change_archived_but_not_yet_committed_accounts_for_its_removal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let (first, second) = two_change_repository(root);
    let reference = vanished_rev(root, "HEAD");
    move_into_archive_uncommitted(root, &second, ChangeState::Archived);
    fs::write(requirements_path(root), drop_block(VANISHED_BASE_REQUIREMENTS, "REQ-auth-001")).unwrap();

    let removed = removed_requirement_ids_on_branch(root, &reference).unwrap();

    assert!(removed.contains("REQ-auth-001"), "{removed:?}");
    assert_eq!(ensure_no_lost_requirements(root, &load_change(root, &first.id).unwrap()), Ok(None));
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
