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
    let git = git_in(root);
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    let record = completed_section_only_current_record(root, SEND_BACK_DELTA);
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
