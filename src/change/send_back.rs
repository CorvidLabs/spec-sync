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
// Spec text the change already applied stays where it is, and the send-back lists it, with the
// text each block had before the change. Once a change has been sent back, `change check` always
// converges: an item the new scope keeps with the same body (a REMOVED block still gone, an ADDED
// or MODIFIED block with the same text) is not applied twice, and an ADDED item kept under the
// same key with a new body replaces the change's own earlier body. An item the new scope drops,
// and a module it drops whose Change Log still carries the change's row, would be left in the spec
// with no change owning it, so check and accept refuse until it is restored, naming the file, the
// item and the text to restore.

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
    /// How many scoped review attempts `review-attempts.json` held when the change was sent back.
    /// None of them counts as a current review any more (CHANGE-12.e); none is removed. `None` on a
    /// send-back recorded before this field existed, which falls back to comparing timestamps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_review_attempts: Option<usize>,
    /// Whether the change had already applied its deltas to the canonical specs.
    pub canonical_applied: bool,
    /// The spec text the change had already applied when it was sent back.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_items: Vec<AppliedDeltaItem>,
    /// The modules whose Change Log carried this change's row (and so its version bump) when it
    /// was sent back.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_modules: Vec<String>,
}

/// One block a change's delta has put into a canonical spec or requirements file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedDeltaItem {
    pub module: String,
    /// `added`, `modified` or `removed`.
    #[serde(default = "modified_operation")]
    pub operation: String,
    /// `requirement` or `section`.
    pub target: String,
    pub key: String,
    /// The body the change applied; empty for a REMOVED item.
    pub content: String,
    /// The block's body before the change applied it, from Git history, when it is known. `None`
    /// for an ADDED block, which did not exist before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
}

fn modified_operation() -> String {
    "modified".into()
}

fn operation_name(operation: DeltaOperation) -> &'static str {
    match operation {
        DeltaOperation::Added => "added",
        DeltaOperation::Modified => "modified",
        DeltaOperation::Removed => "removed",
    }
}

impl AppliedDeltaItem {
    fn delta_target(&self) -> DeltaTarget {
        if self.target == "requirement" {
            DeltaTarget::Requirement
        } else {
            DeltaTarget::SpecSection
        }
    }

    fn delta_operation(&self) -> DeltaOperation {
        match self.operation.as_str() {
            "added" => DeltaOperation::Added,
            "removed" => DeltaOperation::Removed,
            _ => DeltaOperation::Modified,
        }
    }

    fn as_delta_item(&self) -> DeltaItem {
        DeltaItem {
            operation: self.delta_operation(),
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

/// The blocks of `record`'s current deltas that are in the canonical tree, and the modules whose
/// Change Log names the change. A REMOVED item is listed while its block is still gone; every
/// item carries the block's text from before the change when Git history has it.
fn applied_delta_items(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(Vec<AppliedDeltaItem>, Vec<String>), String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut applied = Vec::new();
    let mut modules = Vec::new();
    for module in delta_modules(root, record) {
        let delta_path = delta_path_checked(root, record, module)?;
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
        modules.push(module.clone());
        if !delta_path.exists() {
            continue;
        }
        let delta = read_bounded_change_text(&delta_path, "semantic delta")?;
        let requirements = fs::read_to_string(&requirements_path).unwrap_or_default();
        for item in parse_delta(&delta)? {
            if !delta_item_is_applied(&spec, &requirements, &item) {
                continue;
            }
            let path = match item.target {
                DeltaTarget::Requirement => &requirements_path,
                DeltaTarget::SpecSection => &spec_path,
            };
            let previous_content = if item.operation == DeltaOperation::Added {
                None
            } else {
                pre_change_block(root, record, &spec_path, path, &item)
            };
            applied.push(AppliedDeltaItem {
                module: module.clone(),
                operation: operation_name(item.operation).into(),
                target: match item.target {
                    DeltaTarget::Requirement => "requirement".into(),
                    DeltaTarget::SpecSection => "section".into(),
                },
                key: item.key,
                content: item.content,
                previous_content,
            });
        }
    }
    Ok((applied, modules))
}

fn project_relative(root: &Path, path: &Path) -> Option<String> {
    Some(
        path.strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

/// The body `item`'s block had before the change applied it. The pre-change tree is the parent of
/// the first commit whose spec carries the change's Change Log row, or HEAD while that row is not
/// committed yet. Outside Git, or when the block is not in that tree, the text is unknown.
fn pre_change_block(
    root: &Path,
    record: &ChangeRecord,
    spec_path: &Path,
    path: &Path,
    item: &DeltaItem,
) -> Option<String> {
    let prefix = git_repo_prefix(root).ok()?;
    let spec = project_relative(root, spec_path)?;
    let file = project_relative(root, path)?;
    let row = format!("{}:", record.id);
    let commits = git_output_allow_empty(
        root,
        &["log", "--reverse", "--format=%H", "-S", row.as_str(), "--", spec.as_str()],
    )?;
    let tree = match commits.lines().next() {
        Some(commit) => format!("{}^", commit.trim()),
        None => "HEAD".into(),
    };
    let object = format!("{tree}:{prefix}{file}");
    let text = git_output(root, &["show", object.as_str()])?;
    block_body(&text, item)
}

/// The body under `item`'s heading in `source`, without the heading line.
fn block_body(source: &str, item: &DeltaItem) -> Option<String> {
    let prefix = match item.target {
        DeltaTarget::Requirement => "### ",
        DeltaTarget::SpecSection => "## ",
    };
    let heading = format!("{prefix}{}", item.key);
    let range = markdown_block_range(source, &heading, markdown_prefix_level(prefix))?;
    let block = &source[range];
    let body = block.split_once('\n').map_or("", |(_, rest)| rest);
    Some(body.trim_matches(['\n', ' ', '\t']).to_string())
}

/// After a send-back, an ADDED item the new scope keeps under the same key may change its body.
/// The block is the change's own earlier text when a send-back listed it as added and the canonical
/// block still carries that text; only then is it replaced rather than refused as existing.
fn sent_back_added_block_is_own(
    ledger: &ApprovalLedger,
    module: &str,
    item: &DeltaItem,
    spec: &str,
    requirements: &str,
) -> bool {
    item.operation == DeltaOperation::Added
        && ledger
            .send_backs
            .iter()
            .flat_map(|sent| &sent.applied_items)
            .any(|applied| {
                applied.module == module
                    && applied.delta_operation() == DeltaOperation::Added
                    && applied.delta_target() == item.target
                    && applied.key == item.key
                    && delta_item_is_applied(spec, requirements, &applied.as_delta_item())
            })
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
    let actor = normalize_actor(&actor, "send-back")?;
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("send-back requires a non-empty reason passed with --reason".into());
    }
    let mut ledger = load_approvals(root, &record)?;
    let (applied_items, applied_modules) = applied_delta_items(root, &record)?;
    let audit = SendBackRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: actor.to_string(),
        reason: reason.to_string(),
        timestamp: now(),
        from_state: record.state,
        to_state: ChangeState::Draft,
        cancelled_approvals: ledger.approvals.len(),
        cancelled_review_attempts: Some(recorded_scoped_review_attempts(root, &record)?),
        canonical_applied: record.canonical_applied,
        applied_items,
        applied_modules,
    };
    ledger.send_backs.push(audit.clone());
    record.state = ChangeState::Draft;
    // Materialization has to run again for the new scope. Because the ledger now holds a
    // send-back, it converges on what the canonical tree already carries (see
    // `prepare_module_application`), so the blocks this change applied are not applied twice.
    record.canonical_applied = false;
    record.updated_at = now();
    write_prepared_files(
        root,
        &[
            {
                let path = change_dir(root, &record.id).join("approvals.json");
                let content = json_content_keeping_unknown_members(&path, &ledger)?;
                (path, content)
            },
            {
                let workspace = change_dir(root, &record.id);
                let path = workspace.join("state.json");
                let content = json_content_keeping_unknown_members(
                    &path,
                    relay_gated_record(&workspace, &record).as_ref(),
                )?;
                (path, content)
            },
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

/// Send-backs only append (CHANGE-12.c). No send-back of this change that any commit of its
/// ledger held may be deleted or edited later, in the working tree or in a later commit, because
/// that would bring back the approval it cancelled. The ledger's history is one cached
/// `git log -G send_backs` per path; outside a git repository there is no history to compare with.
fn ensure_committed_send_backs_kept(
    root: &Path,
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
) -> Result<(), String> {
    let Ok(prefix) = git_repo_prefix(root) else {
        return Ok(());
    };
    let mut candidates = Vec::new();
    if let Ok(dir) = find_change_dir(root, &record.id)
        && let Some(relative) = project_relative(root, &dir)
    {
        candidates.push(relative);
    }
    let active = format!("{CHANGES_PATH}/{}", record.id);
    if !candidates.contains(&active) {
        candidates.push(active);
    }
    for candidate in candidates {
        let path = format!("{candidate}/approvals.json");
        let Some(commits) = git_output_allow_empty(
            root,
            &["log", "--format=%H", "-G", "send_backs", "--", path.as_str()],
        ) else {
            continue;
        };
        for commit in commits.lines().map(str::trim).filter(|line| !line.is_empty()) {
            let object = format!("{commit}:{prefix}{path}");
            let Some(text) = git_output(root, &["show", object.as_str()]) else {
                continue;
            };
            let Ok(committed) = serde_json::from_str::<ApprovalLedger>(&text) else {
                continue;
            };
            let kept: Vec<_> = committed
                .send_backs
                .iter()
                .filter(|sent| sent.change_id == record.id)
                .collect();
            if kept.len() > ledger.send_backs.len()
                || kept
                    .iter()
                    .zip(&ledger.send_backs)
                    .any(|(kept, current)| *kept != current)
            {
                let short = &commit[..commit.len().min(12)];
                return Err(format!(
                    "the send-back history of `{}` differs from the one committed in {short}: a committed send-back was removed or edited, which would bring back the approval it cancelled; restore the send-backs in `{path}` from `git show {short}:{prefix}{path}`",
                    record.id
                ));
            }
        }
    }
    Ok(())
}

/// The project audit checks the send-back history of every change it audits, active or
/// archived, so a deleted send-back cannot hide in a committed ledger.
fn send_back_history_errors<'a>(
    root: &Path,
    records: impl IntoIterator<Item = &'a ChangeRecord>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for record in records {
        let Ok(ledger) = load_approvals(root, record) else {
            continue;
        };
        if let Err(error) = ensure_committed_send_backs_kept(root, record, &ledger) {
            errors.push(error);
        }
    }
    errors
}

/// Spec text the change applied before a send-back must not be left in the spec with no change
/// owning it. An item the new scope no longer declares (same module, target and key) is refused
/// while the canonical tree still carries what the change did to it: the body it applied, or for
/// a REMOVED item the block's absence. A module the new scope drops is refused while its Change
/// Log still carries the change's row and version bump. The refusal names what to restore.
fn ensure_no_orphaned_applied_text(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    if ledger
        .send_backs
        .iter()
        .all(|sent| sent.applied_items.is_empty() && sent.applied_modules.is_empty())
    {
        return Ok(());
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut declared: BTreeMap<String, Vec<DeltaItem>> = BTreeMap::new();
    for module in delta_modules(root, record) {
        let delta_path = delta_path_checked(root, record, module)?;
        if delta_path.exists() {
            let delta = read_bounded_change_text(&delta_path, "semantic delta")?;
            declared.insert(module.clone(), parse_delta(&delta)?);
        }
    }
    let mut orphans: Vec<String> = Vec::new();
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
        if !delta_item_is_applied(&spec, &requirements, &item.as_delta_item()) {
            continue;
        }
        let description = item.describe(root);
        let advice = match (item.delta_operation(), &item.previous_content) {
            (DeltaOperation::Added, _) => {
                format!("{description}, which it added: take that block out of the spec")
            }
            (DeltaOperation::Removed, Some(previous)) => format!(
                "{description}, which it removed: restore the block with the text it had before this change:\n\n{previous}\n"
            ),
            (_, Some(previous)) => format!(
                "{description}, which it modified: restore the text it had before this change:\n\n{previous}\n"
            ),
            (DeltaOperation::Removed, None) => format!(
                "{description}, which it removed: restore the block as it was before this change"
            ),
            (_, None) => format!(
                "{description}, which it modified: restore the text it had before this change"
            ),
        };
        if !orphans.contains(&advice) {
            orphans.push(advice);
        }
    }
    for module in ledger.send_backs.iter().flat_map(|sent| &sent.applied_modules) {
        if record.affected_specs.contains(module) {
            continue;
        }
        let Ok((spec_path, _)) = canonical_module_paths(root, &specs_dir, module) else {
            continue;
        };
        let spec = fs::read_to_string(&spec_path).unwrap_or_default();
        if !changelog_records_change(&spec, &record.id) {
            continue;
        }
        let advice = format!(
            "`{}`, a module it no longer declares, still has its Change Log row and version bump: remove the `{}:` row from its `## Change Log` and restore its `version:`",
            project_relative(root, &spec_path).unwrap_or_else(|| spec_path.display().to_string()),
            record.id
        );
        if !orphans.contains(&advice) {
            orphans.push(advice);
        }
    }
    if orphans.is_empty() {
        return Ok(());
    }
    Err(format!(
        "`{}` was re-scoped after it had changed spec text that its deltas no longer declare, so nothing would own it: {}. Make those edits, or keep the items in the change's delta, then run `specsync change check {}` again",
        record.id,
        orphans.join("; "),
        record.id
    ))
}
