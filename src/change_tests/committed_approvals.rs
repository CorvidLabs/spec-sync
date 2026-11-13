// #428: finalize, and ship through it, archives only approvals that history holds. It refuses an
// `approvals.json` that differs from its bytes at `HEAD`, and one whose committed versions do not
// each extend the version committed before them. `relayed_by` is not compared, because
// `change repair-relays` may put a relay back or remove it (REQ-change-relay-repair-001).

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
    record_scoped_review(root, &record.id, "Independent reviewer".into()).unwrap();
    (temp, record)
}

fn approvals_path(root: &Path, record: &ChangeRecord) -> PathBuf {
    change_dir(root, &record.id).join("approvals.json")
}

/// Rewrites one member of the first approval in the working tree's `approvals.json`.
fn edit_first_approval(root: &Path, record: &ChangeRecord, member: &str, value: serde_json::Value) {
    let path = approvals_path(root, record);
    let mut ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let first = &mut ledger["approvals"][0];
    match value {
        serde_json::Value::Null => {
            first.as_object_mut().unwrap().remove(member);
        }
        value => first[member] = value,
    }
    fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
}

#[test]
fn finalize_refuses_approvals_edited_since_head() {
    let (temp, record) = committed_approvals_project();
    let root = temp.path();
    edit_first_approval(root, &record, "actor", serde_json::json!("mallory"));
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains("approvals.json") && refused.contains("HEAD"),
        "{refused}"
    );
    assert!(
        change_dir(root, &record.id).is_dir(),
        "a refused finalize must archive nothing"
    );
}

#[test]
fn finalize_refuses_a_committed_rewrite_of_an_earlier_approval() {
    let mut rewrite = String::new();
    let (temp, record) = reviewed_after(|root, record| {
        edit_first_approval(root, record, "actor", serde_json::json!("mallory"));
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "rewrite the approver"]);
        rewrite = quiet_git(root, &["rev-parse", "--short", "HEAD"]);
    });
    let root = temp.path();
    let refused = finalize_change(root, &record.id).unwrap_err();
    assert!(
        refused.contains("approvals.json") && refused.contains(&rewrite),
        "the refusal names the commit that rewrote the ledger: {refused}"
    );
    assert!(change_dir(root, &record.id).is_dir());
}

#[test]
fn finalize_accepts_a_committed_relay_removal() {
    let (temp, record) = reviewed_after(|root, record| {
        let actor = {
            let ledger: serde_json::Value =
                serde_json::from_slice(&fs::read(approvals_path(root, record)).unwrap()).unwrap();
            ledger["approvals"][0]["actor"].as_str().unwrap().to_string()
        };
        edit_first_approval(
            root,
            record,
            "relayed_by",
            serde_json::json!({
                "decided_by": actor,
                "words": "Approve it.",
                "where": "a meeting",
                "recorded_by": "an agent"
            }),
        );
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "record the relay"]);
        // What `change repair-relays` does to a relay with no clean copy: it removes it.
        edit_first_approval(root, record, "relayed_by", serde_json::Value::Null);
        quiet_git(root, &["add", "--all"]);
        quiet_git(root, &["commit", "-m", "remove the relay"]);
    });
    finalize_change(temp.path(), &record.id)
        .expect("a relay removal is not a rewrite of the ledger");
}
