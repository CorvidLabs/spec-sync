// #428, REQ-change-166: `change finalize`, `change ship` and `change archive` archive only an
// approvals ledger that Git history holds. The working copy must equal the version committed at
// `HEAD` (an accepted change may add the closing records its accepting step wrote), and every
// version committed along `HEAD`'s first-parent history must extend the one before it. The
// `relayed_by` of an approval is not compared, because `change repair-relays` may put a relay back
// or remove it (REQ-change-relay-repair-001).

/// A change reviewed and ready to finalize, with a temporary directory that owns its project.
fn committed_approvals_project() -> (TempDir, ChangeRecord) {
    let temp = TempDir::new().unwrap();
    fast_read_repository(temp.path());
    let record = reviewed_change_ready_to_finalize(temp.path());
    (temp, record)
}

/// A change approved, implemented and committed, not yet reviewed. `edit` commits its own
/// changes to `approvals.json`; the change is then checked and reviewed, ready to finalize.
fn reviewed_after(edit: impl FnOnce(&Path, &ChangeRecord)) -> (TempDir, ChangeRecord) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "implement approved change"]);
    edit(root, &record);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "verify"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    (temp, record)
}

fn approvals_path(root: &Path, record: &ChangeRecord) -> PathBuf {
    change_dir(root, &record.id).join("approvals.json")
}

fn approvals_display(record: &ChangeRecord) -> String {
    format!(".specsync/changes/{}/approvals.json", record.id)
}

fn read_ledger(root: &Path, record: &ChangeRecord) -> serde_json::Value {
    serde_json::from_slice(&fs::read(approvals_path(root, record)).unwrap()).unwrap()
}

fn write_ledger(root: &Path, record: &ChangeRecord, ledger: &serde_json::Value) {
    fs::write(
        approvals_path(root, record),
        serde_json::to_vec_pretty(ledger).unwrap(),
    )
    .unwrap();
}

/// Rewrites one member of the first approval in the working tree's `approvals.json`.
fn edit_first_approval(root: &Path, record: &ChangeRecord, member: &str, value: serde_json::Value) {
    let mut ledger = read_ledger(root, record);
    let first = &mut ledger["approvals"][0];
    match value {
        serde_json::Value::Null => {
            first.as_object_mut().unwrap().remove(member);
        }
        value => first[member] = value,
    }
    write_ledger(root, record, &ledger);
}

/// Appends one more approval, a copy of the first with a later timestamp and `gate`.
fn append_hand_made_approval(root: &Path, record: &ChangeRecord, gate: &str) {
    let mut ledger = read_ledger(root, record);
    let mut extra = ledger["approvals"][0].clone();
    extra["gate"] = serde_json::json!(gate);
    extra["timestamp"] = serde_json::json!(extra["timestamp"].as_u64().unwrap() + 1);
    extra["note"] = serde_json::json!("added by hand");
    ledger["approvals"].as_array_mut().unwrap().push(extra);
    write_ledger(root, record, &ledger);
}

fn short_head(root: &Path) -> String {
    quiet_git(root, &["rev-parse", "--short=12", "HEAD"])
        .trim()
        .to_string()
}

#[test]
fn finalize_refuses_approvals_edited_since_head_and_archives_once_they_are_restored() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    edit_first_approval(root, &record, "actor", serde_json::json!("mallory"));
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains(&approvals_display(&record))
            && refused.contains("HEAD")
            && refused.contains(&format!("git checkout HEAD -- {}", approvals_display(&record))),
        "the refusal names the file, HEAD and the restore: {refused}"
    );
    assert!(
        change_dir(root, &record.id).is_dir(),
        "a refused finalize must archive nothing"
    );
    quiet_git(root, &["checkout", "HEAD", "--", &approvals_display(&record)]);
    finalize_change(root, &record.id).expect("the restored ledger archives");
}

#[test]
fn a_working_copy_that_only_adds_records_is_told_to_commit_it_not_to_restore_it() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    append_hand_made_approval(root, &record, "definition");
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains(&format!("specsync change check {} --commit", record.id)),
        "the way out is the commit: {refused}"
    );
    assert!(
        !refused.contains("git checkout"),
        "a restore would drop the records the working copy adds: {refused}"
    );
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn an_ignored_ledger_is_told_to_stop_ignoring_it_or_force_add_it() {
    let (temp, record) = reviewed_after(|root, record| {
        quiet_git(root, &["rm", "--cached", "-q", &approvals_display(record)]);
        // Ignored, so the fixture's later `git add --all` leaves it out, as `change check
        // --commit` would too: its way out must not be that command (tester2 on #428).
        let exclude = root.join(".git/info/exclude");
        let mut excluded = fs::read_to_string(&exclude).unwrap_or_default();
        excluded.push_str(&format!("/{}\n", approvals_display(record)));
        fs::write(&exclude, excluded).unwrap();
        quiet_git(root, &["commit", "-m", "stop tracking the ledger"]);
    });
    let root = temp.path();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains("is not committed at HEAD")
            && refused.contains("ignored by Git")
            && refused.contains(&format!("git add -f -- {}", approvals_display(&record))),
        "{refused}"
    );
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn a_ledger_not_committed_at_head_is_told_to_commit_it() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    quiet_git(root, &["rm", "--cached", "-q", &approvals_display(&record)]);
    quiet_git(root, &["commit", "-m", "stop tracking the ledger"]);
    let refused =
        approvals_ledger_held_by_history(root, &load_change(root, &record.id).unwrap()).unwrap_err();
    assert!(
        refused.contains("is not committed at HEAD")
            && refused.contains(&format!("specsync change check {} --commit", record.id))
            && !refused.contains("ignored"),
        "{refused}"
    );
}

#[test]
fn finalize_refuses_a_committed_rewrite_of_an_earlier_approval() {
    let mut earlier = String::new();
    let mut rewrite = String::new();
    let (temp, record) = reviewed_after(|root, record| {
        earlier = short_head(root);
        edit_first_approval(root, record, "actor", serde_json::json!("mallory"));
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "rewrite the approver"]);
        rewrite = short_head(root);
    });
    let root = temp.path();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains(&rewrite) && refused.contains(&earlier),
        "the refusal names the commit that rewrote the ledger and the one it rewrote: {refused}"
    );
    assert!(refused.contains("`approvals[0]`"), "and the record: {refused}");
    assert!(
        refused.contains("history") && refused.contains("new id"),
        "and the two ways out: {refused}"
    );
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn a_relay_committed_then_removed_and_restored_still_finalizes() {
    let (temp, record) = reviewed_after(|root, record| {
        let actor = read_ledger(root, record)["approvals"][0]["actor"]
            .as_str()
            .unwrap()
            .to_string();
        let relay = serde_json::json!({
            "decided_by": actor,
            "words": "Approve it.",
            "where": "a meeting",
            "recorded_by": "an agent"
        });
        edit_first_approval(root, record, "relayed_by", relay.clone());
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "record the relay"]);
        // What `change repair-relays` does to a relay with no clean copy: it removes it.
        edit_first_approval(root, record, "relayed_by", serde_json::Value::Null);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "remove the relay"]);
        // And what it does with a clean copy: it puts it back.
        edit_first_approval(root, record, "relayed_by", relay);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "restore the relay"]);
    });
    finalize_change(temp.path(), &record.id)
        .expect("a relay removed or restored is not a rewrite of the ledger");
}

#[test]
fn a_workspace_deleted_and_created_again_under_the_same_id_is_refused() {
    // The workflow-version history refuses this as the change loads, before finalize can ask
    // about its ledger: deleting the change's `state.json` deletes its creation anchor. Option 3
    // (Leif's "All as recommended"): the refusal keeps its sentence and names the commit that
    // removed the workspace and the two ways out.
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "record the review"]);
    let earlier = short_head(root);
    let workspace = format!(".specsync/changes/{}", record.id);
    quiet_git(root, &["rm", "-r", "-q", &workspace]);
    quiet_git(root, &["commit", "-m", "remove the change"]);
    let removed = short_head(root);
    quiet_git(root, &["checkout", &earlier, "--", &workspace]);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "create the change again"]);
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert_eq!(
        refused,
        format!(
            "workflow-version history deleted its immutable creation anchor: commit {removed} \
             removed `{workspace}/state.json`, and a later commit holds the change again under \
             the same id. A commit already in history cannot be undone in place, so take \
             {removed} out of this branch's history, or make the change again under a new id"
        )
    );
    assert!(change_dir(root, &record.id).is_dir(), "nothing is archived");
}

#[test]
fn a_ledger_deleted_and_created_again_under_the_same_id_is_refused() {
    let mut earlier = String::new();
    let mut removed = String::new();
    let mut created = String::new();
    let (temp, record) = reviewed_after(|root, record| {
        earlier = short_head(root);
        quiet_git(root, &["rm", "-q", &approvals_display(record)]);
        quiet_git(root, &["commit", "-m", "remove the ledger"]);
        removed = short_head(root);
        // Created again from the same file, with the approval recorded anew.
        quiet_git(root, &["checkout", &earlier, "--", &approvals_display(record)]);
        let mut ledger = read_ledger(root, record);
        let timestamp = ledger["approvals"][0]["timestamp"].as_u64().unwrap();
        ledger["approvals"][0]["timestamp"] = serde_json::json!(timestamp + 1);
        write_ledger(root, record, &ledger);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "create the ledger again"]);
        created = short_head(root);
    });
    let root = temp.path();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains(&format!("last held in {earlier}")) && refused.contains("removed in"),
        "the refusal names the commit that last held the earlier ledger: {refused}"
    );
    assert!(refused.contains("new id"), "and the two ways out: {refused}");
    assert!(
        refused.contains(&format!("take {removed} and {created} out")),
        "taking out only the re-creation would leave the ledger deleted: {refused}"
    );
    assert!(change_dir(root, &record.id).is_dir(), "nothing is archived");
}

#[test]
fn a_missing_working_ledger_is_named_with_the_commit_to_restore_it_from() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    fs::remove_file(approvals_path(root, &record)).unwrap();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains(&approvals_display(&record))
            && refused.contains("is missing")
            && refused.contains(&format!("git checkout HEAD -- {}", approvals_display(&record))),
        "finalize names the file and how to get it back: {refused}"
    );
    // Deleted in a commit too: the commit before that one last held it.
    let last_held = short_head(root);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "delete the ledger"]);
    let refused =
        approvals_ledger_held_by_history(root, &load_change(root, &record.id).unwrap()).unwrap_err();
    assert!(
        refused.contains(&format!("git checkout {last_held} -- {}", approvals_display(&record))),
        "{refused}"
    );
}

#[test]
fn a_rewrite_on_a_branch_that_was_merged_in_is_refused_at_the_merge_commit() {
    let mut merge = String::new();
    let (temp, record) = reviewed_after(|root, record| {
        quiet_git(root, &["checkout", "-q", "-b", "side"]);
        edit_first_approval(root, record, "actor", serde_json::json!("mallory"));
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "rewrite the approver on a side branch"]);
        quiet_git(root, &["checkout", "-q", "main"]);
        fs::write(root.join("NOTES.md"), "unrelated\n").unwrap();
        quiet_git(root, &["add", "NOTES.md"]);
        quiet_git(root, &["commit", "-m", "unrelated work on main"]);
        quiet_git(root, &["merge", "--no-ff", "-q", "-m", "merge the side branch", "side"]);
        merge = short_head(root);
    });
    let refused = finalize_change(temp.path(), &record.id).unwrap_err();
    assert!(
        refused.contains(&merge),
        "the merge commit brought the rewrite onto the first-parent history: {refused}"
    );
}

#[test]
fn a_finalization_that_stopped_after_acceptance_resumes() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    accept_change_with_gate(root, &record.id, None, None, "finalization", true, true).unwrap();
    finalize_change(root, &record.id)
        .expect("the uncommitted finalization approval is the accepting step's own");
}

#[test]
fn an_accepted_change_archives_with_the_closing_records_accept_wrote() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let mut record = completed_no_spec_record(root);
    record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
    record = start_implementation(root, &record.id).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve and implement"]);
    verify_change(root, &record.id).unwrap();
    accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
    archive_change(root, &record.id).expect("the closing records accept wrote may be uncommitted");
}

#[test]
fn an_accepted_change_whose_working_copy_adds_another_approval_or_changes_one_is_refused() {
    for tamper in ["adds", "changes"] {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fast_read_repository(root);
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "approve and implement"]);
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        if tamper == "adds" {
            append_hand_made_approval(root, &record, "definition");
        } else {
            edit_first_approval(root, &record, "actor", serde_json::json!("mallory"));
        }
        let refused = archive_change(root, &record.id).unwrap_err();
        assert!(
            refused.contains(&approvals_display(&record)),
            "{tamper}: {refused}"
        );
        assert!(change_dir(root, &record.id).is_dir(), "{tamper}: nothing is archived");
    }
}

#[test]
fn a_change_sent_back_and_approved_again_finalizes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = current_workflow_record(root, completed_no_spec_record(root));
    quiet_git(root, &["add", WORKFLOW_V2_BASELINE_PATH]);
    quiet_git(root, &["commit", "-m", "adopt workflow v2"]);
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve"]);
    send_back_change(root, &record.id, "Scope owner".into(), "re-scope".into()).unwrap();
    approve_definition(root, &record.id, Some("Scope owner".into()), None).unwrap();
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "approve again and implement"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id).expect("a send-back and a fresh approval only append");
}

#[test]
fn a_change_reopened_out_of_its_archive_finalizes_again() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "drift after the archive"]);
    reopen_change(root, &record.id, "Recovery operator".into(), "the input drifted".into())
        .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "reopen"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "verify again"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id)
        .expect("the reopened ledger extends the one committed before the archive move");
}

/// REQ-change-166 and the change's third criterion: a relay committed in the approval a reopening
/// embeds, then removed and restored there, as `change repair-relays` does for an approval, still
/// finalizes, through `finalize_change` and not only at the comparison rule.
#[test]
fn a_relay_removed_and_restored_in_a_reopening_s_superseded_approval_still_finalizes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fast_read_repository(root);
    let record = finalized_archived_change(root);
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { false }\n").unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "drift after the archive"]);
    reopen_change(root, &record.id, "Recovery operator".into(), "the input drifted".into())
        .unwrap();
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "reopen"]);
    let actor = read_ledger(root, &record)["reopenings"][0]["superseded_approval"]["actor"]
        .as_str()
        .expect("the reopening embeds the approval it superseded")
        .to_string();
    let relay = serde_json::json!({
        "decided_by": actor,
        "words": "Approve it.",
        "where": "a meeting",
        "recorded_by": "an agent"
    });
    for (value, message) in [
        (relay.clone(), "record the relay in the superseded approval"),
        (serde_json::Value::Null, "remove the relay, as repair-relays does"),
        (relay, "restore the relay, as repair-relays does"),
    ] {
        let mut ledger = read_ledger(root, &record);
        let superseded = &mut ledger["reopenings"][0]["superseded_approval"];
        match value {
            serde_json::Value::Null => {
                superseded.as_object_mut().unwrap().remove("relayed_by");
            }
            value => superseded["relayed_by"] = value,
        }
        write_ledger(root, &record, &ledger);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", message]);
    }
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    quiet_git(root, &["add", "--all"]);
    quiet_git(root, &["commit", "-m", "verify again"]);
    assert!(check_change(root, Some(&record.id)).unwrap().unwrap().passed);
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    finalize_change(root, &record.id)
        .expect("a relay removed or restored in a superseded approval is not a rewrite");
}

/// A small ledger: one definition approval, one reopening embedding a finalization approval, and
/// one send-back, each with the members the comparison reads.
fn rule_ledger() -> serde_json::Value {
    serde_json::json!({
        "approvals": [
            { "gate": "definition", "actor": "Scope owner", "timestamp": 1, "digest": "a", "note": null }
        ],
        "reopenings": [
            {
                "schema_version": 1,
                "actor": "Recovery operator",
                "reason": "drift",
                "timestamp": 2,
                "superseded_approval": {
                    "gate": "finalization", "actor": "specsync:finalization", "timestamp": 1,
                    "digest": "b", "note": null
                }
            }
        ],
        "send_backs": [
            { "schema_version": 1, "actor": "Scope owner", "reason": "rescope", "timestamp": 3 }
        ]
    })
}

fn rewrite_of(older: &serde_json::Value, newer: &serde_json::Value) -> Option<String> {
    approvals_ledger_rewrite(
        &serde_json::to_vec(older).unwrap(),
        &serde_json::to_vec(newer).unwrap(),
    )
    .expect("both read as ledgers")
}

#[test]
fn the_rule_refuses_a_dropped_or_changed_record_in_each_of_the_three_lists() {
    let older = rule_ledger();
    let mut dropped = older.clone();
    dropped["approvals"] = serde_json::json!([]);
    assert_eq!(rewrite_of(&older, &dropped).as_deref(), Some("drops `approvals[0]`"));
    let mut reopening = older.clone();
    reopening["reopenings"][0]["reason"] = serde_json::json!("another reason");
    assert_eq!(rewrite_of(&older, &reopening).as_deref(), Some("changes `reopenings[0]`"));
    let mut send_back = older.clone();
    send_back["send_backs"] = serde_json::json!([]);
    assert_eq!(rewrite_of(&older, &send_back).as_deref(), Some("drops `send_backs[0]`"));
    let mut embedded = older.clone();
    embedded["reopenings"][0]["superseded_approval"]["actor"] = serde_json::json!("mallory");
    assert_eq!(
        rewrite_of(&older, &embedded).as_deref(),
        Some("changes `reopenings[0]`"),
        "every member of the embedded approval but its relay is compared"
    );
    let mut dropped_member = older.clone();
    dropped_member["approvals"][0]
        .as_object_mut()
        .unwrap()
        .remove("note");
    assert_eq!(
        rewrite_of(&older, &dropped_member).as_deref(),
        Some("changes `approvals[0]`"),
        "a member an older SpecSync dropped is a rewrite"
    );
}

#[test]
fn the_rule_passes_what_the_tool_writes() {
    let older = rule_ledger();
    let relay = serde_json::json!({
        "decided_by": "Scope owner", "words": "Approve", "where": "a meeting", "recorded_by": "an agent"
    });
    let mut relayed = older.clone();
    relayed["approvals"][0]["relayed_by"] = relay.clone();
    relayed["reopenings"][0]["superseded_approval"]["relayed_by"] = relay;
    assert_eq!(rewrite_of(&older, &relayed), None, "a relay restored, in either place");
    assert_eq!(rewrite_of(&relayed, &older), None, "a relay removed, in either place");

    let mut backfilled = older.clone();
    backfilled["reopenings"][0]["stale_acceptance_input_digest"] = serde_json::json!("c");
    backfilled["reopenings"][0]["current_acceptance_input_digest"] = serde_json::json!("d");
    assert_eq!(rewrite_of(&older, &backfilled), None, "migrate 5.0's two digest members");
    let mut changed = backfilled.clone();
    changed["reopenings"][0]["current_acceptance_input_digest"] = serde_json::json!("e");
    assert_eq!(
        rewrite_of(&backfilled, &changed).as_deref(),
        Some("changes `reopenings[0]`"),
        "once there, they may not change"
    );

    let mut appended = older.clone();
    appended["approvals"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({ "gate": "definition", "actor": "Scope owner", "timestamp": 4 }));
    appended["scope_adoptions"] = serde_json::json!([]);
    appended["a_member_from_a_newer_specsync"] = serde_json::json!(true);
    assert_eq!(
        rewrite_of(&older, &appended),
        None,
        "appended records and members outside the three lists"
    );
}

#[test]
fn an_accepted_change_may_append_only_the_closing_records_of_its_accepting_step() {
    let head = serde_json::to_vec(&rule_ledger()).unwrap();
    let with = |tail: Vec<serde_json::Value>| {
        let mut ledger = rule_ledger();
        ledger["approvals"].as_array_mut().unwrap().extend(tail);
        serde_json::to_vec(&ledger).unwrap()
    };
    let approval = |gate: &str, note: Option<&str>| {
        serde_json::json!({ "gate": gate, "actor": "Closer", "timestamp": 9, "note": note })
    };
    assert!(appends_only_closing_records(&head, &with(vec![approval("finalization", None)])));
    assert!(appends_only_closing_records(&head, &with(vec![approval("acceptance", None)])));
    assert!(appends_only_closing_records(
        &head,
        &with(vec![
            approval("definition", Some(ACCEPTANCE_NORMALIZING_DEFINITION_NOTE)),
            approval("acceptance", None),
        ])
    ));
    assert!(!appends_only_closing_records(
        &head,
        &with(vec![approval("definition", Some("by hand")), approval("acceptance", None)])
    ));
    assert!(!appends_only_closing_records(&head, &with(vec![approval("definition", None)])));
    assert!(!appends_only_closing_records(
        &head,
        &with(vec![approval("finalization", None), approval("finalization", None)])
    ));
    let mut changed = rule_ledger();
    changed["approvals"][0]["actor"] = serde_json::json!("mallory");
    changed["approvals"]
        .as_array_mut()
        .unwrap()
        .push(approval("acceptance", None));
    assert!(
        !appends_only_closing_records(&head, &serde_json::to_vec(&changed).unwrap()),
        "a closing record does not excuse a change to a committed one"
    );
}

// Rule 3 for #428: a ledger version Git cannot read, as in a blobless clone of a private
// repository or a damaged repository, is refused with the way out, not only the failure.
#[test]
fn a_ledger_version_git_cannot_read_is_refused_with_the_way_out() {
    let mut earlier_blob = String::new();
    let (temp, record) = reviewed_after(|root, record| {
        // Keep every object loose, so the one this test removes is not packed first.
        quiet_git(root, &["config", "gc.auto", "0"]);
        quiet_git(root, &["config", "maintenance.auto", "false"]);
        earlier_blob = quiet_git(
            root,
            &["rev-parse", &format!("HEAD:{}", approvals_display(record))],
        )
        .trim()
        .to_string();
        let relay = serde_json::json!({
            "decided_by": "Scope owner",
            "words": "Approve it.",
            "where": "a meeting",
            "recorded_by": "an agent"
        });
        edit_first_approval(root, record, "relayed_by", relay);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "record the relay"]);
    });
    let root = temp.path();
    let loose = root
        .join(".git/objects")
        .join(&earlier_blob[..2])
        .join(&earlier_blob[2..]);
    assert!(loose.is_file(), "the earlier ledger is a loose object");
    fs::remove_file(&loose).unwrap();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains("failed to read")
            && refused.contains("repair the repository or fetch the missing objects, then run the command again"),
        "{refused}"
    );
    assert!(change_dir(root, &record.id).is_dir(), "nothing is archived");
}
