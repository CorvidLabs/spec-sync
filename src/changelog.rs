//! Automated changelog generation for spec changes between git refs.
//!
//! Compares specs at two git commits/tags and produces a structured diff
//! showing which specs were added, removed, or modified — and which specific
//! fields changed for modified specs.

use crate::parser::parse_frontmatter;
use crate::types::Frontmatter;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

// ─── Types ──────────────────────────────────────────────────────────────

/// A single field change within a modified spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// A spec that was modified between two refs.
#[derive(Debug, Clone)]
pub struct ModifiedSpec {
    pub module: String,
    pub spec_path: String,
    pub changes: Vec<FieldChange>,
}

/// The full changelog comparing two git refs.
#[derive(Debug, Clone)]
pub struct ChangelogReport {
    pub from_ref: String,
    pub to_ref: String,
    pub added: Vec<SpecEntry>,
    pub removed: Vec<SpecEntry>,
    pub modified: Vec<ModifiedSpec>,
}

/// A spec entry (used for added/removed lists).
#[derive(Debug, Clone)]
pub struct SpecEntry {
    pub module: String,
    pub spec_path: String,
    pub status: Option<String>,
    pub version: Option<String>,
}

// ─── Git Helpers ────────────────────────────────────────────────────────

/// List spec files tracked by git at a given ref.
fn list_specs_at_ref(root: &Path, git_ref: &str, specs_dir: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", git_ref, "--", specs_dir])
        .current_dir(root)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.ends_with(".spec.md"))
            .map(|l| l.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Read a file's contents at a given git ref.
fn read_file_at_ref(root: &Path, git_ref: &str, file_path: &str) -> Option<String> {
    let spec = format!("{git_ref}:{file_path}");
    let output = Command::new("git")
        .args(["show", &spec])
        .current_dir(root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"))
    } else {
        None
    }
}

/// Parse a range string like "v0.1..v0.2" or "HEAD~5..HEAD" into (from, to).
pub fn parse_range(range: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = range.splitn(2, "..").collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

// ─── Comparison Logic ──────────────────────────────────────────────────

/// Parse a spec file content into a SpecEntry.
fn spec_entry_from_content(spec_path: &str, content: &str) -> Option<SpecEntry> {
    let parsed = parse_frontmatter(content)?;
    Some(SpecEntry {
        module: parsed
            .frontmatter
            .module
            .unwrap_or_else(|| module_from_path(spec_path)),
        spec_path: spec_path.to_string(),
        status: parsed.frontmatter.status,
        version: parsed.frontmatter.version,
    })
}

/// Extract a module name from a spec file path.
/// `specs/auth/auth.spec.md` -> `auth`
fn module_from_path(path: &str) -> String {
    let p = Path::new(path);
    p.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .strip_suffix(".spec")
        .unwrap_or(p.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown"))
        .to_string()
}

/// Compare frontmatter fields between two versions of a spec.
fn compare_frontmatter(old: &Frontmatter, new: &Frontmatter) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    // Compare status
    if old.status != new.status {
        changes.push(FieldChange {
            field: "status".to_string(),
            old_value: old.status.clone().unwrap_or_default(),
            new_value: new.status.clone().unwrap_or_default(),
        });
    }

    // Compare version
    if old.version != new.version {
        changes.push(FieldChange {
            field: "version".to_string(),
            old_value: old.version.clone().unwrap_or_default(),
            new_value: new.version.clone().unwrap_or_default(),
        });
    }

    // Compare module name
    if old.module != new.module {
        changes.push(FieldChange {
            field: "module".to_string(),
            old_value: old.module.clone().unwrap_or_default(),
            new_value: new.module.clone().unwrap_or_default(),
        });
    }

    // Compare files
    let old_files: BTreeSet<&str> = old.files.iter().map(|s| s.as_str()).collect();
    let new_files: BTreeSet<&str> = new.files.iter().map(|s| s.as_str()).collect();
    if old_files != new_files {
        changes.push(FieldChange {
            field: "files".to_string(),
            old_value: format_list(&old.files),
            new_value: format_list(&new.files),
        });
    }

    // Compare db_tables
    let old_tables: BTreeSet<&str> = old.db_tables.iter().map(|s| s.as_str()).collect();
    let new_tables: BTreeSet<&str> = new.db_tables.iter().map(|s| s.as_str()).collect();
    if old_tables != new_tables {
        changes.push(FieldChange {
            field: "db_tables".to_string(),
            old_value: format_list(&old.db_tables),
            new_value: format_list(&new.db_tables),
        });
    }

    // Compare depends_on
    let old_deps: BTreeSet<&str> = old.depends_on.iter().map(|s| s.as_str()).collect();
    let new_deps: BTreeSet<&str> = new.depends_on.iter().map(|s| s.as_str()).collect();
    if old_deps != new_deps {
        changes.push(FieldChange {
            field: "depends_on".to_string(),
            old_value: format_list(&old.depends_on),
            new_value: format_list(&new.depends_on),
        });
    }

    // Compare agent_policy
    if old.agent_policy != new.agent_policy {
        changes.push(FieldChange {
            field: "agent_policy".to_string(),
            old_value: old.agent_policy.clone().unwrap_or_default(),
            new_value: new.agent_policy.clone().unwrap_or_default(),
        });
    }

    // Compare implements
    if old.implements != new.implements {
        changes.push(FieldChange {
            field: "implements".to_string(),
            old_value: format_u64_list(&old.implements),
            new_value: format_u64_list(&new.implements),
        });
    }

    // Compare tracks
    if old.tracks != new.tracks {
        changes.push(FieldChange {
            field: "tracks".to_string(),
            old_value: format_u64_list(&old.tracks),
            new_value: format_u64_list(&new.tracks),
        });
    }

    changes
}

/// Compare the body sections of two specs (detect section-level changes).
fn compare_sections(old_body: &str, new_body: &str) -> Vec<FieldChange> {
    let old_sections = extract_sections(old_body);
    let new_sections = extract_sections(new_body);

    let mut changes = Vec::new();

    // Find modified and removed sections
    for (name, old_content) in &old_sections {
        match new_sections.get(name) {
            Some(new_content) if new_content != old_content => {
                changes.push(FieldChange {
                    field: format!("section:{name}"),
                    old_value: "(modified)".to_string(),
                    new_value: "(modified)".to_string(),
                });
            }
            None => {
                changes.push(FieldChange {
                    field: format!("section:{name}"),
                    old_value: "(present)".to_string(),
                    new_value: "(removed)".to_string(),
                });
            }
            _ => {}
        }
    }

    // Find added sections
    for name in new_sections.keys() {
        if !old_sections.contains_key(name) {
            changes.push(FieldChange {
                field: format!("section:{name}"),
                old_value: "(absent)".to_string(),
                new_value: "(added)".to_string(),
            });
        }
    }

    changes
}

/// Extract ## sections from markdown body into a map.
fn extract_sections(body: &str) -> BTreeMap<String, String> {
    let mut sections = BTreeMap::new();
    let mut current_name: Option<String> = None;
    let mut current_content = String::new();

    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Don't match ### or deeper
            if !heading.starts_with('#') {
                // Flush previous section
                if let Some(name) = current_name.take() {
                    sections.insert(name, current_content.trim().to_string());
                }
                current_name = Some(heading.trim().to_string());
                current_content = String::new();
                continue;
            }
        }
        if current_name.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Flush last section
    if let Some(name) = current_name {
        sections.insert(name, current_content.trim().to_string());
    }

    sections
}

fn format_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        items.join(", ")
    }
}

fn format_u64_list(items: &[u64]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        items
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// ─── Main Entry Point ──────────────────────────────────────────────────

/// Generate a changelog comparing specs between two git refs.
pub fn generate_changelog(
    root: &Path,
    specs_dir: &str,
    from_ref: &str,
    to_ref: &str,
) -> ChangelogReport {
    let old_specs = list_specs_at_ref(root, from_ref, specs_dir);
    let new_specs = list_specs_at_ref(root, to_ref, specs_dir);

    let old_set: BTreeSet<&str> = old_specs.iter().map(|s| s.as_str()).collect();
    let new_set: BTreeSet<&str> = new_specs.iter().map(|s| s.as_str()).collect();

    // Added specs: in new but not in old
    let mut added = Vec::new();
    for path in &new_specs {
        if !old_set.contains(path.as_str())
            && let Some(content) = read_file_at_ref(root, to_ref, path)
            && let Some(entry) = spec_entry_from_content(path, &content)
        {
            added.push(entry);
        }
    }

    // Removed specs: in old but not in new
    let mut removed = Vec::new();
    for path in &old_specs {
        if !new_set.contains(path.as_str())
            && let Some(content) = read_file_at_ref(root, from_ref, path)
            && let Some(entry) = spec_entry_from_content(path, &content)
        {
            removed.push(entry);
        }
    }

    // Modified specs: in both, but content changed
    let mut modified = Vec::new();
    for path in &new_specs {
        if !old_set.contains(path.as_str()) {
            continue;
        }

        let old_content = match read_file_at_ref(root, from_ref, path) {
            Some(c) => c,
            None => continue,
        };
        let new_content = match read_file_at_ref(root, to_ref, path) {
            Some(c) => c,
            None => continue,
        };

        // Skip if content is identical
        if old_content == new_content {
            continue;
        }

        let old_parsed = parse_frontmatter(&old_content);
        let new_parsed = parse_frontmatter(&new_content);

        let (old_fm, old_body) = match &old_parsed {
            Some(p) => (&p.frontmatter, p.body.as_str()),
            None => continue,
        };
        let (new_fm, new_body) = match &new_parsed {
            Some(p) => (&p.frontmatter, p.body.as_str()),
            None => continue,
        };

        let mut changes = compare_frontmatter(old_fm, new_fm);
        changes.extend(compare_sections(old_body, new_body));

        if !changes.is_empty() {
            let module = new_fm
                .module
                .clone()
                .unwrap_or_else(|| module_from_path(path));
            modified.push(ModifiedSpec {
                module,
                spec_path: path.clone(),
                changes,
            });
        }
    }

    // Sort for deterministic output
    added.sort_by(|a, b| a.module.cmp(&b.module));
    removed.sort_by(|a, b| a.module.cmp(&b.module));
    modified.sort_by(|a, b| a.module.cmp(&b.module));

    ChangelogReport {
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        added,
        removed,
        modified,
    }
}

// ─── Formatting ────────────────────────────────────────────────────────

/// Format the changelog as plain text.
pub fn format_text(report: &ChangelogReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Spec Changelog: {}..{}\n",
        report.from_ref, report.to_ref
    ));
    out.push_str(&format!(
        "{}\n\n",
        "=".repeat(40 + report.from_ref.len() + report.to_ref.len())
    ));

    let total = report.added.len() + report.removed.len() + report.modified.len();
    if total == 0 {
        out.push_str("No spec changes detected.\n");
        return out;
    }

    out.push_str(&format!(
        "Summary: {} added, {} changed, {} removed\n\n",
        report.added.len(),
        report.modified.len(),
        report.removed.len()
    ));

    if !report.added.is_empty() {
        out.push_str("Added\n-----\n");
        for entry in &report.added {
            let status = entry.status.as_deref().unwrap_or("unknown");
            let version = entry.version.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "  + {} (status: {}, version: {})\n    {}\n",
                entry.module, status, version, entry.spec_path
            ));
        }
        out.push('\n');
    }

    if !report.modified.is_empty() {
        out.push_str("Changed\n-------\n");
        for spec in &report.modified {
            out.push_str(&format!("  ~ {} ({})\n", spec.module, spec.spec_path));
            for change in &spec.changes {
                if change.field.starts_with("section:") {
                    let section = change
                        .field
                        .strip_prefix("section:")
                        .unwrap_or(&change.field);
                    if change.new_value == "(added)" {
                        out.push_str(&format!("      + section \"{section}\" added\n"));
                    } else if change.new_value == "(removed)" {
                        out.push_str(&format!("      - section \"{section}\" removed\n"));
                    } else {
                        out.push_str(&format!("      ~ section \"{section}\" modified\n"));
                    }
                } else {
                    out.push_str(&format!(
                        "      {} : \"{}\" -> \"{}\"\n",
                        change.field, change.old_value, change.new_value
                    ));
                }
            }
        }
        out.push('\n');
    }

    if !report.removed.is_empty() {
        out.push_str("Removed\n-------\n");
        for entry in &report.removed {
            let status = entry.status.as_deref().unwrap_or("unknown");
            out.push_str(&format!(
                "  - {} (status: {})\n    {}\n",
                entry.module, status, entry.spec_path
            ));
        }
        out.push('\n');
    }

    out
}

/// Format the changelog as JSON.
pub fn format_json(report: &ChangelogReport) -> String {
    let json = serde_json::json!({
        "from_ref": report.from_ref,
        "to_ref": report.to_ref,
        "summary": {
            "added": report.added.len(),
            "changed": report.modified.len(),
            "removed": report.removed.len(),
        },
        "added": report.added.iter().map(|e| serde_json::json!({
            "module": e.module,
            "spec_path": e.spec_path,
            "status": e.status,
            "version": e.version,
        })).collect::<Vec<_>>(),
        "changed": report.modified.iter().map(|m| serde_json::json!({
            "module": m.module,
            "spec_path": m.spec_path,
            "changes": m.changes.iter().map(|c| serde_json::json!({
                "field": c.field,
                "old_value": c.old_value,
                "new_value": c.new_value,
            })).collect::<Vec<serde_json::Value>>(),
        })).collect::<Vec<_>>(),
        "removed": report.removed.iter().map(|e| serde_json::json!({
            "module": e.module,
            "spec_path": e.spec_path,
            "status": e.status,
            "version": e.version,
        })).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

/// Format the changelog as markdown (Keep-a-Changelog style).
pub fn format_markdown(report: &ChangelogReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "## Spec Changelog: `{}`..`{}`\n\n",
        report.from_ref, report.to_ref
    ));

    let total = report.added.len() + report.removed.len() + report.modified.len();
    if total == 0 {
        out.push_str("No spec changes detected.\n");
        return out;
    }

    out.push_str(&format!(
        "**{}** added, **{}** changed, **{}** removed\n\n",
        report.added.len(),
        report.modified.len(),
        report.removed.len()
    ));

    if !report.added.is_empty() {
        out.push_str("### Added\n\n");
        for entry in &report.added {
            let status = entry.status.as_deref().unwrap_or("unknown");
            let version = entry.version.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "- **{}** (status: `{}`, version: `{}`)\n  - `{}`\n",
                entry.module, status, version, entry.spec_path
            ));
        }
        out.push('\n');
    }

    if !report.modified.is_empty() {
        out.push_str("### Changed\n\n");
        for spec in &report.modified {
            out.push_str(&format!("- **{}** (`{}`)\n", spec.module, spec.spec_path));
            for change in &spec.changes {
                if change.field.starts_with("section:") {
                    let section = change
                        .field
                        .strip_prefix("section:")
                        .unwrap_or(&change.field);
                    if change.new_value == "(added)" {
                        out.push_str(&format!("  - Section \"{}\" added\n", section));
                    } else if change.new_value == "(removed)" {
                        out.push_str(&format!("  - Section \"{}\" removed\n", section));
                    } else {
                        out.push_str(&format!("  - Section \"{}\" modified\n", section));
                    }
                } else {
                    out.push_str(&format!(
                        "  - `{}`: `{}` -> `{}`\n",
                        change.field, change.old_value, change.new_value
                    ));
                }
            }
        }
        out.push('\n');
    }

    if !report.removed.is_empty() {
        out.push_str("### Removed\n\n");
        for entry in &report.removed {
            let status = entry.status.as_deref().unwrap_or("unknown");
            out.push_str(&format!(
                "- **{}** (status: `{}`)\n  - `{}`\n",
                entry.module, status, entry.spec_path
            ));
        }
        out.push('\n');
    }

    out
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Frontmatter;
    use std::fs;
    use tempfile::TempDir;

    // ── parse_range tests ──────────────────────────────────────────────

    #[test]
    fn test_parse_range_valid() {
        let (from, to) = parse_range("v0.1..v0.2").unwrap();
        assert_eq!(from, "v0.1");
        assert_eq!(to, "v0.2");
    }

    #[test]
    fn test_parse_range_head_tilde() {
        let (from, to) = parse_range("HEAD~5..HEAD").unwrap();
        assert_eq!(from, "HEAD~5");
        assert_eq!(to, "HEAD");
    }

    #[test]
    fn test_parse_range_invalid_no_dots() {
        assert!(parse_range("v0.1").is_none());
    }

    #[test]
    fn test_parse_range_invalid_empty_from() {
        assert!(parse_range("..v0.2").is_none());
    }

    #[test]
    fn test_parse_range_invalid_empty_to() {
        assert!(parse_range("v0.1..").is_none());
    }

    #[test]
    fn test_parse_range_commit_hashes() {
        let (from, to) = parse_range("abc1234..def5678").unwrap();
        assert_eq!(from, "abc1234");
        assert_eq!(to, "def5678");
    }

    // ── module_from_path tests ─────────────────────────────────────────

    #[test]
    fn test_module_from_path_standard() {
        assert_eq!(module_from_path("specs/auth/auth.spec.md"), "auth");
    }

    #[test]
    fn test_module_from_path_nested() {
        assert_eq!(
            module_from_path("specs/deep/nested/parser.spec.md"),
            "parser"
        );
    }

    #[test]
    fn test_module_from_path_bare() {
        assert_eq!(module_from_path("validator.spec.md"), "validator");
    }

    // ── compare_frontmatter tests ──────────────────────────────────────

    #[test]
    fn test_compare_frontmatter_no_changes() {
        let fm = Frontmatter {
            module: Some("auth".to_string()),
            version: Some("1".to_string()),
            status: Some("active".to_string()),
            ..Default::default()
        };
        let changes = compare_frontmatter(&fm, &fm);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_compare_frontmatter_status_change() {
        let old = Frontmatter {
            status: Some("draft".to_string()),
            ..Default::default()
        };
        let new = Frontmatter {
            status: Some("active".to_string()),
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "status");
        assert_eq!(changes[0].old_value, "draft");
        assert_eq!(changes[0].new_value, "active");
    }

    #[test]
    fn test_compare_frontmatter_version_change() {
        let old = Frontmatter {
            version: Some("1".to_string()),
            ..Default::default()
        };
        let new = Frontmatter {
            version: Some("2".to_string()),
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "version");
    }

    #[test]
    fn test_compare_frontmatter_files_change() {
        let old = Frontmatter {
            files: vec!["src/auth.ts".to_string()],
            ..Default::default()
        };
        let new = Frontmatter {
            files: vec!["src/auth.ts".to_string(), "src/auth_utils.ts".to_string()],
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "files");
    }

    #[test]
    fn test_compare_frontmatter_depends_on_change() {
        let old = Frontmatter {
            depends_on: vec!["types".to_string()],
            ..Default::default()
        };
        let new = Frontmatter {
            depends_on: vec!["types".to_string(), "config".to_string()],
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "depends_on");
    }

    #[test]
    fn test_compare_frontmatter_multiple_changes() {
        let old = Frontmatter {
            status: Some("draft".to_string()),
            version: Some("1".to_string()),
            files: vec!["src/old.ts".to_string()],
            ..Default::default()
        };
        let new = Frontmatter {
            status: Some("active".to_string()),
            version: Some("2".to_string()),
            files: vec!["src/new.ts".to_string()],
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 3);
        let fields: Vec<&str> = changes.iter().map(|c| c.field.as_str()).collect();
        assert!(fields.contains(&"status"));
        assert!(fields.contains(&"version"));
        assert!(fields.contains(&"files"));
    }

    #[test]
    fn test_compare_frontmatter_implements_change() {
        let old = Frontmatter {
            implements: vec![42],
            ..Default::default()
        };
        let new = Frontmatter {
            implements: vec![42, 57],
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "implements");
        assert_eq!(changes[0].old_value, "42");
        assert_eq!(changes[0].new_value, "42, 57");
    }

    #[test]
    fn test_compare_frontmatter_agent_policy_change() {
        let old = Frontmatter {
            agent_policy: Some("read-only".to_string()),
            ..Default::default()
        };
        let new = Frontmatter {
            agent_policy: Some("read-write".to_string()),
            ..Default::default()
        };
        let changes = compare_frontmatter(&old, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "agent_policy");
    }

    // ── compare_sections tests ─────────────────────────────────────────

    #[test]
    fn test_compare_sections_no_changes() {
        let body = "## Purpose\nDo stuff\n\n## Public API\nStuff\n";
        let changes = compare_sections(body, body);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_compare_sections_modified() {
        let old = "## Purpose\nOld purpose\n\n## Public API\nStuff\n";
        let new = "## Purpose\nNew purpose\n\n## Public API\nStuff\n";
        let changes = compare_sections(old, new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "section:Purpose");
    }

    #[test]
    fn test_compare_sections_added() {
        let old = "## Purpose\nDo stuff\n";
        let new = "## Purpose\nDo stuff\n\n## Invariants\nMust be valid\n";
        let changes = compare_sections(old, new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "section:Invariants");
        assert_eq!(changes[0].new_value, "(added)");
    }

    #[test]
    fn test_compare_sections_removed() {
        let old = "## Purpose\nDo stuff\n\n## Invariants\nMust be valid\n";
        let new = "## Purpose\nDo stuff\n";
        let changes = compare_sections(old, new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "section:Invariants");
        assert_eq!(changes[0].new_value, "(removed)");
    }

    // ── extract_sections tests ─────────────────────────────────────────

    #[test]
    fn test_extract_sections_basic() {
        let body = "## Purpose\nDo auth\n\n## Public API\n| fn | desc |\n";
        let sections = extract_sections(body);
        assert_eq!(sections.len(), 2);
        assert!(sections.contains_key("Purpose"));
        assert!(sections.contains_key("Public API"));
    }

    #[test]
    fn test_extract_sections_ignores_subsections() {
        let body =
            "## Public API\n\n### Exported Functions\n| fn | desc |\n\n## Invariants\nStuff\n";
        let sections = extract_sections(body);
        assert_eq!(sections.len(), 2);
        // ### should be part of the Public API content, not a separate section
        assert!(sections["Public API"].contains("Exported Functions"));
    }

    // ── format helpers tests ───────────────────────────────────────────

    #[test]
    fn test_format_list_empty() {
        assert_eq!(format_list(&[]), "[]");
    }

    #[test]
    fn test_format_list_items() {
        let items = vec!["a".to_string(), "b".to_string()];
        assert_eq!(format_list(&items), "a, b");
    }

    #[test]
    fn test_format_u64_list_empty() {
        assert_eq!(format_u64_list(&[]), "[]");
    }

    #[test]
    fn test_format_u64_list_items() {
        assert_eq!(format_u64_list(&[42, 57]), "42, 57");
    }

    // ── format_text tests ──────────────────────────────────────────────

    #[test]
    fn test_format_text_empty() {
        let report = ChangelogReport {
            from_ref: "v0.1".to_string(),
            to_ref: "v0.2".to_string(),
            added: vec![],
            removed: vec![],
            modified: vec![],
        };
        let text = format_text(&report);
        assert!(text.contains("No spec changes detected"));
        assert!(text.contains("v0.1..v0.2"));
    }

    #[test]
    fn test_format_text_added() {
        let report = ChangelogReport {
            from_ref: "a".to_string(),
            to_ref: "b".to_string(),
            added: vec![SpecEntry {
                module: "auth".to_string(),
                spec_path: "specs/auth/auth.spec.md".to_string(),
                status: Some("active".to_string()),
                version: Some("1".to_string()),
            }],
            removed: vec![],
            modified: vec![],
        };
        let text = format_text(&report);
        assert!(text.contains("Added"));
        assert!(text.contains("auth"));
        assert!(text.contains("1 added"));
    }

    #[test]
    fn test_format_text_modified_with_section_changes() {
        let report = ChangelogReport {
            from_ref: "a".to_string(),
            to_ref: "b".to_string(),
            added: vec![],
            removed: vec![],
            modified: vec![ModifiedSpec {
                module: "parser".to_string(),
                spec_path: "specs/parser/parser.spec.md".to_string(),
                changes: vec![
                    FieldChange {
                        field: "status".to_string(),
                        old_value: "draft".to_string(),
                        new_value: "active".to_string(),
                    },
                    FieldChange {
                        field: "section:Purpose".to_string(),
                        old_value: "(modified)".to_string(),
                        new_value: "(modified)".to_string(),
                    },
                ],
            }],
        };
        let text = format_text(&report);
        assert!(text.contains("Changed"));
        assert!(text.contains("parser"));
        assert!(text.contains("status"));
        assert!(text.contains("section \"Purpose\" modified"));
    }

    // ── format_json tests ──────────────────────────────────────────────

    #[test]
    fn test_format_json_structure() {
        let report = ChangelogReport {
            from_ref: "v1".to_string(),
            to_ref: "v2".to_string(),
            added: vec![SpecEntry {
                module: "auth".to_string(),
                spec_path: "specs/auth/auth.spec.md".to_string(),
                status: Some("active".to_string()),
                version: Some("1".to_string()),
            }],
            removed: vec![],
            modified: vec![],
        };
        let json_str = format_json(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["from_ref"], "v1");
        assert_eq!(parsed["to_ref"], "v2");
        assert_eq!(parsed["summary"]["added"], 1);
        assert_eq!(parsed["summary"]["changed"], 0);
        assert_eq!(parsed["summary"]["removed"], 0);
        assert_eq!(parsed["added"][0]["module"], "auth");
    }

    // ── format_markdown tests ──────────────────────────────────────────

    #[test]
    fn test_format_markdown_empty() {
        let report = ChangelogReport {
            from_ref: "v0.1".to_string(),
            to_ref: "v0.2".to_string(),
            added: vec![],
            removed: vec![],
            modified: vec![],
        };
        let md = format_markdown(&report);
        assert!(md.contains("No spec changes detected"));
    }

    #[test]
    fn test_format_markdown_all_sections() {
        let report = ChangelogReport {
            from_ref: "a".to_string(),
            to_ref: "b".to_string(),
            added: vec![SpecEntry {
                module: "new_mod".to_string(),
                spec_path: "specs/new_mod/new_mod.spec.md".to_string(),
                status: Some("draft".to_string()),
                version: Some("1".to_string()),
            }],
            removed: vec![SpecEntry {
                module: "old_mod".to_string(),
                spec_path: "specs/old_mod/old_mod.spec.md".to_string(),
                status: Some("deprecated".to_string()),
                version: None,
            }],
            modified: vec![ModifiedSpec {
                module: "core".to_string(),
                spec_path: "specs/core/core.spec.md".to_string(),
                changes: vec![FieldChange {
                    field: "version".to_string(),
                    old_value: "1".to_string(),
                    new_value: "2".to_string(),
                }],
            }],
        };
        let md = format_markdown(&report);
        assert!(md.contains("### Added"));
        assert!(md.contains("### Changed"));
        assert!(md.contains("### Removed"));
        assert!(md.contains("**new_mod**"));
        assert!(md.contains("**old_mod**"));
        assert!(md.contains("**core**"));
        assert!(md.contains("`version`"));
    }

    // ── generate_changelog integration tests ───────────────────────────
    // These require a real git repo so we set one up in a temp dir.

    fn setup_git_repo() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Init repo
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(root)
            .output()
            .unwrap();

        // Create initial spec
        let specs_dir = root.join("specs").join("auth");
        fs::create_dir_all(&specs_dir).unwrap();
        fs::write(
            specs_dir.join("auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\nHandle auth\n\n## Public API\n| fn | desc |\n",
        )
        .unwrap();

        // Initial commit
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["tag", "v0.1"])
            .current_dir(root)
            .output()
            .unwrap();

        tmp
    }

    #[test]
    fn test_generate_changelog_no_changes() {
        let tmp = setup_git_repo();
        let report = generate_changelog(tmp.path(), "specs", "v0.1", "HEAD");
        assert!(report.added.is_empty());
        assert!(report.removed.is_empty());
        assert!(report.modified.is_empty());
    }

    #[test]
    fn test_generate_changelog_added_spec() {
        let tmp = setup_git_repo();
        let root = tmp.path();

        // Add a new spec
        let new_dir = root.join("specs").join("api");
        fs::create_dir_all(&new_dir).unwrap();
        fs::write(
            new_dir.join("api.spec.md"),
            "---\nmodule: api\nversion: 1\nstatus: active\nfiles:\n  - src/api.ts\ndb_tables: []\ndepends_on: []\n---\n\n# API\n\n## Purpose\nAPI layer\n",
        )
        .unwrap();

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add api spec"])
            .current_dir(root)
            .output()
            .unwrap();

        let report = generate_changelog(root, "specs", "v0.1", "HEAD");
        assert_eq!(report.added.len(), 1);
        assert_eq!(report.added[0].module, "api");
        assert!(report.removed.is_empty());
        assert!(report.modified.is_empty());
    }

    #[test]
    fn test_generate_changelog_removed_spec() {
        let tmp = setup_git_repo();
        let root = tmp.path();

        // Remove the auth spec
        fs::remove_file(root.join("specs/auth/auth.spec.md")).unwrap();
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "remove auth"])
            .current_dir(root)
            .output()
            .unwrap();

        let report = generate_changelog(root, "specs", "v0.1", "HEAD");
        assert!(report.added.is_empty());
        assert_eq!(report.removed.len(), 1);
        assert_eq!(report.removed[0].module, "auth");
    }

    #[test]
    fn test_generate_changelog_modified_spec() {
        let tmp = setup_git_repo();
        let root = tmp.path();

        // Modify the auth spec
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 2\nstatus: active\nfiles:\n  - src/auth.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\nHandle auth v2\n\n## Public API\n| fn | desc |\n",
        )
        .unwrap();

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update auth"])
            .current_dir(root)
            .output()
            .unwrap();

        let report = generate_changelog(root, "specs", "v0.1", "HEAD");
        assert!(report.added.is_empty());
        assert!(report.removed.is_empty());
        assert_eq!(report.modified.len(), 1);
        assert_eq!(report.modified[0].module, "auth");

        let fields: Vec<&str> = report.modified[0]
            .changes
            .iter()
            .map(|c| c.field.as_str())
            .collect();
        assert!(fields.contains(&"status"));
        assert!(fields.contains(&"version"));
        assert!(fields.contains(&"section:Purpose"));
    }
}
