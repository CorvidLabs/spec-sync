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
    let git = git_in(root);
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), VANISHED_BASE_REQUIREMENTS).unwrap();
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
    assert!(error.contains("cannot read"), "{error}");
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

/// REQ-change-169: a change archived on the branch after the reference accounts for the
/// requirements its archived `## REMOVED` items name.
#[test]
fn a_change_archived_on_the_branch_accounts_for_its_removal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let _record = vanished_repository(root, ADDED_THIRD, &[]);
    let reference = String::from_utf8(
        Command::new("git").args(["rev-parse", "HEAD"]).current_dir(root).output().unwrap().stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let package = root.join(ARCHIVE_PATH).join("2026-11-19-retire-session-one/deltas");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("auth.md"), "## REMOVED\n\n### REQUIREMENT REQ-auth-001\n").unwrap();
    git_in(root)(&["add", "--all"]);
    git_in(root)(&["commit", "-q", "-m", "an archived change's delta"]);

    let removed = removed_requirement_ids_on_branch(root, &reference).unwrap();

    assert!(removed.contains("REQ-auth-001"), "{removed:?}");
}
