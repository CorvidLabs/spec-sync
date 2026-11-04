// Send a change back to draft for re-scoping (#844 option A, CHANGE-12).
//
// A change whose scope turns out wrong after approval used to have one way out: deleting its
// workspace by hand, which lost its audit history, because `change answer` only edits a draft and
// `change reopen` only reopens accepted work. `change send-back` moves an approved, implementing or
// verifying change back to draft so its scope can be answered again. It needs a named person and
// a reason (CHANGE-12.a); it cancels every approval recorded before it, so the new scope needs a
// fresh one (CHANGE-12.b); and it only appends, so nothing already recorded is lost (CHANGE-12.c).
//
// Spec text the change already applied stays where it is. The next `change check` computes what
// is outstanding from the canonical tree itself, so blocks that already carry the approved body
// are converged rather than applied twice.

/// One audited send-back, appended to `approvals.json` under `send_backs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendBackRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
    pub from_state: ChangeState,
    pub to_state: ChangeState,
    /// How many approvals the ledger held when the change was sent back. Every one of them is
    /// cancelled; none is removed.
    pub cancelled_approvals: usize,
    /// Whether the change had already applied its deltas to the canonical specs.
    pub canonical_applied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendBackResult {
    pub change: ChangeRecord,
    pub audit: SendBackRecord,
}

const SEND_BACK_STATES: [ChangeState; 3] = [
    ChangeState::Approved,
    ChangeState::Implementing,
    ChangeState::Verifying,
];

pub fn send_back_change(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
) -> Result<SendBackResult, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(&record, &SEND_BACK_STATES, "send the change back for re-scoping")?;
    let actor = actor.trim();
    if actor.is_empty() {
        return Err("send-back requires a non-empty human actor passed with --actor".into());
    }
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("send-back requires a non-empty reason passed with --reason".into());
    }
    let mut ledger = load_approvals(root, &record)?;
    let audit = SendBackRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: actor.to_string(),
        reason: reason.to_string(),
        timestamp: now(),
        from_state: record.state,
        to_state: ChangeState::Draft,
        cancelled_approvals: ledger.approvals.len(),
        canonical_applied: record.canonical_applied,
    };
    ledger.send_backs.push(audit.clone());
    record.state = ChangeState::Draft;
    // Materialization has to run again for the new scope; the canonical tree, not this flag,
    // decides which blocks are still outstanding.
    record.canonical_applied = false;
    record.updated_at = now();
    write_prepared_files(
        root,
        &[
            (
                change_dir(root, &record.id).join("approvals.json"),
                json_content(&ledger)?,
            ),
            (
                change_dir(root, &record.id).join("state.json"),
                json_content(&record)?,
            ),
            (
                change_dir(root, &record.id).join("change.md"),
                change_markdown_content(&record),
            ),
        ],
    )?;
    Ok(SendBackResult {
        change: record,
        audit,
    })
}

/// A definition approval recorded before the latest send-back no longer counts (CHANGE-12.b).
fn ensure_definition_approval_not_sent_back(
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
    latest_definition_index: usize,
) -> Result<(), String> {
    match ledger.send_backs.last() {
        Some(sent) if sent.cancelled_approvals > latest_definition_index => Err(format!(
            "definition approval was cancelled when `{}` was sent back for re-scoping by {} ({}); run `specsync change approve {} --actor <name>` to approve the new scope",
            record.id, sent.actor, sent.reason, record.id
        )),
        _ => Ok(()),
    }
}
