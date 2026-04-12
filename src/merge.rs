use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process;

use crate::parser::parse_frontmatter;
use crate::validator::find_spec_files;

/// Result of merging a single spec file.
pub struct MergeResult {
    pub spec_path: String,
    pub status: MergeStatus,
    pub details: Vec<String>,
}

pub enum MergeStatus {
    /// File had conflicts and they were resolved automatically.
    Resolved,
    /// File had conflicts that require manual intervention.
    Manual,
    /// File had no conflicts.
    Clean,
}

/// Detect and resolve git merge conflicts in spec files.
/// Returns a list of results — one per conflicted spec file.
pub fn merge_specs(
    root: &Path,
    specs_dir: &Path,
    dry_run: bool,
    all_files: bool,
) -> Vec<MergeResult> {
    let conflicted = if all_files {
        // Scan all spec files for conflict markers
        let spec_files = find_spec_files(specs_dir);
        spec_files
            .into_iter()
            .filter(|p| {
                fs::read_to_string(p)
                    .map(|c| has_conflict_markers(&c))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    } else {
        // Use git to find conflicted spec files
        detect_conflicted_specs(root, specs_dir)
    };

    let mut results = Vec::new();

    for spec_path in &conflicted {
        let content = match fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(e) => {
                results.push(MergeResult {
                    spec_path: rel_path(root, spec_path),
                    status: MergeStatus::Manual,
                    details: vec![format!("Cannot read file: {e}")],
                });
                continue;
            }
        };

        let (resolved, result) = resolve_spec_conflicts(&content, &rel_path(root, spec_path));

        if !dry_run {
            if let MergeStatus::Resolved = &result.status {
                if let Err(e) = fs::write(spec_path, &resolved) {
                    results.push(MergeResult {
                        spec_path: rel_path(root, spec_path),
                        status: MergeStatus::Manual,
                        details: vec![format!("Cannot write file: {e}")],
                    });
                    continue;
                }
            }
        }

        results.push(result);
    }

    results
}

/// Check whether content contains git conflict markers.
pub fn has_conflict_markers(content: &str) -> bool {
    content.contains("\n<<<<<<< ") || content.starts_with("<<<<<<< ")
}

/// Use `git status` to find spec files with merge conflicts.
fn detect_conflicted_specs(root: &Path, specs_dir: &Path) -> Vec<std::path::PathBuf> {
    let output = process::Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let specs_rel = specs_dir
        .strip_prefix(root)
        .unwrap_or(specs_dir)
        .to_string_lossy();

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| l.starts_with(specs_rel.as_ref()) && l.ends_with(".md"))
        .map(|l| root.join(l))
        .collect()
}

/// Resolve conflicts in a single spec file.
/// Returns (resolved_content, merge_result).
fn resolve_spec_conflicts(content: &str, path: &str) -> (String, MergeResult) {
    let mut details = Vec::new();
    let mut all_resolved = true;

    // Split the file into regions: clean text and conflict blocks
    let regions = parse_conflict_regions(content);

    let mut output = String::new();

    for region in &regions {
        match region {
            Region::Clean(text) => output.push_str(text),
            Region::Conflict {
                ours,
                theirs,
                marker_label,
            } => {
                // Determine what section this conflict is in
                let section = detect_section(&output);

                match resolve_conflict(ours, theirs, &section) {
                    Resolution::Auto(merged) => {
                        details.push(format!(
                            "Auto-resolved in {}: {}",
                            section.as_deref().unwrap_or("unknown section"),
                            marker_label
                        ));
                        output.push_str(&merged);
                    }
                    Resolution::Manual => {
                        details.push(format!(
                            "Manual resolution needed in {}: {}",
                            section.as_deref().unwrap_or("unknown section"),
                            marker_label
                        ));
                        all_resolved = false;
                        // Preserve the conflict markers
                        output.push_str(&format!(
                            "<<<<<<< {marker_label}\n{ours}=======\n{theirs}>>>>>>> {marker_label}\n"
                        ));
                    }
                }
            }
        }
    }

    // If everything was resolved, validate the result parses
    if all_resolved
        && !output.is_empty()
        && parse_frontmatter(&output).is_none()
        && content.contains("---\n")
    {
        details.push("Warning: resolved file has invalid frontmatter".to_string());
    }

    let status = if !all_resolved {
        MergeStatus::Manual
    } else if details.is_empty() {
        MergeStatus::Clean
    } else {
        MergeStatus::Resolved
    };

    (
        output,
        MergeResult {
            spec_path: path.to_string(),
            status,
            details,
        },
    )
}

enum Region {
    Clean(String),
    Conflict {
        ours: String,
        theirs: String,
        marker_label: String,
    },
}

/// Parse content into clean regions and conflict blocks.
fn parse_conflict_regions(content: &str) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut clean_buf = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(label) = line.strip_prefix("<<<<<<< ") {
            // Flush clean buffer
            if !clean_buf.is_empty() {
                regions.push(Region::Clean(clean_buf.clone()));
                clean_buf.clear();
            }

            let marker_label = label.to_string();
            let mut ours = String::new();
            let mut theirs = String::new();
            let mut in_theirs = false;

            for inner_line in lines.by_ref() {
                if inner_line == "=======" {
                    in_theirs = true;
                } else if inner_line.starts_with(">>>>>>> ") {
                    break;
                } else if in_theirs {
                    theirs.push_str(inner_line);
                    theirs.push('\n');
                } else {
                    ours.push_str(inner_line);
                    ours.push('\n');
                }
            }

            regions.push(Region::Conflict {
                ours,
                theirs,
                marker_label,
            });
        } else {
            clean_buf.push_str(line);
            clean_buf.push('\n');
        }
    }

    if !clean_buf.is_empty() {
        regions.push(Region::Clean(clean_buf));
    }

    regions
}

/// Detect which markdown section the cursor is currently in,
/// based on the content already emitted.
fn detect_section(content_so_far: &str) -> Option<String> {
    // Find the last ## heading
    content_so_far
        .lines()
        .rev()
        .find(|l| l.starts_with("## "))
        .map(|l| l.trim_start_matches("## ").trim().to_string())
}

enum Resolution {
    Auto(String),
    Manual,
}

/// Try to auto-resolve a conflict based on section context.
fn resolve_conflict(ours: &str, theirs: &str, section: &Option<String>) -> Resolution {
    let section_name = section.as_deref().unwrap_or("");

    match section_name {
        // Changelog: merge rows chronologically
        "Change Log" => resolve_changelog_conflict(ours, theirs),

        // Frontmatter region (before any ## heading): merge frontmatter
        "" => resolve_frontmatter_conflict(ours, theirs),

        // Any section with only table rows: try table merge
        _ => {
            if is_pure_table_rows(ours) && is_pure_table_rows(theirs) {
                resolve_table_conflict(ours, theirs)
            } else {
                Resolution::Manual
            }
        }
    }
}

/// Check if text consists only of markdown table rows (lines starting with |).
fn is_pure_table_rows(text: &str) -> bool {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .all(|l| l.trim_start().starts_with('|'))
}

/// Merge changelog table rows by date (union, sorted chronologically).
fn resolve_changelog_conflict(ours: &str, theirs: &str) -> Resolution {
    let our_rows = parse_table_rows(ours);
    let their_rows = parse_table_rows(theirs);

    if our_rows.is_empty() && their_rows.is_empty() {
        return Resolution::Manual;
    }

    // Deduplicate by full row content, preserve chronological order
    let mut seen = HashSet::new();
    let mut all_rows: Vec<&str> = Vec::new();

    for row in our_rows.iter().chain(their_rows.iter()) {
        let normalized = row.trim();
        if seen.insert(normalized) {
            all_rows.push(row);
        }
    }

    // Sort by date (first cell) — dates in ISO format sort lexicographically
    all_rows.sort_by_key(|a| extract_first_cell(a));

    let merged = all_rows
        .iter()
        .map(|r| r.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Resolution::Auto(format!("{merged}\n"))
}

/// Merge generic table rows (union, deduplicated by first cell / key).
fn resolve_table_conflict(ours: &str, theirs: &str) -> Resolution {
    let our_rows = parse_table_rows(ours);
    let their_rows = parse_table_rows(theirs);

    if our_rows.is_empty() && their_rows.is_empty() {
        return Resolution::Manual;
    }

    // Deduplicate by first cell (e.g., symbol name)
    let mut seen = HashMap::new();
    let mut order = Vec::new();

    for row in our_rows.iter().chain(their_rows.iter()) {
        let key = extract_first_cell(row);
        if !seen.contains_key(&key) {
            order.push(key.clone());
        }
        // Theirs wins on conflict (latest change takes precedence)
        seen.insert(key, row.trim_end().to_string());
    }

    let merged = order
        .iter()
        .filter_map(|k| seen.get(k))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    Resolution::Auto(format!("{merged}\n"))
}

/// Merge frontmatter YAML fields.
/// Lists (files, depends_on, db_tables) are unioned.
/// Scalars: theirs wins if different.
fn resolve_frontmatter_conflict(ours: &str, theirs: &str) -> Resolution {
    // Parse both sides as YAML-like key-value pairs
    let our_fields = parse_yaml_fields(ours);
    let their_fields = parse_yaml_fields(theirs);

    if our_fields.is_empty() && their_fields.is_empty() {
        return Resolution::Manual;
    }

    let list_keys: HashSet<&str> = ["files", "db_tables", "depends_on"].into_iter().collect();

    let mut merged_lines = Vec::new();
    let mut handled = HashSet::new();

    // Process in order of our fields first, then any new fields from theirs
    let all_keys: Vec<String> = {
        let mut keys = Vec::new();
        for (k, _) in &our_fields {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        for (k, _) in &their_fields {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        keys
    };

    for key in &all_keys {
        if handled.contains(key.as_str()) {
            continue;
        }
        handled.insert(key.as_str());

        let our_val = our_fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);
        let their_val = their_fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);

        match (our_val, their_val) {
            (Some(YamlValue::List(a)), Some(YamlValue::List(b)))
                if list_keys.contains(key.as_str()) =>
            {
                // Union the lists
                let mut combined = a.clone();
                for item in b {
                    if !combined.contains(item) {
                        combined.push(item.clone());
                    }
                }
                combined.sort();
                if combined.is_empty() {
                    merged_lines.push(format!("{key}: []"));
                } else {
                    merged_lines.push(format!("{key}:"));
                    for item in &combined {
                        merged_lines.push(format!("  - {item}"));
                    }
                }
            }
            (_, Some(val)) => {
                // Theirs wins for scalars (or if only theirs has it)
                merged_lines.push(format_yaml_field(key, val));
            }
            (Some(val), None) => {
                merged_lines.push(format_yaml_field(key, val));
            }
            (None, None) => {}
        }
    }

    let result = merged_lines.join("\n");
    Resolution::Auto(format!("{result}\n"))
}

#[derive(Clone, Debug)]
enum YamlValue {
    Scalar(String),
    List(Vec<String>),
}

/// Simple YAML field parser (handles our zero-dep YAML subset).
fn parse_yaml_fields(text: &str) -> Vec<(String, YamlValue)> {
    let mut fields = Vec::new();
    let mut current_key: Option<String> = None;
    let mut current_list: Vec<String> = Vec::new();

    for line in text.lines() {
        if let Some(stripped) = line.trim_start().strip_prefix("- ") {
            if current_key.is_some() {
                current_list.push(stripped.trim().to_string());
            }
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key.is_empty() || key.contains(' ') {
                continue;
            }

            // Flush previous
            if let Some(prev_key) = current_key.take() {
                fields.push((prev_key, YamlValue::List(current_list.clone())));
                current_list.clear();
            }

            let value = line[colon_pos + 1..].trim();
            if value.is_empty() || value == "[]" {
                current_key = Some(key.to_string());
                current_list.clear();
            } else {
                fields.push((key.to_string(), YamlValue::Scalar(value.to_string())));
            }
        }
    }

    if let Some(prev_key) = current_key.take() {
        fields.push((prev_key, YamlValue::List(current_list)));
    }

    fields
}

fn format_yaml_field(key: &str, value: &YamlValue) -> String {
    match value {
        YamlValue::Scalar(s) => format!("{key}: {s}"),
        YamlValue::List(items) if items.is_empty() => format!("{key}: []"),
        YamlValue::List(items) => {
            let mut lines = vec![format!("{key}:")];
            for item in items {
                lines.push(format!("  - {item}"));
            }
            lines.join("\n")
        }
    }
}

/// Parse markdown table data rows from text (skip header/separator).
fn parse_table_rows(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with('|')
                && !t.starts_with("| -")
                && !t.starts_with("|--")
                && !t.starts_with("|-")
        })
        .collect()
}

/// Extract the first cell value from a markdown table row.
fn extract_first_cell(row: &str) -> String {
    let parts: Vec<&str> = row.split('|').collect();
    if parts.len() >= 2 {
        parts[1].trim().to_string()
    } else {
        String::new()
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Print merge results to stdout (text format).
pub fn print_results(results: &[MergeResult], dry_run: bool) {
    if results.is_empty() {
        println!("{}", "No spec files with merge conflicts found.".green());
        return;
    }

    let mut resolved_count = 0;
    let mut manual_count = 0;

    for r in results {
        match r.status {
            MergeStatus::Resolved => {
                resolved_count += 1;
                let verb = if dry_run { "would resolve" } else { "resolved" };
                println!("  {} {} {}", "✓".green(), verb.green(), r.spec_path.bold());
            }
            MergeStatus::Manual => {
                manual_count += 1;
                println!(
                    "  {} {} {}",
                    "✗".red(),
                    "needs manual merge:".red(),
                    r.spec_path.bold()
                );
            }
            MergeStatus::Clean => {}
        }

        for detail in &r.details {
            println!("    {detail}");
        }
    }

    println!();
    if resolved_count > 0 {
        let verb = if dry_run {
            "can be auto-resolved"
        } else {
            "auto-resolved"
        };
        println!(
            "{} {} spec file(s) {verb}.",
            "Summary:".bold(),
            resolved_count
        );
    }
    if manual_count > 0 {
        println!(
            "{} {} spec file(s) need manual resolution.",
            "Summary:".bold(),
            manual_count
        );
    }
}

/// Format results as JSON.
pub fn results_to_json(results: &[MergeResult]) -> String {
    let items: Vec<String> = results
        .iter()
        .map(|r| {
            let status = match r.status {
                MergeStatus::Resolved => "resolved",
                MergeStatus::Manual => "manual",
                MergeStatus::Clean => "clean",
            };
            let details_json: Vec<String> = r
                .details
                .iter()
                .map(|d| format!("\"{}\"", d.replace('\"', "\\\"")))
                .collect();
            format!(
                "    {{\"path\": \"{}\", \"status\": \"{}\", \"details\": [{}]}}",
                r.spec_path.replace('\"', "\\\""),
                status,
                details_json.join(", ")
            )
        })
        .collect();

    format!("{{\n  \"results\": [\n{}\n  ]\n}}", items.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_conflict_markers() {
        assert!(has_conflict_markers(
            "some text\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n"
        ));
        assert!(!has_conflict_markers("clean file\nno conflicts\n"));
    }

    #[test]
    fn test_parse_conflict_regions() {
        let content =
            "before\n<<<<<<< HEAD\nours line\n=======\ntheirs line\n>>>>>>> branch\nafter\n";
        let regions = parse_conflict_regions(content);
        assert_eq!(regions.len(), 3);
        match &regions[0] {
            Region::Clean(s) => assert_eq!(s, "before\n"),
            _ => panic!("expected Clean"),
        }
        match &regions[1] {
            Region::Conflict {
                ours,
                theirs,
                marker_label,
            } => {
                assert_eq!(ours, "ours line\n");
                assert_eq!(theirs, "theirs line\n");
                assert_eq!(marker_label, "HEAD");
            }
            _ => panic!("expected Conflict"),
        }
        match &regions[2] {
            Region::Clean(s) => assert_eq!(s, "after\n"),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn test_resolve_changelog_conflict() {
        let ours = "| 2026-01-01 | Added auth |\n| 2026-01-15 | Fixed login |\n";
        let theirs = "| 2026-01-01 | Added auth |\n| 2026-01-10 | Added signup |\n";

        match resolve_changelog_conflict(ours, theirs) {
            Resolution::Auto(merged) => {
                assert!(merged.contains("Added auth"));
                assert!(merged.contains("Fixed login"));
                assert!(merged.contains("Added signup"));
                // Check chronological order
                let lines: Vec<&str> = merged.lines().collect();
                assert_eq!(lines.len(), 3);
                assert!(lines[0].contains("2026-01-01"));
                assert!(lines[1].contains("2026-01-10"));
                assert!(lines[2].contains("2026-01-15"));
            }
            Resolution::Manual => panic!("expected auto resolution"),
        }
    }

    #[test]
    fn test_resolve_table_conflict() {
        let ours = "| `createAuth` | config: Config | Auth | Creates auth |\n";
        let theirs = "| `createAuth` | config: Config | Auth | Updated desc |\n| `validateToken` | token: string | bool | Validates |\n";

        match resolve_table_conflict(ours, theirs) {
            Resolution::Auto(merged) => {
                assert!(merged.contains("validateToken"));
                // theirs wins for createAuth
                assert!(merged.contains("Updated desc"));
                assert!(!merged.contains("Creates auth"));
            }
            Resolution::Manual => panic!("expected auto resolution"),
        }
    }

    #[test]
    fn test_resolve_frontmatter_conflict() {
        let ours =
            "module: auth\nversion: 2\nfiles:\n  - src/auth.ts\n  - src/login.ts\ndepends_on: []\n";
        let theirs = "module: auth\nversion: 3\nfiles:\n  - src/auth.ts\n  - src/signup.ts\ndepends_on: []\n";

        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Auto(merged) => {
                // Theirs wins for scalar (version)
                assert!(merged.contains("version: 3"));
                // Lists are unioned
                assert!(merged.contains("src/auth.ts"));
                assert!(merged.contains("src/login.ts"));
                assert!(merged.contains("src/signup.ts"));
            }
            Resolution::Manual => panic!("expected auto resolution"),
        }
    }

    #[test]
    fn test_full_spec_conflict_resolution() {
        let content = r#"---
<<<<<<< HEAD
module: auth
version: 2
status: active
files:
  - src/auth.ts
  - src/login.ts
db_tables: []
depends_on: []
=======
module: auth
version: 3
status: active
files:
  - src/auth.ts
  - src/signup.ts
db_tables: []
depends_on: []
>>>>>>> feature-branch
---

## Purpose

Auth module.

## Change Log

| Date | Change |
|------|--------|
<<<<<<< HEAD
| 2026-01-01 | Initial spec |
| 2026-01-15 | Added login |
=======
| 2026-01-01 | Initial spec |
| 2026-01-10 | Added signup |
>>>>>>> feature-branch
"#;

        let (resolved, result) = resolve_spec_conflicts(content, "specs/auth/auth.spec.md");
        assert!(matches!(result.status, MergeStatus::Resolved));
        assert!(!has_conflict_markers(&resolved));
        // Frontmatter: version 3 (theirs wins), files unioned
        assert!(resolved.contains("version: 3"));
        assert!(resolved.contains("src/login.ts"));
        assert!(resolved.contains("src/signup.ts"));
        // Changelog: all entries merged chronologically
        assert!(resolved.contains("Added login"));
        assert!(resolved.contains("Added signup"));
    }

    #[test]
    fn test_manual_fallback_for_prose() {
        let content = "## Purpose\n\n<<<<<<< HEAD\nThis is our purpose description.\n=======\nThis is their different purpose.\n>>>>>>> branch\n";
        let (resolved, result) = resolve_spec_conflicts(content, "test.spec.md");
        // Prose conflicts should remain for manual resolution
        assert!(matches!(result.status, MergeStatus::Manual));
        assert!(has_conflict_markers(&resolved));
    }

    #[test]
    fn test_parse_yaml_fields() {
        let yaml = "module: auth\nversion: 1\nfiles:\n  - src/a.ts\n  - src/b.ts\ndb_tables: []\n";
        let fields = parse_yaml_fields(yaml);
        assert_eq!(fields.len(), 4);
        assert!(matches!(&fields[0], (k, YamlValue::Scalar(v)) if k == "module" && v == "auth"));
        assert!(matches!(&fields[2], (k, YamlValue::List(v)) if k == "files" && v.len() == 2));
    }

    #[test]
    fn test_is_pure_table_rows() {
        assert!(is_pure_table_rows("| a | b |\n| c | d |\n"));
        assert!(!is_pure_table_rows("some text\n| a | b |\n"));
        assert!(is_pure_table_rows("| a | b |\n\n| c | d |\n"));
    }
}
