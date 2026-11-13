//! Ancestor-aware merge for one module contract.
//!
//! Git invokes this as a merge driver (`specsync merge-file %O %A %B %P`) when
//! `.gitattributes` marks `*.spec.md` and `requirements.md`. The marker-based
//! resolver in `merge.rs` is a different path: it still takes `max(version)` and
//! still refuses prose, because a conflict hunk does not carry the ancestor.
//! This driver does, so two branches that each bumped the same module can land
//! together.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Output;

use crate::git_utils::git_cmd;

/// Result of merging one contract file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeContract {
    /// Merged bytes. Conflict markers remain here when `clean` is false.
    pub content: String,
    /// True only when every region merged and the content has no conflict markers.
    pub clean: bool,
}

/// Merge `ancestor`, `ours`, and `theirs` for `path` (`%P`).
///
/// `path` selects the contract kind. A module spec unions `files`, `db_tables`,
/// and `depends_on`, counts every numeric version bump from the ancestor, and
/// unions change-log rows. A requirements file unions blocks by ID. The same
/// new ID with two bodies is kept twice so `check` can refuse it. An existing
/// block edited differently on both sides stays a conflict. Any other prose
/// that both sides changed differently stays a conflict.
pub fn merge_contract(ancestor: &str, ours: &str, theirs: &str, path: &str) -> MergeContract {
    if ours == theirs || theirs == ancestor {
        return MergeContract {
            content: ours.to_string(),
            clean: true,
        };
    }
    if ours == ancestor {
        return MergeContract {
            content: theirs.to_string(),
            clean: true,
        };
    }

    let crlf = ours.contains("\r\n");
    let ancestor = normalize_newlines(ancestor);
    let ours_n = normalize_newlines(ours);
    let theirs_n = normalize_newlines(theirs);
    let merged = match contract_kind(path) {
        ContractKind::Requirements => merge_requirements(&ancestor, &ours_n, &theirs_n),
        ContractKind::Spec => merge_spec(&ancestor, &ours_n, &theirs_n),
        ContractKind::Text => merge_text_region(&ancestor, &ours_n, &theirs_n),
    };
    let content = if crlf {
        merged.content.replace('\n', "\r\n")
    } else {
        merged.content
    };
    let clean = merged.clean && !content.contains("<<<<<<<");
    MergeContract { content, clean }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContractKind {
    Spec,
    Requirements,
    Text,
}

fn contract_kind(path: &str) -> ContractKind {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    if name == "requirements.md" || name.ends_with(".req.md") {
        ContractKind::Requirements
    } else if name.ends_with(".spec.md") || path.is_empty() {
        ContractKind::Spec
    } else {
        ContractKind::Text
    }
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn merge_spec(ancestor: &str, ours: &str, theirs: &str) -> MergeContract {
    let (Some((base_fm, base_body)), Some((ours_fm, ours_body)), Some((theirs_fm, theirs_body))) = (
        split_frontmatter(ancestor),
        split_frontmatter(ours),
        split_frontmatter(theirs),
    ) else {
        return merge_text_region(ancestor, ours, theirs);
    };

    let (base_pre, base_log, base_post) = split_changelog(&base_body);
    let (ours_pre, ours_log, ours_post) = split_changelog(&ours_body);
    let (theirs_pre, theirs_log, theirs_post) = split_changelog(&theirs_body);

    let log = merge_changelog(&base_log, &ours_log, &theirs_log);
    let pre = merge_text_region(&base_pre, &ours_pre, &theirs_pre);
    let post = merge_text_region(&base_post, &ours_post, &theirs_post);
    let fm = merge_frontmatter(&base_fm, &ours_fm, &theirs_fm, log.version);
    let content = format!(
        "{}{}{}{}",
        fm.content, pre.content, log.content, post.content
    );
    MergeContract {
        content,
        clean: fm.clean && pre.clean && log.clean && post.clean,
    }
}

fn merge_requirements(ancestor: &str, ours: &str, theirs: &str) -> MergeContract {
    let (
        Some((base_pre, base_blocks)),
        Some((ours_pre, ours_blocks)),
        Some((theirs_pre, theirs_blocks)),
    ) = (
        parse_requirement_doc(ancestor),
        parse_requirement_doc(ours),
        parse_requirement_doc(theirs),
    )
    else {
        return merge_text_region(ancestor, ours, theirs);
    };
    // A side that already holds one ID twice (an earlier kept-twice merge, or a
    // heading the parser could not tell from an example) cannot be merged by ID:
    // `find` would pick one body and drop the other.
    if has_duplicate_block_ids(&base_blocks)
        || has_duplicate_block_ids(&ours_blocks)
        || has_duplicate_block_ids(&theirs_blocks)
    {
        return merge_text_region(ancestor, ours, theirs);
    }

    let pre = merge_text_region(&base_pre, &ours_pre, &theirs_pre);
    let mut clean = pre.clean;
    let mut content = pre.content;

    let base_ids: BTreeSet<&str> = base_blocks.iter().map(|block| block.id.as_str()).collect();
    // #874: the ancestor's order is the default, not the answer. Compare each side's order over
    // the ids surviving on BOTH sides (t384's review of #1058: comparing full surviving sets let
    // a deletion mask a reorder). A side that reordered those while the other kept the ancestor's
    // order wins outright; two different reorders conflict rather than one silently prevailing.
    let ours_ids: BTreeSet<&str> = ours_blocks.iter().map(|block| block.id.as_str()).collect();
    let theirs_ids: BTreeSet<&str> = theirs_blocks
        .iter()
        .map(|block| block.id.as_str())
        .collect();
    let common: BTreeSet<&str> = base_ids
        .iter()
        .copied()
        .filter(|id| ours_ids.contains(id) && theirs_ids.contains(id))
        .collect();
    let common_order = |blocks: &[ReqBlock]| -> Vec<String> {
        blocks
            .iter()
            .filter(|block| common.contains(block.id.as_str()))
            .map(|block| block.id.clone())
            .collect()
    };
    let base_common = common_order(&base_blocks);
    let ours_common = common_order(&ours_blocks);
    let theirs_common = common_order(&theirs_blocks);
    let surviving_order = |blocks: &[ReqBlock]| -> Vec<String> {
        blocks
            .iter()
            .filter(|block| base_ids.contains(block.id.as_str()))
            .map(|block| block.id.clone())
            .collect()
    };
    let mut order: Vec<String> = if ours_common == theirs_common {
        surviving_order(&ours_blocks)
    } else if ours_common == base_common {
        surviving_order(&theirs_blocks)
    } else if theirs_common == base_common {
        surviving_order(&ours_blocks)
    } else {
        // Two different reorders: no per-block sequence can hold both, so hand the whole
        // document to the text merge, which writes <<<<<<< markers with each side's order
        // under its own label (the duplicate-ID guard above does the same).
        let merged = merge_text_region(ancestor, ours, theirs);
        // A clean text merge is not automatically a valid one here. When both sides move
        // the SAME block to different places, the line merge sees a shared deletion plus
        // two non-overlapping inserts and writes the block twice (t384 on #1058). Accept a
        // clean result only when re-parsing finds every surviving id exactly once;
        // otherwise write the conflict ourselves so no order prevails quietly.
        if merged.clean && reparse_holds_each_id_once(&merged.content, &ours_ids, &theirs_ids) {
            return merged;
        }
        if merged.clean {
            return MergeContract {
                content: conflict_block(ours, theirs),
                clean: false,
            };
        }
        return merged;
    };
    // The winning side's sequence may omit ids the loser kept (a winner-side deletion is
    // judged per-block below, never by absence from the sequence). Re-insert each missing
    // id after its nearest base predecessor so a delete/edit conflict keeps its document
    // position instead of landing at the end (t384's nit on #1058).
    for (base_index, block) in base_blocks.iter().enumerate() {
        let kept_by_either =
            ours_ids.contains(block.id.as_str()) || theirs_ids.contains(block.id.as_str());
        if !kept_by_either || order.iter().any(|id| id == &block.id) {
            continue;
        }
        let position = base_blocks[..base_index]
            .iter()
            .rev()
            .find_map(|predecessor| {
                order
                    .iter()
                    .position(|id| id == &predecessor.id)
                    .map(|position| position + 1)
            })
            .unwrap_or(0);
        order.insert(position, block.id.clone());
    }
    let mut added = Vec::new();
    for block in ours_blocks.iter().chain(theirs_blocks.iter()) {
        if !base_ids.contains(block.id.as_str()) && !added.iter().any(|id: &String| id == &block.id)
        {
            added.push(block.id.clone());
        }
    }
    added.sort();
    order.extend(added);

    for id in order {
        let base = base_blocks.iter().find(|block| block.id == id);
        let our = ours_blocks.iter().find(|block| block.id == id);
        let their = theirs_blocks.iter().find(|block| block.id == id);
        match (base, our, their) {
            (None, Some(our), None) => push_block(&mut content, &our.text),
            (None, None, Some(their)) => push_block(&mut content, &their.text),
            (None, Some(our), Some(their)) if our.text == their.text => {
                push_block(&mut content, &our.text);
            }
            // Both branches allocated this ID. Keep both bodies. `check` names the ID.
            (None, Some(our), Some(their)) => {
                push_block(&mut content, &our.text);
                push_block(&mut content, &their.text);
            }
            (Some(_), Some(our), Some(their)) if our.text == their.text => {
                push_block(&mut content, &our.text);
            }
            (Some(base), Some(our), Some(their)) if our.text == base.text => {
                push_block(&mut content, &their.text);
            }
            (Some(base), Some(our), Some(their)) if their.text == base.text => {
                push_block(&mut content, &our.text);
            }
            (Some(base), Some(our), Some(their)) => {
                let merged = merge_text_region(&base.text, &our.text, &their.text);
                clean = clean && merged.clean;
                push_block(&mut content, &merged.content);
            }
            (Some(_), None, None) => {}
            (Some(base), None, Some(their)) if their.text == base.text => {}
            (Some(base), Some(our), None) if our.text == base.text => {}
            // One side deleted the block and the other edited it. The deleting
            // side is shown empty, so choosing it does not restore the old text.
            (Some(_), None, Some(their)) => {
                clean = false;
                push_block(&mut content, &conflict_block("", &their.text));
            }
            (Some(_), Some(our), None) => {
                clean = false;
                push_block(&mut content, &conflict_block(&our.text, ""));
            }
            (None, None, None) => {}
        }
    }

    // The assembled file ends with whichever block sorts last. Keep the
    // current side's final-newline state, as every other region does.
    let content = restore_region_newline(ours, content);
    MergeContract { content, clean }
}

struct ReqBlock {
    id: String,
    text: String,
}

/// A clean whole-text merge of two divergent reorders is only valid when re-parsing it finds
/// every surviving block id exactly once. `git merge-file` resolves "both sides moved the same
/// block to different places" as a shared deletion plus two non-overlapping inserts — a clean
/// exit with the block written twice (t384 on #1058). The expected set is the union of what
/// each side kept: a block deleted on both sides is rightly absent, and nothing either side
/// kept may be missing.
fn reparse_holds_each_id_once(
    merged: &str,
    ours_ids: &BTreeSet<&str>,
    theirs_ids: &BTreeSet<&str>,
) -> bool {
    let Some((_, blocks)) = parse_requirement_doc(merged) else {
        return false;
    };
    let expected: BTreeSet<&str> = ours_ids.union(theirs_ids).copied().collect();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for block in &blocks {
        *counts.entry(block.id.as_str()).or_default() += 1;
    }
    counts.len() == expected.len()
        && counts.keys().all(|id| expected.contains(id))
        && counts.values().all(|count| *count == 1)
}

/// Appends `block`, starting it on its own line. A block whose file had no final
/// newline would otherwise run straight into the next `### REQ-` heading.
fn push_block(content: &mut String, block: &str) {
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(block);
}

fn has_duplicate_block_ids(blocks: &[ReqBlock]) -> bool {
    let mut seen = BTreeSet::new();
    blocks.iter().any(|block| !seen.insert(block.id.as_str()))
}

/// Marks each line that is a fence delimiter or sits inside a fenced code block.
///
/// CommonMark rules: at most three leading spaces, then three or more backticks
/// or tildes (a backtick fence's info string holds no backtick). The fence closes
/// on a run of the same character at least as long, followed only by spaces or
/// tabs. An unclosed fence runs to the end of the text.
fn fenced_lines(lines: &[&str]) -> Vec<bool> {
    let mut open: Option<(char, usize)> = None;
    lines
        .iter()
        .map(|line| {
            let bare = line.trim_end_matches(['\n', '\r']);
            let indent = bare.chars().take_while(|ch| *ch == ' ').count();
            let run = (indent <= 3).then(|| &bare[indent..]).and_then(|rest| {
                let marker = rest.chars().next().filter(|ch| matches!(ch, '`' | '~'))?;
                let length = rest.chars().take_while(|ch| *ch == marker).count();
                (length >= 3).then_some((marker, length, &rest[length..]))
            });
            match (open, run) {
                (Some((marker, length)), Some((ch, run_length, after)))
                    if ch == marker && run_length >= length && after.trim().is_empty() =>
                {
                    open = None;
                    true
                }
                (Some(_), _) => true,
                (None, Some((ch, run_length, after))) if !(ch == '`' && after.contains('`')) => {
                    open = Some((ch, run_length));
                    true
                }
                (None, _) => false,
            }
        })
        .collect()
}

/// A Markdown ATX heading of level 1 to 3 outside a fence (`#`, `##`, `###`).
fn is_section_heading(line: &str) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let level = rest.chars().take_while(|ch| *ch == '#').count();
    (1..=3).contains(&level)
        && rest[level..]
            .chars()
            .next()
            .is_none_or(|ch| ch == ' ' || ch == '\t')
}

fn parse_requirement_doc(text: &str) -> Option<(String, Vec<ReqBlock>)> {
    let mut blocks = Vec::new();
    let mut preamble_end = None;
    let mut lines = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        lines.push((offset, line));
        offset += line.len();
    }
    if offset < text.len() {
        lines.push((offset, &text[offset..]));
    }
    let fenced = fenced_lines(&lines.iter().map(|(_, line)| *line).collect::<Vec<_>>());
    let mut starts = Vec::new();
    for (index, (at, line)) in lines.iter().enumerate() {
        if fenced[index] {
            continue;
        }
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some(id) = requirement_heading_id(bare) {
            if preamble_end.is_none() {
                preamble_end = Some(*at);
            }
            starts.push((index, id.to_string()));
        } else if preamble_end.is_some() && is_section_heading(bare) {
            // A section after the first requirement would be absorbed into the
            // block before it, and duplicated when both sides add a block there.
            return None;
        }
    }
    // A scaffold with no requirement yet is all preamble and no blocks, so the
    // first IDs two branches add are still merged by ID.
    let Some(preamble_end) = preamble_end else {
        return Some((text.to_string(), Vec::new()));
    };
    for (pos, (index, id)) in starts.iter().enumerate() {
        let end = starts
            .get(pos + 1)
            .map(|(next, _)| lines[*next].0)
            .unwrap_or(text.len());
        blocks.push(ReqBlock {
            id: id.clone(),
            text: text[lines[*index].0..end].to_string(),
        });
    }
    Some((text[..preamble_end].to_string(), blocks))
}

fn requirement_heading_id(line: &str) -> Option<&str> {
    // An ATX heading may be indented by up to three spaces; four is indented code.
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let rest = line[indent..].strip_prefix("### ")?.trim();
    let token = rest.split_whitespace().next().unwrap_or(rest);
    let token =
        token.trim_matches(|ch: char| matches!(ch, ':' | ',' | '.' | ';' | ')' | '(' | ']' | '['));
    is_requirement_id(token).then_some(token)
}

fn is_requirement_id(token: &str) -> bool {
    let mut parts = token.split('-');
    if parts.next() != Some("REQ") {
        return false;
    }
    let module = match parts.next() {
        Some(module)
            if !module.is_empty() && module.chars().all(|ch| ch.is_ascii_alphanumeric()) =>
        {
            module
        }
        _ => return false,
    };
    let _ = module;
    let rest: Vec<&str> = parts.collect();
    let Some(last) = rest.last() else {
        return false;
    };
    !rest.is_empty()
        && rest
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_alphanumeric()))
        && last.chars().all(|ch| ch.is_ascii_digit())
}

fn merge_changelog(ancestor: &str, ours: &str, theirs: &str) -> ChangelogMerge {
    if ours == theirs || theirs == ancestor {
        return ChangelogMerge::clean(ours.to_string(), None);
    }
    if ours == ancestor {
        return ChangelogMerge::clean(theirs.to_string(), None);
    }
    match (
        parse_changelog_table(ancestor),
        parse_changelog_table(ours),
        parse_changelog_table(theirs),
    ) {
        (Some(base), Some(our), Some(their))
            if headers_match(&base, &our, &their)
                && keeps_every_row(&base, &our)
                && keeps_every_row(&base, &their) =>
        {
            let mut merged = union_changelog_tables(&base, &our, &their);
            if !ours.ends_with('\n') {
                while merged.content.ends_with('\n') {
                    merged.content.pop();
                }
            }
            merged
        }
        _ => {
            let merged = merge_text_region(ancestor, ours, theirs);
            ChangelogMerge {
                content: merged.content,
                clean: merged.clean,
                version: None,
            }
        }
    }
}

struct ChangelogMerge {
    content: String,
    clean: bool,
    version: Option<u64>,
}

impl ChangelogMerge {
    fn clean(content: String, version: Option<u64>) -> Self {
        Self {
            content,
            clean: true,
            version,
        }
    }
}

struct ChangeTable {
    prefix: String,
    header_cells: Vec<String>,
    rows: Vec<String>,
    suffix: String,
}

fn parse_changelog_table(section: &str) -> Option<ChangeTable> {
    if section.is_empty() {
        return None;
    }
    let lines: Vec<&str> = section.split_inclusive('\n').collect();
    if !section.ends_with('\n') {
        // split_inclusive keeps a trailing partial line. Nothing extra to do.
    }
    let header_at = lines.iter().position(|line| {
        let bare = line.trim_end_matches('\n').trim();
        bare.starts_with('|') && bare.ends_with('|') && !is_separator_row(bare)
    })?;
    let separator_at = header_at + 1;
    let separator = lines.get(separator_at)?;
    if !is_separator_row(separator.trim_end_matches('\n').trim()) {
        return None;
    }
    let prefix = lines[..=separator_at].concat();
    let header_cells = table_cells(lines[header_at].trim_end_matches('\n'));
    let mut rows = Vec::new();
    let mut suffix_at = lines.len();
    for (index, line) in lines.iter().enumerate().skip(separator_at + 1) {
        let bare = line.trim_end_matches('\n');
        let trimmed = bare.trim();
        // A blank line ends a Markdown table. It and everything after it, prose or another
        // table, are the suffix, merged as text and never as change-log rows (#875).
        if trimmed.is_empty() {
            suffix_at = index;
            break;
        }
        if trimmed.starts_with('|') && !is_separator_row(trimmed) {
            rows.push(trimmed.to_string());
            continue;
        }
        suffix_at = index;
        break;
    }
    let suffix = lines[suffix_at..].concat();
    Some(ChangeTable {
        prefix,
        header_cells,
        rows,
        suffix,
    })
}

/// Whether `side` still holds every ancestor row byte-for-byte. An edited or
/// removed ancestor row is not an addition, so the union does not apply.
fn keeps_every_row(base: &ChangeTable, side: &ChangeTable) -> bool {
    base.rows.iter().all(|row| side.rows.contains(row))
}

fn headers_match(base: &ChangeTable, ours: &ChangeTable, theirs: &ChangeTable) -> bool {
    let norm = |cells: &[String]| {
        cells
            .iter()
            .map(|cell| cell.to_ascii_lowercase())
            .collect::<Vec<_>>()
    };
    let base = norm(&base.header_cells);
    norm(&ours.header_cells) == base && norm(&theirs.header_cells) == base
}

fn union_changelog_tables(
    base: &ChangeTable,
    ours: &ChangeTable,
    theirs: &ChangeTable,
) -> ChangelogMerge {
    let base_rows: BTreeSet<&str> = base.rows.iter().map(String::as_str).collect();
    let mut rows: Vec<String> = Vec::new();
    for row in ours.rows.iter().chain(theirs.rows.iter()) {
        if !rows.iter().any(|existing| existing == row) {
            rows.push(row.clone());
        }
    }
    rows.sort_by(|left, right| {
        first_cell(left)
            .cmp(&first_cell(right))
            .then_with(|| left.cmp(right))
    });

    let version_at = ours
        .header_cells
        .iter()
        .position(|cell| cell.eq_ignore_ascii_case("version"));
    let mut assigned = None;
    if let Some(index) = version_at {
        let base_max = base
            .rows
            .iter()
            .filter_map(|row| row_cell(row, index))
            .filter_map(|cell| cell.parse::<u64>().ok())
            .max();
        if let Some(mut next) = base_max {
            let mut numbered = Vec::with_capacity(rows.len());
            let mut changed = false;
            for row in rows {
                if base_rows.contains(row.as_str()) {
                    numbered.push(row);
                    continue;
                }
                let Some(cell) = row_cell(&row, index) else {
                    return changelog_conflict(ours, theirs);
                };
                if cell.parse::<u64>().is_err() {
                    return changelog_conflict(ours, theirs);
                }
                // No higher `u64` to number this row with: a conflict, never a panic or a
                // wrap to 0 that would number it below the ancestor (#870, REQ-merge-006).
                let Some(following) = next.checked_add(1) else {
                    return changelog_conflict(ours, theirs);
                };
                next = following;
                numbered.push(rewrite_row_cell(&row, index, &next.to_string()));
                changed = true;
            }
            if changed {
                assigned = Some(next);
            }
            rows = numbered;
        } else if [ours, theirs].iter().all(|side| {
            side.rows
                .iter()
                .any(|row| !base_rows.contains(row.as_str()))
        }) {
            // No ancestor version to count from, and both sides added rows: their
            // version cells cannot be renumbered, so the table stays a conflict.
            return changelog_conflict(ours, theirs);
        }
    }

    let prefix = merge_text_region(&base.prefix, &ours.prefix, &theirs.prefix);
    let suffix = merge_text_region(&base.suffix, &ours.suffix, &theirs.suffix);
    let mut content = prefix.content;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    for row in &rows {
        content.push_str(row);
        content.push('\n');
    }
    content.push_str(&suffix.content);
    ChangelogMerge {
        content,
        clean: prefix.clean && suffix.clean,
        version: assigned,
    }
}

fn changelog_conflict(ours: &ChangeTable, theirs: &ChangeTable) -> ChangelogMerge {
    ChangelogMerge {
        content: conflict_block(&render_table(ours), &render_table(theirs)),
        clean: false,
        version: None,
    }
}

fn render_table(table: &ChangeTable) -> String {
    let mut content = table.prefix.clone();
    for row in &table.rows {
        content.push_str(row);
        content.push('\n');
    }
    content.push_str(&table.suffix);
    content
}

fn first_cell(row: &str) -> String {
    row_cell(row, 0).unwrap_or_default()
}

fn row_cell(row: &str, index: usize) -> Option<String> {
    table_cells(row).into_iter().nth(index)
}

fn rewrite_row_cell(row: &str, index: usize, value: &str) -> String {
    let mut cells = table_cells(row);
    if let Some(cell) = cells.get_mut(index) {
        *cell = value.to_string();
    }
    format!("| {} |", cells.join(" | "))
}

/// Cells of one table row. An escaped `\|` is part of a cell, not a delimiter.
fn table_cells(row: &str) -> Vec<String> {
    let inner = row.trim();
    let inner = inner.strip_prefix('|').unwrap_or(inner);
    let inner = match inner.strip_suffix('|') {
        Some(stripped) if !stripped.ends_with('\\') => stripped,
        _ => inner,
    };
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            cell.push(ch);
            escaped = false;
        } else if ch == '\\' {
            cell.push(ch);
            escaped = true;
        } else if ch == '|' {
            cells.push(cell.trim().to_string());
            cell.clear();
        } else {
            cell.push(ch);
        }
    }
    cells.push(cell.trim().to_string());
    cells
}

fn is_separator_row(row: &str) -> bool {
    let inner = row.trim().trim_matches('|');
    !inner.is_empty()
        && inner.split('|').all(|cell| {
            let cell = cell.trim();
            !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':')
        })
}

fn split_changelog(body: &str) -> (String, String, String) {
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    // A fenced example of a `## Change Log` heading is quoted text, not the section.
    let fenced = fenced_lines(&lines);
    let Some(start) = lines.iter().enumerate().position(|(index, line)| {
        !fenced[index] && line.trim_end_matches(['\n', '\r']) == "## Change Log"
    }) else {
        return (body.to_string(), String::new(), String::new());
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(index, line)| {
            !fenced[*index] && line.trim_end_matches(['\n', '\r']).starts_with("## ")
        })
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    (
        lines[..start].concat(),
        lines[start..end].concat(),
        lines[end..].concat(),
    )
}

fn split_frontmatter(text: &str) -> Option<(String, String)> {
    let rest = text.strip_prefix("---\n")?;
    let mut consumed = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches('\n') == "---" {
            let frontmatter = format!("---\n{}", &rest[..consumed + line.len()]);
            let body = rest[consumed + line.len()..].to_string();
            return Some((frontmatter, body));
        }
        consumed += line.len();
    }
    None
}

#[derive(Clone, Debug)]
struct Field {
    key: String,
    lines: Vec<String>,
    value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FieldValue {
    Scalar(String),
    List(Vec<String>),
}

fn merge_frontmatter(
    ancestor: &str,
    ours: &str,
    theirs: &str,
    version_floor: Option<u64>,
) -> MergeContract {
    let (Some(base_fields), Some(our_fields), Some(their_fields)) = (
        parse_fields(ancestor),
        parse_fields(ours),
        parse_fields(theirs),
    ) else {
        return merge_text_region(ancestor, ours, theirs);
    };
    if has_duplicate_keys(&base_fields)
        || has_duplicate_keys(&our_fields)
        || has_duplicate_keys(&their_fields)
    {
        return merge_text_region(ancestor, ours, theirs);
    }

    let mut keys = Vec::new();
    for fields in [&base_fields, &our_fields, &their_fields] {
        for field in fields {
            if !keys.iter().any(|key: &String| key == &field.key) {
                keys.push(field.key.clone());
            }
        }
    }

    let mut clean = true;
    let mut rendered = Vec::new();
    for key in keys {
        let base = base_fields.iter().find(|field| field.key == key);
        let our = our_fields.iter().find(|field| field.key == key);
        let their = their_fields.iter().find(|field| field.key == key);
        match merge_one_field(base, our, their, version_floor) {
            FieldMerge::Keep(text) if text.is_empty() => {}
            FieldMerge::Keep(text) => rendered.push(text),
            FieldMerge::Conflict(text) => {
                clean = false;
                rendered.push(text);
            }
        }
    }
    let mut content = format!("---\n{}\n---\n", rendered.join("\n"));
    if ancestor.ends_with("\n\n") || ours.ends_with("\n\n") {
        // The blank line between the fence and the body lives in the body, not here.
    }
    if !ancestor.ends_with('\n') && content.ends_with('\n') && !ours.ends_with('\n') {
        content.pop();
    }
    MergeContract { content, clean }
}

enum FieldMerge {
    Keep(String),
    Conflict(String),
}

fn merge_one_field(
    base: Option<&Field>,
    ours: Option<&Field>,
    theirs: Option<&Field>,
    version_floor: Option<u64>,
) -> FieldMerge {
    match (base, ours, theirs) {
        (None, None, None) => FieldMerge::Keep(String::new()),
        (None, Some(ours), None) => FieldMerge::Keep(ours.lines.join("\n")),
        (None, None, Some(theirs)) => FieldMerge::Keep(theirs.lines.join("\n")),
        (Some(_), None, None) => FieldMerge::Keep(String::new()),
        (Some(base), Some(ours), None) if ours.lines == base.lines => {
            FieldMerge::Keep(String::new())
        }
        (Some(base), None, Some(theirs)) if theirs.lines == base.lines => {
            FieldMerge::Keep(String::new())
        }
        // One side deleted the field and the other edited it. The deleting side
        // is shown empty, so choosing it does not restore the old value.
        (Some(_), Some(ours), None) => {
            FieldMerge::Conflict(conflict_block(&ours.lines.join("\n"), ""))
        }
        (Some(_), None, Some(theirs)) => {
            FieldMerge::Conflict(conflict_block("", &theirs.lines.join("\n")))
        }
        (base, Some(ours), Some(theirs)) => merge_present_field(base, ours, theirs, version_floor),
    }
}

fn merge_present_field(
    base: Option<&Field>,
    ours: &Field,
    theirs: &Field,
    version_floor: Option<u64>,
) -> FieldMerge {
    if ours.key == "version" {
        return merge_version_field(base, ours, theirs, version_floor);
    }
    if ours.lines == theirs.lines {
        return FieldMerge::Keep(ours.lines.join("\n"));
    }
    if base.is_some_and(|base| base.lines == ours.lines) {
        return FieldMerge::Keep(theirs.lines.join("\n"));
    }
    if base.is_some_and(|base| base.lines == theirs.lines) {
        return FieldMerge::Keep(ours.lines.join("\n"));
    }
    if matches!(ours.key.as_str(), "files" | "db_tables" | "depends_on")
        && let (FieldValue::List(our_items), FieldValue::List(their_items)) =
            (&ours.value, &theirs.value)
    {
        let mut combined = Vec::new();
        for item in our_items.iter().chain(their_items.iter()) {
            if !combined.contains(item) {
                combined.push(item.clone());
            }
        }
        combined.sort();
        let mut lines = vec![format!("{}:", ours.key)];
        if combined.is_empty() {
            return FieldMerge::Keep(format!("{}: []", ours.key));
        }
        for item in combined {
            lines.push(format!("  - {}", render_list_item(&item)));
        }
        return FieldMerge::Keep(lines.join("\n"));
    }
    FieldMerge::Conflict(conflict_block(
        &ours.lines.join("\n"),
        &theirs.lines.join("\n"),
    ))
}

fn merge_version_field(
    base: Option<&Field>,
    ours: &Field,
    theirs: &Field,
    version_floor: Option<u64>,
) -> FieldMerge {
    let FieldValue::Scalar(our_raw) = &ours.value else {
        return FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &theirs.lines.join("\n"),
        ));
    };
    let FieldValue::Scalar(their_raw) = &theirs.value else {
        return FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &theirs.lines.join("\n"),
        ));
    };
    let base_raw = base.and_then(|field| match &field.value {
        FieldValue::Scalar(raw) => Some(raw.as_str()),
        _ => None,
    });
    let combined = match (
        base_raw.and_then(parse_numeric_version),
        parse_numeric_version(our_raw),
        parse_numeric_version(their_raw),
    ) {
        (Some(base), Some(ours_n), Some(theirs_n)) => combine_versions(base, ours_n, theirs_n),
        _ => None,
    };
    // Equal values written with different scalar syntax (`6` and `"6"`) stay a
    // conflict, as do any two styles that differ (merge invariant 22).
    if combined.is_some() && our_raw != their_raw && quote_style(our_raw) != quote_style(their_raw)
    {
        return FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &theirs.lines.join("\n"),
        ));
    }
    let Some(combined) = combined else {
        if our_raw == their_raw {
            return FieldMerge::Keep(ours.lines.join("\n"));
        }
        if base_raw.is_some_and(|raw| raw == our_raw) {
            return FieldMerge::Keep(theirs.lines.join("\n"));
        }
        if base_raw.is_some_and(|raw| raw == their_raw) {
            return FieldMerge::Keep(ours.lines.join("\n"));
        }
        return FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &theirs.lines.join("\n"),
        ));
    };
    // What follows the number (the closing quote's tail or a ` # comment`) merges three
    // ways, so the result is the same in either merge order and a one-sided note survives;
    // two different notes are a conflict (#876, REQ-merge-006).
    let our_note = version_decoration(our_raw);
    let their_note = version_decoration(their_raw);
    let base_note = base_raw.map(version_decoration);
    let note_side = if our_note == their_note || base_note == Some(their_note) {
        our_raw
    } else if base_note == Some(our_note) {
        their_raw
    } else {
        return FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &theirs.lines.join("\n"),
        ));
    };
    let raised = match (combined, version_floor) {
        (NumericVersion::Int(value), Some(floor)) => NumericVersion::Int(value.max(floor)),
        (other, _) => other,
    };
    if let NumericVersion::Int(value) = raised
        && parse_numeric_version(our_raw) == Some(NumericVersion::Int(value))
        && our_raw == their_raw
    {
        return FieldMerge::Keep(ours.lines.join("\n"));
    }
    if let NumericVersion::Int(value) = raised
        && parse_numeric_version(their_raw) == Some(NumericVersion::Int(value))
        && version_floor.is_none()
        && note_side == their_raw
    {
        return FieldMerge::Keep(theirs.lines.join("\n"));
    }
    let rendered = match raised {
        NumericVersion::Int(value) => value.to_string(),
        NumericVersion::Semver {
            major,
            minor,
            patch,
        } => format!("{major}.{minor}.{patch}"),
    };
    let line = format!("version: {}", rewrite_scalar(note_side, &rendered));
    FieldMerge::Keep(line)
}

/// The text after a version scalar's value: what follows the closing quote, or the ` #`
/// comment of a plain scalar. `rewrite_scalar` keeps exactly this part.
fn version_decoration(raw: &str) -> &str {
    let raw = raw.trim();
    if let Some(quote) = raw.chars().next().filter(|ch| *ch == '"' || *ch == '\'') {
        let rest = &raw[quote.len_utf8()..];
        return rest
            .find(quote)
            .map_or("", |close| &rest[close + quote.len_utf8()..]);
    }
    raw.find(" #").map_or("", |comment| &raw[comment..])
}

/// The quote character a scalar opens with, if any.
fn quote_style(raw: &str) -> Option<char> {
    raw.trim()
        .chars()
        .next()
        .filter(|ch| *ch == '"' || *ch == '\'')
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NumericVersion {
    Int(u64),
    Semver { major: u64, minor: u64, patch: u64 },
}

fn parse_numeric_version(raw: &str) -> Option<NumericVersion> {
    let inner = scalar_inner(raw)?;
    if let Ok(value) = inner.parse::<u64>() {
        return Some(NumericVersion::Int(value));
    }
    let mut parts = inner.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(NumericVersion::Semver {
        major,
        minor,
        patch,
    })
}

fn scalar_inner(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    let (body, _) = if let Some(quote) = raw.chars().next().filter(|ch| *ch == '"' || *ch == '\'') {
        let rest = &raw[quote.len_utf8()..];
        let close = rest.find(quote)?;
        (&rest[..close], &rest[close + quote.len_utf8()..])
    } else {
        let comment = raw.find(" #").unwrap_or(raw.len());
        (&raw[..comment], &raw[comment..])
    };
    let body = body.trim();
    (!body.is_empty()).then_some(body)
}

fn rewrite_scalar(raw: &str, new_value: &str) -> String {
    let raw = raw.trim();
    if let Some(quote) = raw.chars().next().filter(|ch| *ch == '"' || *ch == '\'') {
        let rest = &raw[quote.len_utf8()..];
        if let Some(close) = rest.find(quote) {
            let suffix = &rest[close + quote.len_utf8()..];
            return format!("{quote}{new_value}{quote}{suffix}");
        }
    }
    let comment = raw.find(" #").unwrap_or(raw.len());
    format!("{new_value}{}", &raw[comment..])
}

fn combine_versions(
    base: NumericVersion,
    ours: NumericVersion,
    theirs: NumericVersion,
) -> Option<NumericVersion> {
    match (base, ours, theirs) {
        (NumericVersion::Int(base), NumericVersion::Int(ours), NumericVersion::Int(theirs)) => {
            let sum = ours.checked_add(theirs)?.checked_sub(base)?;
            Some(NumericVersion::Int(sum.max(ours).max(theirs)))
        }
        (
            NumericVersion::Semver {
                major: base_major,
                minor: base_minor,
                patch: base_patch,
            },
            NumericVersion::Semver {
                major: our_major,
                minor: our_minor,
                patch: our_patch,
            },
            NumericVersion::Semver {
                major: their_major,
                minor: their_minor,
                patch: their_patch,
            },
        ) if base_major == our_major
            && our_major == their_major
            && base_minor == our_minor
            && our_minor == their_minor =>
        {
            let patch = our_patch
                .checked_add(their_patch)?
                .checked_sub(base_patch)?;
            Some(NumericVersion::Semver {
                major: our_major,
                minor: our_minor,
                patch: patch.max(our_patch).max(their_patch),
            })
        }
        _ => None,
    }
}

fn parse_fields(frontmatter: &str) -> Option<Vec<Field>> {
    let inner = frontmatter
        .strip_prefix("---\n")?
        .strip_suffix("---\n")
        .or_else(|| frontmatter.strip_prefix("---\n")?.strip_suffix("---"))?;
    let mut fields = Vec::new();
    let mut current: Option<Field> = None;
    for line in inner.lines() {
        if line.trim().is_empty() {
            if let Some(field) = current.as_mut() {
                field.lines.push(line.to_string());
            }
            continue;
        }
        if line.trim_start().starts_with("- ") {
            let field = current.as_mut()?;
            let FieldValue::List(items) = &mut field.value else {
                return None;
            };
            // The item is a YAML scalar: quotes are not part of the value and an
            // unquoted ` #` starts a comment. An item this cannot read keeps the
            // whole frontmatter on the plain text merge.
            items.push(parse_list_scalar(line.trim().trim_start_matches("- "))?);
            field.lines.push(line.to_string());
            continue;
        }
        let (key, value) = split_field_key(line)?;
        if let Some(field) = current.take() {
            fields.push(field);
        }
        // An empty value starts a block list (`files:` followed by `-` items).
        // `key: []` is an empty list too. A bare `key:` with no items stays an
        // empty list and is rendered from its original line when that line is unchanged.
        let parsed = if value.is_empty() || value == "[]" {
            FieldValue::List(Vec::new())
        } else if let Some(items) = parse_inline_list(&value) {
            FieldValue::List(items)
        } else {
            FieldValue::Scalar(value)
        };
        current = Some(Field {
            key,
            lines: vec![line.to_string()],
            value: parsed,
        });
    }
    if let Some(field) = current {
        fields.push(field);
    }
    Some(fields)
}

fn split_field_key(line: &str) -> Option<(String, String)> {
    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }
    let colon = line.find(':')?;
    let key = &line[..colon];
    if key.is_empty() || key.contains(' ') {
        return None;
    }
    Some((key.to_string(), line[colon + 1..].trim().to_string()))
}

/// Items of a flow list such as `["src/a,b.rs", src/c.rs]`. A comma inside a
/// quoted item is part of that item. Anything this cannot read (an unclosed
/// quote, a nested flow collection) is `None`, so the field is not unioned.
fn parse_inline_list(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        match quote {
            Some('"') if ch == '\\' => {
                current.push(ch);
                current.push(chars.next()?);
            }
            Some(open) if ch == open => {
                if open == '\'' && chars.peek() == Some(&'\'') {
                    current.push(ch);
                    current.push(chars.next()?);
                } else {
                    current.push(ch);
                    quote = None;
                }
            }
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => {
                current.push(ch);
                quote = Some(ch);
            }
            None if matches!(ch, '[' | ']' | '{' | '}') => return None,
            None if ch == ',' => {
                items.push(parse_list_scalar(current.trim())?);
                current.clear();
            }
            None => current.push(ch),
        }
    }
    if quote.is_some() {
        return None;
    }
    if !current.trim().is_empty() {
        items.push(parse_list_scalar(current.trim())?);
    }
    Some(items)
}

/// One list item read as a YAML scalar: the quoted value, or the plain value
/// before an ` #` comment. `None` for an empty item or text after a closing quote.
fn parse_list_scalar(raw: &str) -> Option<String> {
    let raw = raw.trim();
    match raw.chars().next() {
        Some('"') => {
            let mut value = String::new();
            let mut chars = raw[1..].char_indices();
            while let Some((at, ch)) = chars.next() {
                match ch {
                    '\\' => match chars.next()?.1 {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        _ => return None,
                    },
                    '"' => {
                        return trailing_is_comment(&raw[1 + at + 1..]).then_some(value);
                    }
                    other => value.push(other),
                }
            }
            None
        }
        Some('\'') => {
            let mut value = String::new();
            let body = &raw[1..];
            let mut chars = body.char_indices().peekable();
            while let Some((at, ch)) = chars.next() {
                if ch == '\'' {
                    if chars.peek().map(|(_, next)| *next) == Some('\'') {
                        chars.next();
                        value.push('\'');
                        continue;
                    }
                    return trailing_is_comment(&body[at + 1..]).then_some(value);
                }
                value.push(ch);
            }
            None
        }
        Some(_) => {
            let value = match raw.find(" #").or_else(|| raw.find("\t#")) {
                Some(comment) => raw[..comment].trim_end(),
                None => raw,
            };
            (!value.is_empty()).then(|| value.to_string())
        }
        None => None,
    }
}

fn trailing_is_comment(rest: &str) -> bool {
    let rest = rest.trim_start();
    rest.is_empty() || rest.starts_with('#')
}

/// Renders a merged list item. A plain path stays plain; anything YAML could
/// read differently is double-quoted.
fn render_list_item(item: &str) -> String {
    let plain = !item.is_empty()
        && !item.starts_with(['-', '?', ':', '!', '&', '*', '|', '>', '%', '@', '`', '#'])
        && item.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-' | '+' | '~')
        });
    if plain && !yaml_reads_as_non_string(item) {
        item.to_string()
    } else {
        format!("\"{}\"", item.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Whether YAML (1.1 or 1.2) would read this plain scalar as something other than a string:
/// a boolean, a null, or a number. Such an item is written quoted so a union keeps it a
/// string (#872, REQ-merge-006). Quoting more than needed is safe; it stays a string.
fn yaml_reads_as_non_string(item: &str) -> bool {
    let lower = item.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n" | "null" | "~"
    ) {
        return true;
    }
    if item.parse::<f64>().is_ok() {
        return true;
    }
    let unsigned = lower.strip_prefix(['+', '-']).unwrap_or(&lower);
    if matches!(unsigned, ".inf" | ".nan") {
        return true;
    }
    let radix = |prefix: &str, digit: fn(char) -> bool| {
        unsigned
            .strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|ch| digit(ch) || ch == '_'))
    };
    if radix("0x", |ch| ch.is_ascii_hexdigit())
        || radix("0o", |ch| matches!(ch, '0'..='7'))
        || radix("0b", |ch| matches!(ch, '0' | '1'))
    {
        return true;
    }
    // YAML 1.1 numbers may carry underscores anywhere in the digits, exponent forms included
    // (`1_000`, `1_000.5e+2`): read it as a number once they are gone.
    item.contains('_') && item.replace('_', "").parse::<f64>().is_ok()
}

fn has_duplicate_keys(fields: &[Field]) -> bool {
    let mut seen = BTreeSet::new();
    fields.iter().any(|field| !seen.insert(field.key.as_str()))
}

fn merge_text_region(ancestor: &str, ours: &str, theirs: &str) -> MergeContract {
    if ours == theirs || theirs == ancestor {
        return MergeContract {
            content: ours.to_string(),
            clean: true,
        };
    }
    if ours == ancestor {
        return MergeContract {
            content: theirs.to_string(),
            clean: true,
        };
    }
    match line_merge(ancestor, ours, theirs) {
        Ok(content) => MergeContract {
            content,
            clean: true,
        },
        Err(content) => MergeContract {
            content,
            clean: false,
        },
    }
}

fn line_merge(ancestor: &str, ours: &str, theirs: &str) -> Result<String, String> {
    let dir =
        tempfile::tempdir().map_err(|error| conflict_block(ours, theirs) + &error.to_string())?;
    let base_path = dir.path().join("base");
    let ours_path = dir.path().join("ours");
    let theirs_path = dir.path().join("theirs");
    let write = |path: &Path, text: &str| -> Result<(), String> {
        let mut stored = text.to_string();
        if !stored.ends_with('\n') {
            stored.push('\n');
        }
        fs::write(path, stored).map_err(|error| error.to_string())
    };
    if let Err(error) = write(&base_path, ancestor)
        .and_then(|_| write(&ours_path, ours))
        .and_then(|_| write(&theirs_path, theirs))
    {
        return Err(conflict_block(ours, theirs) + &error);
    }
    let output = git_cmd(dir.path())
        .args(["merge-file", "-p", "--diff3"])
        .arg(&ours_path)
        .arg(&base_path)
        .arg(&theirs_path)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(_) => return Err(conflict_block(ours, theirs)),
    };
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let text = restore_region_newline(ours, text);
    if merge_file_clean(&output) && !text.contains("<<<<<<<") {
        Ok(text)
    } else if text.contains("<<<<<<<") {
        Err(text)
    } else {
        Err(conflict_block(ours, theirs))
    }
}

fn merge_file_clean(output: &Output) -> bool {
    output.status.success()
}

fn restore_region_newline(template: &str, mut text: String) -> String {
    if template.ends_with('\n') {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text
    } else {
        while text.ends_with('\n') {
            text.pop();
        }
        text
    }
}

fn conflict_block(ours: &str, theirs: &str) -> String {
    format!("<<<<<<< ours\n{ours}\n=======\n{theirs}\n>>>>>>> theirs\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(version: &str, files: &[&str], rows: &[&str]) -> String {
        let files = files
            .iter()
            .map(|file| format!("  - {file}\n"))
            .collect::<String>();
        let rows = rows
            .iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>();
        format!(
            "---\nmodule: cli\nversion: {version}\nstatus: stable\nfiles:\n{files}db_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Purpose\n\nThe command line.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n{rows}"
        )
    }

    fn requirements(blocks: &[&str]) -> String {
        let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n### REQ-cli-001\n\nThe system SHALL start.\n\nAcceptance Criteria\n- It starts.\n".to_string();
        for block in blocks {
            text.push('\n');
            text.push_str(block);
            if !block.ends_with('\n') {
                text.push('\n');
            }
        }
        text
    }

    #[test]
    fn parallel_bumps_merge_commutatively_and_count_both() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "6",
            &["src/cli.rs", "src/a.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let theirs = spec(
            "6",
            &["src/cli.rs", "src/b.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-b: add b |",
            ],
        );
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/cli.spec.md");
        assert!(forward.clean, "{}", forward.content);
        assert!(backward.clean, "{}", backward.content);
        assert_eq!(forward.content, backward.content);
        assert!(
            forward.content.contains("version: 7\n"),
            "{}",
            forward.content
        );
        assert!(forward.content.contains("src/a.rs"));
        assert!(forward.content.contains("src/b.rs"));
        assert!(forward.content.contains("change-a: add a"));
        assert!(forward.content.contains("change-b: add b"));
        assert!(!forward.content.contains("<<<<<<<"));
    }

    #[test]
    fn different_version_numbers_still_merge_to_one_line() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "6",
            &["src/cli.rs", "src/a.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let theirs = spec(
            "7",
            &["src/cli.rs", "src/c.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-02-01 | mid: already |",
                "| 2026-11-05 | change-c: run C |",
            ],
        );
        let merged = merge_contract(&base, &ours, &theirs, "cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(
            merged.content.contains("version: 8\n"),
            "{}",
            merged.content
        );
        assert_eq!(
            merged.content.matches("version:").count(),
            1,
            "{}",
            merged.content
        );
    }

    #[test]
    fn quoted_version_keeps_its_quotes_when_both_bumps_count() {
        let base = spec(
            "\"5\"",
            &["src/cli.rs"],
            &["| 2026-01-01 | base: initial |"],
        );
        let ours = spec(
            "\"6\"",
            &["src/cli.rs", "src/a.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let theirs = spec(
            "\"6\"",
            &["src/cli.rs", "src/b.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-b: add b |",
            ],
        );
        let merged = merge_contract(&base, &ours, &theirs, "cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(
            merged.content.contains("version: \"7\"\n"),
            "{}",
            merged.content
        );
    }

    // #874: a one-sided reorder of existing requirement blocks is kept, not silently dropped.
    #[test]
    fn a_one_sided_requirement_reorder_is_kept() {
        let one =
            "### REQ-cli-001\n\nThe system SHALL start.\n\nAcceptance Criteria\n- It starts.\n";
        let two = "### REQ-cli-002\n\nThe system SHALL run.\n\nAcceptance Criteria\n- It runs.\n";
        let three =
            "### REQ-cli-003\n\nThe system SHALL stop.\n\nAcceptance Criteria\n- It stops.\n";
        let doc = |blocks: &[&str]| {
            let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n".to_string();
            for block in blocks {
                text.push_str(block);
                text.push('\n');
            }
            text
        };
        let base = doc(&[one, two]);
        // Ours swaps the two blocks; theirs only adds a third.
        let ours = doc(&[two, one]);
        let theirs = doc(&[one, two, three]);
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/requirements.md");
        assert!(forward.clean, "{}", forward.content);
        assert!(backward.clean, "{}", backward.content);
        let pos = |text: &str, needle: &str| text.find(needle).unwrap();
        assert!(
            pos(&forward.content, "REQ-cli-002") < pos(&forward.content, "REQ-cli-001"),
            "the reorder is kept:\n{}",
            forward.content
        );
        assert!(
            pos(&forward.content, "REQ-cli-001") < pos(&forward.content, "REQ-cli-003"),
            "the addition lands after the reordered blocks:\n{}",
            forward.content
        );
        assert_eq!(forward.content, backward.content);
    }

    // t384 on #1058: a reorder on one side plus a deletion on the other must keep the reorder.
    #[test]
    fn a_reorder_plus_a_deletion_keeps_the_reorder() {
        let one =
            "### REQ-cli-001\n\nThe system SHALL start.\n\nAcceptance Criteria\n- It starts.\n";
        let two = "### REQ-cli-002\n\nThe system SHALL run.\n\nAcceptance Criteria\n- It runs.\n";
        let three =
            "### REQ-cli-003\n\nThe system SHALL stop.\n\nAcceptance Criteria\n- It stops.\n";
        let doc = |blocks: &[&str]| {
            let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n".to_string();
            for block in blocks {
                text.push_str(block);
                text.push('\n');
            }
            text
        };
        let base = doc(&[one, two, three]);
        let ours = doc(&[two, one, three]); // reordered
        let theirs = doc(&[one, two]); // deleted three
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/requirements.md");
        for merged in [&forward, &backward] {
            assert!(merged.clean, "{}", merged.content);
            assert!(
                !merged.content.contains("REQ-cli-003"),
                "the deletion stands: {}",
                merged.content
            );
            assert!(
                merged.content.find("REQ-cli-002").unwrap()
                    < merged.content.find("REQ-cli-001").unwrap(),
                "the reorder is kept: {}",
                merged.content
            );
        }
        assert_eq!(forward.content, backward.content);
    }

    // #874: two different reorders conflict rather than one silently prevailing.
    #[test]
    fn divergent_requirement_reorders_conflict() {
        let one =
            "### REQ-cli-001\n\nThe system SHALL start.\n\nAcceptance Criteria\n- It starts.\n";
        let two = "### REQ-cli-002\n\nThe system SHALL run.\n\nAcceptance Criteria\n- It runs.\n";
        let three =
            "### REQ-cli-003\n\nThe system SHALL stop.\n\nAcceptance Criteria\n- It stops.\n";
        let doc = |blocks: &[&str]| {
            let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n".to_string();
            for block in blocks {
                text.push_str(block);
                text.push('\n');
            }
            text
        };
        let base = doc(&[one, two, three]);
        let ours = doc(&[two, one, three]);
        let theirs = doc(&[three, one, two]);
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert!(
            !merged.clean,
            "divergent reorders must conflict:\n{}",
            merged.content
        );
        assert!(
            merged.content.contains("<<<<<<<"),
            "a conflict holds both sides under their own labels:\n{}",
            merged.content
        );
    }

    // t384 on #1058: both sides moving the SAME block to different places is a divergent
    // reorder the line merge resolves "clean" — a shared deletion plus two non-overlapping
    // inserts writes the block twice, and the merged file then fails `specsync check`.
    // The merge must conflict with markers instead.
    #[test]
    fn divergent_moves_of_the_same_block_conflict_with_markers() {
        let block = |n: usize| {
            format!(
                "### REQ-cli-00{n}\n\nThe system SHALL do {n}.\n\nAcceptance Criteria\n- {n} done.\n"
            )
        };
        let doc = |sequence: &[usize]| {
            let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n".to_string();
            for n in sequence {
                text.push_str(&block(*n));
                text.push('\n');
            }
            text
        };
        let base = doc(&[1, 2, 3, 4, 5]);
        let ours = doc(&[3, 1, 2, 4, 5]);
        let theirs = doc(&[1, 2, 4, 5, 3]);
        for (a, b) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, a, b, "specs/cli/requirements.md");
            assert!(
                !merged.clean,
                "divergent moves of one block must not merge clean:\n{}",
                merged.content
            );
            assert!(
                merged.content.contains("<<<<<<<"),
                "the conflict holds both sides under their own labels:\n{}",
                merged.content
            );
        }
    }

    // t384's nit on #1058: a block one side deleted and the other edited keeps its document
    // position — re-inserted after its base predecessor, not pushed to the end.
    #[test]
    fn a_delete_edit_conflict_keeps_its_document_position() {
        let block = |n: usize, text: &str| {
            format!(
                "### REQ-cli-00{n}\n\nThe system SHALL {text}.\n\nAcceptance Criteria\n- {text}.\n"
            )
        };
        let doc = |blocks: &[String]| {
            let mut text = "---\nspec: cli.spec.md\n---\n\n# Requirements\n\n".to_string();
            for block in blocks {
                text.push_str(block);
                text.push('\n');
            }
            text
        };
        let base = doc(&[block(1, "start"), block(2, "run"), block(3, "stop")]);
        let deleted = doc(&[block(1, "start"), block(3, "stop")]);
        let edited = doc(&[block(1, "start"), block(2, "sprint"), block(3, "stop")]);
        for (ours, theirs) in [(&deleted, &edited), (&edited, &deleted)] {
            let merged = merge_contract(&base, ours, theirs, "specs/cli/requirements.md");
            assert!(
                !merged.clean,
                "delete/edit must conflict:\n{}",
                merged.content
            );
            let first = merged.content.find("### REQ-cli-001").unwrap();
            let marker = merged.content.find("<<<<<<<").unwrap();
            let third = merged.content.find("### REQ-cli-003").unwrap();
            assert!(
                first < marker && marker < third,
                "the conflicted block keeps its position between 001 and 003:\n{}",
                merged.content
            );
        }
    }

    #[test]
    fn requirement_additions_merge_in_either_order_and_duplicate_ids_are_kept() {
        let base = requirements(&[]);
        let ours = requirements(&[
            "### REQ-cli-002\n\nThe system SHALL run A.\n\nAcceptance Criteria\n- A runs.\n",
        ]);
        let theirs = requirements(&[
            "### REQ-cli-003\n\nThe system SHALL run B.\n\nAcceptance Criteria\n- B runs.\n",
        ]);
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/requirements.md");
        assert!(forward.clean, "{}", forward.content);
        assert_eq!(forward.content, backward.content);
        assert!(forward.content.contains("### REQ-cli-002"));
        assert!(forward.content.contains("### REQ-cli-003"));
        assert_eq!(forward.content.matches("### REQ-cli-001").count(), 1);

        let duplicate = requirements(&[
            "### REQ-cli-002\n\nThe system SHALL run A.\n\nAcceptance Criteria\n- A runs.\n",
        ]);
        let other = requirements(&[
            "### REQ-cli-002\n\nThe system SHALL run B instead.\n\nAcceptance Criteria\n- B runs.\n",
        ]);
        let merged = merge_contract(&base, &duplicate, &other, "requirements.md");
        assert!(merged.clean, "{}", merged.content);
        assert_eq!(
            merged.content.matches("### REQ-cli-002").count(),
            2,
            "{}",
            merged.content
        );
        assert!(merged.content.contains("run A"));
        assert!(merged.content.contains("run B instead"));
    }

    #[test]
    fn divergent_prose_stays_a_conflict() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = base.replace("The command line.", "The command line from A.");
        let theirs = base.replace("The command line.", "The command line from B.");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(!merged.clean, "divergent purpose must not merge cleanly");
        assert!(merged.content.contains("<<<<<<<"), "{}", merged.content);
        assert!(merged.content.contains("from A"));
        assert!(merged.content.contains("from B"));
    }

    #[test]
    fn one_sided_edit_is_returned_unchanged() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "6",
            &["src/cli.rs", "src/a.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let merged = merge_contract(&base, &ours, &base, "cli.spec.md");
        assert!(merged.clean);
        assert_eq!(merged.content, ours);
    }

    // P1 (review): a fenced example headed like a requirement must not shadow a
    // real requirement with that ID and drop it from a "clean" merge.
    #[test]
    fn fenced_requirement_example_never_drops_a_real_requirement() {
        let example = "Example:\n\n```markdown\n### REQ-cli-002\n\nAn example only.\n```\n";
        let base = requirements(&[]).replace(
            "# Requirements\n\n",
            &format!("# Requirements\n\n{example}\n"),
        );
        let real =
            "### REQ-cli-002\n\nThe system SHALL stop.\n\nAcceptance Criteria\n- It stops.\n";
        let ours = format!("{base}\n{real}");
        let theirs = base.replace("The system SHALL start.", "The system SHALL start quickly.");
        for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, left, right, "specs/cli/requirements.md");
            assert!(
                merged.content.contains("The system SHALL stop."),
                "real REQ-cli-002 was dropped:\n{}",
                merged.content
            );
            assert!(
                merged.content.contains("An example only."),
                "{}",
                merged.content
            );
            if merged.clean {
                assert!(
                    merged.content.contains("start quickly"),
                    "{}",
                    merged.content
                );
            }
        }
    }

    // P1 (review): a one-sided edit to the Change Log text before the table must
    // survive a row union, in either merge order.
    #[test]
    fn changelog_prefix_edit_survives_a_row_union_in_either_order() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "6",
            &["src/cli.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let theirs = base.replace(
            "## Change Log\n\n",
            "## Change Log\n\nNewest rows last.\n\n",
        );
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/cli.spec.md");
        for merged in [&forward, &backward] {
            assert!(merged.clean, "{}", merged.content);
            assert!(
                merged.content.contains("Newest rows last."),
                "{}",
                merged.content
            );
            assert!(
                merged.content.contains("change-a: add a"),
                "{}",
                merged.content
            );
        }
        assert_eq!(forward.content, backward.content);
    }

    // P1 (review): a quoted comma inside an inline list item is one path, not two.
    #[test]
    fn quoted_comma_in_an_inline_list_stays_one_item() {
        let make = |files: &str| {
            format!(
                "---\nmodule: cli\nversion: 5\nstatus: stable\nfiles: {files}\ndb_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Purpose\n\nThe command line.\n"
            )
        };
        let base = make(r#"["src/a,b.rs"]"#);
        let ours = make(r#"["src/a,b.rs", "src/c.rs"]"#);
        let theirs = make(r#"["src/a,b.rs", "src/d.rs"]"#);
        for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
            assert!(!merged.content.contains("- src/a\n"), "{}", merged.content);
            assert!(!merged.content.contains("- b.rs"), "{}", merged.content);
            if merged.clean {
                assert!(merged.content.contains("src/a,b.rs"), "{}", merged.content);
                assert!(merged.content.contains("src/c.rs"), "{}", merged.content);
                assert!(merged.content.contains("src/d.rs"), "{}", merged.content);
            }
        }
    }

    // P2: equal versions written with different scalar syntax stay a conflict (merge invariant 22).
    #[test]
    fn version_scalar_syntax_divergence_stays_a_conflict() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = base.replace("version: 5", "version: 6");
        let theirs = base.replace("version: 5", "version: \"6\"");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(!merged.clean, "{}", merged.content);
    }

    // P2: two added blocks from files with no final newline stay on separate lines.
    #[test]
    fn appended_blocks_without_a_final_newline_start_on_their_own_line() {
        let base = requirements(&[]);
        let ours = format!("{base}\n### REQ-cli-002\n\nThe system SHALL stop.");
        let theirs = format!("{base}\n### REQ-cli-002\n\nThe system SHALL pause.");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert_eq!(
            merged.content.matches("\n### REQ-cli-002").count(),
            2,
            "{}",
            merged.content
        );
    }

    // P2: a deleted-versus-edited requirement shows the deleting side as empty.
    #[test]
    fn deleted_versus_edited_requirement_shows_the_deletion() {
        let block = "### REQ-cli-002\n\nThe system SHALL stop.\n";
        let base = requirements(&[block]);
        let ours = requirements(&[]);
        let theirs = requirements(&["### REQ-cli-002\n\nThe system SHALL stop now.\n"]);
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert!(!merged.clean);
        assert!(
            merged
                .content
                .contains("<<<<<<< ours\n\n=======\n### REQ-cli-002\n\nThe system SHALL stop now."),
            "{}",
            merged.content
        );
        assert!(
            !merged.content.contains("SHALL stop.\n"),
            "{}",
            merged.content
        );
    }

    // P2: an escaped pipe inside a cell survives renumbering.
    #[test]
    fn escaped_pipe_in_a_changelog_cell_survives_renumbering() {
        let table = |rows: &str| {
            format!(
                "---\nmodule: cli\nversion: 2\nstatus: stable\nfiles:\n  - src/cli.rs\ndb_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Change Log\n\n| Version | Date | Change |\n|---------|------|--------|\n| 1 | 2026-01-01 | initial |\n{rows}"
            )
        };
        let base = table("");
        let ours = table("| 2 | 2026-11-05 | change-a: A \\| B |\n");
        let theirs = table("| 2 | 2026-11-05 | change-b: C |\n");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(
            merged.content.contains("change-a: A \\| B |"),
            "{}",
            merged.content
        );
    }

    // P2: an edited ancestor row is not a new row, so it is not unioned and renumbered.
    #[test]
    fn edited_ancestor_changelog_row_is_not_counted_as_new() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "5",
            &["src/cli.rs"],
            &["| 2026-01-01 | base: initial release |"],
        );
        let theirs = spec(
            "6",
            &["src/cli.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-b: add b |",
            ],
        );
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(
            !(merged.clean
                && merged.content.contains("base: initial |")
                && merged.content.contains("initial release")),
            "obsolete and edited rows were both kept in a clean merge:\n{}",
            merged.content
        );
    }

    // P2: a fenced `## Change Log` example does not stand in for the real section.
    #[test]
    fn fenced_changelog_heading_is_not_the_changelog() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]).replace(
            "The command line.\n",
            "The command line.\n\n```markdown\n## Change Log\n```\n",
        );
        let ours = base.replace("version: 5", "version: 6") + "| 2026-11-05 | change-a: add a |\n";
        let theirs =
            base.replace("version: 5", "version: 6") + "| 2026-11-05 | change-b: add b |\n";
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(merged.content.contains("change-a: add a"));
        assert!(merged.content.contains("change-b: add b"));
    }

    // P2: the changelog keeps the current side's missing final newline.
    #[test]
    fn changelog_union_keeps_a_missing_final_newline() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let ours = spec(
            "6",
            &["src/cli.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-a: add a |",
            ],
        );
        let ours = ours.trim_end_matches('\n').to_string();
        let theirs = spec(
            "6",
            &["src/cli.rs"],
            &[
                "| 2026-01-01 | base: initial |",
                "| 2026-11-05 | change-b: add b |",
            ],
        );
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(!merged.content.ends_with('\n'), "{:?}", merged.content);
    }

    // P2: a YAML comment on a block-list item is not part of the path.
    #[test]
    fn block_list_comment_is_not_part_of_the_item() {
        let base = spec("5", &["src/base.rs"], &[]);
        let ours = spec("5", &["src/base.rs # note", "src/a.rs"], &[]);
        let theirs = spec("5", &["src/base.rs", "src/b.rs"], &[]);
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert_eq!(
            merged.content.matches("src/base.rs").count(),
            1,
            "{}",
            merged.content
        );
    }

    // P2: the first requirements two branches add to an empty scaffold both land.
    #[test]
    fn first_ids_added_to_a_heading_free_scaffold_both_land() {
        let base = "---\nspec: cli.spec.md\n---\n\n# Requirements\n".to_string();
        let ours = format!("{base}\n### REQ-cli-001\n\nThe system SHALL start.\n");
        let theirs = format!("{base}\n### REQ-cli-002\n\nThe system SHALL stop.\n");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(merged.content.contains("### REQ-cli-001"));
        assert!(merged.content.contains("### REQ-cli-002"));
    }

    // P2: a section after the last requirement is not copied into each added block.
    #[test]
    fn trailing_section_is_not_duplicated_by_added_blocks() {
        let tail = "\n## Notes\n\nShared notes.\n";
        let base = format!("{}{tail}", requirements(&[]));
        let ours = base.replace(
            tail,
            &format!("\n### REQ-cli-002\n\nThe system SHALL stop.\n{tail}"),
        );
        let theirs = base.replace(
            tail,
            &format!("\n### REQ-cli-003\n\nThe system SHALL pause.\n{tail}"),
        );
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert!(
            merged.content.matches("Shared notes.").count() <= 1,
            "{}",
            merged.content
        );
        assert!(merged.content.contains("SHALL stop."), "{}", merged.content);
        assert!(
            merged.content.contains("SHALL pause."),
            "{}",
            merged.content
        );
    }

    // Review (2906dc30): the merged requirements file keeps the current side's final newline.
    #[test]
    fn requirements_union_keeps_the_current_final_newline() {
        let base = requirements(&[]);
        let ours = format!("{base}\n### REQ-cli-002\n\nThe system SHALL stop.\n");
        let theirs = format!("{base}\n### REQ-cli-003\n\nThe system SHALL pause.");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/requirements.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(merged.content.ends_with('\n'), "{:?}", merged.content);
        let merged = merge_contract(&base, &theirs, &ours, "specs/cli/requirements.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(!merged.content.ends_with('\n'), "{:?}", merged.content);
    }

    // Review (2906dc30): an ATX heading indented by up to three spaces is a requirement heading.
    #[test]
    fn indented_requirement_heading_is_a_block() {
        assert_eq!(
            requirement_heading_id("   ### REQ-cli-002"),
            Some("REQ-cli-002")
        );
        assert_eq!(requirement_heading_id("    ### REQ-cli-002"), None);
    }

    // Review (d2973cf1): a deleted-versus-edited frontmatter field shows the deletion.
    #[test]
    fn deleted_versus_edited_frontmatter_field_shows_the_deletion() {
        let base = spec("5", &["src/cli.rs"], &[])
            .replace("status: stable\n", "status: stable\nowner: alice\n");
        let ours = base.replace("owner: alice\n", "");
        let theirs = base.replace("owner: alice\n", "owner: bob\n");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(!merged.clean, "{}", merged.content);
        assert!(
            merged
                .content
                .contains("<<<<<<< ours\n\n=======\nowner: bob"),
            "{}",
            merged.content
        );
        assert!(
            !merged.content.contains("owner: alice"),
            "{}",
            merged.content
        );
        let merged = merge_contract(&base, &theirs, &ours, "specs/cli/cli.spec.md");
        assert!(
            merged
                .content
                .contains("owner: bob\n=======\n\n>>>>>>> theirs"),
            "{}",
            merged.content
        );
        assert!(
            !merged.content.contains("owner: alice"),
            "{}",
            merged.content
        );
    }

    // Review (d2973cf1): two first rows on an empty Version table do not merge clean
    // with the same version.
    #[test]
    fn first_version_rows_on_an_empty_table_are_not_left_equal() {
        let table = |rows: &str| {
            format!(
                "---\nmodule: cli\nversion: 1\nstatus: stable\nfiles:\n  - src/cli.rs\ndb_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Change Log\n\n| Version | Date | Change |\n|---------|------|--------|\n{rows}"
            )
        };
        let base = table("");
        let ours = table("| 1 | 2026-11-05 | change-a |\n");
        let theirs = table("| 1 | 2026-11-05 | change-b |\n");
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(
            !(merged.clean
                && merged.content.contains("| 1 | 2026-11-05 | change-a |")
                && merged.content.contains("| 1 | 2026-11-05 | change-b |")),
            "{}",
            merged.content
        );
    }

    // #872 (REQ-merge-006): a quoted list item that YAML would read as a number, boolean or
    // null is still a string after a union. Plain paths stay plain.
    #[test]
    fn yaml_typed_list_items_stay_strings_after_a_union() {
        let make = |files: &str, tables: &str, deps: &str| {
            format!(
                "---\nmodule: cli\nversion: 5\nstatus: stable\nfiles: {files}\ndb_tables: {tables}\ndepends_on: {deps}\n---\n\n# CLI\n\n## Purpose\n\nThe command line.\n"
            )
        };
        for typed in [
            "123",
            "1.5",
            "1e3",
            "0x1F",
            "0o17",
            ".inf",
            ".NaN",
            "true",
            "False",
            "null",
            "Null",
            "~",
            "yes",
            "No",
            "on",
            "OFF",
            "1_000",
            "1_000.5e+2",
            "+1_000.5",
            "0x_1F",
        ] {
            let base = make(&format!(r#"["src/cli.rs", "{typed}"]"#), "[]", "[]");
            let ours = make(
                &format!(r#"["src/cli.rs", "{typed}", "src/a.rs"]"#),
                "[]",
                "[]",
            );
            let theirs = make(
                &format!(r#"["src/cli.rs", "{typed}", "src/b.rs"]"#),
                "[]",
                "[]",
            );
            for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
                let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
                assert!(merged.clean, "{typed}: {}", merged.content);
                assert!(
                    merged.content.contains(&format!("  - \"{typed}\"\n")),
                    "`{typed}` must stay a quoted string: {}",
                    merged.content
                );
                assert!(
                    merged.content.contains("  - src/a.rs\n"),
                    "{}",
                    merged.content
                );
            }
        }
        let base = make(r#"["src/cli.rs"]"#, r#"["true"]"#, r#"["null"]"#);
        let ours = make(
            r#"["src/cli.rs"]"#,
            r#"["true", "users"]"#,
            r#"["null", "specs/a/a.spec.md"]"#,
        );
        let theirs = make(
            r#"["src/cli.rs"]"#,
            r#"["true", "orders"]"#,
            r#"["null", "specs/b/b.spec.md"]"#,
        );
        let merged = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        assert!(merged.clean, "{}", merged.content);
        assert!(
            merged
                .content
                .contains("db_tables:\n  - orders\n  - \"true\"\n  - users\n"),
            "{}",
            merged.content
        );
        assert!(
            merged.content.contains(
                "depends_on:\n  - \"null\"\n  - specs/a/a.spec.md\n  - specs/b/b.spec.md\n"
            ),
            "{}",
            merged.content
        );
    }

    // #895 (REQ-merge-004, MERGE-1): a Change Log section that opens with a stray, headerless
    // row before the real table still merges two appended rows cleanly, in either order: the
    // table's header is the row a separator follows, and the stray row stays as text before it.
    #[test]
    fn two_appended_rows_merge_after_a_stray_leading_change_log_row() {
        let make = |version: u64, rows: &str| {
            format!(
                "---\nmodule: cli\nversion: {version}\nstatus: stable\nfiles:\n  - src/cli.rs\ndb_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Purpose\n\nThe command line.\n\n## Change Log\n\n| 2026-07-30 | A stray row with no header |\n\n| Date | Change |\n|------|--------|\n| 2026-08-01 | initial |\n{rows}"
            )
        };
        let base = make(168, "");
        let ours = make(169, "| 2026-11-01 | ours |\n");
        let theirs = make(169, "| 2026-11-02 | theirs |\n");
        for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
            assert!(merged.clean, "{}", merged.content);
            assert!(
                merged.content.contains("version: 170\n"),
                "{}",
                merged.content
            );
            for row in [
                "| 2026-07-30 | A stray row with no header |\n",
                "| 2026-08-01 | initial |\n",
                "| 2026-11-01 | ours |\n",
                "| 2026-11-02 | theirs |\n",
            ] {
                assert_eq!(
                    merged.content.matches(row).count(),
                    1,
                    "{row:?}: {}",
                    merged.content
                );
            }
        }
    }

    // #875 (REQ-merge-006): the first blank line after the change-log rows ends the table.
    // Prose or a second table after it comes out byte-for-byte, and that table's rows are not
    // sorted into the change log.
    #[test]
    fn text_after_the_change_log_table_survives_a_row_union() {
        for after in [
            "\nNotes after the table.\n\n| Key | Value |\n|-----|-------|\n| z | 1 |\n| a | 2 |\n",
            "\n| Key | Value |\n|-----|-------|\n| z | 1 |\n| a | 2 |\n",
        ] {
            let with = |version: &str, rows: &[&str]| {
                format!("{}{after}", spec(version, &["src/cli.rs"], rows))
            };
            let base = with("5", &["| 2026-01-01 | base: initial |"]);
            let ours = with(
                "6",
                &[
                    "| 2026-01-01 | base: initial |",
                    "| 2026-11-05 | change-a: add a |",
                ],
            );
            let theirs = with(
                "6",
                &[
                    "| 2026-01-01 | base: initial |",
                    "| 2026-11-05 | change-b: add b |",
                ],
            );
            let expected = with(
                "7",
                &[
                    "| 2026-01-01 | base: initial |",
                    "| 2026-11-05 | change-a: add a |",
                    "| 2026-11-05 | change-b: add b |",
                ],
            );
            for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
                let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
                assert!(merged.clean, "{}", merged.content);
                assert_eq!(merged.content, expected);
            }
        }
    }

    // #876 (REQ-merge-006): when both sides bump `version:`, the comment after the number
    // merges three ways and the result does not depend on the merge order.
    #[test]
    fn version_comments_merge_three_ways_in_either_order() {
        let base = spec("5", &["src/cli.rs"], &["| 2026-01-01 | base: initial |"]);
        let bump = |value: &str| base.replace("version: 5\n", &format!("version: {value}\n"));

        // A comment added on one side only survives, whichever side it is on.
        let plain = bump("6");
        let noted = bump("6 # B");
        for (left, right) in [(&plain, &noted), (&noted, &plain)] {
            let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
            assert!(merged.clean, "{}", merged.content);
            assert!(
                merged.content.contains("version: 7 # B\n"),
                "{}",
                merged.content
            );
        }

        // The same comment on two otherwise different changes is kept, and both bumps count.
        let rows = ["| 2026-01-01 | base: initial |"];
        let ours = spec("6 # A", &["src/cli.rs", "src/a.rs"], &rows);
        let theirs = spec("6 # A", &["src/cli.rs", "src/b.rs"], &rows);
        for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
            assert!(merged.clean, "{}", merged.content);
            assert!(
                merged.content.contains("version: 7 # A\n"),
                "{}",
                merged.content
            );
        }

        // Two different comments are a conflict that keeps both, in either order. Like every
        // conflict the driver writes, each side sits under its own label (`ours` is the branch
        // merged into), so the two orders are mirror images: the same lines, halves swapped.
        let ours = bump("6 # A");
        let theirs = bump("6 # B");
        let forward = merge_contract(&base, &ours, &theirs, "specs/cli/cli.spec.md");
        let backward = merge_contract(&base, &theirs, &ours, "specs/cli/cli.spec.md");
        for merged in [&forward, &backward] {
            assert!(!merged.clean, "{}", merged.content);
            assert!(merged.content.contains("# A"), "{}", merged.content);
            assert!(merged.content.contains("# B"), "{}", merged.content);
        }
        assert!(
            forward.content.contains(
                "<<<<<<< ours\nversion: 6 # A\n=======\nversion: 6 # B\n>>>>>>> theirs\n"
            ),
            "{}",
            forward.content
        );
        assert_eq!(
            backward.content,
            forward.content.replace(
                "<<<<<<< ours\nversion: 6 # A\n=======\nversion: 6 # B\n>>>>>>> theirs\n",
                "<<<<<<< ours\nversion: 6 # B\n=======\nversion: 6 # A\n>>>>>>> theirs\n"
            )
        );
    }

    // #870 (REQ-merge-006): renumbering a new change-log `Version` cell past `u64::MAX` leaves
    // the table a conflict instead of panicking or wrapping to 0.
    #[test]
    fn changelog_version_renumbering_stops_at_u64_max() {
        let table = |rows: &str| {
            format!(
                "---\nmodule: cli\nversion: 2\nstatus: stable\nfiles:\n  - src/cli.rs\ndb_tables: []\ndepends_on: []\n---\n\n# CLI\n\n## Change Log\n\n| Version | Date | Change |\n|---------|------|--------|\n| {} | 2026-01-01 | initial |\n{rows}",
                u64::MAX
            )
        };
        let base = table("");
        let ours = table("| 1 | 2026-11-05 | change-a |\n");
        let theirs = table("| 1 | 2026-11-05 | change-b |\n");
        for (left, right) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge_contract(&base, left, right, "specs/cli/cli.spec.md");
            assert!(!merged.clean, "{}", merged.content);
            assert!(!merged.content.contains("| 0 |"), "{}", merged.content);
            assert!(merged.content.contains("change-a"), "{}", merged.content);
            assert!(merged.content.contains("change-b"), "{}", merged.content);
        }
    }
}
