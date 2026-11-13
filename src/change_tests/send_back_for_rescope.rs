// Issue #844, option A: a change whose scope turned out wrong after approval can be sent back to
// draft to be re-scoped, instead of having its workspace deleted by hand (CHANGE-12).
//
// The fixture is the chough#69 order in miniature: approve, materialize and verify, then find the
// scope wrong. Before this change the only scope verb, `change answer`, refused anything that was
// not a draft, and nothing could move a verifying change back.

const SEND_BACK_DELTA: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials and sessions.\n";

const SEND_BACK_REASON: &str = "the session code moved into src/session.rs";

/// The auth spec owns both files, so widening the change's paths stays inside its one module.
fn auth_spec_owning_session_code(root: &Path) {
    let spec = root.join("specs/auth/auth.spec.md");
    let content = fs::read_to_string(&spec)
        .unwrap()
        .replacen("  - src/auth.rs\n", "  - src/auth.rs\n  - src/session.rs\n", 1);
    fs::write(spec, content).unwrap();
    fs::write(root.join("src/session.rs"), "// Sessions.\n").unwrap();
}

/// A change at `verifying`: approved, materialized and verified against its first scope.
fn verifying_change(root: &Path) -> ChangeRecord {
    let record = completed_section_only_current_record(root, SEND_BACK_DELTA);
    auth_spec_owning_session_code(root);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let record = load_change(root, &record.id).unwrap();
    assert_eq!(record.state, ChangeState::Verifying, "fixture must reach verifying");
    record
}

fn approvals_bytes(root: &Path, record: &ChangeRecord) -> serde_json::Value {
    serde_json::from_slice(&fs::read(change_dir(root, &record.id).join("approvals.json")).unwrap())
        .unwrap()
}

/// DISCRIMINATOR for CHANGE-12: a verifying change is sent back, its scope is answered again,
/// and it goes on to a fresh approval, check, verification and acceptance under its own name.
#[test]
fn a_verifying_change_sent_back_can_be_re_scoped_and_delivered() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);

    let sent = send_back_change(
        root,
        &record.id,
        "Scope owner".into(),
        SEND_BACK_REASON.into(),
    )
    .unwrap();
    assert_eq!(sent.change.state, ChangeState::Draft);

    let rescoped = answer_question(
        root,
        &record.id,
        "affected_paths",
        "src/auth.rs,src/session.rs",
    )
    .unwrap();
    assert_eq!(
        rescoped.affected_paths,
        vec!["src/auth.rs".to_string(), "src/session.rs".to_string()]
    );

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let accepted = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
    assert_eq!(accepted.state, ChangeState::Accepted);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert_eq!(
        spec.matches("Auth tracks credentials and sessions.").count(),
        1,
        "the already-applied section must not be applied twice: {spec}"
    );
}

/// DISCRIMINATOR for CHANGE-12.a: no named person or no reason, no send-back, and nothing moves.
#[test]
fn sending_back_needs_a_named_person_and_a_reason() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);
    let before = approvals_bytes(root, &record);

    let no_actor =
        send_back_change(root, &record.id, "  ".into(), SEND_BACK_REASON.into()).unwrap_err();
    assert!(no_actor.contains("--actor"), "{no_actor}");
    let no_reason =
        send_back_change(root, &record.id, "Scope owner".into(), " ".into()).unwrap_err();
    assert!(no_reason.contains("--reason"), "{no_reason}");

    assert_eq!(load_change(root, &record.id).unwrap().state, ChangeState::Verifying);
    assert_eq!(approvals_bytes(root, &record), before);
}

/// DISCRIMINATOR for CHANGE-12.a and 12.c: the person and the reason stay in the change's
/// history, and every approval and the verification evidence already recorded is kept.
#[test]
fn a_send_back_is_kept_in_the_change_history_and_loses_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);
    let before = approvals_bytes(root, &record);
    let verification = fs::read(change_dir(root, &record.id).join("verification.json")).unwrap();

    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();

    let after = approvals_bytes(root, &record);
    assert_eq!(
        after["approvals"], before["approvals"],
        "every approval already recorded must survive the send-back unchanged"
    );
    let sent = &after["send_backs"][0];
    assert_eq!(sent["actor"], "Scope owner");
    assert_eq!(sent["reason"], SEND_BACK_REASON);
    assert_eq!(sent["from_state"], "verifying");
    assert_eq!(
        fs::read(change_dir(root, &record.id).join("verification.json")).unwrap(),
        verification,
        "the earlier verification evidence must be kept"
    );

    // A second send-back after re-approval appends; it never rewrites the first.
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    send_back_change(root, &record.id, "Second owner".into(), "still wrong".into()).unwrap();
    let twice = approvals_bytes(root, &record);
    assert_eq!(twice["send_backs"][0], *sent);
    assert_eq!(twice["send_backs"][1]["actor"], "Second owner");
    assert_eq!(twice["send_backs"][1]["from_state"], "approved");
}

/// DISCRIMINATOR for CHANGE-12.b: the approval granted before the send-back no longer counts,
/// even when the scope is not changed at all, until someone approves again.
#[test]
fn sending_back_cancels_the_approval_until_a_fresh_one() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);

    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();

    let error = ensure_definition_approval_valid(root, &load_change(root, &record.id).unwrap())
        .unwrap_err();
    assert!(error.contains("sent back"), "{error}");
    assert!(
        error.contains(&format!("specsync change approve {}", record.id)),
        "the refusal must name the fresh approval: {error}"
    );
    start_implementation(root, &record.id).unwrap_err();

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    ensure_definition_approval_valid(root, &load_change(root, &record.id).unwrap()).unwrap();
}

/// CONTROL: only approved, implementing and verifying changes can be sent back. A draft has
/// nothing to cancel, and an accepted or archived change already has `change reopen`.
#[test]
fn only_an_approved_unfinished_change_can_be_sent_back() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let draft = completed_section_only_current_record(root, SEND_BACK_DELTA);
    let error =
        send_back_change(root, &draft.id, "Scope owner".into(), SEND_BACK_REASON.into())
            .unwrap_err();
    assert!(error.contains("expected approved or implementing or verifying"), "{error}");
    assert_eq!(load_change(root, &draft.id).unwrap().state, ChangeState::Draft);
}

fn git_in(root: &Path) -> impl Fn(&[&str]) + '_ {
    move |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git command failed: {args:?}"
        );
    }
}

/// A git repository whose first commit holds the auth spec and a draft section-only change.
fn committed_spec_change(root: &Path) -> ChangeRecord {
    committed_change_with_delta(root, SEND_BACK_DELTA)
}

fn committed_change_with_delta(root: &Path, delta: &str) -> ChangeRecord {
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, delta);
    auth_spec_owning_session_code(root);
    git(&["add", "--all"]);
    git(&["commit", "-m", "base"]);
    record
}

/// CHANGE-12 end to end on a real branch, with a change that owns a spec: it is checked, which
/// applies its delta, sent back, approved again, checked, reviewed and finalized. The archived
/// package, which still carries the send-back and the cancelled approval, passes the audit.
#[test]
fn a_sent_back_change_finalizes_and_its_archive_passes_audit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    let record = committed_spec_change(root);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    git(&["add", "--all"]);
    git(&["commit", "-m", "first check"]);

    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    git(&["add", "--all"]);
    git(&["commit", "-m", "check the re-approved scope"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let destination = finalize_change(root, &record.id).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "finalize"]);

    let ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(destination.join("approvals.json")).unwrap()).unwrap();
    assert_eq!(ledger["send_backs"][0]["actor"], "Scope owner");
    assert_eq!(ledger["send_backs"][0]["applied_items"][0]["key"], "Purpose");
    let report = audit_project(root);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert_eq!(spec.matches("Auth tracks credentials and sessions.").count(), 1, "{spec}");
}

/// DISCRIMINATOR for CHANGE-12.c (#846 review): a re-scope that drops an item the change had
/// already applied would leave that text in the spec with nothing owning it. `change check`
/// refuses, naming the spec file and the section, until the text is taken out of the spec or the
/// item is kept in the delta.
#[test]
fn a_re_scope_that_drops_applied_text_is_refused_until_the_text_is_removed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    fs::write(
        change_dir(root, &record.id).join("deltas/auth.md"),
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nSessions expire.\n",
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("specs/auth/auth.spec.md"), "{error}");
    assert!(error.contains("Purpose"), "{error}");

    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path)
        .unwrap()
        .replace("Auth tracks credentials and sessions.", "Auth.");
    fs::write(&spec_path, spec).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
}

/// DISCRIMINATOR for CHANGE-12.b (#846 review): every reader of the definition approval honours
/// the send-back, including the event resolver historical and archived evidence go through.
#[test]
fn the_definition_approval_event_resolver_honours_a_send_back() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = verifying_change(root);
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    let record = load_change(root, &record.id).unwrap();
    let ledger = load_approvals(root, &record).unwrap();
    let digest = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .unwrap()
        .digest
        .clone();
    let error = resolve_definition_approval_event(&record, &ledger, &digest, None, "")
        .map(|_| ())
        .unwrap_err();
    assert!(error.contains("sent back"), "{error}");
}

/// DISCRIMINATOR for CHANGE-12.b (#846 review): deleting a committed send-back from
/// `approvals.json` does not bring the cancelled approval back.
#[test]
fn deleting_a_committed_send_back_does_not_revive_the_cancelled_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    let record = committed_spec_change(root);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "send back"]);

    let path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    ledger.as_object_mut().unwrap().remove("send_backs");
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(change_dir(root, &record.id).join("state.json")).unwrap())
            .unwrap();
    state["state"] = "implementing".into();
    fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
    fs::write(
        change_dir(root, &record.id).join("state.json"),
        serde_json::to_vec_pretty(&state).unwrap(),
    )
    .unwrap();

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("send-back"), "{error}");
}

// Review round two on 6e9abc9 (0xLeif, CHANGES_REQUESTED).

const REMOVED_DELTA: &str = "## REMOVED\n\n### SPEC SECTION Legacy Notes\n";

fn check_passes(root: &Path, id: &str) {
    let report = check_change(root, Some(id)).unwrap().unwrap();
    assert!(report.passed, "{:?}", report.commands);
}

fn approve_check_and_commit(root: &Path, id: &str, message: &str) {
    approve_definition(root, id, Some("Scope owner".into()), None).unwrap();
    check_passes(root, id);
    let git = git_in(root);
    git(&["add", "--all"]);
    git(&["commit", "-m", message]);
}

/// DISCRIMINATOR (High): a REMOVED item the change already applied stays converged after a
/// send-back, so the re-approved change can be materialized and checked again.
#[test]
fn a_sent_back_change_with_a_removed_item_can_be_checked_again() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // A REMOVED-only delta has no semantic acceptance item, so it could never verify; the
    // MODIFIED item lets the last step run `change check` as well as materialize.
    let record =
        completed_section_only_current_record(root, &format!("{SEND_BACK_DELTA}\n{REMOVED_DELTA}"));
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    answer_question(root, &record.id, "affected_paths", "src/auth.rs").unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    check_passes(root, &record.id);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(!spec.contains("## Legacy Notes"), "{spec}");
}

/// DISCRIMINATOR (High): an ADDED item kept under the same key with a new body replaces the
/// change's own earlier body after a send-back instead of failing as an existing block.
#[test]
fn a_sent_back_change_can_change_the_body_of_an_item_it_added() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(
        root,
        "## ADDED\n\n### SPEC SECTION Sessions\n\nSessions last a day.\n",
    );
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    fs::write(
        change_dir(root, &record.id).join("deltas/auth.md"),
        "## ADDED\n\n### SPEC SECTION Sessions\n\nSessions last an hour.\n",
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_passes(root, &record.id);
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(spec.contains("Sessions last an hour."), "{spec}");
    assert!(!spec.contains("Sessions last a day."), "{spec}");
    assert_eq!(spec.matches("## Sessions").count(), 1, "{spec}");
}

const OWN_ADDED_REQUIREMENT: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL keep a session alive for a day.\n\nAcceptance Criteria\n- A session survives a day.\n";

const REVISED_OWN_ADDED_REQUIREMENT: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL keep a session alive for an hour.\n\nAcceptance Criteria\n- A session survives an hour.\n";

/// A change that added REQ-auth-002, materialized it and was sent back.
fn sent_back_after_adding_a_requirement(root: &Path) -> ChangeRecord {
    let record = completed_section_only_current_record(root, OWN_ADDED_REQUIREMENT);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    fs::write(
        change_dir(root, &record.id).join("deltas/auth.md"),
        REVISED_OWN_ADDED_REQUIREMENT,
    )
    .unwrap();
    record
}

/// #1001: approve accepts a sent-back change's revised body for a requirement it added itself,
/// as materialization already does, and materialization then replaces the change's own block.
#[test]
fn a_sent_back_change_can_be_re_approved_with_a_revised_requirement_it_added() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = sent_back_after_adding_a_requirement(root);
    approve_definition(root, &record.id, Some("Scope owner".into()), None)
        .expect("the canonical block is this change's own earlier text, so approve accepts the revision");
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    let requirements = fs::read_to_string(root.join("specs/auth/requirements.md")).unwrap();
    assert!(requirements.contains("alive for an hour."), "{requirements}");
    assert!(!requirements.contains("alive for a day."), "{requirements}");
    assert_eq!(requirements.matches("### REQ-auth-002").count(), 1, "{requirements}");
}

/// #1001: the exception covers only the change's own text. A block whose canonical text
/// someone else changed after the send-back is still refused as an existing block.
#[test]
fn approve_still_refuses_an_added_requirement_whose_canonical_text_someone_else_changed() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = sent_back_after_adding_a_requirement(root);
    let path = root.join("specs/auth/requirements.md");
    let foreign = fs::read_to_string(&path)
        .unwrap()
        .replace("alive for a day.", "alive for a week.")
        .replace("survives a day.", "survives a week.");
    fs::write(&path, foreign).unwrap();
    let error = approve_definition(root, &record.id, Some("Scope owner".into()), None)
        .expect_err("text the change did not write is not its own");
    assert!(
        error.contains("cannot add existing block `REQ-auth-002`"),
        "{error}"
    );
}

/// DISCRIMINATOR (Medium): a dropped MODIFIED item replaced text that existed before the change,
/// so the refusal says to restore that text, not to take the section out.
#[test]
fn a_dropped_modified_item_is_refused_with_advice_to_restore_its_earlier_text() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = committed_spec_change(root);
    approve_check_and_commit(root, &record.id, "first check");
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    fs::write(
        change_dir(root, &record.id).join("deltas/auth.md"),
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nSessions expire.\n",
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("specs/auth/auth.spec.md` section `Purpose`"), "{error}");
    assert!(error.contains("restore"), "{error}");
    assert!(error.contains("Auth."), "the refusal must quote the earlier text: {error}");
    assert!(!error.contains("Take that text out"), "{error}");
}

/// DISCRIMINATOR (Medium): a REMOVED item the re-scope drops would leave the section deleted with
/// no change owning the deletion. Check refuses, naming it, until the section is restored.
#[test]
fn a_dropped_removed_item_is_refused_until_the_section_is_restored() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = committed_change_with_delta(root, &format!("{SEND_BACK_DELTA}\n{REMOVED_DELTA}"));
    approve_check_and_commit(root, &record.id, "first check");
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    fs::write(change_dir(root, &record.id).join("deltas/auth.md"), SEND_BACK_DELTA).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("specs/auth/auth.spec.md` section `Legacy Notes`"), "{error}");
    assert!(error.contains("Retained for compatibility."), "{error}");

    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap().replace(
        "## Change Log",
        "## Legacy Notes\n\nRetained for compatibility.\n\n## Change Log",
    );
    fs::write(&spec_path, spec).unwrap();
    check_passes(root, &record.id);
}

/// DISCRIMINATOR (Medium): a module dropped from `affected_specs` keeps the change's version bump
/// and Change Log row, so check refuses and names that row until it is reverted.
#[test]
fn a_module_dropped_from_the_scope_is_refused_while_it_keeps_the_change_log_row() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, SEND_BACK_DELTA);
    let auth = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    fs::create_dir_all(root.join("specs/session")).unwrap();
    fs::write(
        root.join("specs/session/session.spec.md"),
        auth.replace("module: auth", "module: session")
            .replace("src/auth.rs", "src/session.rs")
            .replace("# Auth", "# Session"),
    )
    .unwrap();
    fs::write(root.join("src/session.rs"), "// Sessions.\n").unwrap();
    answer_question(root, &record.id, "affected_specs", "auth,session").unwrap();
    answer_question(root, &record.id, "affected_paths", "src/auth.rs,src/session.rs").unwrap();
    fs::write(
        change_dir(root, &record.id).join("deltas/session.md"),
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nSessions expire.\n",
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_passes(root, &record.id);

    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    answer_question(root, &record.id, "affected_specs", "auth").unwrap();
    answer_question(root, &record.id, "affected_paths", "src/auth.rs").unwrap();
    fs::remove_file(change_dir(root, &record.id).join("deltas/session.md")).unwrap();
    let session = root.join("specs/session/session.spec.md");
    let restored = fs::read_to_string(&session)
        .unwrap()
        .replace("Sessions expire.", "Stable.");
    fs::write(&session, restored).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("specs/session/session.spec.md"), "{error}");
    assert!(error.contains("Change Log"), "{error}");
}

/// DISCRIMINATOR (Low): deleting a send-back and committing the deletion does not bring the
/// cancelled approval back, and the project audit reports it.
#[test]
fn a_send_back_deleted_in_a_later_commit_is_refused_and_audited() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = git_in(root);
    let record = committed_spec_change(root);
    approve_check_and_commit(root, &record.id, "first check");
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "send back"]);

    let path = change_dir(root, &record.id).join("approvals.json");
    let mut ledger: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    ledger.as_object_mut().unwrap().remove("send_backs");
    fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
    let state_path = change_dir(root, &record.id).join("state.json");
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    state["state"] = "verifying".into();
    state["canonical_applied"] = true.into();
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "drop the send-back"]);

    let error = check_change(root, Some(&record.id)).map(|_| ()).unwrap_err();
    assert!(error.contains("send-back"), "{error}");
    let report = audit_project(root);
    assert!(
        report.errors.iter().any(|error| error.contains("send-back")),
        "{:?}",
        report.errors
    );
}

/// DISCRIMINATOR (High), 0xLeif's exact repro: a REMOVED-only delta through approve, start,
/// materialize, send-back, answer, approve, start and materialize again.
#[test]
fn a_sent_back_removed_only_change_materializes_again() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, REMOVED_DELTA);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    let version = |root: &Path| {
        fs::read_to_string(root.join("specs/auth/auth.spec.md"))
            .unwrap()
            .lines()
            .find(|line| line.starts_with("version:"))
            .unwrap()
            .to_string()
    };
    let first_version = version(root);
    send_back_change(root, &record.id, "Scope owner".into(), SEND_BACK_REASON.into()).unwrap();
    answer_question(root, &record.id, "affected_paths", "src/auth.rs").unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(!spec.contains("## Legacy Notes"), "{spec}");
    assert_eq!(spec.matches("add-passkeys:").count(), 1, "{spec}");
    assert_eq!(version(root), first_version, "the version is bumped once: {spec}");
}

/// #914: a review recorded before a send-back stays readable history, bound to the approval the
/// send-back cancelled, and does not stop the fresh review recorded after re-approval. The stale
/// review is committed with the re-approval, as `change check --commit` would, so the
/// committed-history walk sees it too. The criteria change, so the earlier review cannot count.
#[test]
fn a_review_recorded_before_a_send_back_is_history_and_a_fresh_one_counts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    }
    let check_and_commit = |message: &str| {
        assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", message]);
        assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    };
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_and_commit("implement approved change");
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let reviewed = load_change(root, &record.id).unwrap();
    let earlier = load_scoped_review(root, &reviewed).unwrap();
    assert!(scoped_review_is_current(root, &reviewed, &earlier));

    send_back_change(
        root,
        &record.id,
        "Scope owner".into(),
        "the review found a scope gap".into(),
    )
    .unwrap();
    answer_question(
        root,
        &record.id,
        "acceptance_criteria",
        "The verification records the reviewed scope",
    )
    .unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    check_and_commit("re-scope and re-approve after the send-back");
    let verifying = load_change(root, &record.id).unwrap();
    assert_eq!(verifying.state, ChangeState::Verifying);

    // The split itself, on the real ledger: the earlier review binds the approval the send-back
    // cancelled, which is valid history and binds no current review.
    let approvals = load_approvals(root, &verifying).unwrap();
    assert!(historical_definition_approver_for_review(&approvals, &earlier).is_ok());
    assert!(definition_approver_for_review_contract(&approvals, &earlier).is_err());

    let kept = load_scoped_review(root, &verifying)
        .expect("the review history stays readable after a send-back");
    assert_eq!(kept, earlier, "the earlier review is kept unchanged");
    assert!(
        matches!(
            scoped_review_currency(root, &verifying, &kept),
            ScopedReviewCurrency::Stale(reason) if reason.contains("send-back cancelled")
        ),
        "a review bound only to the cancelled approval never counts"
    );

    record_scoped_review(root, &record.id, "Independent reviewer".into())
        .expect("a fresh review records after re-approval");
    let fresh = load_scoped_review(root, &verifying).unwrap();
    assert_ne!(fresh, earlier);
    assert!(scoped_review_is_current(root, &verifying, &fresh));
    assert!(
        definition_approver_for_review_contract(&approvals, &fresh).is_ok(),
        "the fresh review binds the approval recorded after the send-back"
    );
}

/// #960 (CHANGE-12.e): a send-back cancels the review recorded before it even when the scope comes
/// back unchanged. The re-approval then has the same digest the earlier review followed, and a
/// `change check` without `--commit` leaves HEAD and every digest the review binds where they were,
/// so only the send-back's record of how many attempts preceded it tells the two apart.
#[test]
fn a_review_recorded_before_a_send_back_never_counts_after_an_identical_reapproval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // Every record this test writes lands in the same second, as it can on a fast machine, so the
    // assertions cannot depend on two records' timestamps differing (CI-7, #977).
    let _clock = PinnedTestClock::at(1_800_000_000);
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    if quiet_git(root, &["ls-files", WORKFLOW_V2_BASELINE_PATH]).is_empty() {
        quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
        quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    }
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    let reviewed = load_change(root, &record.id).unwrap();
    let earlier = load_scoped_review(root, &reviewed).unwrap();
    assert!(scoped_review_is_current(root, &reviewed, &earlier));

    let sent = send_back_change(
        root,
        &record.id,
        "Scope owner".into(),
        "the scope needs another look".into(),
    )
    .unwrap();
    assert_eq!(
        sent.audit.cancelled_review_attempts,
        Some(1),
        "the send-back records the review attempt it cancels"
    );
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    let reapproved = load_change(root, &record.id).unwrap();
    assert_eq!(
        definition_digest(root, &reapproved).unwrap(),
        earlier.contract_digest,
        "the scope came back unchanged"
    );

    let kept = load_scoped_review(root, &reapproved).expect("the review history stays readable");
    assert_eq!(kept, earlier);
    assert!(
        matches!(
            scoped_review_currency(root, &reapproved, &kept),
            ScopedReviewCurrency::Stale(reason) if reason.contains("before the change was last sent back")
        ),
        "a review recorded before the send-back never counts, even after an identical re-approval"
    );

    // In the same second the fresh review can equal the earlier one field for field, so the
    // proof that a second review was recorded is the ledger, not the records' inequality.
    let before = recorded_scoped_review_attempts(root, &reapproved).unwrap();
    record_scoped_review(root, &record.id, "Independent reviewer".into())
        .expect("a fresh review records after the re-approval");
    let fresh = load_scoped_review(root, &reapproved).unwrap();
    let attempts = load_scoped_review_attempt_ledger(root, &reapproved)
        .unwrap()
        .expect("the review history exists");
    assert_eq!(
        attempts.reviews.len(),
        before + 1,
        "the fresh review is appended"
    );
    assert_eq!(attempts.reviews.last(), Some(&fresh));
    assert!(scoped_review_is_current(root, &reapproved, &fresh));
}

/// The rule itself: a send-back that recorded its attempt count answers by position; one recorded
/// before that count existed answers by time, strictly, so the archives such send-backs already
/// carry, whose reviews all came later, keep their verdict.
#[test]
fn a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time() {
    let review = |timestamp: u64| ScopedReviewRecord {
        schema_version: 2,
        change_id: "add-passkeys".into(),
        reviewer: "Independent reviewer".into(),
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
        timestamp,
        relayed_by: None,
    };
    let send_back = |cancelled_review_attempts: Option<usize>| SendBackRecord {
        schema_version: 1,
        change_id: "add-passkeys".into(),
        actor: "Scope owner".into(),
        reason: SEND_BACK_REASON.into(),
        timestamp: 100,
        from_state: ChangeState::Verifying,
        to_state: ChangeState::Draft,
        cancelled_approvals: 1,
        cancelled_review_attempts,
        canonical_applied: false,
        applied_items: Vec::new(),
        applied_modules: Vec::new(),
    };

    let counted = send_back(Some(1));
    assert!(!review_recorded_after_send_back(&counted, Some(0), &review(500)));
    assert!(review_recorded_after_send_back(&counted, Some(1), &review(100)));
    assert!(!review_recorded_after_send_back(&counted, None, &review(500)));

    let legacy = send_back(None);
    assert!(!review_recorded_after_send_back(&legacy, Some(0), &review(99)));
    assert!(!review_recorded_after_send_back(&legacy, Some(0), &review(100)));
    assert!(review_recorded_after_send_back(&legacy, Some(0), &review(101)));

    // The count is written only when present, so a send-back recorded before it keeps its bytes.
    let bytes = serde_json::to_string(&legacy).unwrap();
    assert!(!bytes.contains("cancelled_review_attempts"), "{bytes}");
    let parsed: SendBackRecord = serde_json::from_str(&bytes).unwrap();
    assert_eq!(parsed, legacy);
}

// #844 item 4, #1070 (REQ-change-161): a spec split moved a requirement into `sessions` while
// `auth` still owns the code. A change declares both modules with a delta only for `sessions`;
// `auth` is a code-only module, allowed because it owns one of the change's files.

const SESSIONS_DELTA: &str = "## MODIFIED\n\n### SPEC SECTION Purpose\n\nSessions track logins.\n";

/// A second module, `sessions`, owning `src/session.rs`.
fn sessions_spec(root: &Path) {
    fs::create_dir_all(root.join("specs/sessions")).unwrap();
    fs::write(root.join("src/session.rs"), "// Sessions.\n").unwrap();
    let auth = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    fs::write(
        root.join("specs/sessions/sessions.spec.md"),
        auth.replace("module: auth", "module: sessions")
            .replace("src/auth.rs", "src/session.rs")
            .replace("# Auth\n\n## Purpose\n\nAuth.", "# Sessions\n\n## Purpose\n\nSessions."),
    )
    .unwrap();
}

/// A draft declaring `auth` and `sessions` over `paths`, with a delta for `sessions` only.
fn split_scope_draft(root: &Path, paths: &str) -> ChangeRecord {
    let record = completed_section_only_current_record(root, SEND_BACK_DELTA);
    sessions_spec(root);
    answer_question(root, &record.id, "affected_specs", "auth,sessions").unwrap();
    answer_question(root, &record.id, "affected_paths", paths).unwrap();
    answer_question(root, &record.id, "constraints", "none").unwrap();
    fs::remove_file(delta_path(root, &record, "auth")).unwrap();
    fs::write(delta_path(root, &record, "sessions"), SESSIONS_DELTA).unwrap();
    load_change(root, &record.id).unwrap()
}

#[test]
fn a_module_that_owns_a_changed_file_may_be_declared_without_a_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = split_scope_draft(root, "src/auth.rs");
    let auth_before = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();

    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();
    materialize_change_deltas(root, &record.id).unwrap();
    verify_change(root, &record.id).unwrap();
    let accepted = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
    assert_eq!(accepted.state, ChangeState::Accepted);

    let sessions = fs::read_to_string(root.join("specs/sessions/sessions.spec.md")).unwrap();
    assert!(sessions.contains("Sessions track logins."), "{sessions}");
    assert_eq!(
        fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap(),
        auth_before,
        "a code-only module's spec is not edited"
    );
}

#[test]
fn a_module_with_no_delta_that_owns_none_of_the_files_is_refused() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = split_scope_draft(root, "src/session.rs");
    let refused = approve_definition(root, &record.id, Some("Scope owner".into()), None)
        .unwrap_err();
    assert!(
        refused.contains("auth") && refused.contains("delta"),
        "a module with no delta must own one of the change's files: {refused}"
    );
}

// REQ-change-161: dropping an approved delta after approval does not turn its module into a
// code-only one; the approval covered that wording, so the change is refused until approved again.
#[test]
fn an_approved_delta_deleted_later_is_still_a_changed_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = completed_section_only_current_record(root, SEND_BACK_DELTA);
    sessions_spec(root);
    answer_question(root, &record.id, "affected_specs", "auth,sessions").unwrap();
    answer_question(root, &record.id, "affected_paths", "src/auth.rs").unwrap();
    answer_question(root, &record.id, "constraints", "none").unwrap();
    fs::write(delta_path(root, &record, "sessions"), SESSIONS_DELTA).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    start_implementation(root, &record.id).unwrap();

    fs::remove_file(delta_path(root, &record, "auth")).unwrap();
    let refused = materialize_change_deltas(root, &record.id).unwrap_err();
    assert!(
        refused.contains("auth") && refused.contains("changed after approval"),
        "{refused}"
    );
}
