// #1085: `change check` rebases a stale `## MODIFIED` requirement delta over main's edit when the
// two edits do not overlap, keeping the approval and recording a rebase every reader recomputes
// (REQ-change-161). Anything it cannot merge keeps the #843 refusal (REQ-change-122).

const REBASE_BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

/// The change inserts one word into the first sentence.
const REBASE_DELTA: &str = "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

const MERGED_BODY: &str = "The system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after twelve hours.";

/// A repository where `delta` was approved against `REBASE_BASE_REQUIREMENTS` and committed, and
/// main then replaced `from` with `to` in the canonical requirements and committed.
/// A repository with an `origin` (a bare repository inside `.git`, so the working tree never sees
/// it) where `delta` was approved against `REBASE_BASE_REQUIREMENTS`; then main replaced `from` with
/// `to` in the canonical requirements, committed it and pushed it, so `origin/main` holds main's
/// text the way a merged pull request leaves it.
fn rebase_repository(root: &Path, delta: &str, from: &str, to: &str) -> ChangeRecord {
    let record = approved_repository_with_origin(root, delta);
    edit_requirements(root, from, to);
    let git = git_in(root);
    git(&["add", "--all"]);
    git(&["commit", "-m", "main edits REQ-auth-001"]);
    git(&["push", "-q", "origin", "main"]);
    record
}

/// `delta` approved against `REBASE_BASE_REQUIREMENTS`, committed and pushed to a bare `origin`.
fn approved_repository_with_origin(root: &Path, delta: &str) -> ChangeRecord {
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approved against the base"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    record
}

fn edit_requirements(root: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(requirements_path(root)).unwrap();
    let moved = text.replace(from, to);
    assert_ne!(moved, text, "the fixture edit must apply");
    fs::write(requirements_path(root), moved).unwrap();
}

fn rebased_repository(root: &Path) -> ChangeRecord {
    rebase_repository(root, REBASE_DELTA, "after one day.", "after twelve hours.")
}

/// REQ-change-161: the word merge takes each side's edit where the other left the text alone,
/// and refuses where both changed the same words or inserted at the same point.
#[test]
fn merge_words_merges_edits_apart_and_refuses_edits_that_touch() {
    let base = "keep one session per user and expire it after one day";
    assert_eq!(
        merge_words(
            base,
            "keep one session per user and expire it after twelve hours",
            "keep exactly one session per user and expire it after one day",
        )
        .as_deref(),
        Some("keep exactly one session per user and expire it after twelve hours")
    );
    // Same words changed on both sides.
    assert_eq!(
        merge_words(base, "keep a single session per user and expire it after one day", "keep exactly one session per user and expire it after one day"),
        None
    );
    // Two insertions at one point.
    assert_eq!(merge_words("a b", "a x b", "a y b"), None);
    // The same change on both sides is no conflict.
    assert_eq!(merge_words("a b", "a x b", "a x b").as_deref(), Some("a x b"));
    // An unmoved base takes the approved body as it is.
    assert_eq!(merge_words("a b", "a b", "c d").as_deref(), Some("c d"));
    // Line breaks travel with their words.
    assert_eq!(
        merge_words("S\n\nAC\n- one", "S\n\nAC\n- one changed", "S2\n\nAC\n- one").as_deref(),
        Some("S2\n\nAC\n- one changed")
    );
    let long = "word ".repeat(MAX_REBASE_TOKENS + 1);
    assert_eq!(merge_words(&long, &long, "word"), None, "an oversized side refuses");
}

#[test]
fn word_tokens_join_back_to_the_text() {
    for text in ["", "a", "  lead and trail  ", "one\n\ntwo\tthree", "x\r\ny"] {
        assert_eq!(word_tokens(text).concat(), text);
    }
    assert_eq!(word_tokens(" a b\n"), [" ", "a ", "b\n"]);
}

/// REQ-change-161 end to end at the unit level: the merge reaches the canonical requirement, the
/// delta and the binding move together, no approval is added, and a second check, a re-approval
/// and the base guard all accept the rebased delta.
#[test]
fn a_rebase_moves_the_bound_delta_and_base_and_materializes_the_merge() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    let approvals = load_approvals(root, &record).unwrap().approvals.len();

    materialize_change_deltas(root, &record.id).unwrap();

    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    assert!(requirements.contains(MERGED_BODY), "{requirements}");
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    assert!(delta.contains(MERGED_BODY), "{delta}");
    let ledger = load_approvals(root, &record).unwrap();
    assert_eq!(ledger.approvals.len(), approvals, "no approval is added");
    assert_eq!(ledger.delta_rebases.len(), 1);
    let rebase = &ledger.delta_rebases[0];
    assert_eq!(rebase.actor, DELTA_REBASE_ACTOR);
    assert_eq!(rebase.item, "auth: REQUIREMENT REQ-auth-001");
    assert_eq!(rebase.delta_before, REBASE_DELTA);
    assert!(rebase.base_block.contains("after one day."), "{rebase:?}");
    assert!(rebase.main_block.contains("after twelve hours."), "{rebase:?}");
    assert!(changed_approved_delta_modules(root, &record).unwrap().is_empty());
    ensure_approved_delta_bases_current(root, &record).unwrap();
    assert_eq!(
        recorded_delta_rebases(root, &record),
        vec![DeltaRebaseSummary {
            module: "auth".into(),
            requirement: "REQ-auth-001".into(),
            rebased_at: rebase.timestamp,
        }]
    );

    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements);
    assert_eq!(load_approvals(root, &record).unwrap().delta_rebases.len(), 1);

    // A later approval of the rebased delta as it stands carries the rebased base forward.
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    ensure_approved_delta_bases_current(root, &record).unwrap();
    assert!(changed_approved_delta_modules(root, &record).unwrap().is_empty());
}

/// REQ-change-161: a record is recomputed, not trusted. Editing what it holds, or the delta it
/// claims to have produced, is refused and names the record.
#[test]
fn a_rebase_record_that_does_not_recompute_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let honest = fs::read_to_string(&ledger_path).unwrap();

    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.delta_rebases[0].main_block = ledger.delta_rebases[0]
        .main_block
        .replace("twelve hours", "twelve years");
    write_json(&ledger_path, &ledger).unwrap();
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        error.contains("delta rebase 1") && error.contains("does not recompute"),
        "{error}"
    );
    assert!(
        error.contains(&format!("specsync change approve {}", record.id)),
        "{error}"
    );
    assert!(!summarize_change(root, &record).approval_valid);

    // A delta edited after the rebase is the ordinary "changed after approval".
    fs::write(&ledger_path, &honest).unwrap();
    let delta = delta_path(root, &record, "auth");
    let rebased = fs::read_to_string(&delta).unwrap();
    fs::write(&delta, rebased.replace("exactly one", "at most one")).unwrap();
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(error.contains("changed after approval"), "{error}");
}

/// REQ-change-161: acceptance reads the same effective claims, so a rebased change accepts.
#[test]
fn a_rebased_change_verifies_and_accepts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    let testing = change_dir(root, &record.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap_or_default();
    content.push_str(
        "\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-001 | session test |\n",
    );
    fs::write(&testing, content).unwrap();
    let record = start_implementation(root, &record.id).unwrap();
    let verification = check_change(root, Some(&record.id)).unwrap().unwrap();
    assert!(verification.passed, "{verification:?}");
    assert!(
        fs::read_to_string(requirements_path(root))
            .unwrap()
            .contains(MERGED_BODY)
    );
    let accepted = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
    assert_eq!(accepted.state, ChangeState::Accepted);
}

/// REQ-change-161: what the merge does not cover keeps the #843 refusal, and nothing is written.
#[test]
fn what_the_rebase_does_not_cover_keeps_the_refusal() {
    fn assert_refused_untouched(root: &Path, record: &ChangeRecord, what: &str) {
        let requirements = fs::read_to_string(requirements_path(root)).unwrap_or_default();
        let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
        let delta = fs::read_to_string(delta_path(root, record, "auth")).unwrap();
        let error = materialize_change_deltas(root, &record.id).unwrap_err();
        assert!(error.contains("has changed since"), "{what}: {error}");
        assert_eq!(fs::read_to_string(requirements_path(root)).unwrap_or_default(), requirements, "{what}");
        assert_eq!(fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(), spec, "{what}");
        assert_eq!(fs::read_to_string(delta_path(root, record, "auth")).unwrap(), delta, "{what}");
        assert!(load_approvals(root, record).unwrap().delta_rebases.is_empty(), "{what}");
    }

    // Overlapping edits.
    let temp = TempDir::new().unwrap();
    let record = rebase_repository(temp.path(), REBASE_DELTA, "keep one session", "keep a single session");
    assert_refused_untouched(temp.path(), &record, "overlap");

    // A MODIFIED spec section, with edits words apart.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Legacy Notes\n\nRetained for compatibility with every client.\n",
    );
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approved"]);
    let spec_path = root.join("specs/auth/auth.spec.md");
    let moved = fs::read_to_string(&spec_path)
        .unwrap()
        .replace("Retained for compatibility.", "Kept, retained for compatibility.");
    fs::write(&spec_path, moved).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "main edits the section"]);
    assert_refused_untouched(root, &record, "spec section");

    // A base Git cannot give back: no repository at all.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, REBASE_DELTA);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    fs::write(
        requirements_path(root),
        REBASE_BASE_REQUIREMENTS.replace("after one day.", "after twelve hours."),
    )
    .unwrap();
    assert_refused_untouched(root, &record, "no history");
}

/// REQ-change-161: a workflow-v1 change's definition digest hashes its delta bodies, so a rebase
/// would change what its approval signed; it keeps the refusal.
#[test]
fn a_workflow_v1_change_keeps_the_refusal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_record(root, REBASE_DELTA);
    assert_eq!(record.workflow_version, 1);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approved"]);
    fs::write(
        requirements_path(root),
        REBASE_BASE_REQUIREMENTS.replace("after one day.", "after twelve hours."),
    )
    .unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "main edits"]);
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("has changed since"), "{error}");
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
}

/// fable-lead's #1089 review: main's block comes from main. A sentence added on the change's own
/// branch, before the first check, after materialization or while resolving a merge, is not
/// text the default branch held, so it is never merged into the approved delta: the check refuses
/// it as stale, exactly as before, and writes nothing.
#[test]
fn a_branch_side_edit_is_never_rebased_into_the_delta() {
    fn assert_refused_untouched(root: &Path, record: &ChangeRecord, what: &str) {
        let requirements = fs::read_to_string(requirements_path(root)).unwrap();
        let delta = fs::read_to_string(delta_path(root, record, "auth")).unwrap();
        let ledger = fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap();
        let error = materialize_change_deltas(root, &record.id).unwrap_err();
        assert!(error.contains("has changed since"), "{what}: {error}");
        assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements, "{what}");
        assert_eq!(fs::read_to_string(delta_path(root, record, "auth")).unwrap(), delta, "{what}");
        assert_eq!(
            fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap(),
            ledger,
            "{what}"
        );
    }

    // Before the first check: a hand edit, committed on the branch but never on main.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    edit_requirements(root, "after one day.", "after one day, or at sign-out.");
    git_in(root)(&["commit", "-qam", "a branch-side edit"]);
    assert_refused_untouched(root, &record, "before the first check");
    // The refusal names the way on: take main's text for the requirement and check again.
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        error.contains("git show origin/main:specs/auth/requirements.md"),
        "{error}"
    );

    // After materialization: the same edit, made to the materialized requirement.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    materialize_change_deltas(root, &record.id).unwrap();
    edit_requirements(root, "after one day.", "after one day, or at sign-out.");
    assert_refused_untouched(root, &record, "after materialization");

    // A merge resolution that keeps main's edit and adds a sentence of its own.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebase_repository(root, REBASE_DELTA, "after one day.", "after twelve hours.");
    edit_requirements(root, "after twelve hours.", "after twelve hours, or at sign-out.");
    assert_refused_untouched(root, &record, "conflict resolution");

    // With no default branch to read main's text from, there is nothing to rebase over.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    git_in(root)(&["remote", "remove", "origin"]);
    edit_requirements(root, "after one day.", "after twelve hours.");
    git_in(root)(&["commit", "-qam", "main edits, no origin"]);
    assert_refused_untouched(root, &record, "no default branch");
}

/// fable-lead's #1089 review, reader side: while the change is delivered, every read re-finds the
/// record's main block in the default branch's history. A record whose main block the default
/// branch no longer holds (here, origin/main is moved back past it) is refused, naming the record.
#[test]
fn a_rebase_whose_main_block_the_default_branch_does_not_hold_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(load_approvals(root, &record).unwrap().delta_rebases.len(), 1);
    let before_edit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD~1"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    git_in(root)(&["update-ref", "refs/remotes/origin/main", before_edit.trim()]);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("delta rebase 1") && error.contains("default branch"),
        "{error}"
    );
    assert!(!summarize_change(root, &record).approval_valid);
}


/// fable-lead's second #1089 review point: "text the default branch held" must mean held after the
/// approval-time base. Main went v1 -> v2, the change was approved against v2, and the branch's
/// working tree holds v1. v1 is in main's history, but older than the base, so taking it as main's
/// block would revert main's v1-to-v2 edit inside the delta with the approval intact. Refused.
#[test]
fn an_older_main_text_than_the_base_is_never_rebased_into_the_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, REBASE_DELTA);
    // v1, committed and pushed, then main moves to v2 (the base the change is approved against).
    let v1 = REBASE_BASE_REQUIREMENTS.replace("after one day.", "after two days.");
    fs::write(requirements_path(root), &v1).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "v1"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "v2"]);
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "approved against v2"]);
    git(&["push", "-q", "origin", "main"]);
    // The branch's working tree goes back to v1.
    fs::write(requirements_path(root), &v1).unwrap();
    git(&["commit", "-qam", "back to v1 on the branch"]);
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("has changed since"), "{error}");
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert!(load_approvals(root, &record).unwrap().delta_rebases.is_empty());
}

/// corvid-agent's review of #1089: a refusal writes nothing, even when another item of the same
/// change could have been rebased. REQ-auth-001's edits are words apart; REQ-auth-002's overlap.
/// The check refuses REQ-auth-002, and the delta and approvals.json stay byte for byte.
#[test]
fn a_mixed_rebase_refusal_writes_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let two = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n\n### REQ-auth-002\n\nThe system SHALL let a user sign in with a passkey.\n\nAcceptance Criteria\n- A passkey signs the user in.\n";
    let delta = "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL let a user sign in with a passkey or a key.\n\nAcceptance Criteria\n- A passkey signs the user in.\n";
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), two).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "approved"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    // Main edits REQ-auth-001 apart from the change's words, and REQ-auth-002 on the same words.
    edit_requirements(root, "after one day.", "after twelve hours.");
    edit_requirements(root, "with a passkey.", "with a passkey or a card.");
    git(&["commit", "-qam", "main edits both"]);
    git(&["push", "-q", "origin", "main"]);
    let delta_before = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let ledger_before = fs::read_to_string(&ledger_path).unwrap();
    let requirements_before = fs::read_to_string(requirements_path(root)).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("has changed since"), "{error}");
    assert!(error.contains("REQ-auth-002"), "{error}");
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta_before);
    assert_eq!(fs::read_to_string(&ledger_path).unwrap(), ledger_before);
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements_before);
}

/// Codex's review of #1089 (P1 at merge_words): with a repeated word, two LCS alignments could
/// anchor it at different occurrences and merge edits to the same words. Anchors are words unique
/// in all three texts, so an ambiguous stretch refuses.
#[test]
fn a_repeated_word_never_aligns_two_edits_to_the_same_words() {
    assert_eq!(merge_words("c a b d", "b b d", "b d"), None);
    assert_eq!(merge_words("x x y", "x x z", "x w y"), None);
    // Words apart still merge when anchors separate them.
    assert_eq!(
        merge_words("one two three four", "one two three FOUR", "ONE two three four").as_deref(),
        Some("ONE two three FOUR")
    );
}

/// Codex's review of #1089 (P2 at the approval key): a re-approval in the same second with the same
/// digest is a different approval, so it does not inherit the earlier approval's rebases.
#[test]
fn a_same_second_re_approval_does_not_inherit_earlier_rebases() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    materialize_change_deltas(root, &record.id).unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger = load_approvals(root, &record).unwrap();
    assert_eq!(ledger.delta_rebases.len(), 1);
    let rebase = ledger.delta_rebases[0].clone();
    // A later approval with the same time and digest as the one the rebase extends.
    let copy = ledger.approvals[rebase.approval_index].clone();
    ledger.approvals.push(copy);
    write_json(&ledger_path, &ledger).unwrap();
    let ledger = load_approvals(root, &record).unwrap();
    let latest = ledger.approvals.last().unwrap();
    let current = load_change(root, &record.id).unwrap();

    let (digests, _) = effective_delta_claims(root, &current, &ledger, latest).unwrap();

    assert_ne!(
        digests.unwrap().get("auth"),
        Some(&rebase.delta_after_digest),
        "the copy binds its own digests, not the earlier approval's rebase"
    );
}

/// Codex's review of #1089 (P2 at status): `change status` lists a rebase only when it recomputes,
/// so it never says "the approval stands" for a record the binding refuses.
#[test]
fn status_lists_no_rebase_that_does_not_recompute() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(recorded_delta_rebases(root, &record).len(), 1);
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger = load_approvals(root, &record).unwrap();
    ledger.delta_rebases[0].main_block = ledger.delta_rebases[0]
        .main_block
        .replace("twelve hours", "twelve years");
    write_json(&ledger_path, &ledger).unwrap();

    assert!(recorded_delta_rebases(root, &record).is_empty());
    assert!(!summarize_change(root, &record).approval_valid);
}
