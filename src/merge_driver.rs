//! Ancestor-aware merge for one module contract.
//!
//! Git invokes this as a merge driver (`specsync merge-file %O %A %B %P`) when
//! `.gitattributes` marks `*.spec.md` and `requirements.md`. The marker-based
//! resolver in `merge.rs` is a different path: it still takes `max(version)` and
//! still refuses prose, because a conflict hunk does not carry the ancestor.
//! This driver does, so two branches that each bumped the same module can land
//! together.

use std::collections::BTreeSet;
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

    let pre = merge_text_region(&base_pre, &ours_pre, &theirs_pre);
    let mut clean = pre.clean;
    let mut content = pre.content;

    let base_ids: BTreeSet<&str> = base_blocks.iter().map(|block| block.id.as_str()).collect();
    let mut order: Vec<String> = Vec::new();
    for block in &base_blocks {
        if !order.iter().any(|id| id == &block.id) {
            order.push(block.id.clone());
        }
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
            (None, Some(our), None) => content.push_str(&our.text),
            (None, None, Some(their)) => content.push_str(&their.text),
            (None, Some(our), Some(their)) if our.text == their.text => {
                content.push_str(&our.text);
            }
            // Both branches allocated this ID. Keep both bodies. `check` names the ID.
            (None, Some(our), Some(their)) => {
                content.push_str(&our.text);
                content.push_str(&their.text);
            }
            (Some(_), Some(our), Some(their)) if our.text == their.text => {
                content.push_str(&our.text);
            }
            (Some(base), Some(our), Some(their)) if our.text == base.text => {
                content.push_str(&their.text);
            }
            (Some(base), Some(our), Some(their)) if their.text == base.text => {
                content.push_str(&our.text);
            }
            (Some(base), Some(our), Some(their)) => {
                let merged = merge_text_region(&base.text, &our.text, &their.text);
                clean = clean && merged.clean;
                content.push_str(&merged.content);
            }
            (Some(_), None, None) => {}
            (Some(base), None, Some(their)) if their.text == base.text => {}
            (Some(base), Some(our), None) if our.text == base.text => {}
            (Some(base), None, Some(their)) => {
                clean = false;
                content.push_str(&conflict_block(&base.text, &their.text));
            }
            (Some(base), Some(our), None) => {
                clean = false;
                content.push_str(&conflict_block(&our.text, &base.text));
            }
            (None, None, None) => {}
        }
    }

    MergeContract { content, clean }
}

struct ReqBlock {
    id: String,
    text: String,
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
    let mut starts = Vec::new();
    for (index, (at, line)) in lines.iter().enumerate() {
        let bare = line.trim_end_matches(['\n', '\r']);
        if let Some(id) = requirement_heading_id(bare) {
            if preamble_end.is_none() {
                preamble_end = Some(*at);
            }
            starts.push((index, id.to_string()));
        }
    }
    let preamble_end = preamble_end?;
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
    let rest = line.strip_prefix("### ")?.trim();
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
        (Some(base), Some(our), Some(their)) if headers_match(&base, &our, &their) => {
            union_changelog_tables(&base, &our, &their)
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
        if trimmed.is_empty() {
            if lines[index + 1..]
                .iter()
                .all(|later| later.trim().is_empty())
            {
                suffix_at = index;
                break;
            }
            continue;
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
                next += 1;
                numbered.push(rewrite_row_cell(&row, index, &next.to_string()));
                changed = true;
            }
            if changed {
                assigned = Some(next);
            }
            rows = numbered;
        }
    }

    let suffix = merge_text_region(&base.suffix, &ours.suffix, &theirs.suffix);
    let mut content = ours.prefix.clone();
    for row in &rows {
        content.push_str(row);
        content.push('\n');
    }
    content.push_str(&suffix.content);
    ChangelogMerge {
        content,
        clean: suffix.clean,
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

fn table_cells(row: &str) -> Vec<String> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
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
    let Some(start) = lines
        .iter()
        .position(|line| line.trim_end_matches(['\n', '\r']) == "## Change Log")
    else {
        return (body.to_string(), String::new(), String::new());
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.trim_end_matches(['\n', '\r']).starts_with("## "))
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
        (Some(base), Some(ours), None) => FieldMerge::Conflict(conflict_block(
            &ours.lines.join("\n"),
            &base.lines.join("\n"),
        )),
        (Some(base), None, Some(theirs)) => FieldMerge::Conflict(conflict_block(
            &base.lines.join("\n"),
            &theirs.lines.join("\n"),
        )),
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
            lines.push(format!("  - {item}"));
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
    let line = format!("version: {}", rewrite_scalar(our_raw, &rendered));
    FieldMerge::Keep(line)
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
            items.push(line.trim().trim_start_matches("- ").trim().to_string());
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

fn parse_inline_list(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }
    Some(
        inner
            .split(',')
            .map(|item| item.trim().trim_matches(['\'', '"']).to_string())
            .filter(|item| !item.is_empty())
            .collect(),
    )
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
}
