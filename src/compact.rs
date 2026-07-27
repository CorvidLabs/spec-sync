use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::validator::find_spec_files;

const SUMMARY_MARKER: &str = "<!-- specsync:compact:v1 -->";

/// Result of compacting a single spec's changelog.
#[derive(Debug)]
pub struct CompactResult {
    pub spec_path: String,
    #[allow(dead_code)]
    pub original_entries: usize,
    pub compacted_entries: usize,
    pub removed: usize,
    pub applied: bool,
}

/// A failed compact read, parse, staging, or publish operation.
pub struct CompactFailure {
    pub spec_path: String,
    pub operation: &'static str,
    pub message: String,
}

/// Complete outcome of a repository-wide compact invocation.
pub struct CompactReport {
    pub results: Vec<CompactResult>,
    pub failures: Vec<CompactFailure>,
    pub planned: usize,
    pub succeeded: usize,
}

impl CompactReport {
    pub fn complete(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn partial(&self) -> bool {
        self.succeeded > 0 && self.succeeded < self.planned
    }
}

struct CompactPlan {
    path: PathBuf,
    replacement: String,
    result: CompactResult,
}

/// Compact changelog entries across all specs.
/// Keeps the last `keep` entries and summarizes older ones.
pub fn compact_changelogs(
    root: &Path,
    specs_dir: &Path,
    keep: usize,
    dry_run: bool,
) -> CompactReport {
    let spec_files = find_spec_files(specs_dir);
    let mut plans = Vec::new();
    let mut failures = Vec::new();

    for spec_path in &spec_files {
        let rel_path = relative_display_path(root, spec_path);
        let content = match fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(error) => {
                failures.push(CompactFailure {
                    spec_path: rel_path,
                    operation: "read",
                    message: error.to_string(),
                });
                continue;
            }
        };

        match compact_spec_changelog(&content, &rel_path, keep) {
            Ok(Some((replacement, result))) if result.removed > 0 => {
                plans.push(CompactPlan {
                    path: spec_path.clone(),
                    replacement,
                    result,
                });
            }
            Ok(_) => {}
            Err(message) => failures.push(CompactFailure {
                spec_path: rel_path,
                operation: "parse",
                message,
            }),
        }
    }

    let planned = plans.len();
    if dry_run || !failures.is_empty() {
        return CompactReport {
            results: plans.into_iter().map(|plan| plan.result).collect(),
            failures,
            planned,
            succeeded: 0,
        };
    }

    let mut staged = Vec::with_capacity(planned);
    for plan in plans {
        match stage_replacement(&plan.path, plan.replacement.as_bytes()) {
            Ok(temporary) => staged.push((temporary, plan)),
            Err(error) => failures.push(CompactFailure {
                spec_path: plan.result.spec_path.clone(),
                operation: "stage",
                message: error.to_string(),
            }),
        }
    }

    if !failures.is_empty() {
        return CompactReport {
            results: staged.into_iter().map(|(_, plan)| plan.result).collect(),
            failures,
            planned,
            succeeded: 0,
        };
    }

    let mut results = Vec::with_capacity(planned);
    let mut succeeded = 0usize;
    for (temporary, mut plan) in staged {
        match temporary.persist(&plan.path) {
            Ok(_) => {
                plan.result.applied = true;
                succeeded += 1;
                results.push(plan.result);
            }
            Err(error) => {
                failures.push(CompactFailure {
                    spec_path: plan.result.spec_path.clone(),
                    operation: "publish",
                    message: error.error.to_string(),
                });
                results.push(plan.result);
            }
        }
    }

    CompactReport {
        results,
        failures,
        planned,
        succeeded,
    }
}

/// Compact the changelog in a single spec file's content.
/// Returns (new_content, result) if the changelog was found.
fn compact_spec_changelog(
    content: &str,
    rel_path: &str,
    keep: usize,
) -> Result<Option<(String, CompactResult)>, String> {
    // Find the ## Change Log section
    let changelog_marker = "## Change Log";
    let Some(cl_start) = content.find(changelog_marker) else {
        return Ok(None);
    };

    // Find where this section ends (next ## heading or EOF)
    let after_header = cl_start + changelog_marker.len();
    let section_end = content[after_header..]
        .find("\n## ")
        .map(|p| after_header + p)
        .unwrap_or(content.len());

    let section = &content[cl_start..section_end];
    let lines = source_lines(section);
    let Some(header_index) = lines
        .iter()
        .position(|line| line.content.trim().starts_with('|'))
    else {
        return Ok(None);
    };
    let header_cells = split_cells(lines[header_index].content.trim());
    if header_cells.len() < 2 {
        return Err("Change Log table header must contain at least two columns".to_string());
    }
    let separator_index = header_index + 1;
    let Some(separator) = lines.get(separator_index) else {
        return Err("Change Log table is missing its separator row".to_string());
    };
    let separator_cells = split_cells(separator.content.trim());
    if separator_cells.len() != header_cells.len()
        || !separator_cells.iter().all(|cell| is_separator_cell(cell))
    {
        return Err(format!(
            "Change Log separator has {} columns; expected {}",
            separator_cells.len(),
            header_cells.len()
        ));
    }

    // Only the first contiguous table is the changelog table. Later tables in
    // the section are prose and must remain byte-for-byte untouched.
    let mut data_rows: Vec<(usize, &str)> = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(separator_index + 1) {
        let trimmed = line.content.trim();
        if !trimmed.starts_with('|') {
            break;
        }
        let cells = split_cells(trimmed);
        if cells.len() != header_cells.len() {
            return Err(format!(
                "Change Log row {} has {} columns; expected {}",
                index + 1,
                cells.len(),
                header_cells.len()
            ));
        }
        data_rows.push((index, trimmed));
    }

    // #417: recognize our OWN compaction summary rows so a re-run replaces
    // (folds) them instead of re-compacting them as ordinary entries — the
    // accumulated count and range survive, making compact idempotent.
    let (summary_rows, entries): (Vec<_>, Vec<_>) =
        data_rows.into_iter().partition(|(_, l)| is_summary_row(l));
    if summary_rows.len() > 1 {
        return Err(
            "Change Log contains multiple SpecSync compaction summaries; refusing ambiguous folding"
                .to_string(),
        );
    }

    let old_count = summary_rows
        .iter()
        .filter_map(|(_, row)| summary_metadata(row))
        .map(|(count, _)| count)
        .next()
        .unwrap_or(0);
    let old_range_start = summary_rows
        .iter()
        .find_map(|(_, row)| summary_metadata(row).map(|(_, start)| start));

    let total = entries.len();
    if total <= keep {
        return Ok(Some((
            content.to_string(),
            CompactResult {
                spec_path: rel_path.to_string(),
                original_entries: total,
                compacted_entries: total,
                removed: 0,
                applied: false,
            },
        )));
    }

    // Keep the last `keep` entries, summarize the rest
    let to_remove = total - keep;
    let removed_rows = &entries[..to_remove];

    // Fold any previous summary into the new one: counts add, the range keeps
    // its original start and extends to the newest compacted entry.
    let new_count = old_count
        .checked_add(to_remove as u64)
        .ok_or_else(|| "SpecSync compaction summary count overflowed u64".to_string())?;
    let first_date = old_range_start
        .unwrap_or_else(|| extract_first_cell(removed_rows.first().map(|(_, l)| *l).unwrap_or("")));
    let last_date = extract_first_cell(removed_rows.last().map(|(_, l)| *l).unwrap_or(""));

    let col_count = header_cells.len();

    let summary_row = {
        let mut cells: Vec<String> = Vec::with_capacity(col_count);
        cells.push(format!("{first_date} — {last_date}"));
        for _ in 0..col_count.saturating_sub(2) {
            cells.push("—".to_string());
        }
        cells.push(format!(
            "Compacted: {new_count} {} {SUMMARY_MARKER}",
            if new_count == 1 { "entry" } else { "entries" }
        ));
        format!("| {} |", cells.join(" | "))
    };

    // Remove the compacted entries AND any previous summary row; insert the
    // new summary at the earliest removed position.
    let remove_indices: std::collections::HashSet<usize> = removed_rows
        .iter()
        .chain(summary_rows.iter())
        .map(|(i, _)| *i)
        .collect();
    let insert_at = remove_indices.iter().min().copied();

    // Reconstruct from inclusive source lines so every untouched line keeps
    // its original LF/CRLF terminator (including mixed-ending files).
    let mut new_section = String::with_capacity(section.len());
    for (i, line) in lines.iter().enumerate() {
        if remove_indices.contains(&i) {
            if Some(i) == insert_at {
                new_section.push_str(&summary_row);
                new_section.push_str(line.ending);
            }
            // Skip this line (it was compacted / an outdated summary)
        } else {
            new_section.push_str(line.raw);
        }
    }

    let mut new_content = String::new();
    new_content.push_str(&content[..cl_start]);
    new_content.push_str(&new_section);
    new_content.push_str(&content[section_end..]);

    Ok(Some((
        new_content,
        CompactResult {
            spec_path: rel_path.to_string(),
            original_entries: total,
            compacted_entries: keep, // entries actually kept (summary row excluded)
            removed: to_remove,
            applied: false,
        },
    )))
}

/// Whether a changelog table row is a spec-sync compaction summary row.
fn is_summary_row(row: &str) -> bool {
    summary_metadata(row).is_some()
}

/// Parse the metadata that uniquely identifies a spec-sync summary row.
///
/// Requiring both a non-empty range and an exact `Compacted: N entry|entries`
/// final cell prevents ordinary user-authored changelog text beginning with
/// `Compacted:` from being deleted as if it were tool-owned state.
fn summary_metadata(row: &str) -> Option<(u64, String)> {
    let cells = split_cells(row);
    if cells.len() < 2 || cells[1..cells.len() - 1].iter().any(|cell| cell != "—") {
        return None;
    }
    let (range_start, range_end) = cells.first()?.split_once(" — ")?;
    let range_start = range_start.trim();
    if range_start.is_empty() || range_end.trim().is_empty() {
        return None;
    }

    let mut summary_parts = cells
        .last()?
        .trim()
        .strip_suffix(SUMMARY_MARKER)?
        .trim_end()
        .strip_prefix("Compacted: ")?
        .split_whitespace();
    let count: u64 = summary_parts.next()?.parse().ok()?;
    let label = summary_parts.next()?;
    if (count == 1 && label != "entry")
        || (count != 1 && label != "entries")
        || summary_parts.next().is_some()
    {
        return None;
    }

    Some((count, range_start.to_string()))
}

/// Split a markdown table row into cells, honoring odd escaped backslash runs
/// and Markdown code spans containing pipes.
fn split_cells(row: &str) -> Vec<String> {
    let inner = row.trim();
    let mut cells = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut index = usize::from(chars.first() == Some(&'|'));
    let mut code_delimiter = None;
    let mut backslash_run = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '`' {
            let start = index;
            while index < chars.len() && chars[index] == '`' {
                index += 1;
            }
            let run = index - start;
            match code_delimiter {
                Some(delimiter) if delimiter == run => code_delimiter = None,
                None => code_delimiter = Some(run),
                _ => {}
            }
            cur.extend(std::iter::repeat_n('`', run));
            backslash_run = 0;
            continue;
        }

        if ch == '|' && code_delimiter.is_none() && backslash_run.is_multiple_of(2) {
            cells.push(cur.trim().to_string());
            cur.clear();
            backslash_run = 0;
            index += 1;
            continue;
        }

        cur.push(ch);
        backslash_run = if ch == '\\' { backslash_run + 1 } else { 0 };
        index += 1;
    }

    if !cur.trim().is_empty() {
        cells.push(cur.trim().to_string());
    }
    cells
}

/// Extract the first cell value from a markdown table row.
fn extract_first_cell(row: &str) -> String {
    split_cells(row)
        .into_iter()
        .next()
        .unwrap_or_else(|| "?".to_string())
}

struct SourceLine<'a> {
    raw: &'a str,
    content: &'a str,
    ending: &'a str,
}

fn source_lines(source: &str) -> Vec<SourceLine<'_>> {
    source
        .split_inclusive('\n')
        .map(|raw| {
            let without_lf = raw.strip_suffix('\n').unwrap_or(raw);
            let content = without_lf.strip_suffix('\r').unwrap_or(without_lf);
            SourceLine {
                raw,
                content,
                ending: &raw[content.len()..],
            }
        })
        .collect()
}

fn is_separator_cell(cell: &str) -> bool {
    let trimmed = cell.trim().trim_matches(':');
    trimmed.len() >= 3 && trimmed.chars().all(|character| character == '-')
}

fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn stage_replacement(path: &Path, replacement: &[u8]) -> io::Result<tempfile::NamedTempFile> {
    let metadata = fs::metadata(path)?;
    if metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "target is read-only",
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "target has no parent directory",
        )
    })?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".specsync-compact-")
        .tempfile_in(parent)?;
    temporary.write_all(replacement)?;
    temporary.as_file_mut().sync_all()?;
    temporary
        .as_file_mut()
        .set_permissions(metadata.permissions())?;
    Ok(temporary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_changelog() {
        let content = r#"---
module: test
version: 1
status: active
files:
  - src/test.rs
---

## Purpose

Test module.

## Change Log

| Date | Change |
|------|--------|
| 2026-01-01 | First |
| 2026-01-15 | Second |
| 2026-02-01 | Third |
| 2026-02-15 | Fourth |
| 2026-03-01 | Fifth |
"#;

        let (new_content, result) = compact_spec_changelog(content, "test.spec.md", 3)
            .unwrap()
            .unwrap();
        assert_eq!(result.original_entries, 5);
        assert_eq!(result.removed, 2);
        assert!(new_content.contains("Compacted: 2 entries"));
        assert!(new_content.contains("| 2026-02-01 | Third |"));
        assert!(new_content.contains("| 2026-03-01 | Fifth |"));
        assert!(!new_content.contains("| 2026-01-01 | First |"));
    }

    #[test]
    fn test_compact_no_change_needed() {
        let content = r#"## Change Log

| Date | Change |
|------|--------|
| 2026-03-01 | Only entry |
"#;

        let (_, result) = compact_spec_changelog(content, "test.spec.md", 5)
            .unwrap()
            .unwrap();
        assert_eq!(result.removed, 0);
    }

    #[test]
    fn test_compact_three_column_table() {
        let content = r#"## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-01-01 | alice | First |
| 2026-02-01 | bob | Second |
| 2026-03-01 | carol | Third |
"#;

        let (new_content, result) = compact_spec_changelog(content, "test.spec.md", 1)
            .unwrap()
            .unwrap();
        assert_eq!(result.removed, 2);
        assert!(new_content.contains("| — |")); // compacted author column
        assert!(new_content.contains("Compacted: 2 entries"));
    }

    // ── #417 regressions ────────────────────────────────────────────

    fn changelog_21() -> String {
        let mut s = String::from("## Change Log\n\n| Date | Change |\n|------|--------|\n");
        for i in 1..=21 {
            s.push_str(&format!("| 2026-01-{i:02} | Change number {i} |\n"));
        }
        s
    }

    #[test]
    fn compact_is_idempotent() {
        // #417 core regression: re-running compact must not re-compact its own
        // summary row — count and range survive, nothing changes.
        let (once, r1) = compact_spec_changelog(&changelog_21(), "t.spec.md", 5)
            .unwrap()
            .unwrap();
        assert_eq!(r1.removed, 16);
        assert!(once.contains("Compacted: 16 entries"));
        assert!(once.contains("| 2026-01-01 — 2026-01-16 |"));

        let (twice, r2) = compact_spec_changelog(&once, "t.spec.md", 5)
            .unwrap()
            .unwrap();
        assert_eq!(r2.removed, 0, "second run must be a no-op");
        assert_eq!(once, twice, "second run must not change the file");
    }

    #[test]
    fn compact_folds_existing_summary_when_more_entries_arrive() {
        let (once, _) = compact_spec_changelog(&changelog_21(), "t.spec.md", 5)
            .unwrap()
            .unwrap();
        // Two new entries land, pushing past --keep 5 again.
        let extended = once.replace(
            "| 2026-01-21 | Change number 21 |",
            "| 2026-01-21 | Change number 21 |\n| 2026-01-22 | Change number 22 |\n| 2026-01-23 | Change number 23 |",
        );
        let (twice, r2) = compact_spec_changelog(&extended, "t.spec.md", 5)
            .unwrap()
            .unwrap();
        assert_eq!(r2.removed, 2);
        assert!(twice.contains("Compacted: 18 entries"), "{twice}");
        assert!(twice.contains("| 2026-01-01 — 2026-01-18 |"), "{twice}");
        // Exactly one summary row.
        assert_eq!(twice.matches("Compacted: ").count(), 1, "{twice}");
    }

    #[test]
    fn compact_preserves_trailing_newline() {
        let content = changelog_21();
        assert!(content.ends_with('\n'));
        let (new_content, _) = compact_spec_changelog(&content, "t.spec.md", 5)
            .unwrap()
            .unwrap();
        assert!(new_content.ends_with('\n'), "trailing newline stripped");
    }

    #[test]
    fn compact_summary_row_matches_four_column_table() {
        let mut content = String::from(
            "## Change Log\n\n| Date | PR | Author | Change |\n|------|----|--------|--------|\n",
        );
        for i in 1..=8 {
            content.push_str(&format!("| 2026-01-{i:02} | #{i} | dev | Change {i} |\n"));
        }
        let (new_content, _) = compact_spec_changelog(&content, "t.spec.md", 3)
            .unwrap()
            .unwrap();
        let summary = new_content
            .lines()
            .find(|l| l.contains("Compacted: "))
            .unwrap();
        assert_eq!(split_cells(summary).len(), 4, "malformed row: {summary}");
    }

    #[test]
    fn compact_handles_escaped_pipes_in_cells() {
        let mut content = String::from("## Change Log\n\n| Date | Change |\n|------|--------|\n");
        for i in 1..=6 {
            content.push_str(&format!("| 2026-01-{i:02} | Change {i} |\n"));
        }
        content.push_str("| 2026-02-01 | Uses a \\| b syntax |\n");
        let (new_content, r) = compact_spec_changelog(&content, "t.spec.md", 2)
            .unwrap()
            .unwrap();
        assert_eq!(r.removed, 5);
        assert!(
            new_content.contains("Uses a \\| b syntax"),
            "escaped-pipe cell truncated/dropped:\n{new_content}"
        );
        assert!(
            new_content.contains("2026-01-01 — 2026-01-05"),
            "{new_content}"
        );
    }

    #[test]
    fn split_cells_preserves_one_escaped_pipe() {
        let cells = split_cells("| 2026-01-01 | Uses a \\| b syntax |");
        assert_eq!(cells, ["2026-01-01", "Uses a \\| b syntax"]);
    }

    #[test]
    fn compact_does_not_treat_user_text_as_a_summary_row() {
        let mut content = String::from("## Change Log\n\n| Date | Change |\n|------|--------|\n");
        for i in 1..=6 {
            content.push_str(&format!("| 2026-01-{i:02} | Change {i} |\n"));
        }
        content.push_str("| 2026-01-07 | Compacted: manual migration notes |\n");

        let (new_content, result) = compact_spec_changelog(&content, "t.spec.md", 2)
            .unwrap()
            .unwrap();

        assert_eq!(result.removed, 5);
        assert!(
            new_content.contains("| 2026-01-07 | Compacted: manual migration notes |"),
            "a user-authored recent row was mistaken for a tool summary:\n{new_content}"
        );
    }

    #[test]
    fn compact_requires_summary_placeholders_in_wide_tables() {
        let mut content = String::from(
            "## Change Log\n\n| Date | Author | Change |\n|------|--------|--------|\n",
        );
        for i in 1..=6 {
            content.push_str(&format!("| 2026-01-{i:02} | dev | Change {i} |\n"));
        }
        content.push_str("| 2026-01-01 — 2026-01-07 | maintainer | Compacted: 2 entries |\n");

        let (new_content, result) = compact_spec_changelog(&content, "t.spec.md", 2)
            .unwrap()
            .unwrap();

        assert_eq!(result.removed, 5);
        assert!(
            new_content.contains("| 2026-01-01 — 2026-01-07 | maintainer | Compacted: 2 entries |"),
            "a user-authored wide row was mistaken for a tool summary:\n{new_content}"
        );
    }

    #[test]
    fn compact_uses_singular_for_one_entry() {
        let content = changelog_21();
        let (new_content, r) = compact_spec_changelog(&content, "t.spec.md", 20)
            .unwrap()
            .unwrap();
        assert_eq!(r.removed, 1);
        assert!(
            new_content.contains(&format!("Compacted: 1 entry {SUMMARY_MARKER} |")),
            "{new_content}"
        );
        assert!(!new_content.contains("1 entries"));
    }

    #[test]
    fn compact_reports_kept_entries_not_summary_row() {
        // #417: --keep 5 used to report "(kept 6)" because the summary row was
        // counted as a kept entry.
        let (_, r) = compact_spec_changelog(&changelog_21(), "t.spec.md", 5)
            .unwrap()
            .unwrap();
        assert_eq!(r.compacted_entries, 5);
    }

    #[test]
    fn compact_preserves_exact_shape_user_row_without_marker() {
        let mut content = String::from("## Change Log\n\n| Date | Change |\n|------|--------|\n");
        for i in 1..=6 {
            content.push_str(&format!("| 2026-01-{i:02} | Change {i} |\n"));
        }
        content.push_str("| 2025-01-01 — 2025-01-31 | Compacted: 2 entries |\n");

        let (new_content, result) = compact_spec_changelog(&content, "t.spec.md", 2)
            .unwrap()
            .unwrap();

        assert_eq!(result.removed, 5);
        assert!(
            new_content.contains("| 2025-01-01 — 2025-01-31 | Compacted: 2 entries |"),
            "an unmarked user row was claimed as tool state:\n{new_content}"
        );
        assert!(new_content.contains(SUMMARY_MARKER));
    }

    #[test]
    fn compact_rejects_multiple_marked_summaries() {
        let marker = SUMMARY_MARKER;
        let content = format!(
            "## Change Log\n\n| Date | Change |\n|------|--------|\n\
             | 2026-01-01 — 2026-01-02 | Compacted: 2 entries {marker} |\n\
             | 2026-01-03 — 2026-01-04 | Compacted: 2 entries {marker} |\n\
             | 2026-01-05 | Fifth |\n"
        );

        let error = compact_spec_changelog(&content, "t.spec.md", 0).unwrap_err();
        assert!(error.contains("multiple SpecSync compaction summaries"));
    }

    #[test]
    fn compact_preserves_crlf_and_mixed_line_endings() {
        let content = concat!(
            "## Change Log\r\n",
            "\r\n",
            "| Date | Change |\r\n",
            "|------|--------|\r\n",
            "| 2026-01-01 | First |\r\n",
            "| 2026-01-02 | Second |\n",
            "| 2026-01-03 | Third |\r\n",
            "\r\n",
            "## Next\r\n",
            "untouched\n",
        );

        let (new_content, _) = compact_spec_changelog(content, "t.spec.md", 1)
            .unwrap()
            .unwrap();

        assert!(new_content.starts_with("## Change Log\r\n\r\n"));
        assert!(new_content.contains(SUMMARY_MARKER));
        assert!(new_content.contains("| 2026-01-03 | Third |\r\n\r\n## Next\r\nuntouched\n"));
        assert!(!new_content.contains("| 2026-01-02 | Second |\n"));
    }

    #[test]
    fn split_cells_honors_backslash_parity_and_code_spans() {
        assert_eq!(
            split_cells(r"| date | one \| pipe |"),
            ["date", r"one \| pipe"]
        );
        assert_eq!(
            split_cells(r"| date | two \\| columns |"),
            ["date", r"two \\", "columns"]
        );
        assert_eq!(
            split_cells("| date | `code | pipe` |"),
            ["date", "`code | pipe`"]
        );
        assert_eq!(
            split_cells("| date | ``code ` | pipe`` |"),
            ["date", "``code ` | pipe``"]
        );
    }

    #[test]
    fn compact_ignores_secondary_tables_after_changelog_table() {
        let content = concat!(
            "## Change Log\n\n",
            "| Date | Change |\n",
            "|------|--------|\n",
            "| 2026-01-01 | First |\n",
            "| 2026-01-02 | Second |\n",
            "| 2026-01-03 | Third |\n",
            "\n",
            "| Key | Value |\n",
            "|-----|-------|\n",
            "| A | B |\n",
        );

        let (new_content, result) = compact_spec_changelog(content, "t.spec.md", 1)
            .unwrap()
            .unwrap();

        assert_eq!(result.removed, 2);
        assert!(new_content.contains("| Key | Value |\n|-----|-------|\n| A | B |\n"));
    }

    #[test]
    fn compact_supports_keep_zero() {
        let (new_content, result) = compact_spec_changelog(&changelog_21(), "t.spec.md", 0)
            .unwrap()
            .unwrap();

        assert_eq!(result.removed, 21);
        assert_eq!(result.compacted_entries, 0);
        assert_eq!(new_content.matches(SUMMARY_MARKER).count(), 1);
        assert!(!new_content.contains("| 2026-01-21 | Change number 21 |"));
    }

    #[test]
    fn compact_rejects_summary_count_overflow() {
        let content = format!(
            "## Change Log\n\n| Date | Change |\n|------|--------|\n\
             | 2026-01-01 — 2026-01-02 | Compacted: {} entries {} |\n\
             | 2026-01-03 | Third |\n",
            u64::MAX,
            SUMMARY_MARKER
        );

        let error = compact_spec_changelog(&content, "t.spec.md", 0).unwrap_err();
        assert!(error.contains("overflowed u64"));
    }

    #[cfg(unix)]
    #[test]
    fn compact_preflight_failure_prevents_all_writes() {
        use std::os::unix::fs::PermissionsExt;

        let temporary = tempfile::tempdir().unwrap();
        let specs = temporary.path().join("specs");
        fs::create_dir_all(specs.join("a")).unwrap();
        fs::create_dir_all(specs.join("b")).unwrap();
        let first = specs.join("a/a.spec.md");
        let second = specs.join("b/b.spec.md");
        let content = changelog_21();
        fs::write(&first, &content).unwrap();
        fs::write(&second, &content).unwrap();
        let mut permissions = fs::metadata(&second).unwrap().permissions();
        permissions.set_mode(0o444);
        fs::set_permissions(&second, permissions).unwrap();

        let report = compact_changelogs(temporary.path(), &specs, 5, false);

        assert!(!report.complete());
        assert_eq!(report.succeeded, 0);
        assert_eq!(fs::read_to_string(&first).unwrap(), content);
        assert_eq!(fs::read_to_string(&second).unwrap(), content);
    }
}
