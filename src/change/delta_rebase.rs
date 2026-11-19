// Rebasing a stale MODIFIED requirement delta over main's edit (#1085, REQ-change-161).
//
// A `## MODIFIED` body replaces a requirement's whole text, written against the block as it stood
// at approval. When main edits that requirement afterwards, applying the old body would delete
// main's edit, so #843 refuses the delta and the human re-approves a hand-rebased body even when
// the two edits are nowhere near each other. Here `change check` merges the approval-time text,
// main's text and the approved body word by word; when the edits do not overlap it rewrites the
// delta to the merge and records the rebase, and the approval stands.
//
// The record is not a claim to be trusted. It holds its own inputs (the approval-time block, main's
// block and the delta before the rebase), and every reader recomputes the merge and its digests
// from them. A record that does not recompute is refused, so a rebase can only ever carry exactly
// the merge of approved wording and main's wording, never wording nobody approved (CHANGE-6.a).

/// The longest side, in tokens, the word merge compares; a larger block keeps the refusal.
const MAX_REBASE_TOKENS: usize = 4_000;

/// Who records a rebase. A rebase is never a human decision, so it is never recorded as one.
const DELTA_REBASE_ACTOR: &str = "specsync:rebase";

/// One rebase of one `## MODIFIED` requirement item over main's edit (REQ-change-161).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaRebaseRecord {
    pub schema_version: u32,
    /// The module whose delta was rebased.
    pub module: String,
    /// The item's base key, e.g. `auth: REQUIREMENT REQ-auth-001`.
    pub item: String,
    /// The definition approval this rebase extends, by its timestamp and digest.
    pub approval_timestamp: u64,
    pub approval_digest: String,
    pub actor: String,
    pub timestamp: u64,
    /// The canonical block the approved body was written against, trimmed.
    pub base_block: String,
    /// The canonical block main had when the delta was rebased over it, trimmed.
    pub main_block: String,
    /// The module's whole delta before this rebase.
    pub delta_before: String,
    /// The digest of the module's delta after this rebase.
    pub delta_after_digest: String,
}

/// One rebase as `change status` shows it (REQ-cmd-change-038).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaRebaseSummary {
    pub module: String,
    pub requirement: String,
    pub rebased_at: u64,
}

/// The UTC calendar date of a Unix timestamp, `YYYY-MM-DD`, for status lines.
pub fn utc_date_of(seconds: u64) -> String {
    utc_date(seconds)
}

/// A text split into words, each with the whitespace that follows it; leading whitespace is a
/// token of its own. Joining the tokens gives the text back byte for byte.
fn word_tokens(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut in_space = text.starts_with(char::is_whitespace);
    for (index, character) in text.char_indices() {
        let space = character.is_whitespace();
        if !space && in_space && index > start {
            tokens.push(&text[start..index]);
            start = index;
        }
        in_space = space;
    }
    if start < text.len() {
        tokens.push(&text[start..]);
    }
    tokens
}

/// The word-level three-way merge of `approved` and `current` over their common `base`, or `None`
/// when the two edits overlap. Aligned diff3-style on the base, as `three_way_keeps_body` aligns
/// lines: between stable words, a stretch that only one side changed takes that side; a stretch
/// both sides changed, including two insertions at one point, is an overlap unless both sides
/// made the same change.
fn merge_words(base: &str, current: &str, approved: &str) -> Option<String> {
    let base = word_tokens(base);
    let current = word_tokens(current);
    let approved = word_tokens(approved);
    if [base.len(), current.len(), approved.len()]
        .iter()
        .any(|tokens| *tokens > MAX_REBASE_TOKENS)
    {
        return None;
    }
    let to_current = longest_common_subsequence_map(&base, &current);
    let to_approved = longest_common_subsequence_map(&base, &approved);
    let (mut b, mut c, mut a) = (0usize, 0usize, 0usize);
    let mut merged = String::new();
    loop {
        while b < base.len() && to_current[b] == Some(c) && to_approved[b] == Some(a) {
            merged.push_str(base[b]);
            b += 1;
            c += 1;
            a += 1;
        }
        if b == base.len() && c == current.len() && a == approved.len() {
            return Some(merged);
        }
        let next = (b..base.len()).find(|&index| {
            to_current[index].is_some_and(|token| token >= c)
                && to_approved[index].is_some_and(|token| token >= a)
        });
        let (base_end, current_end, approved_end) = match next {
            Some(index) => (
                index,
                to_current[index].unwrap_or(current.len()),
                to_approved[index].unwrap_or(approved.len()),
            ),
            None => (base.len(), current.len(), approved.len()),
        };
        let base_chunk = &base[b..base_end];
        let current_chunk = &current[c..current_end];
        let approved_chunk = &approved[a..approved_end];
        let taken = if current_chunk == base_chunk {
            approved_chunk
        } else if approved_chunk == base_chunk || approved_chunk == current_chunk {
            current_chunk
        } else {
            return None;
        };
        merged.extend(taken.iter().copied());
        b = base_end;
        c = current_end;
        a = approved_end;
        if next.is_none() {
            return Some(merged);
        }
    }
}

/// A block as the base digest frames it: CRLF folded, surrounding blank lines and spaces trimmed.
fn trimmed_block(block: &str) -> String {
    block
        .replace("\r\n", "\n")
        .trim_matches(['\n', ' ', '\t'])
        .to_string()
}

/// The body of a trimmed block: everything after its heading line, trimmed.
fn trimmed_block_body(block: &str) -> &str {
    block
        .split_once('\n')
        .map_or("", |(_, body)| body)
        .trim_matches(['\n', ' ', '\t'])
}

/// The base digest of `item`'s block when the block is `block`, framed exactly as
/// `canonical_block_digest` frames a block it finds in a canonical file.
fn item_block_digest(module: &str, item: &DeltaItem, block: &str) -> String {
    let mut digest = FramedDigest::new(APPROVED_DELTA_BASE_DOMAIN);
    digest.frame(b"key", delta_base_key(module, item).as_bytes());
    digest.frame(b"block", trimmed_block(block).as_bytes());
    digest.finish()
}

/// The `## MODIFIED` requirement item of `delta` whose base key is `key`.
fn modified_requirement_item(module: &str, delta: &str, key: &str) -> Option<DeltaItem> {
    parse_delta(delta).ok()?.into_iter().find(|item| {
        item.operation == DeltaOperation::Modified
            && item.target == DeltaTarget::Requirement
            && delta_base_key(module, item) == key
    })
}

/// `delta` with `item`'s body replaced by `body`, every other byte kept.
fn rebased_delta_text(delta: &str, item: &DeltaItem, body: &str) -> Result<String, String> {
    apply_markdown_block(
        delta,
        "### ",
        &format!("REQUIREMENT {}", item.key),
        body,
        DeltaOperation::Modified,
    )
}

/// The delta `record` produces from its own inputs: the merge of its base block, main's block and
/// the approved body in its pre-rebase delta. `Err` names what does not recompute.
fn recompute_rebase(record: &DeltaRebaseRecord) -> Result<(DeltaItem, String), String> {
    let item = modified_requirement_item(&record.module, &record.delta_before, &record.item)
        .ok_or_else(|| "its delta has no such MODIFIED requirement".to_string())?;
    let approved = trimmed_block(&item.content);
    let merged = merge_words(
        trimmed_block_body(&record.base_block),
        trimmed_block_body(&record.main_block),
        &approved,
    )
    .ok_or_else(|| "its texts do not merge without overlap".to_string())?;
    let after = rebased_delta_text(&record.delta_before, &item, &merged)?;
    if delta_body_digest(&record.module, &after) != record.delta_after_digest {
        return Err("its merge does not give the delta it records".into());
    }
    Ok((item, after))
}

/// A definition approval's delta claims: each module's delta digest, and each replacing item's
/// canonical base, keyed as the approval keys them.
type DeltaClaims = (
    Option<BTreeMap<String, String>>,
    Option<BTreeMap<String, String>>,
);

/// The delta digests and canonical bases `approval` binds once every rebase recorded under it is
/// folded in, in order (REQ-change-161). Each rebase must follow the digest and base its
/// predecessor left and must recompute from its own texts; one that does not is refused, naming it.
/// `None` claims stay `None`, and a rebase under an approval without claims is refused.
fn effective_delta_claims(
    id: &str,
    ledger: &ApprovalLedger,
    approval: &ApprovalRecord,
) -> Result<DeltaClaims, String> {
    let mut digests = approval.approved_delta_digests.clone();
    let mut bases = approval.approved_delta_bases.clone();
    for (index, rebase) in ledger.delta_rebases.iter().enumerate() {
        if rebase.approval_timestamp != approval.timestamp
            || rebase.approval_digest != approval.digest
        {
            continue;
        }
        let refuse = |why: &str| {
            format!(
                "delta rebase {} in {id}'s approvals.json ({}) does not recompute: {why}; restore approvals.json and the delta from Git, or approve the delta as it stands with `specsync change approve {id}`",
                index + 1,
                rebase.item
            )
        };
        if rebase.schema_version != 1 {
            return Err(refuse("its schema version is not 1"));
        }
        let (Some(digests), Some(bases)) = (digests.as_mut(), bases.as_mut()) else {
            return Err(refuse("its approval records no delta digests or bases"));
        };
        if digests.get(&rebase.module)
            != Some(&delta_body_digest(&rebase.module, &rebase.delta_before))
        {
            return Err(refuse("its delta before the rebase is not the approved delta"));
        }
        let (item, _) = recompute_rebase(rebase).map_err(|why| refuse(&why))?;
        if bases.get(&rebase.item)
            != Some(&item_block_digest(&rebase.module, &item, &rebase.base_block))
        {
            return Err(refuse("its base block is not the approved base"));
        }
        digests.insert(rebase.module.clone(), rebase.delta_after_digest.clone());
        bases.insert(
            rebase.item.clone(),
            item_block_digest(&rebase.module, &item, &rebase.main_block),
        );
    }
    Ok((digests, bases))
}

/// The rebases recorded under the change's effective definition approval, for `change status`
/// (REQ-cmd-change-038). Empty when there are none or the ledger cannot be read; a record that
/// does not recompute is reported by the binding, not here.
pub fn recorded_delta_rebases(root: &Path, record: &ChangeRecord) -> Vec<DeltaRebaseSummary> {
    let Ok(ledger) = load_approvals(root, record) else {
        return Vec::new();
    };
    let Ok(approval) = effective_definition_approval(root, record, &ledger) else {
        return Vec::new();
    };
    ledger
        .delta_rebases
        .iter()
        .filter(|rebase| {
            rebase.approval_timestamp == approval.timestamp
                && rebase.approval_digest == approval.digest
        })
        .map(|rebase| DeltaRebaseSummary {
            module: rebase.module.clone(),
            requirement: rebase
                .item
                .rsplit_once(" REQUIREMENT ")
                .map_or_else(|| rebase.item.clone(), |(_, key)| key.to_string()),
            rebased_at: rebase.timestamp,
        })
        .collect()
}

/// The canonical block whose base digest an approval recorded, read from the newest commit that
/// holds it; the full trimmed block, so a rebase record can show its digest. `None` outside Git,
/// past the history bound, or when no commit matches.
fn recover_approved_base_block(
    root: &Path,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> Option<String> {
    let prefix = git_repo_prefix(root).ok()?;
    let file = project_relative(root, path)?;
    let limit = MOVED_BASE_HISTORY_LIMIT.to_string();
    let commits = git_output_allow_empty(
        root,
        &["log", "--format=%H", "-n", limit.as_str(), "--", file.as_str()],
    )?;
    let heading = format!("### {}", item.key);
    for commit in commits.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let object = format!("{commit}:{prefix}{file}");
        let text = git_output_allow_empty(root, &["show", object.as_str()]).unwrap_or_default();
        if canonical_block_digest(module, "", &text, item) == base {
            let range = markdown_block_range(&text, &heading, 3)?;
            return Some(trimmed_block(&text[range]));
        }
    }
    None
}

/// Rebase every unapplied `## MODIFIED` requirement item of `record` whose canonical block moved
/// since its approval, when main's edit and the change's edit do not overlap (REQ-change-161).
/// Each rebase rewrites the item's body in the delta and appends a record to `approvals.json`, in
/// one prepared write. An item that cannot be rebased is left alone, so the base check that runs
/// next refuses it exactly as before. Returns the number of rebases written.
///
/// Workflow-v1 changes are left out: their definition digest hashes the delta bodies, so a
/// rebase there would change what the approval signed. CI never writes.
fn rebase_stale_modified_requirements(root: &Path, record: &ChangeRecord) -> Result<usize, String> {
    if record.no_spec_change || record.workflow_version < 2 || is_ci_project(root) {
        return Ok(0);
    }
    let mut ledger = load_approvals(root, record)?;
    let approval = effective_definition_approval(root, record, &ledger)?.clone();
    let (Some(mut digests), Some(mut bases)) = effective_delta_claims(&record.id, &ledger, &approval)?
    else {
        return Ok(0);
    };
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut prepared = Vec::new();
    let mut rebases = Vec::new();
    for module in &record.affected_specs {
        let delta_path = delta_path_checked(root, record, module)?;
        let mut delta = read_bounded_change_text(&delta_path, "semantic delta")
            .map_err(|error| format!("semantic delta for {module}: {error}"))?;
        if digests.get(module) != Some(&delta_body_digest(module, &delta)) {
            // Not the approved wording; the delta-body check refuses it.
            continue;
        }
        let (spec_path, requirements_path) = canonical_module_paths(root, &specs_dir, module)?;
        let spec = read_canonical_or_absent(root, &spec_path)?;
        let requirements = read_canonical_or_absent(root, &requirements_path)?;
        let items = parse_delta(&delta)?;
        let mut changed = false;
        for item in items.iter().filter(|item| {
            item.operation == DeltaOperation::Modified && item.target == DeltaTarget::Requirement
        }) {
            if delta_item_is_applied(&spec, &requirements, item) {
                continue;
            }
            let key = delta_base_key(module, item);
            let Some(base) = bases.get(&key).cloned() else {
                continue;
            };
            if canonical_block_digest(module, &spec, &requirements, item) == base {
                continue;
            }
            let Some(range) = markdown_block_range(&requirements, &format!("### {}", item.key), 3)
            else {
                continue;
            };
            let main_block = trimmed_block(&requirements[range]);
            let Some(base_block) =
                recover_approved_base_block(root, &requirements_path, module, item, &base)
            else {
                continue;
            };
            let Some(current) = modified_requirement_item(module, &delta, &key) else {
                continue;
            };
            let Some(merged) = merge_words(
                trimmed_block_body(&base_block),
                trimmed_block_body(&main_block),
                &trimmed_block(&current.content),
            ) else {
                continue;
            };
            let Ok(after) = rebased_delta_text(&delta, &current, &merged) else {
                continue;
            };
            let rebase = DeltaRebaseRecord {
                schema_version: 1,
                module: module.clone(),
                item: key.clone(),
                approval_timestamp: approval.timestamp,
                approval_digest: approval.digest.clone(),
                actor: DELTA_REBASE_ACTOR.into(),
                timestamp: now(),
                base_block,
                main_block: main_block.clone(),
                delta_before: delta.clone(),
                delta_after_digest: delta_body_digest(module, &after),
            };
            // Written only if it recomputes, exactly as every later reader will recompute it.
            if recompute_rebase(&rebase).is_err() {
                continue;
            }
            digests.insert(module.clone(), rebase.delta_after_digest.clone());
            bases.insert(key, item_block_digest(module, &current, &main_block));
            rebases.push(rebase);
            delta = after;
            changed = true;
        }
        if changed {
            prepared.push((delta_path, delta));
        }
    }
    if rebases.is_empty() {
        return Ok(0);
    }
    let count = rebases.len();
    ledger.delta_rebases.extend(rebases);
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    prepared.push((
        approvals_path.clone(),
        json_content_keeping_unknown_members(&approvals_path, &ledger)?,
    ));
    write_prepared_files(root, &prepared)?;
    Ok(count)
}
