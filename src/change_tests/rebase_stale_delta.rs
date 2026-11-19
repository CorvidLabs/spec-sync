// #1085: `change check` rebases a stale `## MODIFIED` requirement delta over main's edit when the
// two edits do not overlap, keeping the approval and recording a rebase every reader recomputes
// (REQ-change-161). Anything it cannot merge keeps the #843 refusal (REQ-change-122).

const REBASE_BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

/// The change inserts one word into the first sentence.
const REBASE_DELTA: &str = "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

const MERGED_BODY: &str = "The system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after twelve hours.";

/// A repository where `delta` was approved against `REBASE_BASE_REQUIREMENTS` and committed, and
/// main then replaced `from` with `to` in the canonical requirements and committed.
fn rebase_repository(root: &Path, delta: &str, from: &str, to: &str) -> ChangeRecord {
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    fs::write(requirements_path(root), REBASE_BASE_REQUIREMENTS).unwrap();
    let record = approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approved against the base"]);
    let moved = fs::read_to_string(requirements_path(root))
        .unwrap()
        .replace(from, to);
    fs::write(requirements_path(root), moved).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "main edits REQ-auth-001"]);
    record
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
