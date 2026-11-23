// #1085, option 2 (Leif's "propose, don't approve"): when main has edited a `## MODIFIED`
// requirement since the change's approved base and the two edits do not overlap, `change check`
// writes the word merge into the delta as a proposal and refuses; once a person approves it, the
// check applies it (REQ-change-rebase-proposal-001). Anything it cannot merge keeps the #843 refusal, naming its
// cause (REQ-change-122). Only the delta is ever written: never the canonical requirement, never
// approvals.json.

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

/// The check proposes `merged` for REQ-auth-001 and refuses, writing only the delta: the canonical
/// requirements and approvals.json stay byte for byte, status reports the delta as changed after
/// approval, and a second check refuses it as such without proposing again (REQ-change-rebase-proposal-001).
fn assert_proposed(root: &Path, record: &ChangeRecord, merged: &str, how: &str) {
    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let ledger = fs::read_to_string(&ledger_path).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("proposed rebase")
            && error.contains("auth: REQ-auth-001")
            && error.contains(&format!("specsync change approve {}", record.id)),
        "{how}: {error}"
    );
    let delta = fs::read_to_string(delta_path(root, record, "auth")).unwrap();
    assert!(delta.contains(merged), "{how}: {delta}");
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements, "{how}");
    assert_eq!(fs::read_to_string(&ledger_path).unwrap(), ledger, "{how}: no approval is written");
    assert_eq!(
        changed_approved_delta_modules(root, record).unwrap(),
        vec!["auth".to_string()],
        "{how}"
    );
    assert!(!summarize_change(root, record).approval_valid, "{how}");
    let again = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(again.contains("changed after approval"), "{how}: {again}");
    assert_eq!(
        fs::read_to_string(delta_path(root, record, "auth")).unwrap(),
        delta,
        "{how}: proposed once"
    );
}

/// A person approves the proposal, and the next check applies it.
fn approve_and_apply(root: &Path, record: &ChangeRecord, merged: &str, how: &str) {
    approve_definition(root, &record.id, Some("Scope owner".into()), None)
        .unwrap_or_else(|error| panic!("{how}: {error}"));
    materialize_change_deltas(root, &record.id).unwrap_or_else(|error| panic!("{how}: {error}"));
    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    assert!(requirements.contains(merged), "{how}: {requirements}");
    ensure_approved_delta_bases_current(root, record).unwrap();
    assert!(changed_approved_delta_modules(root, record).unwrap().is_empty(), "{how}");
}

/// The bytes of the change's approvals.json.
fn ledger_bytes(root: &Path, record: &ChangeRecord) -> String {
    fs::read_to_string(change_dir(root, &record.id).join("approvals.json")).unwrap()
}

/// REQ-change-rebase-proposal-001: the word merge takes each side's edit where the other left the text alone,
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
fn rebasing_a_delta_preserves_surrounding_bytes_and_replaces_nested_body() {
    let original = "## MODIFIED\r\n\r\n### requirement REQ-auth-001  \r\n\r\nOld body.\r\n\r\n### Scenario\r\n\r\nOld scenario.\r\n\r\n\r\n### REQUIREMENT REQ-auth-002\r\n\r\nUntouched body.";
    let expected = "## MODIFIED\r\n\r\n### requirement REQ-auth-001  \r\n\r\nNew body.\r\n\r\n### Scenario\r\n\r\nNew scenario.\r\n\r\n\r\n### REQUIREMENT REQ-auth-002\r\n\r\nUntouched body.";
    let items = parse_delta_with_body_ranges(original).unwrap();
    assert_eq!(items.len(), 2);
    let mut actual = original.to_string();
    replace_delta_item_body(
        &mut actual,
        items[0].1.clone(),
        "New body.\n\n### Scenario\n\nNew scenario.",
    );
    assert_eq!(actual, expected);
}

#[test]
fn word_tokens_join_back_to_the_text() {
    for text in ["", "a", "  lead and trail  ", "one\n\ntwo\tthree", "x\r\ny"] {
        assert_eq!(word_tokens(text).concat(), text);
    }
    assert_eq!(word_tokens(" a b\n"), [" ", "a ", "b\n"]);
}

/// REQ-change-rebase-proposal-001 end to end at the unit level: the check writes the merge into the delta as a
/// proposal and refuses; nothing reaches the canonical requirement until a person approves it;
/// then the check applies it, and checking again changes nothing.
#[test]
fn a_non_overlapping_main_edit_is_proposed_and_applies_once_approved() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);

    assert_proposed(root, &record, MERGED_BODY, "proposed");
    approve_and_apply(root, &record, MERGED_BODY, "approved");

    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements);
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
}

/// REQ-change-rebase-proposal-001: an approved proposal is ordinary approved wording, so the change verifies and
/// accepts.
#[test]
fn an_approved_proposal_verifies_and_accepts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    let testing = change_dir(root, &record.id).join("testing.md");
    let mut content = fs::read_to_string(&testing).unwrap_or_default();
    content.push_str(
        "\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-001 | session test |\n",
    );
    fs::write(&testing, content).unwrap();
    materialize_change_deltas(root, &record.id).unwrap_err();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
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

/// REQ-change-rebase-proposal-001: what the merge does not cover keeps the #843 refusal, and nothing is written.
#[test]
fn what_the_rebase_does_not_cover_keeps_the_refusal() {
    fn assert_refused_untouched(root: &Path, record: &ChangeRecord, what: &str) -> String {
        let requirements = fs::read_to_string(requirements_path(root)).unwrap_or_default();
        let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
        let delta = fs::read_to_string(delta_path(root, record, "auth")).unwrap();
        let ledger = ledger_bytes(root, record);
        let error = materialize_change_deltas(root, &record.id).unwrap_err();
        assert!(error.contains("has changed since"), "{what}: {error}");
        assert_eq!(fs::read_to_string(requirements_path(root)).unwrap_or_default(), requirements, "{what}");
        assert_eq!(fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(), spec, "{what}");
        assert_eq!(fs::read_to_string(delta_path(root, record, "auth")).unwrap(), delta, "{what}");
        assert_eq!(ledger_bytes(root, record), ledger, "{what}");
        error
    }

    // Overlapping edits: refused, naming the cause and the way on.
    let temp = TempDir::new().unwrap();
    let record = rebase_repository(temp.path(), REBASE_DELTA, "keep one session", "keep a single session");
    let error = assert_refused_untouched(temp.path(), &record, "overlap");
    assert!(
        error.contains("main's edit at") && error.contains("overlaps this change's words"),
        "{error}"
    );
    assert!(!error.contains("proposed rebase"), "{error}");

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

/// REQ-change-rebase-proposal-001: a workflow-v1 change's definition digest hashes its delta bodies, so a rebase
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
fn a_branch_side_edit_is_never_proposed_into_the_delta() {
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
    // The refusal names the way on: take main's text for the requirement and check again, and
    // the check proposes a rebase for a person to approve.
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        error.contains("git show origin/main:specs/auth/requirements.md")
            && error.contains("proposes")
            && !error.contains("the approval stands"),
        "{error}"
    );

    // After materialization: the same edit, made to the materialized requirement.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    materialize_change_deltas(root, &record.id).unwrap();
    edit_requirements(root, "after one day.", "after one day, or at sign-out.");
    assert_refused_untouched(root, &record, "after materialization");

    // A merge resolution that keeps main's edit and adds a sentence of its own: main edited the
    // requirement, and the working tree holds none of the three texts, which is named.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebase_repository(root, REBASE_DELTA, "after one day.", "after twelve hours.");
    edit_requirements(root, "after twelve hours.", "after twelve hours, or at sign-out.");
    assert_refused_untouched(root, &record, "conflict resolution");
    let error = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(error.contains("is none of main's text"), "{error}");

    // With no default branch to read main's text from, there is nothing to rebase over.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    git_in(root)(&["remote", "remove", "origin"]);
    edit_requirements(root, "after one day.", "after twelve hours.");
    git_in(root)(&["commit", "-qam", "main edits, no origin"]);
    assert_refused_untouched(root, &record, "no default branch");
}

/// fable-lead's second #1089 review point: "text the default branch held" must mean held after the
/// approval-time base. Main went v1 -> v2, the change was approved against v2, and the branch's
/// working tree holds v1. v1 is in main's history, but older than the base, so taking it as main's
/// block would revert main's v1-to-v2 edit inside the delta with the approval intact. Refused.
#[test]
fn an_older_main_text_than_the_base_is_never_proposed_into_the_delta() {
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
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("has changed since"), "{error}");
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}

/// corvid-agent's review of #1089: when one item of a change cannot be proposed, no item is.
/// REQ-auth-001's edits are words apart; REQ-auth-002's overlap. The check refuses REQ-auth-002,
/// naming the overlap, and the delta and approvals.json stay byte for byte.
#[test]
fn a_mixed_refusal_proposes_nothing() {
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
    assert!(
        error.contains("REQ-auth-002 (main's edit at") && error.contains("overlaps"),
        "{error}"
    );
    assert!(!error.contains("proposed rebase"), "{error}");
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

/// ss-lead's review of #1089 (F2): "text the default branch held" means main's own line, walked
/// with `--first-parent`. An intermediate text of a side branch that its merge did not keep was
/// never main's, so a working tree holding it is refused, not rebased.
#[test]
fn a_side_branch_text_its_merge_did_not_keep_is_never_main_s_block() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    let git = git_in(root);
    git(&["checkout", "-q", "-b", "side"]);
    edit_requirements(root, "after one day.", "after one day, or at sign-out.");
    git(&["commit", "-qam", "side: an intermediate text"]);
    edit_requirements(root, "after one day, or at sign-out.", "after twelve hours.");
    git(&["commit", "-qam", "side: the text the merge keeps"]);
    git(&["checkout", "-q", "main"]);
    git(&["merge", "-q", "--no-ff", "-m", "merge side", "side"]);
    git(&["push", "-q", "origin", "main"]);
    // The change's branch holds the side branch's intermediate text.
    edit_requirements(root, "after twelve hours.", "after one day, or at sign-out.");
    git(&["commit", "-qam", "the intermediate text on the change's branch"]);
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("has changed since"), "{error}");
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}

/// ss-lead's review of #1089 (F1): a commit of the default branch whose tree lists the
/// requirement's file but whose text cannot be read refuses the rebase. Skipping it could walk past
/// the approval-time text to an older one. Here the approval-time commit's file object is gone,
/// the branch's own approval commit still lets the base be recovered, and the working tree holds
/// the older text: without the refusal it would be merged, reverting main's edit.
#[test]
fn an_unreadable_default_branch_text_refuses_the_proposal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, REBASE_DELTA);
    let v1 = REBASE_BASE_REQUIREMENTS.replace("after one day.", "after two days.");
    fs::write(requirements_path(root), &v1).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "v1"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "v2, the approval-time text"]);
    git(&["push", "-q", "origin", "main"]);
    let main_blob = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD:specs/auth/requirements.md"])
            .current_dir(root)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    // The change's branch: approved against v2, with a note of its own outside the requirement,
    // so its file is a different object that still holds the approval-time block.
    let noted = |text: &str| text.replace("# Requirements\n", "# Requirements\n\nBranch note.\n");
    fs::write(requirements_path(root), noted(REBASE_BASE_REQUIREMENTS)).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "approved on the branch"]);
    fs::write(requirements_path(root), noted(&v1)).unwrap();
    git(&["commit", "-qam", "the older text on the branch"]);
    // Main's approval-time object can no longer be read.
    let loose = root
        .join(".git/objects")
        .join(&main_blob[..2])
        .join(&main_blob[2..]);
    fs::remove_file(&loose).unwrap();
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    // The refusal names its cause and the way on, never "rewrite and re-approve" (the tester's run
    // at 0d6ca1cd).
    assert!(
        error.contains("REQ-auth-001")
            && error.contains("cannot be read from Git")
            && error.contains("git fetch")
            && !error.contains("re-run `specsync change approve"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}

/// Codex's review of #1089 (P1 at the merge): a word one side deleted never comes back through the
/// other side's insertion, so a moved word against a deletion refuses, in either direction.
#[test]
fn a_word_one_side_deleted_is_never_put_back_by_the_other() {
    assert_eq!(merge_words("a b c", "a c", "b a c"), None);
    assert_eq!(merge_words("a b c", "b a c", "a c"), None);
    // An insertion of a word nobody deleted still merges.
    assert_eq!(merge_words("a b c x", "a c x", "a b c d x").as_deref(), Some("a c d x"));
}

/// ss-lead's review of the option-2 wording (Leif's rule 3): a requirement longer than the merge
/// reads (4,000 words a side) is refused, naming that limit and the way on, and nothing is written.
#[test]
fn a_requirement_longer_than_the_merge_reads_is_refused_naming_the_limit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let padding = "word ".repeat(MAX_REBASE_TOKENS);
    let long = REBASE_DELTA.replace(
        "The system SHALL keep exactly one session per user.",
        &format!("The system SHALL keep exactly one session per user, {padding}."),
    );
    let record = rebase_repository(root, &long, "after one day.", "after twelve hours.");
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("REQ-auth-001")
            && error.contains("longer than the merge reads (4,000 words a side)")
            && error.contains("rewrite the body onto main's text by hand")
            && error.contains(&format!("specsync change approve {}", record.id)),
        "{error}"
    );
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}

/// Codex's review of #1089 (P2 at the hint): the way on names the file of a module with a stale
/// requirement, never one whose only stale item is a section. `auth` sorts first and has only a
/// stale section; `billing` has the stale requirement.
#[test]
fn the_rebase_hint_names_the_module_with_the_stale_requirement() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let mut record = completed_section_only_current_record(
        root,
        "## MODIFIED\n\n### SPEC SECTION Legacy Notes\n\nRetained for compatibility with every client.\n",
    );
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    fs::create_dir_all(root.join("specs/billing")).unwrap();
    fs::write(root.join("src/billing.rs"), "// Billing module.\n").unwrap();
    let billing_spec = fs::read_to_string(root.join("specs/auth/auth.spec.md"))
        .unwrap()
        .replace("module: auth", "module: billing")
        .replace("src/auth.rs", "src/billing.rs")
        .replace("# Auth\n", "# Billing\n");
    fs::write(root.join("specs/billing/billing.spec.md"), &billing_spec).unwrap();
    let billing = "---\nspec: billing.spec.md\n---\n\n# Requirements\n\n### REQ-billing-001\n\nThe system SHALL bill once per month.\n\nAcceptance Criteria\n- One bill per month.\n";
    fs::write(root.join("specs/billing/requirements.md"), billing).unwrap();
    record.affected_specs = vec!["auth".into(), "billing".into()];
    save_change(root, &record).unwrap();
    fs::write(
        delta_path(root, &record, "billing"),
        "## MODIFIED\n\n### REQUIREMENT REQ-billing-001\n\nThe system SHALL bill exactly once per month.\n\nAcceptance Criteria\n- One bill per month.\n",
    )
    .unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "approved"]);
    let origin = root.join(".git/test-origin.git");
    git(&["init", "-q", "--bare", "-b", "main", origin.to_str().unwrap()]);
    git(&["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&["push", "-q", "origin", "main"]);
    let spec = root.join("specs/auth/auth.spec.md");
    let moved = fs::read_to_string(&spec)
        .unwrap()
        .replace("Retained for compatibility.", "Kept, retained for compatibility.");
    fs::write(&spec, moved).unwrap();
    fs::write(
        root.join("specs/billing/requirements.md"),
        billing.replace("- One bill per month.\n", "- One bill per month.\n- Bills are emailed.\n"),
    )
    .unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("git show origin/main:specs/billing/requirements.md"), "{error}");
    assert!(!error.contains("specs/auth/requirements.md"), "{error}");
}

/// Run Git, failing with its output.
fn git_shown(root: &Path, args: &[&str]) {
    let output = Command::new("git").args(args).current_dir(root).output().unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn rev(root: &Path, revision: &str) -> String {
    let output = Command::new("git")
        .args(["rev-parse", revision])
        .current_dir(root)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// The change on its own branch `change`, materialized and committed; then main replaces `from`
/// with `to` on `main`, commits and pushes it, and `change` is checked out again, not yet merged.
fn materialized_then_main_edits(root: &Path, record: &ChangeRecord, from: &str, to: &str) {
    let git = git_in(root);
    git(&["checkout", "-q", "-b", "change"]);
    materialize_change_deltas(root, &record.id).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "materialize on the change's branch"]);
    git(&["checkout", "-q", "main"]);
    edit_requirements(root, from, to);
    git(&["commit", "-qam", "main edits REQ-auth-001"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
}

/// The tester's #1089 regression, first half: the delta was materialized before main's first
/// edit, and then the branch merged main. Main's block is read at HEAD's merge-base with the
/// default branch, never from the working tree, so each of the three working-tree texts gets the
/// same proposal: Git's own clean merge, a resolution to the change's side, and a resolution to
/// main's side. Once approved, each applies.
#[test]
fn a_materialized_change_that_merged_main_gets_the_same_proposal_whichever_side_the_merge_kept() {
    for (how, merge) in [
        ("Git's clean merge", &["merge", "-q", "--no-edit", "origin/main"][..]),
        ("resolved to the change's side", &["merge", "-q", "-s", "ours", "--no-edit", "origin/main"][..]),
        ("resolved to main's side", &["merge", "-q", "--no-edit", "origin/main"][..]),
    ] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = approved_repository_with_origin(root, REBASE_DELTA);
        // Main edits a line the change's materialization leaves apart (it adds a blank line at the
        // end of the file, which a last-line edit would touch), so Git merges it cleanly.
        materialized_then_main_edits(
            root,
            &record,
            "- One session per user.",
            "- One session per user and device.",
        );
        let git = git_in(root);
        git_shown(root, merge);
        if how == "resolved to main's side" {
            git(&["checkout", "origin/main", "--", "specs/auth/requirements.md"]);
            git(&["commit", "-qm", "take main's requirement"]);
        }
        let merged = "The system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user and device.\n- A session expires after one day.";

        assert_proposed(root, &record, merged, how);
        approve_and_apply(root, &record, merged, how);
    }
}

/// The tester's #1089 regression, second half: a second real main edit after an approved proposal
/// was materialized. The branch's block is then the change's text plus both of main's edits, which
/// main never held; it is the merge of main's earlier block (the base the approval recorded),
/// main's new block and the approved body, so it is proposed again.
#[test]
fn a_second_main_edit_after_an_approved_proposal_is_proposed_again() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    assert_proposed(root, &record, MERGED_BODY, "first edit");
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    materialized_then_main_edits(
        root,
        &record,
        "- One session per user.",
        "- One session per user and device.",
    );
    git_shown(root, &["merge", "-q", "--no-edit", "origin/main"]);

    let twice = MERGED_BODY.replace("- One session per user.", "- One session per user and device.");
    assert_proposed(root, &record, &twice, "second edit");
    approve_and_apply(root, &record, &twice, "second edit");
}

/// ss-lead's #1089 rule (3): an item main has edited past its approved base is stale even when
/// the working tree carries the change's body, since keeping that body drops main's edit. When
/// main's next edit overlaps the change's words and the branch kept its own text, the check
/// refuses, naming the overlap, and writes nothing.
#[test]
fn an_overlapping_main_edit_after_an_approved_proposal_is_refused_even_when_the_branch_kept_its_text() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = rebased_repository(root);
    materialize_change_deltas(root, &record.id).unwrap_err();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    materialized_then_main_edits(root, &record, "keep one session", "keep a single session");
    git_in(root)(&["merge", "-q", "-s", "ours", "--no-edit", "origin/main"]);
    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger_path = change_dir(root, &record.id).join("approvals.json");
    let ledger = fs::read_to_string(&ledger_path).unwrap();

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("has changed since")
            && error.contains("REQ-auth-001 (main's edit at")
            && error.contains("overlaps"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements);
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(fs::read_to_string(&ledger_path).unwrap(), ledger);
}

/// Codex's third review of #1089 (P1 at merge_words): both sides moving one word to different
/// places once gave `b a c b d`. Both remove it from where it stood and each adds it elsewhere, so
/// the move rule refuses. A move only one side makes merges with the other side's edit elsewhere.
#[test]
fn a_word_both_sides_move_refuses_and_a_one_sided_move_merges() {
    assert_eq!(merge_words("a b c d", "b a c d", "a c b d"), None);
    assert_eq!(
        merge_words("a b c d e", "a b c d E", "a c b d e").as_deref(),
        Some("a c b d E")
    );
    assert_eq!(
        merge_words("a b c d e", "a c b d e", "a b c d E").as_deref(),
        Some("a c b d E")
    );
}

/// R2 (ss-lead's review of #1089, measured by the tester): a word one side deleted that the other
/// side's new text also uses elsewhere is no move, so the rebase merges. Only a word both sides
/// removed from one stretch the same way and one side alone adds elsewhere refuses.
#[test]
fn a_word_one_side_deleted_that_the_other_uses_elsewhere_still_merges() {
    assert_eq!(
        merge_words(
            "The system SHALL end the session after one day of no use.",
            "The system SHALL end the session after one day of no use. It SHALL tell the user why.",
            "The system SHALL end each session after one day of no use.",
        )
        .as_deref(),
        Some("The system SHALL end each session after one day of no use. It SHALL tell the user why.")
    );
    assert_eq!(
        merge_words(
            "Run the delta check now.",
            "Run the delta check now. Each delta is named.",
            "Run the delta, check now.",
        )
        .as_deref(),
        Some("Run the delta, check now. Each delta is named.")
    );
    // A word both deleted at one place and one side adds at another is still a move.
    assert_eq!(merge_words("a b c", "a c", "b a c"), None);
    assert_eq!(merge_words("a w b c", "a b c", "a b w c"), None);
}

/// Codex's third review of #1089 (P1 at main_side's caller), as ss-lead narrowed it: with a default
/// branch, an applied item whose main text Git cannot read is refused, never read as "main did
/// not edit it". Without a default branch an applied item materializes as before.
#[test]
fn an_applied_item_whose_main_text_git_cannot_read_is_refused_only_with_a_default_branch() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    let git = git_in(root);
    git(&["checkout", "-q", "-b", "change"]);
    materialize_change_deltas(root, &record.id).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "materialize"]);
    let blob = rev(root, "origin/main:specs/auth/requirements.md");
    fs::remove_file(root.join(".git/objects").join(&blob[..2]).join(&blob[2..])).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(error.contains("REQ-auth-001") && error.contains("cannot be read from Git"), "{error}");
    assert_eq!(ledger_bytes(root, &record), ledger);

    // No default branch: the applied item is not main's edit to merge, exactly as before.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    let git = git_in(root);
    git(&["remote", "remove", "origin"]);
    materialize_change_deltas(root, &record.id).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "materialize"]);
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
}

/// Codex's fourth review of #1089 (P1 at :130), the tester's case: both sides move a repeated
/// word, and the merge kept one `alpha` where main and the change each keep two. The word-count
/// check refuses any merge that loses or invents a word.
#[test]
fn a_repeated_word_both_sides_move_is_refused_by_the_word_count() {
    assert_eq!(merge_words("a b a c d", "a b c a d", "b a c a d"), None);
    assert_eq!(
        merge_words(
            "alpha beta alpha gamma delta",
            "alpha beta gamma alpha delta",
            "beta alpha gamma alpha delta",
        ),
        None
    );
}

/// Codex's fourth review of #1089 (P1 at :599), the tester's case: REQ-auth-001 is materialized on
/// the branch, main deletes the block, and the merge keeps the branch's block. Main has edited
/// the item (its block is gone), so keeping the change's body would bring the requirement back.
#[test]
fn a_requirement_main_deleted_is_refused_even_when_the_branch_kept_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    let git = git_in(root);
    git(&["checkout", "-q", "-b", "change"]);
    materialize_change_deltas(root, &record.id).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-qm", "materialize"]);
    git(&["checkout", "-q", "main"]);
    fs::write(
        requirements_path(root),
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n",
    )
    .unwrap();
    git(&["commit", "-qam", "main deletes REQ-auth-001"]);
    git(&["push", "-q", "origin", "main"]);
    git(&["checkout", "-q", "change"]);
    git(&["merge", "-q", "-s", "ours", "--no-edit", "origin/main"]);
    let requirements = fs::read_to_string(requirements_path(root)).unwrap();
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("REQ-auth-001 (main deleted this requirement at")
            && error.contains("drop the item from the delta"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(requirements_path(root)).unwrap(), requirements);
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}

/// Codex's fourth review of #1089 (P1 at :883), the tester's case: main edits the heading, and the
/// branch puts the old heading back. A rebase compares blocks with their headings, and the delta
/// cannot carry main's heading, so it is refused instead of losing it.
#[test]
fn a_heading_main_edited_is_never_lost_by_a_proposal() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = approved_repository_with_origin(root, REBASE_DELTA);
    edit_requirements(root, "### REQ-auth-001\n", "### REQ-auth-001: One session per user\n");
    let git = git_in(root);
    git(&["commit", "-qam", "main edits the heading"]);
    git(&["push", "-q", "origin", "main"]);
    edit_requirements(root, "### REQ-auth-001: One session per user\n", "### REQ-auth-001\n");
    git(&["commit", "-qam", "the branch puts the old heading back"]);
    let delta = fs::read_to_string(delta_path(root, &record, "auth")).unwrap();
    let ledger = ledger_bytes(root, &record);

    let error = materialize_change_deltas(root, &record.id).unwrap_err();

    assert!(
        error.contains("REQ-auth-001 (main edited this requirement's heading at"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(delta_path(root, &record, "auth")).unwrap(), delta);
    assert_eq!(ledger_bytes(root, &record), ledger);
}
