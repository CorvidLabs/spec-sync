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
    /// The definition approval this rebase extends: its position in `approvals`, its timestamp and
    /// its digest, so a re-approval in the same second with the same digest never inherits it.
    pub approval_index: usize,
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
/// when the two edits overlap or cannot be aligned unambiguously.
///
/// The texts are aligned only on anchors: words that occur exactly once in each of the three, in
/// an order that rises in all three, found again inside each gap between anchors. A repeated word
/// is never an anchor, so it can never be matched to one occurrence on one side and another on the
/// other, which is how an LCS alignment once let two edits to the same words merge (Codex's review
/// of #1089: base `c a b d`, main `b b d`, approved `b d`). Between anchors, a stretch only one side
/// changed takes that side; a stretch both changed, the same way excepted, is an overlap, and so is
/// a stretch both changed with no anchor inside to split it.
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
    let mut merged = Vec::new();
    merge_token_ranges(&base, &current, &approved, &mut merged)?;
    Some(merged.concat())
}

fn merge_token_ranges<'a>(
    base: &[&'a str],
    current: &[&'a str],
    approved: &[&'a str],
    merged: &mut Vec<&'a str>,
) -> Option<()> {
    if current == base {
        merged.extend_from_slice(approved);
        return Some(());
    }
    if approved == base || approved == current {
        merged.extend_from_slice(current);
        return Some(());
    }
    let anchors = unique_anchors(base, current, approved);
    if anchors.is_empty() {
        return None;
    }
    let (mut b, mut c, mut a) = (0usize, 0usize, 0usize);
    for (base_at, current_at, approved_at) in anchors {
        merge_token_ranges(
            &base[b..base_at],
            &current[c..current_at],
            &approved[a..approved_at],
            merged,
        )?;
        merged.push(base[base_at]);
        b = base_at + 1;
        c = current_at + 1;
        a = approved_at + 1;
    }
    merge_token_ranges(&base[b..], &current[c..], &approved[a..], merged)
}

/// The words that occur exactly once in each of `base`, `current` and `approved`, as index triples,
/// keeping the longest chain whose indices rise in all three.
fn unique_anchors(base: &[&str], current: &[&str], approved: &[&str]) -> Vec<(usize, usize, usize)> {
    fn once<'a>(tokens: &[&'a str]) -> BTreeMap<&'a str, Option<usize>> {
        let mut seen: BTreeMap<&str, Option<usize>> = BTreeMap::new();
        for (index, token) in tokens.iter().enumerate() {
            seen.entry(token)
                .and_modify(|slot| *slot = None)
                .or_insert(Some(index));
        }
        seen
    }
    let (in_current, in_approved) = (once(current), once(approved));
    let candidates: Vec<(usize, usize, usize)> = once(base)
        .into_iter()
        .filter_map(|(token, base_at)| {
            Some((
                base_at?,
                (*in_current.get(token)?)?,
                (*in_approved.get(token)?)?,
            ))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // Longest chain rising in the current and approved indices (the base index already rises).
    let mut length = vec![1usize; candidates.len()];
    let mut previous = vec![None; candidates.len()];
    for later in 0..candidates.len() {
        for earlier in 0..later {
            if candidates[earlier].1 < candidates[later].1
                && candidates[earlier].2 < candidates[later].2
                && length[earlier] + 1 > length[later]
            {
                length[later] = length[earlier] + 1;
                previous[later] = Some(earlier);
            }
        }
    }
    let Some(mut at) = (0..candidates.len()).max_by_key(|&index| (length[index], usize::MAX - index))
    else {
        return Vec::new();
    };
    let mut chain = vec![candidates[at]];
    while let Some(earlier) = previous[at] {
        chain.push(candidates[earlier]);
        at = earlier;
    }
    chain.reverse();
    chain
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
///
/// For a change still being delivered, each record's main block must also be text the default
/// branch held: found by digest in the default branch's history of the requirement's file
/// (fable-lead's #1089 review). A block that only the change's own branch holds, such as a sentence
/// added by hand or in a conflict resolution, is wording nobody approved, so it is refused, and so
/// is a record whose default branch cannot be read. An accepted or archived change is not re-judged
/// against a default branch that has since moved on; its acceptance bound the canonical text.
fn effective_delta_claims(
    root: &Path,
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
    approval: &ApprovalRecord,
) -> Result<DeltaClaims, String> {
    let id = record.id.as_str();
    let delivering = matches!(
        record.state,
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
    );
    let mut digests = approval.approved_delta_digests.clone();
    let mut bases = approval.approved_delta_bases.clone();
    let approval_index = approval_position(ledger, approval);
    for (index, rebase) in ledger.delta_rebases.iter().enumerate() {
        if !rebase_extends(rebase, approval_index, approval) {
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
        if delivering {
            let specs_dir = crate::config::load_config(root).specs_dir;
            let (_, requirements_path) = canonical_module_paths(root, &specs_dir, &rebase.module)?;
            let main_digest = item_block_digest(&rebase.module, &item, &rebase.main_block);
            match block_on_default_branch(
                root,
                &requirements_path,
                &rebase.module,
                &item,
                &main_digest,
                &item_block_digest(&rebase.module, &item, &rebase.base_block),
            ) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(refuse(
                        "its main block is not text the default branch held, so it would carry wording nobody approved",
                    ));
                }
                Err(why) => return Err(refuse(&why)),
            }
        }
        digests.insert(rebase.module.clone(), rebase.delta_after_digest.clone());
        bases.insert(
            rebase.item.clone(),
            item_block_digest(&rebase.module, &item, &rebase.main_block),
        );
    }
    Ok((digests, bases))
}

/// Where `approval` sits in `ledger.approvals`, by identity when it is one of the ledger's own
/// records; `None` for a copy, which no rebase extends.
fn approval_position(ledger: &ApprovalLedger, approval: &ApprovalRecord) -> Option<usize> {
    ledger
        .approvals
        .iter()
        .position(|candidate| std::ptr::eq(candidate, approval))
}

/// Whether `rebase` extends the approval at `approval_index`: the same position, time and digest,
/// so a re-approval in the same second with the same digest does not inherit an earlier
/// approval's rebases (Codex's review of #1089).
fn rebase_extends(
    rebase: &DeltaRebaseRecord,
    approval_index: Option<usize>,
    approval: &ApprovalRecord,
) -> bool {
    approval_index == Some(rebase.approval_index)
        && rebase.approval_timestamp == approval.timestamp
        && rebase.approval_digest == approval.digest
}

/// The rebases recorded under the change's effective definition approval, for `change status`
/// (REQ-cmd-change-038). Empty when there are none, when the ledger cannot be read, and when the
/// records do not recompute: status never says "the approval stands" for a rebase the binding
/// refuses (Codex's review of #1089); the binding's own refusal says what is wrong.
pub fn recorded_delta_rebases(root: &Path, record: &ChangeRecord) -> Vec<DeltaRebaseSummary> {
    let Ok(ledger) = load_approvals(root, record) else {
        return Vec::new();
    };
    let Ok(approval) = effective_definition_approval(root, record, &ledger) else {
        return Vec::new();
    };
    if effective_delta_claims(root, record, &ledger, approval).is_err() {
        return Vec::new();
    }
    let approval_index = approval_position(&ledger, approval);
    ledger
        .delta_rebases
        .iter()
        .filter(|rebase| rebase_extends(rebase, approval_index, approval))
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

/// Whether the default branch (`origin/HEAD`, else `origin/main` or `origin/master`) held `item`'s
/// block with digest `digest` more recently than the block with digest `base`, in the newest
/// `MOVED_BASE_HISTORY_LIMIT` commits that touched `path`. `Err` when there is no default branch to read, or Git cannot read it; the rebase then
/// keeps today's refusal (fable-lead's #1089 review: main's block comes from main, never from the
/// change's own working tree).
fn block_on_default_branch(
    root: &Path,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    digest: &str,
    base: &str,
) -> Result<bool, String> {
    let default = remote_default_ref(root).ok_or_else(|| {
        "no default branch to read main's text from (origin/HEAD, origin/main or origin/master)"
            .to_string()
    })?;
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    let file = project_relative(root, path)
        .ok_or_else(|| "the requirement's file is outside the project".to_string())?;
    let limit = MOVED_BASE_HISTORY_LIMIT.to_string();
    let commits = git_output_allow_empty(
        root,
        &[
            "log",
            "--format=%H",
            "-n",
            limit.as_str(),
            "--end-of-options",
            default.as_str(),
            "--",
            file.as_str(),
        ],
    )
    .ok_or_else(|| format!("cannot read the history of {file} on {default}"))?;
    // Newest first: main's block counts only if the branch held it more recently than the
    // approval-time base. Meeting the base first means the block is older text main replaced, and
    // taking it would revert main's edit inside the delta.
    for commit in commits.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let object = format!("{commit}:{prefix}{file}");
        let text = git_output_allow_empty(root, &["show", object.as_str()]).unwrap_or_default();
        let held = canonical_block_digest(module, "", &text, item);
        if held == digest {
            return Ok(true);
        }
        if held == base {
            return Ok(false);
        }
    }
    Ok(false)
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
    let (approval, approval_index, claims) = {
        let effective = effective_definition_approval(root, record, &ledger)?;
        let index = approval_position(&ledger, effective)
            .ok_or_else(|| "the effective definition approval is not in the ledger".to_string())?;
        let claims = effective_delta_claims(root, record, &ledger, effective)?;
        (effective.clone(), index, claims)
    };
    let (Some(mut digests), Some(mut bases)) = claims else {
        return Ok(0);
    };
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut prepared = Vec::new();
    let mut rebases = Vec::new();
    let mut left_stale = false;
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
        // Every replacing item whose canonical block moved is either rebased here or left stale.
        // One left stale means the base check refuses next, and a refusal writes nothing
        // (corvid-agent's review of #1089), so the rebases staged for the others are dropped.
        for item in items.iter().filter(|item| {
            item.operation != DeltaOperation::Added && !is_change_log_section(item)
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
            let staged = (item.operation == DeltaOperation::Modified
                && item.target == DeltaTarget::Requirement)
                .then(|| {
                    stage_rebase(
                        root,
                        module,
                        &requirements_path,
                        &requirements,
                        &delta,
                        &key,
                        &base,
                        (approval_index, &approval),
                    )
                })
                .flatten();
            let Some((rebase, after, main_base)) = staged else {
                left_stale = true;
                continue;
            };
            digests.insert(module.clone(), rebase.delta_after_digest.clone());
            bases.insert(key, main_base);
            rebases.push(rebase);
            delta = after;
            changed = true;
        }
        if changed {
            prepared.push((delta_path, delta));
        }
    }
    if left_stale {
        return Ok(0);
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

/// One rebase of the `## MODIFIED` requirement `key` in `delta`, staged in memory: the record, the
/// rebased delta, and the base the item binds after it. `None` when the item cannot be rebased:
/// main's block is not text the default branch held after the approval-time base, the base text is
/// gone from Git, or the edits overlap. Nothing is written here.
#[allow(clippy::too_many_arguments)]
fn stage_rebase(
    root: &Path,
    module: &str,
    requirements_path: &Path,
    requirements: &str,
    delta: &str,
    key: &str,
    base: &str,
    (approval_index, approval): (usize, &ApprovalRecord),
) -> Option<(DeltaRebaseRecord, String, String)> {
    let current = modified_requirement_item(module, delta, key)?;
    let range = markdown_block_range(requirements, &format!("### {}", current.key), 3)?;
    let main_block = trimmed_block(&requirements[range]);
    // Main's block comes from main, and from after the base: the working tree's block must be text
    // the default branch held more recently than the approval-time text, or the rebase would carry
    // a branch-side edit, or revert one of main's, inside the approved delta (fable-lead's #1089
    // review).
    let main_base = item_block_digest(module, &current, &main_block);
    if !block_on_default_branch(root, requirements_path, module, &current, &main_base, base)
        .unwrap_or(false)
    {
        return None;
    }
    let base_block = recover_approved_base_block(root, requirements_path, module, &current, base)?;
    let merged = merge_words(
        trimmed_block_body(&base_block),
        trimmed_block_body(&main_block),
        &trimmed_block(&current.content),
    )?;
    let after = rebased_delta_text(delta, &current, &merged).ok()?;
    let rebase = DeltaRebaseRecord {
        schema_version: 1,
        module: module.to_string(),
        item: key.to_string(),
        approval_index,
        approval_timestamp: approval.timestamp,
        approval_digest: approval.digest.clone(),
        actor: DELTA_REBASE_ACTOR.into(),
        timestamp: now(),
        base_block,
        main_block,
        delta_before: delta.to_string(),
        delta_after_digest: delta_body_digest(module, &after),
    };
    // Staged only if it recomputes, exactly as every later reader will recompute it.
    recompute_rebase(&rebase).ok()?;
    Some((rebase, after, main_base))
}
