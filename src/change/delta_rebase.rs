// Proposing a rebase of a stale MODIFIED requirement delta over main's edit (#1085,
// REQ-change-161; Leif's option 2, "propose, don't approve").
//
// A `## MODIFIED` body replaces a requirement's whole text, written against the block as it stood
// at approval. When main edits that requirement afterwards, applying the old body would delete
// main's edit, so #843 refuses the delta and a person rewrites the body onto main's text by hand,
// even when the two edits are nowhere near each other. Here `change check` merges the
// approval-time text, main's text and the approved body word by word; when the edits do not
// overlap it writes the merge into the delta as a proposal and refuses. Nothing else is written:
// the delta then differs from its approval, so nothing reaches the canonical requirement until a
// person approves the proposal with `change approve` (CHANGE-6.a). Every case the merge cannot
// settle is refused instead, naming its cause and the way on.

/// The longest side, in tokens, the word merge compares; a larger block keeps the refusal.
const MAX_REBASE_TOKENS: usize = 4_000;

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
/// when the two edits overlap, when either moves a word, or when they cannot be aligned
/// unambiguously.
///
/// The texts are aligned only on anchors: words that occur exactly once in each of the three,
/// found again inside each gap between anchors. A repeated word is never an anchor, so it can never
/// be matched to one occurrence on one side and another on the other, which is how an LCS
/// alignment once let two edits to the same words merge (Codex's review of #1089: base `c a b d`,
/// main `b b d`, approved `b d`). Between anchors, a stretch only one side changed takes that side;
/// a stretch both changed, the same way excepted, is an overlap, and so is a stretch both changed
/// with no anchor inside to split it.
///
/// Moves refuse by one rule, not one counterexample at a time (ss-lead's review of #1089): a word
/// both sides removed from one stretch the same way, that one side alone adds in another stretch,
/// is a move against the other side's edit, and refuses. That covers a move against a deletion
/// (Codex: base `a b c`, main `a c`, approved `b a c`) and both sides moving one word to different
/// places, where both remove it from where it stood (Codex: base `a b c d`, main `b a c d`,
/// approved `a c b d` once gave `b a c b d`). A move only one side makes is a deletion and an
/// insertion of that side alone, and merges like any other edit. A word only one side deleted is
/// that side's own edit, so the other side may use it elsewhere: the change drops one `the`, main
/// appends a sentence with `the`, and they merge. Refusing every anchor word out of order (R1)
/// was measured and rejected: it refused #360's REQ-cli-012, where main alone moved a clause.
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
    let mut merge = WordMerge::default();
    merge.ranges(&base, &current, &approved)?;
    if merge
        .one_side_added
        .iter()
        .any(|word| merge.jointly_removed.contains(word))
    {
        return None;
    }
    // A three-way merge neither loses nor invents words: each word's merged count lies between the
    // base's count plus every deletion of it both sides made and the base's count plus every
    // addition of it both sides made. Whatever the alignment did, a merge outside that range lost
    // or duplicated a word, and it cannot be proposed (Codex's fourth review of #1089: both sides
    // moving a repeated `alpha` once kept one where each side keeps two). A range, not the sum: an
    // edit both sides made the same way is counted once, as the merge keeps it, and that is how the
    // real stale requirements rebuilt from Chough's history rebase (measured, ss-lead's choice).
    if !keeps_every_word_count(&base, &current, &approved, &merge.merged) {
        return None;
    }
    Some(merge.merged.concat())
}

/// Whether every word (whitespace aside) occurs in `merged` no fewer times than `base` keeps after
/// the deletions of both sides, and no more than `base` holds with the additions of both sides.
fn keeps_every_word_count(
    base: &[&str],
    current: &[&str],
    approved: &[&str],
    merged: &[&str],
) -> bool {
    fn count<'a>(tokens: &[&'a str]) -> BTreeMap<&'a str, isize> {
        let mut counts = BTreeMap::new();
        for token in tokens {
            *counts.entry(token.trim()).or_insert(0) += 1;
        }
        counts
    }
    let (base, current, approved, merged) =
        (count(base), count(current), count(approved), count(merged));
    let words: BTreeSet<&str> = base
        .keys()
        .chain(current.keys())
        .chain(approved.keys())
        .chain(merged.keys())
        .copied()
        .collect();
    words.into_iter().all(|word| {
        let of = |counts: &BTreeMap<&str, isize>| counts.get(word).copied().unwrap_or(0);
        let held = of(&base);
        let (main_change, approved_change) = (of(&current) - held, of(&approved) - held);
        let kept = of(&merged);
        kept >= held + main_change.min(0) + approved_change.min(0)
            && kept <= held + main_change.max(0) + approved_change.max(0)
    })
}

/// A word merge in progress: the tokens merged so far, the words both sides removed from one
/// stretch the same way, and the words one side alone added.
#[derive(Default)]
struct WordMerge<'a> {
    merged: Vec<&'a str>,
    jointly_removed: BTreeSet<&'a str>,
    one_side_added: BTreeSet<&'a str>,
}

impl<'a> WordMerge<'a> {
    fn ranges(
        &mut self,
        base: &[&'a str],
        current: &[&'a str],
        approved: &[&'a str],
    ) -> Option<()> {
        if approved == current {
            self.jointly_removed.extend(surplus_words(base, current));
            self.merged.extend_from_slice(current);
            return Some(());
        }
        if current == base {
            self.one_side_added.extend(surplus_words(approved, base));
            self.merged.extend_from_slice(approved);
            return Some(());
        }
        if approved == base {
            self.one_side_added.extend(surplus_words(current, base));
            self.merged.extend_from_slice(current);
            return Some(());
        }
        let anchors = unique_anchors(base, current, approved);
        if anchors.is_empty() {
            return None;
        }
        let (mut b, mut c, mut a) = (0usize, 0usize, 0usize);
        for (base_at, current_at, approved_at) in anchors {
            self.ranges(
                &base[b..base_at],
                &current[c..current_at],
                &approved[a..approved_at],
            )?;
            self.merged.push(base[base_at]);
            b = base_at + 1;
            c = current_at + 1;
            a = approved_at + 1;
        }
        self.ranges(&base[b..], &current[c..], &approved[a..])
    }
}

/// The words (whitespace aside) that occur more often in `more` than in `less`.
fn surplus_words<'a>(more: &[&'a str], less: &[&'a str]) -> Vec<&'a str> {
    let mut counts: BTreeMap<&'a str, isize> = BTreeMap::new();
    for token in more {
        *counts.entry(token.trim()).or_insert(0) += 1;
    }
    for token in less {
        *counts.entry(token.trim()).or_insert(0) -= 1;
    }
    counts
        .into_iter()
        .filter(|(_, surplus)| *surplus > 0)
        .map(|(word, _)| word)
        .collect()
}

/// The words that occur exactly once in each of `base`, `current` and `approved`, as index triples,
/// keeping the longest chain whose indices rise in all three. A word left out of the chain is one a
/// side moved; the gaps then hold its deletion and its insertion as that side's edits.
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

/// Main's side of one requirement, read from Git and never from the working tree (REQ-change-161):
/// the merge-base of HEAD and the default branch, which is on that branch's first-parent line, the
/// requirement's block there, and the approval-time (or last rebase's) block found earlier on
/// that line.
struct MainSide {
    /// Main deleted the requirement: its block is absent at the merge-base, after a text that
    /// held the base (Codex's fourth review of #1089). Nothing can be merged into an absent block.
    deleted: bool,
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

/// The heading line of a trimmed block.
fn block_heading(block: &str) -> &str {
    block.lines().next().unwrap_or("").trim_end()
}

/// Main's side of `item` when main has edited it past `base`: its block at HEAD's merge-base with
/// the default branch differs from `base` and from the delta's body, and `base` is a text the
/// default branch's first-parent line held before it. `Ok(None)` when main has no such edit, or
/// there is no default branch; `Err` when there is one and Git cannot read what this needs.
fn main_side(
    root: &Path,
    path: &Path,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> Result<Option<MainSide>, String> {
    // No default branch (a local-only project): there is no main edit to merge, exactly as before
    // #1085. With one, a merge-base, tree, text or history Git cannot read is an `Err`, never "main
    // did not edit it" (Codex's third review of #1089, as ss-lead narrowed it).
    if remote_default_ref(root).is_none() {
        return Ok(None);
    }
    let (_, commit) = main_merge_base(root)?;
    let prefix = git_repo_prefix(root).map_err(|error| format!("cannot read Git: {error}"))?;
    let file = project_relative(root, path)
        .ok_or_else(|| "the requirement's file is outside the project".to_string())?;
    let block = block_at(root, &prefix, &file, &commit, item)?;
    if block
        .as_deref()
        .is_some_and(|block| item_block_digest(module, item, block) == base)
    {
        return Ok(None);
    }
    let Some(base_block) =
        base_on_line_before(root, &prefix, &file, &commit, module, item, base)?
    else {
        return Ok(None);
    };
    let Some(block) = block else {
        // Main held the base and then deleted the requirement: an edit of main's that keeping
        // the change's body would undo (Codex's fourth review of #1089).
        return Ok(Some(MainSide {
            deleted: true,
            commit,
            block: String::new(),
            base_block,
        }));
    };
    // A block equal to the change's own text, heading and body, is not main's edit: after a change
    // is merged before it is finalized (#654), HEAD is the default branch and the merge-base holds
    // the change's body under the base's heading (ss-lead's review of #1089).
    if block_heading(&block) == block_heading(&base_block)
        && trimmed_block_body(&block) == trimmed_block(&item.content)
    {
        return Ok(None);
    }
    Ok(Some(MainSide {
        deleted: false,
        commit,
        block,
        base_block,
    }))
}

/// What the check makes of one `## MODIFIED` requirement item, given main's side of it.
enum MainEdit {
    /// Main has not edited it past its base, there is no default branch, or the check runs in CI.
    Unedited,
    /// Main has edited it and the edits merge: the merged body, and main's commit.
    Proposal { body: String, commit: String },
    /// Main has edited it and no proposal can be made: the cause and the way on, as a parenthetical
    /// after the requirement's id.
    Refused(String),
    /// A default branch exists and Git cannot read main's text: why.
    Unreadable(String),
}

/// What main's edit of `item` means for a workflow-v2 change outside CI (REQ-change-161): main's
/// side is read from Git, and the item's working-tree block is read from `requirements`, the text
/// of the canonical file at `path`. Each refusal names its cause and the way on (Leif's rule 3).
#[allow(clippy::too_many_arguments)]
fn main_edit(
    root: &Path,
    record: &ChangeRecord,
    path: &Path,
    requirements: &str,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> MainEdit {
    if record.workflow_version < 2
        || is_ci_project(root)
        || item.operation != DeltaOperation::Modified
        || item.target != DeltaTarget::Requirement
    {
        return MainEdit::Unedited;
    }
    let main = match main_side(root, path, module, item, base) {
        Ok(Some(main)) => main,
        Ok(None) => return MainEdit::Unedited,
        Err(why) => return MainEdit::Unreadable(why),
    };
    let at = short_commit(&main.commit);
    let shown = project_relative(root, path)
        .map(|file| format!(" (`git show {at}:{file}` prints it)"))
        .unwrap_or_default();
    let by_hand = format!("rewrite the body onto main's text by hand{shown}, then approve it");
    if main.deleted {
        return MainEdit::Refused(format!(
            " (main deleted this requirement at {at}; rewrite the delta onto main's text by hand: drop the item from the delta, or ask for the requirement back on main, then approve)"
        ));
    }
    // The delta carries a body, never a heading, so a heading main edited cannot be carried into a
    // proposal; it is refused rather than lost (Codex's fourth review of #1089).
    if block_heading(&main.block) != block_heading(&main.base_block) {
        return MainEdit::Refused(format!(
            " (main edited this requirement's heading at {at}; a delta carries a body, not a heading, so {by_hand})"
        ));
    }
    let base_body = trimmed_block_body(&main.base_block);
    let main_body = trimmed_block_body(&main.block);
    let approved = trimmed_block(&item.content);
    if [base_body, main_body, approved.as_str()]
        .iter()
        .any(|text| word_tokens(text).len() > MAX_REBASE_TOKENS)
    {
        return MainEdit::Refused(format!(
            " (main edited this requirement at {at}, and it is longer than the merge reads (4,000 words a side); {by_hand})"
        ));
    }
    let Some(merged) = merge_words(base_body, main_body, &approved) else {
        return MainEdit::Refused(format!(
            " (main's edit at {at} overlaps this change's words; {by_hand})"
        ));
    };
    // The working tree's block must be main's block, the delta's body (the change applied, or a
    // conflict resolved to its side) or already their merge (Git's own clean merge), headings
    // compared too. Any other text, such as a branch-side sentence or an older text of main's, is
    // refused (fable-lead's and ss-lead's reviews of #1089).
    let working = markdown_block_range(requirements, &format!("### {}", item.key), 3)
        .map(|range| trimmed_block(&requirements[range]))
        .filter(|block| block_heading(block) == block_heading(&main.base_block));
    match working.as_deref().map(trimmed_block_body) {
        Some(body) if body == main_body || body == approved || body == merged => {
            MainEdit::Proposal {
                body: merged,
                commit: main.commit,
            }
        }
        _ => MainEdit::Refused(format!(
            " (main edited this requirement at {at}, and the working tree's block is none of main's text, this change's body or their merge; set it to main's text{shown} and check again, or {by_hand})"
        )),
    }
}

/// Why the base check refuses `item` on main's side, as a parenthetical after its id
/// (REQ-change-161): main has edited it past `base`, which is stale even when the working tree
/// carries the change's own body, since keeping that body drops main's edit. `None` when main has
/// not, as before, including with no default branch and in CI.
#[allow(clippy::too_many_arguments)]
fn main_edit_refusal(
    root: &Path,
    record: &ChangeRecord,
    path: &Path,
    requirements: &str,
    module: &str,
    item: &DeltaItem,
    base: &str,
) -> Option<String> {
    match main_edit(root, record, path, requirements, module, item, base) {
        MainEdit::Unedited => None,
        // Reached only when another item of the change could not be proposed: none is then.
        MainEdit::Proposal { commit, .. } => Some(format!(
            " (main edited this requirement at {}; the check proposes a rebase of it once every other listed item can be proposed)",
            short_commit(&commit)
        )),
        MainEdit::Refused(cause) => Some(cause),
        MainEdit::Unreadable(why) => Some(format!(
            " (main's text cannot be read from Git: {why}; repair the repository or fetch the default branch, then check again)"
        )),
    }
}

/// Propose a rebase of every `## MODIFIED` requirement item of `record` that main has edited
/// since its approved base, and refuse (REQ-change-161). Main's block is read from Git at HEAD's
/// merge-base with the default branch. Each proposal writes the merged body into the item in the
/// delta, and nothing else: the canonical requirement and `approvals.json` are untouched, and the
/// delta no longer matches its approval, so the next check refuses it as changed after approval
/// until a person approves it with `change approve`.
///
/// When any replacing item of the change is stale and cannot be proposed, nothing is written and
/// the base check that runs next refuses, naming each item's cause. When a default branch exists
/// and Git cannot read main's text, the refusal is made here instead, naming its cause and its way
/// on, because the base check's "rewrite and re-approve" would mislead for it (the tester's run at
/// 0d6ca1cd).
///
/// Workflow-v1 changes are left out: their definition digest hashes the delta bodies. CI never
/// writes, so it never proposes.
fn propose_stale_modified_rebases(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if record.no_spec_change || record.workflow_version < 2 || is_ci_project(root) {
        return Ok(());
    }
    let ledger = load_approvals(root, record)?;
    let approval = effective_definition_approval(root, record, &ledger)?;
    let (Some(digests), Some(bases)) = (
        approval.approved_delta_digests.as_ref(),
        approval.approved_delta_bases.as_ref(),
    ) else {
        return Ok(());
    };
    let specs_dir = crate::config::load_config(root).specs_dir;
    let id = record.id.as_str();
    let mut prepared = Vec::new();
    let mut proposed: Vec<String> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut held_back = false;
    for module in &record.affected_specs {
        let delta_path = delta_path_checked(root, record, module)?;
        let mut delta = read_bounded_change_text(&delta_path, "semantic delta")
            .map_err(|error| format!("semantic delta for {module}: {error}"))?;
        if digests.get(module) != Some(&delta_body_digest(module, &delta)) {
            // Not the approved wording; the delta-body check refuses it, and a hand edit is never
            // overwritten by a proposal.
            continue;
        }
        let (spec_path, requirements_path) = canonical_module_paths(root, &specs_dir, module)?;
        let spec = read_canonical_or_absent(root, &spec_path)?;
        let requirements = read_canonical_or_absent(root, &requirements_path)?;
        let items = parse_delta(&delta)?;
        let mut changed = false;
        for item in items.iter().filter(|item| {
            item.operation != DeltaOperation::Added && !is_change_log_section(item)
        }) {
            let Some(base) = bases.get(&delta_base_key(module, item)) else {
                continue;
            };
            let stale = !delta_item_is_applied(&spec, &requirements, item)
                && canonical_block_digest(module, &spec, &requirements, item) != *base;
            match main_edit(root, record, &requirements_path, &requirements, module, item, base) {
                MainEdit::Unedited => held_back |= stale,
                MainEdit::Proposal { body, commit } => {
                    delta = rebased_delta_text(&delta, item, &body)?;
                    changed = true;
                    proposed.push(format!(
                        "{module}: {} (main's text at {})",
                        item.key,
                        short_commit(&commit)
                    ));
                }
                MainEdit::Refused(_) => held_back = true,
                MainEdit::Unreadable(why) => unreadable.push(format!(
                    "{module}: {}: main's text cannot be read from Git ({why}); repair the repository or fetch the default branch (`git fetch origin`), then run `specsync change check {}` again",
                    item.key,
                    shell_word(id)
                )),
            }
        }
        if changed {
            prepared.push((delta_path, delta));
        }
    }
    if !unreadable.is_empty() {
        return Err(format!(
            "semantic delta for {} could not be proposed over main's edit, and nothing was written: {}",
            shell_word(id),
            unreadable.join("; ")
        ));
    }
    if held_back || proposed.is_empty() {
        return Ok(());
    }
    write_prepared_files(root, &prepared)?;
    Err(format!(
        "semantic delta for {} now holds a proposed rebase over main's edit, and nothing else was written: {}. A proposal is not approved wording: read it in the delta, approve it with `specsync change approve {}`, then run `specsync change check {}`",
        shell_word(id),
        proposed.join("; "),
        shell_word(id),
        shell_word(id)
    ))
}
