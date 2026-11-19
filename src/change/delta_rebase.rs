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
// block, the commit main's block was read at, and the delta before the rebase), and every reader
// recomputes the merge and its digests from them. A record that does not recompute is refused, so a rebase can only ever carry exactly
// the merge of approved wording and main's wording, never wording nobody approved (CHANGE-6.a).

/// The longest side, in tokens, the word merge compares; a larger block keeps the refusal.
const MAX_REBASE_TOKENS: usize = 4_000;

/// The largest module delta a rebase copies into its record. Each record holds the whole delta
/// before it, so an unbounded delta times many records could make `approvals.json` and every read
/// of it huge (Codex's review of #1089); past the bound the item keeps today's refusal.
const MAX_REBASED_DELTA_BYTES: usize = 256 * 1024;

/// The most rebase records one change's `approvals.json` holds; past it the item keeps today's
/// refusal and needs a fresh approval, which starts a new set.
const MAX_DELTA_REBASES: usize = 64;

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
    /// The commit `main_block` was read at: the merge-base of HEAD and the default branch, on that
    /// branch's first-parent line. Empty in a record without one, which every reader refuses.
    #[serde(default)]
    pub main_commit: String,
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
    let guard = MoveGuard {
        main_deleted: deleted_words(&base, &current),
        approved_deleted: deleted_words(&base, &approved),
    };
    let mut merged = Vec::new();
    merge_token_ranges(&base, &current, &approved, &mut merged, &guard)?;
    Some(merged.concat())
}

/// The words each side deleted from the base, by count, so a merge never puts back, through the
/// other side's insertion, a word one side removed. Codex's review of #1089: base `a b c`, main
/// `a c`, approved `b a c` anchors `a` and `c` and would return `b a c`, restoring the `b` main
/// deleted because the approved body moved it.
struct MoveGuard<'a> {
    main_deleted: BTreeSet<&'a str>,
    approved_deleted: BTreeSet<&'a str>,
}

/// Words (whitespace aside) that occur fewer times in `side` than in `base`.
fn deleted_words<'a>(base: &[&'a str], side: &[&'a str]) -> BTreeSet<&'a str> {
    let count = |tokens: &[&'a str]| {
        let mut counts: BTreeMap<&'a str, usize> = BTreeMap::new();
        for token in tokens {
            *counts.entry(token.trim()).or_insert(0) += 1;
        }
        counts
    };
    let in_side = count(side);
    count(base)
        .into_iter()
        .filter(|(word, times)| in_side.get(word).copied().unwrap_or(0) < *times)
        .map(|(word, _)| word)
        .collect()
}

/// Whether `taken` inserts, relative to `base_chunk`, a word in `deleted`.
fn reinserts(taken: &[&str], base_chunk: &[&str], deleted: &BTreeSet<&str>) -> bool {
    let mut surplus: BTreeMap<&str, isize> = BTreeMap::new();
    for token in taken {
        *surplus.entry(token.trim()).or_insert(0) += 1;
    }
    for token in base_chunk {
        *surplus.entry(token.trim()).or_insert(0) -= 1;
    }
    surplus
        .into_iter()
        .any(|(word, extra)| extra > 0 && deleted.contains(word))
}

fn merge_token_ranges<'a>(
    base: &[&'a str],
    current: &[&'a str],
    approved: &[&'a str],
    merged: &mut Vec<&'a str>,
    guard: &MoveGuard<'_>,
) -> Option<()> {
    if approved == current {
        merged.extend_from_slice(current);
        return Some(());
    }
    if current == base {
        if reinserts(approved, base, &guard.main_deleted) {
            return None;
        }
        merged.extend_from_slice(approved);
        return Some(());
    }
    if approved == base {
        if reinserts(current, base, &guard.approved_deleted) {
            return None;
        }
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
            guard,
        )?;
        merged.push(base[base_at]);
        b = base_at + 1;
        c = current_at + 1;
        a = approved_at + 1;
    }
    merge_token_ranges(&base[b..], &current[c..], &approved[a..], merged, guard)
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
/// For a change still being delivered, each record's main commit must also be on the default
/// branch's first-parent line and in HEAD's history, and hold the record's main block after its
/// base block (`check_rebase_main_commit`; fable-lead's and ss-lead's #1089 reviews). A block only
/// the change's own branch holds is wording nobody approved, so it is refused, and so is a record
/// whose default branch cannot be read. An accepted or archived change is not re-judged against a
/// default branch that has since moved on; its acceptance bound the canonical text.
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
            check_rebase_main_commit(root, &requirements_path, &rebase.module, &item, rebase)
                .map_err(|why| refuse(&why))?;
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

/// Why a change's recorded rebases are refused, for `change status` (REQ-change-161): the same
/// refusal `change check` prints, naming the record. `None` when every record recomputes, when
/// there are none, and when the ledger or the effective approval cannot be read (status reports
/// those on their own).
pub fn refused_delta_rebase(root: &Path, record: &ChangeRecord) -> Option<String> {
    let ledger = load_approvals(root, record).ok()?;
    let approval = effective_definition_approval(root, record, &ledger).ok()?;
    effective_delta_claims(root, record, &ledger, approval).err()
}

/// Main's side of one requirement, read from Git and never from the working tree (REQ-change-161):
/// the merge-base of HEAD and the default branch, which is on that branch's first-parent line, the
/// requirement's block there, and the approval-time (or last rebase's) block found earlier on
/// that line.
struct MainSide {
    commit: String,
    block: String,
    base_block: String,
}

/// The default branch (`origin/HEAD`, else `origin/main` or `origin/master`) and the merge-base of
/// HEAD with it. `Err` when there is no default branch, Git cannot answer, or the merge-base is not
/// on the default branch's first-parent line: a side branch's commit was never main's text
/// (ss-lead's review of #1089).
fn main_merge_base(root: &Path) -> Result<(String, String), String> {
    let default = remote_default_ref(root).ok_or_else(|| {
        "no default branch to read main's text from (origin/HEAD, origin/main or origin/master)"
            .to_string()
    })?;
    let commit = git_output_allow_empty(root, &["merge-base", "HEAD", default.as_str()])
        .map(|text| text.trim().to_string())
        .filter(|commit| is_canonical_commit_id(commit))
        .ok_or_else(|| format!("cannot find where HEAD meets {default}"))?;
    if !on_first_parent_line(root, &default, &commit) {
        return Err(format!(
            "the merge-base of HEAD and {default} is not on the default branch's first-parent line"
        ));
    }
    Ok((default, commit))
}

/// Whether `commit` is on `default`'s first-parent line: walking first parents from `default`,
/// the first commit `commit` can reach is `commit` itself.
fn on_first_parent_line(root: &Path, default: &str, commit: &str) -> bool {
    let range = format!("{commit}..{default}");
    let Some(count) = git_output_allow_empty(
        root,
        &["rev-list", "--first-parent", "--count", "--end-of-options", range.as_str()],
    ) else {
        return false;
    };
    let at = format!("{default}~{}^{{commit}}", count.trim());
    git_output_allow_empty(root, &["rev-parse", "--verify", "--end-of-options", at.as_str()])
        .is_some_and(|resolved| resolved.trim() == commit)
}

/// Whether `commit` is in HEAD's history.
fn is_ancestor_of_head(root: &Path, commit: &str) -> bool {
    git_output_allow_empty(
        root,
        &["merge-base", "--is-ancestor", "--end-of-options", commit, "HEAD"],
    )
    .is_some()
}

/// `item`'s trimmed block in `file` at `commit`. `Ok(None)` when the commit's tree does not list
/// the file or the file has no such block; `Err` when the tree lists the file and its text cannot
/// be read, which refuses, never skips (ss-lead's review of #1089).
fn block_at(
    root: &Path,
    prefix: &str,
    file: &str,
    commit: &str,
    item: &DeltaItem,
) -> Result<Option<String>, String> {
    let listed = git_output_allow_empty(
        root,
        &["ls-tree", "--name-only", "--end-of-options", commit, "--", file],
    )
    .ok_or_else(|| format!("cannot read the tree of {commit}"))?;
    if listed.trim().is_empty() {
        return Ok(None);
    }
    let object = format!("{commit}:{prefix}{file}");
    let text = git_output_allow_empty(root, &["show", object.as_str()])
        .ok_or_else(|| format!("cannot read {file} at {commit}"))?;
    Ok(markdown_block_range(&text, &format!("### {}", item.key), 3)
        .map(|range| trimmed_block(&text[range])))
}

/// The block with digest `base`, found on the first-parent line at or before `commit`, in the
/// newest `MOVED_BASE_HISTORY_LIMIT` commits that touched `file`. `None` when the line never held
/// it, so the text at `commit` is not main's edit of the base: the base is the change's own text,
/// or an older text main replaced.
fn base_on_line_before(
    root: &Path,
    prefix: &str,
    file: &str,
    commit: &str,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> Result<Option<String>, String> {
    let limit = MOVED_BASE_HISTORY_LIMIT.to_string();
    let commits = git_output_allow_empty(
        root,
        &[
            "log",
            "--first-parent",
            "--format=%H",
            "-n",
            limit.as_str(),
            "--end-of-options",
            commit,
            "--",
            file,
        ],
    )
    .ok_or_else(|| format!("cannot read the history of {file} at {commit}"))?;
    for held in commits.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(block) = block_at(root, prefix, file, held, item)?
            && item_block_digest(module, item, &block) == base
        {
            return Ok(Some(block));
        }
    }
    Ok(None)
}

/// Main's side of `item` when main has edited it past `base`: its block at HEAD's merge-base with
/// the default branch differs from `base` and from the delta's body, and `base` is a text the
/// default branch's first-parent line held before it. `Ok(None)` when main has no such edit;
/// `Err` when Git cannot say.
fn main_side(
    root: &Path,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> Result<Option<MainSide>, String> {
    let (_, commit) = main_merge_base(root)?;
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    let file = project_relative(root, path)
        .ok_or_else(|| "the requirement's file is outside the project".to_string())?;
    let Some(block) = block_at(root, &prefix, &file, &commit, item)? else {
        return Ok(None);
    };
    // A block equal to the delta's body is the change's own text, not main's edit: after a change
    // is merged before it is finalized (#654), HEAD is the default branch and the merge-base holds
    // the change's body (ss-lead's review of #1089).
    if item_block_digest(module, item, &block) == base
        || trimmed_block_body(&block) == trimmed_block(&item.content)
    {
        return Ok(None);
    }
    let Some(base_block) =
        base_on_line_before(root, &prefix, &file, &commit, module, item, base)?
    else {
        return Ok(None);
    };
    Ok(Some(MainSide {
        commit,
        block,
        base_block,
    }))
}

/// Whether main has edited `item` past the base `base` and the change has not been rebased over
/// it, for the base check (REQ-change-161): a workflow-v2 `## MODIFIED` requirement outside CI
/// whose block at HEAD's merge-base with the default branch is newer text than its base. Such an
/// item is stale even when the working tree carries the change's own body, since keeping that body
/// drops main's edit. `false` whenever Git cannot say, so the item keeps today's check.
fn main_moved_past_base(
    root: &Path,
    record: &ChangeRecord,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> bool {
    record.workflow_version >= 2
        && !is_ci_project(root)
        && item.operation == DeltaOperation::Modified
        && item.target == DeltaTarget::Requirement
        && matches!(main_side(root, path, module, item, base), Ok(Some(_)))
}

/// What a reader checks of a record's main commit while the change is delivered: it names a
/// commit on the default branch's first-parent line, in HEAD's history, that holds the record's
/// main block, after a text with the record's base block on that line.
fn check_rebase_main_commit(
    root: &Path,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    rebase: &DeltaRebaseRecord,
) -> Result<(), String> {
    let commit = rebase.main_commit.as_str();
    if !is_canonical_commit_id(commit) {
        return Err("it names no main commit".into());
    }
    let default = remote_default_ref(root).ok_or_else(|| {
        "no default branch to read main's text from (origin/HEAD, origin/main or origin/master)"
            .to_string()
    })?;
    if !on_first_parent_line(root, &default, commit) {
        return Err(format!(
            "its main commit {commit} is not on the default branch's first-parent line ({default})"
        ));
    }
    if !is_ancestor_of_head(root, commit) {
        return Err(format!(
            "its main commit {commit} is not in this branch's history"
        ));
    }
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    let file = project_relative(root, path)
        .ok_or_else(|| "the requirement's file is outside the project".to_string())?;
    let held = block_at(root, &prefix, &file, commit, item)?;
    if held.as_deref().map(|block| item_block_digest(module, item, block))
        != Some(item_block_digest(module, item, &rebase.main_block))
    {
        return Err(format!(
            "its main commit {commit} on the default branch does not hold its main block"
        ));
    }
    let base = item_block_digest(module, item, &rebase.base_block);
    if base_on_line_before(root, &prefix, &file, commit, module, item, &base)?.is_none() {
        return Err(format!(
            "its base block is not a text the default branch held before its main commit {commit}"
        ));
    }
    Ok(())
}

/// Rebase every `## MODIFIED` requirement item of `record` whose block main has edited since its
/// approved base, when main's edit and the change's edit do not overlap (REQ-change-161). Main's
/// block is read from Git at HEAD's merge-base with the default branch. Each rebase rewrites the
/// item's body in the delta, and in the canonical requirement when the working tree carries the
/// change's body, and appends a record to `approvals.json`, all in one prepared write. An item
/// that cannot be rebased is left alone, so the base check that runs next refuses it, and then no
/// item is rebased. Returns the number of rebases written.
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
        let mut requirements = read_canonical_or_absent(root, &requirements_path)?;
        let items = parse_delta(&delta)?;
        let (mut delta_changed, mut requirements_changed) = (false, false);
        // Every replacing item that is stale, or that main has edited past its base, is either
        // rebased here or left stale. One left stale means the base check refuses next, and a
        // refusal writes nothing (corvid-agent's review of #1089), so the rebases staged for the
        // others are dropped.
        for item in items.iter().filter(|item| {
            item.operation != DeltaOperation::Added && !is_change_log_section(item)
        }) {
            let key = delta_base_key(module, item);
            let Some(base) = bases.get(&key).cloned() else {
                continue;
            };
            let stale = !delta_item_is_applied(&spec, &requirements, item)
                && canonical_block_digest(module, &spec, &requirements, item) != base;
            let main = (item.operation == DeltaOperation::Modified
                && item.target == DeltaTarget::Requirement)
                .then(|| main_side(root, &requirements_path, module, item, &base).ok().flatten())
                .flatten();
            let Some(main) = main else {
                left_stale |= stale;
                continue;
            };
            let staged = stage_rebase(
                module,
                &requirements,
                &delta,
                &key,
                &main,
                (approval_index, &approval),
            );
            let Some((rebase, after, main_base, requirements_after)) = staged else {
                left_stale = true;
                continue;
            };
            digests.insert(module.clone(), rebase.delta_after_digest.clone());
            bases.insert(key, main_base);
            rebases.push(rebase);
            delta = after;
            delta_changed = true;
            if let Some(text) = requirements_after {
                requirements = text;
                requirements_changed = true;
            }
        }
        if delta_changed {
            prepared.push((delta_path, delta));
        }
        if requirements_changed {
            prepared.push((requirements_path, requirements));
        }
    }
    if left_stale || ledger.delta_rebases.len() + rebases.len() > MAX_DELTA_REBASES {
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

/// One rebase of the `## MODIFIED` requirement `key` in `delta` over `main`, staged in memory: the
/// record, the rebased delta, the base the item binds after it, and the canonical requirements
/// when they must change too. `None` when the item cannot be rebased: the edits overlap, the delta
/// is over the size bound, or the working tree's block is not one of the three texts a rebase
/// accepts (main's block, the delta's body, or already their merge). Nothing is written here.
fn stage_rebase(
    module: &str,
    requirements: &str,
    delta: &str,
    key: &str,
    main: &MainSide,
    (approval_index, approval): (usize, &ApprovalRecord),
) -> Option<(DeltaRebaseRecord, String, String, Option<String>)> {
    if delta.len() > MAX_REBASED_DELTA_BYTES {
        return None;
    }
    let current = modified_requirement_item(module, delta, key)?;
    let approved = trimmed_block(&current.content);
    let main_body = trimmed_block_body(&main.block);
    let merged = merge_words(trimmed_block_body(&main.base_block), main_body, &approved)?;
    // The working tree's block must be main's block, the delta's body (the change applied, or a
    // conflict resolved to its side) or already their merge (Git's own clean merge). Any other
    // text, such as a branch-side sentence or an older text of main's, keeps the refusal
    // (fable-lead's and ss-lead's reviews of #1089).
    let range = markdown_block_range(requirements, &format!("### {}", current.key), 3)?;
    let working_block = trimmed_block(&requirements[range]);
    let working = trimmed_block_body(&working_block);
    let requirements_after = if working == main_body || working == merged {
        None
    } else if working == approved {
        Some(
            apply_markdown_block(
                requirements,
                "### ",
                &current.key,
                &merged,
                DeltaOperation::Modified,
            )
            .ok()?,
        )
    } else {
        return None;
    };
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
        base_block: main.base_block.clone(),
        main_block: main.block.clone(),
        main_commit: main.commit.clone(),
        delta_before: delta.to_string(),
        delta_after_digest: delta_body_digest(module, &after),
    };
    // Staged only if it recomputes, exactly as every later reader will recompute it.
    recompute_rebase(&rebase).ok()?;
    let main_base = item_block_digest(module, &current, &main.block);
    Some((rebase, after, main_base, requirements_after))
}
