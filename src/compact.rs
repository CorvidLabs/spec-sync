use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::validator::find_spec_files;

/// Result of compacting a single spec's changelog.
pub struct CompactResult {
    pub spec_path: String,
    #[allow(dead_code)]
    pub original_entries: usize,
    pub compacted_entries: usize,
    pub removed: usize,
}

/// Compact changelog entries across all specs.
/// Keeps the last `keep` entries and summarizes older ones.
pub fn compact_changelogs(
    root: &Path,
    specs_dir: &Path,
    keep: usize,
    dry_run: bool,
) -> Vec<CompactResult> {
    let spec_files = find_spec_files(specs_dir);
    let mut results = Vec::new();

    for spec_path in &spec_files {
        let content = match fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();

        if let Some((new_content, result)) = compact_spec_changelog(&content, &rel_path, keep)
            && result.removed > 0 {
                if !dry_run
                    && let Err(e) = fs::write(spec_path, &new_content) {
                        eprintln!(
                            "{} Failed to write {}: {e}",
                            "error:".red().bold(),
                            rel_path
                        );
                        continue;
                    }
                results.push(result);
            }
    }

    results
}

/// Compact the changelog in a single spec file's content.
/// Returns (new_content, result) if the changelog was found.
fn compact_spec_changelog(
    content: &str,
    rel_path: &str,
    keep: usize,
) -> Option<(String, CompactResult)> {
    // Find the ## Change Log section
    let changelog_marker = "## Change Log";
    let cl_start = content.find(changelog_marker)?;

    // Find where this section ends (next ## heading or EOF)
    let after_header = cl_start + changelog_marker.len();
    let section_end = content[after_header..]
        .find("\n## ")
        .map(|p| after_header + p)
        .unwrap_or(content.len());

    let section = &content[cl_start..section_end];
    let lines: Vec<&str> = section.lines().collect();

    // Find table rows: lines starting with | that are not header/separator.
    // The first two table lines are always header + separator; data rows follow.
    let mut header_lines: Vec<usize> = Vec::new();
    let mut data_rows: Vec<(usize, &str)> = Vec::new();
    let mut table_line_count = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        table_line_count += 1;
        // First two table lines are header row and separator row
        if table_line_count <= 2 {
            header_lines.push(i);
            continue;
        }
        // Data row
        data_rows.push((i, trimmed));
    }

    let total = data_rows.len();
    if total <= keep {
        return Some((
            content.to_string(),
            CompactResult {
                spec_path: rel_path.to_string(),
                original_entries: total,
                compacted_entries: total,
                removed: 0,
            },
        ));
    }

    // Keep the last `keep` entries, summarize the rest
    let to_remove = total - keep;
    let removed_rows = &data_rows[..to_remove];

    // Extract date range from removed entries
    let first_date = extract_first_cell(removed_rows.first().map(|(_, l)| *l).unwrap_or(""));
    let last_date = extract_first_cell(removed_rows.last().map(|(_, l)| *l).unwrap_or(""));

    // Detect column count from first data row
    let col_count = data_rows
        .first()
        .map(|(_, l)| l.matches('|').count().saturating_sub(1))
        .unwrap_or(2);

    let summary_row = if col_count >= 3 {
        format!("| {first_date} — {last_date} | — | Compacted: {to_remove} entries |")
    } else {
        format!("| {first_date} — {last_date} | Compacted: {to_remove} entries |")
    };

    // Build the indices to remove
    let remove_indices: std::collections::HashSet<usize> =
        removed_rows.iter().map(|(i, _)| *i).collect();

    // Reconstruct the section
    let mut new_lines: Vec<String> = Vec::new();
    let mut inserted_summary = false;

    for (i, line) in lines.iter().enumerate() {
        if remove_indices.contains(&i) {
            if !inserted_summary {
                new_lines.push(summary_row.clone());
                inserted_summary = true;
            }
            // Skip this line (it was compacted)
        } else {
            new_lines.push(line.to_string());
        }
    }

    let new_section = new_lines.join("\n");
    let mut new_content = String::new();
    new_content.push_str(&content[..cl_start]);
    new_content.push_str(&new_section);
    new_content.push_str(&content[section_end..]);

    Some((
        new_content,
        CompactResult {
            spec_path: rel_path.to_string(),
            original_entries: total,
            compacted_entries: keep + 1, // kept + summary
            removed: to_remove,
        },
    ))
}

/// Extract the first cell value from a markdown table row.
fn extract_first_cell(row: &str) -> String {
    let parts: Vec<&str> = row.split('|').collect();
    if parts.len() >= 2 {
        parts[1].trim().to_string()
    } else {
        "?".to_string()
    }
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

        let (new_content, result) = compact_spec_changelog(content, "test.spec.md", 3).unwrap();
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

        let (_, result) = compact_spec_changelog(content, "test.spec.md", 5).unwrap();
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

        let (new_content, result) = compact_spec_changelog(content, "test.spec.md", 1).unwrap();
        assert_eq!(result.removed, 2);
        assert!(new_content.contains("| — |")); // author placeholder
        assert!(new_content.contains("Compacted: 2 entries"));
    }
}
