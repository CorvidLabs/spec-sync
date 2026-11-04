// Send a change back to draft for re-scoping (#844 option A, CHANGE-12).
//
// A change whose scope turns out wrong after approval used to have one way out: deleting its
// workspace by hand, which lost its audit history, because `change answer` only edits a draft and
// `change reopen` only reopens accepted work. `change send-back` moves an approved, implementing or
// verifying change back to draft so its scope can be answered again. It needs a named person and
// a reason (CHANGE-12.a); it cancels the definition approval recorded before it, so the new scope
// needs a fresh one (CHANGE-12.b); and it only appends, so nothing already recorded is lost
// (CHANGE-12.c).
//
// Spec text the change already applied stays where it is, and the send-back lists it. The next
// `change check` converges an item the new scope keeps with the same body rather than applying it
// twice. An item the new scope drops would be left in the spec with no change owning it, so check
// and accept refuse while that text is still there, naming the file and the item.

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
    /// How many approvals the ledger held when the change was sent back. Every definition
    /// approval among them is cancelled; none is removed.
    pub cancelled_approvals: usize,
    /// Whether the change had already applied its deltas to the canonical specs.
    pub canonical_applied: bool,
    /// The spec text the change had already applied when it was sent back.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_items: Vec<AppliedDeltaItem>,
}

/// One block a change's delta has put into a canonical spec or requirements file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedDeltaItem {
    pub module: String,
    /// `requirement` or `section`.
    pub target: String,
    pub key: String,
    pub content: String,
}

impl AppliedDeltaItem {
    fn delta_target(&self) -> DeltaTarget {
        if self.target == "requirement" {
            DeltaTarget::Requirement
        } else {
            DeltaTarget::SpecSection
        }
    }

    fn as_delta_item(&self) -> DeltaItem {
        DeltaItem {
            operation: DeltaOperation::Modified,
            target: self.delta_target(),
            key: self.key.clone(),
            content: self.content.clone(),
        }
    }

    /// For messages, e.g. "`specs/auth/auth.spec.md` section `Purpose`".
    fn describe(&self, root: &Path) -> String {
        let specs_dir = crate::config::load_config(root).specs_dir;
        let paths = canonical_module_paths(root, &specs_dir, &self.module).ok();
        let display = |path: &Path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        };
        match (self.delta_target(), paths) {
            (DeltaTarget::Requirement, Some((_, requirements))) => {
                format!("`{}` requirement `{}`", display(&requirements), self.key)
            }
            (DeltaTarget::SpecSection, Some((spec, _))) => {
                format!("`{}` section `{}`", display(&spec), self.key)
            }
            (_, None) => format!("module `{}` {} `{}`", self.module, self.target, self.key),
        }
    }
}

/// The blocks of `record`'s current deltas that are in the canonical tree, for every module whose
/// Change Log names the change. A REMOVED item leaves no text behind, so it is not listed.
fn applied_delta_items(root: &Path, record: &ChangeRecord) -> Result<Vec<AppliedDeltaItem>, String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut applied = Vec::new();
    for module in &record.affected_specs {
        let delta_path = delta_path_checked(root, record, module)?;
        if !delta_path.exists() {
            continue;
        }
        let delta = read_bounded_change_text(&delta_path, "semantic delta")?;
        let Ok((spec_path, requirements_path)) = canonical_module_paths(root, &specs_dir, module)
        else {
            continue;
        };
        let Ok(spec) = fs::read_to_string(&spec_path) else {
            continue;
        };
        if !changelog_records_change(&spec, &record.id) {
            continue;
        }
        let requirements = fs::read_to_string(&requirements_path).unwrap_or_default();
        for item in parse_delta(&delta)? {
            if item.operation == DeltaOperation::Removed
                || !delta_item_is_applied(&spec, &requirements, &item)
            {
                continue;
            }
            applied.push(AppliedDeltaItem {
                module: module.clone(),
                target: match item.target {
                    DeltaTarget::Requirement => "requirement".into(),
                    DeltaTarget::SpecSection => "section".into(),
                },
                key: item.key,
                content: item.content,
            });
        }
    }
    Ok(applied)
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
    let applied_items = applied_delta_items(root, &record)?;
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
        applied_items,
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

/// How many leading ledger approvals the send-backs have cancelled.
fn send_back_cancelled_prefix(ledger: &ApprovalLedger) -> usize {
    ledger
        .send_backs
        .iter()
        .map(|sent| sent.cancelled_approvals)
        .max()
        .unwrap_or(0)
}

/// A definition approval recorded before the latest send-back no longer counts (CHANGE-12.b).
fn ensure_definition_approval_not_sent_back(
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
    latest_definition_index: usize,
) -> Result<(), String> {
    let cancelled = send_back_cancelled_prefix(ledger);
    match ledger
        .send_backs
        .iter()
        .rfind(|sent| sent.cancelled_approvals == cancelled)
    {
        Some(sent) if cancelled > latest_definition_index => Err(format!(
            "definition approval was cancelled when `{}` was sent back for re-scoping by {} ({}); run `specsync change approve {} --actor <name>` to approve the new scope",
            record.id, sent.actor, sent.reason, record.id
        )),
        _ => Ok(()),
    }
}

/// Send-backs only append (CHANGE-12.c). A send-back the committed ledger already holds cannot be
/// deleted or edited to bring the approval it cancelled back. Outside a git repository, or before
/// the ledger is first committed, there is no committed history to compare with.
fn ensure_committed_send_backs_kept(
    root: &Path,
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
) -> Result<(), String> {
    let Ok(prefix) = git_repo_prefix(root) else {
        return Ok(());
    };
    let Ok(dir) = find_change_dir(root, &record.id) else {
        return Ok(());
    };
    let mut candidates = Vec::new();
    if let Ok(relative) = dir.strip_prefix(root) {
        candidates.push(relative.to_string_lossy().replace('\\', "/"));
    }
    candidates.push(format!("{CHANGES_PATH}/{}", record.id));
    for candidate in candidates {
        let path = format!("{prefix}{candidate}/approvals.json");
        let object = format!("HEAD:{path}");
        let Some(text) = git_output(root, &["show", object.as_str()]) else {
            continue;
        };
        let Ok(committed) = serde_json::from_str::<ApprovalLedger>(&text) else {
            continue;
        };
        if committed.send_backs.len() > ledger.send_backs.len()
            || committed
                .send_backs
                .iter()
                .zip(&ledger.send_backs)
                .any(|(kept, current)| kept != current)
        {
            return Err(format!(
                "the send-back history of `{}` differs from the committed ledger at HEAD: a committed send-back was removed or edited, which would bring back the approval it cancelled; restore `{candidate}/approvals.json`",
                record.id
            ));
        }
        return Ok(());
    }
    Ok(())
}

/// Spec text the change applied before a send-back must not be left in the spec with no change
/// owning it. An item the new scope no longer declares (same module, target and key) is refused
/// while the canonical tree still carries the body the change applied.
fn ensure_no_orphaned_applied_text(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    if ledger.send_backs.iter().all(|sent| sent.applied_items.is_empty()) {
        return Ok(());
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut declared: BTreeMap<String, Vec<DeltaItem>> = BTreeMap::new();
    for module in &record.affected_specs {
        let delta_path = delta_path_checked(root, record, module)?;
        if delta_path.exists() {
            let delta = read_bounded_change_text(&delta_path, "semantic delta")?;
            declared.insert(module.clone(), parse_delta(&delta)?);
        }
    }
    let mut orphans = Vec::new();
    for item in ledger.send_backs.iter().flat_map(|sent| &sent.applied_items) {
        let kept = declared.get(&item.module).is_some_and(|items| {
            items
                .iter()
                .any(|current| current.target == item.delta_target() && current.key == item.key)
        });
        if kept {
            continue;
        }
        let Ok((spec_path, requirements_path)) =
            canonical_module_paths(root, &specs_dir, &item.module)
        else {
            continue;
        };
        let spec = fs::read_to_string(&spec_path).unwrap_or_default();
        let requirements = fs::read_to_string(&requirements_path).unwrap_or_default();
        if delta_item_is_applied(&spec, &requirements, &item.as_delta_item()) {
            let description = item.describe(root);
            if !orphans.contains(&description) {
                orphans.push(description);
            }
        }
    }
    if orphans.is_empty() {
        return Ok(());
    }
    Err(format!(
        "`{}` was re-scoped after it had applied spec text that its deltas no longer declare, so nothing would own it: {}. Take that text out of the spec, or keep the item in the change's delta, then run `specsync change check {}` again",
        record.id,
        orphans.join(", "),
        record.id
    ))
}
