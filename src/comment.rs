//! GitHub PR comment formatting with spec links and actionable suggestions.
//!
//! Produces GitHub-flavored markdown output designed for posting as PR comments,
//! including direct links to spec files, actionable checklists, and diff-aware
//! suggestions for updating specs.

use crate::types::{CoverageReport, ValidationResult};
use std::path::Path;

/// Information about a spec violation suitable for PR comment rendering.
#[derive(Debug, Clone)]
pub struct SpecViolation {
    /// Relative path to the spec file (e.g., `specs/auth.spec.md`).
    pub spec_path: String,
    /// Error messages from validation.
    pub errors: Vec<String>,
    /// Warning messages from validation.
    pub warnings: Vec<String>,
    /// Actionable fix suggestions (reserved for future diff-aware suggestions).
    #[allow(dead_code)]
    pub fixes: Vec<String>,
}

impl SpecViolation {
    /// Build a violation from a `ValidationResult`.
    pub fn from_result(result: &ValidationResult) -> Self {
        Self {
            spec_path: result.spec_path.clone(),
            errors: result.errors.clone(),
            warnings: result.warnings.clone(),
            fixes: result.fixes.clone(),
        }
    }
}

/// Build a GitHub-friendly file link.  When `repo` and `branch` are known we
/// produce a full `https://github.com/…/blob/…` URL; otherwise we fall back to
/// a relative markdown link.
fn spec_link(spec_path: &str, repo: Option<&str>, branch: Option<&str>) -> String {
    if let (Some(repo), Some(branch)) = (repo, branch) {
        format!("[`{spec_path}`](https://github.com/{repo}/blob/{branch}/{spec_path})")
    } else {
        format!("`{spec_path}`")
    }
}

/// Classify an error message into an actionable suggestion.
fn suggestion_for_error(error: &str) -> String {
    if error.starts_with("Missing required section") {
        let section = error
            .strip_prefix("Missing required section: ")
            .unwrap_or(error);
        format!("Add a **{section}** section to the spec")
    } else if error.starts_with("Source file") && error.contains("not found") {
        format!("Update the `files` list in frontmatter -- {error}")
    } else if error.starts_with("DB table") {
        format!("Verify database schema references -- {error}")
    } else if error.starts_with("Frontmatter") || error.starts_with("Missing or malformed") {
        format!("Fix the YAML frontmatter block -- {error}")
    } else if error.starts_with("Dependency spec") {
        format!("Create or fix the referenced dependency spec -- {error}")
    } else if error.starts_with("Schema column") {
        format!("Update the spec's column documentation -- {error}")
    } else if error.starts_with("Spec documents") {
        format!("Remove stale file references from the spec -- {error}")
    } else {
        format!("Review and fix -- {error}")
    }
}

/// Classify a warning message into an actionable suggestion.
fn suggestion_for_warning(warning: &str) -> String {
    if warning.starts_with("Export '") {
        let symbol = warning
            .strip_prefix("Export '")
            .and_then(|s| s.split('\'').next())
            .unwrap_or("?");
        format!("Add `{symbol}` to the **Public API** table in the spec")
    } else if warning.starts_with("Consumed By") {
        format!("Review cross-module dependency -- {warning}")
    } else if warning.starts_with("Schema column") {
        format!("Update column documentation -- {warning}")
    } else {
        format!("Review -- {warning}")
    }
}

/// Render the full GitHub PR comment for `specsync check --format github`.
#[allow(clippy::too_many_arguments)]
pub fn render_check_comment(
    total: usize,
    passed: usize,
    warnings: usize,
    errors: usize,
    all_errors: &[String],
    all_warnings: &[String],
    coverage: &CoverageReport,
    overall_passed: bool,
    repo: Option<&str>,
    branch: Option<&str>,
) -> String {
    let mut out = String::new();

    // Header
    let status = if overall_passed { "Passed" } else { "Failed" };
    let icon = if overall_passed { "✅" } else { "❌" };
    out.push_str(&format!("## {icon} SpecSync: {status}\n\n"));

    // Summary table
    out.push_str("| Metric | Value |\n|--------|-------|\n");
    out.push_str(&format!("| Specs checked | {total} |\n"));
    out.push_str(&format!("| Passed | {passed} |\n"));
    out.push_str(&format!("| Errors | {errors} |\n"));
    out.push_str(&format!("| Warnings | {warnings} |\n"));
    out.push_str(&format!(
        "| File coverage | {}% ({}/{}) |\n",
        coverage.coverage_percent, coverage.specced_file_count, coverage.total_source_files
    ));
    out.push_str(&format!(
        "| LOC coverage | {}% ({}/{}) |\n\n",
        coverage.loc_coverage_percent, coverage.specced_loc, coverage.total_loc
    ));

    // Errors with spec links and actionable suggestions
    if !all_errors.is_empty() {
        out.push_str("### Errors\n\n");
        let grouped = group_by_spec(all_errors);
        for (spec, messages) in &grouped {
            let link = spec_link(spec, repo, branch);
            out.push_str(&format!("**{link}**\n\n"));
            for msg in messages {
                out.push_str(&format!("- {msg}\n"));
            }
            out.push('\n');
        }
    }

    // Warnings with spec links
    if !all_warnings.is_empty() {
        out.push_str("### Warnings\n\n");
        let grouped = group_by_spec(all_warnings);
        for (spec, messages) in &grouped {
            let link = spec_link(spec, repo, branch);
            out.push_str(&format!("**{link}**\n\n"));
            for msg in messages {
                out.push_str(&format!("- {msg}\n"));
            }
            out.push('\n');
        }
    }

    // Actionable checklist
    if !all_errors.is_empty() || !all_warnings.is_empty() {
        out.push_str("### Action Items\n\n");
        for err in all_errors {
            let raw = strip_spec_prefix(err);
            let suggestion = suggestion_for_error(raw);
            out.push_str(&format!("- [ ] {suggestion}\n"));
        }
        for warn in all_warnings {
            let raw = strip_spec_prefix(warn);
            let suggestion = suggestion_for_warning(raw);
            out.push_str(&format!("- [ ] {suggestion}\n"));
        }
        out.push('\n');
    }

    // Uncovered files
    if !coverage.unspecced_files.is_empty() {
        out.push_str("### Unspecced Files\n\n");
        out.push_str("The following source files have no spec coverage:\n\n");
        let limit = 15;
        for f in coverage.unspecced_files.iter().take(limit) {
            out.push_str(&format!("- `{f}`\n"));
        }
        if coverage.unspecced_files.len() > limit {
            out.push_str(&format!(
                "- _...and {} more_\n",
                coverage.unspecced_files.len() - limit
            ));
        }
        out.push_str("\nRun `specsync generate` to scaffold specs for these files.\n\n");
    }

    // Footer
    out.push_str("---\n");
    out.push_str("_Generated by [specsync](https://github.com/CorvidLabs/spec-sync) · ");
    out.push_str("Run `specsync check --format github` to reproduce_\n");

    out
}

/// Render a GitHub PR comment for the `specsync comment` subcommand, combining
/// check results with diff-aware suggestions.
pub fn render_comment_body(
    violations: &[SpecViolation],
    coverage: &CoverageReport,
    repo: Option<&str>,
    branch: Option<&str>,
) -> String {
    let total = violations.len();
    let errors: usize = violations.iter().map(|v| v.errors.len()).sum();
    let warnings: usize = violations.iter().map(|v| v.warnings.len()).sum();
    let passed = violations.iter().filter(|v| v.errors.is_empty()).count();
    let overall_passed = errors == 0;

    let all_errors: Vec<String> = violations
        .iter()
        .flat_map(|v| v.errors.iter().map(|e| format!("{}: {e}", v.spec_path)))
        .collect();
    let all_warnings: Vec<String> = violations
        .iter()
        .flat_map(|v| v.warnings.iter().map(|w| format!("{}: {w}", v.spec_path)))
        .collect();

    render_check_comment(
        total,
        passed,
        warnings,
        errors,
        &all_errors,
        &all_warnings,
        coverage,
        overall_passed,
        repo,
        branch,
    )
}

/// Group prefixed messages (`spec/path: message`) by spec path.
/// Returns a vector of (spec_path, messages) preserving insertion order.
fn group_by_spec(messages: &[String]) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for msg in messages {
        let (spec, remainder) = split_spec_prefix(msg);
        if let Some(&idx) = seen.get(&spec) {
            groups[idx].1.push(remainder.to_string());
        } else {
            seen.insert(spec.clone(), groups.len());
            groups.push((spec, vec![remainder.to_string()]));
        }
    }
    groups
}

/// Split a `"spec/path.md: error message"` string into (spec_path, message).
fn split_spec_prefix(s: &str) -> (String, &str) {
    if let Some(idx) = s.find(": ") {
        let prefix = &s[..idx];
        // Only treat it as a spec path if it looks like a file path
        if prefix.contains('/') || prefix.ends_with(".md") {
            return (prefix.to_string(), &s[idx + 2..]);
        }
    }
    ("unknown".to_string(), s)
}

/// Strip the `"spec/path.md: "` prefix from a message, returning just the error text.
fn strip_spec_prefix(s: &str) -> &str {
    if let Some(idx) = s.find(": ") {
        let prefix = &s[..idx];
        if prefix.contains('/') || prefix.ends_with(".md") {
            return &s[idx + 2..];
        }
    }
    s
}

/// Detect the current git branch name.
pub fn detect_branch(root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_link_with_repo() {
        let link = spec_link("specs/auth.spec.md", Some("owner/repo"), Some("main"));
        assert_eq!(
            link,
            "[`specs/auth.spec.md`](https://github.com/owner/repo/blob/main/specs/auth.spec.md)"
        );
    }

    #[test]
    fn test_spec_link_without_repo() {
        let link = spec_link("specs/auth.spec.md", None, None);
        assert_eq!(link, "`specs/auth.spec.md`");
    }

    #[test]
    fn test_suggestion_for_missing_section() {
        let s = suggestion_for_error("Missing required section: Purpose");
        assert_eq!(s, "Add a **Purpose** section to the spec");
    }

    #[test]
    fn test_suggestion_for_source_file_not_found() {
        let s = suggestion_for_error("Source file src/foo.rs not found");
        assert!(s.contains("Update the `files` list"));
    }

    #[test]
    fn test_suggestion_for_db_table() {
        let s = suggestion_for_error("DB table users not found in schema");
        assert!(s.contains("Verify database schema"));
    }

    #[test]
    fn test_suggestion_for_frontmatter() {
        let s = suggestion_for_error("Frontmatter missing module field");
        assert!(s.contains("Fix the YAML frontmatter"));
    }

    #[test]
    fn test_suggestion_for_dependency() {
        let s = suggestion_for_error("Dependency spec core not found");
        assert!(s.contains("Create or fix the referenced dependency"));
    }

    #[test]
    fn test_suggestion_for_schema_column_error() {
        let s = suggestion_for_error("Schema column 'email' not found");
        assert!(s.contains("Update the spec's column documentation"));
    }

    #[test]
    fn test_suggestion_for_stale_file_ref() {
        let s = suggestion_for_error("Spec documents file src/old.rs which was deleted");
        assert!(s.contains("Remove stale file references"));
    }

    #[test]
    fn test_suggestion_for_generic_error() {
        let s = suggestion_for_error("Something completely unexpected");
        assert!(s.starts_with("Review and fix -- "));
    }

    #[test]
    fn test_suggestion_for_export_warning() {
        let s = suggestion_for_warning("Export 'MyClass' is not documented in spec");
        assert!(s.contains("Add `MyClass` to the **Public API** table"));
    }

    #[test]
    fn test_suggestion_for_consumed_by_warning() {
        let s = suggestion_for_warning("Consumed By module x uses y");
        assert!(s.contains("Review cross-module dependency"));
    }

    #[test]
    fn test_suggestion_for_schema_column_warning() {
        let s = suggestion_for_warning("Schema column 'name' type mismatch");
        assert!(s.contains("Update column documentation"));
    }

    #[test]
    fn test_suggestion_for_generic_warning() {
        let s = suggestion_for_warning("Something unexpected");
        assert!(s.starts_with("Review -- "));
    }

    #[test]
    fn test_group_by_spec() {
        let messages = vec![
            "specs/auth.spec.md: error one".to_string(),
            "specs/auth.spec.md: error two".to_string(),
            "specs/api.spec.md: error three".to_string(),
        ];
        let grouped = group_by_spec(&messages);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].0, "specs/auth.spec.md");
        assert_eq!(grouped[0].1.len(), 2);
        assert_eq!(grouped[1].0, "specs/api.spec.md");
        assert_eq!(grouped[1].1.len(), 1);
    }

    #[test]
    fn test_group_by_spec_no_prefix() {
        let messages = vec!["plain message".to_string()];
        let grouped = group_by_spec(&messages);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].0, "unknown");
    }

    #[test]
    fn test_split_spec_prefix() {
        let (spec, msg) = split_spec_prefix("specs/auth.spec.md: Missing section");
        assert_eq!(spec, "specs/auth.spec.md");
        assert_eq!(msg, "Missing section");
    }

    #[test]
    fn test_split_spec_prefix_no_path() {
        let (spec, msg) = split_spec_prefix("just a plain message");
        assert_eq!(spec, "unknown");
        assert_eq!(msg, "just a plain message");
    }

    #[test]
    fn test_strip_spec_prefix() {
        assert_eq!(
            strip_spec_prefix("specs/auth.spec.md: Missing section"),
            "Missing section"
        );
        assert_eq!(strip_spec_prefix("no prefix here"), "no prefix here");
    }

    #[test]
    fn test_render_check_comment_passed() {
        let coverage = CoverageReport {
            total_source_files: 10,
            specced_file_count: 10,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 1000,
            specced_loc: 1000,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        };
        let output = render_check_comment(
            5,
            5,
            0,
            0,
            &[],
            &[],
            &coverage,
            true,
            Some("owner/repo"),
            Some("main"),
        );
        assert!(output.contains("SpecSync: Passed"));
        assert!(output.contains("| Specs checked | 5 |"));
        assert!(!output.contains("### Errors"));
        assert!(!output.contains("### Action Items"));
    }

    #[test]
    fn test_render_check_comment_failed_with_errors() {
        let coverage = CoverageReport {
            total_source_files: 10,
            specced_file_count: 8,
            unspecced_files: vec!["src/new.rs".to_string(), "src/other.rs".to_string()],
            unspecced_modules: vec![],
            coverage_percent: 80,
            total_loc: 1000,
            specced_loc: 800,
            loc_coverage_percent: 80,
            unspecced_file_loc: vec![],
        };
        let errors = vec![
            "specs/auth.spec.md: Missing required section: Purpose".to_string(),
            "specs/auth.spec.md: Source file src/auth.rs not found".to_string(),
            "specs/api.spec.md: DB table users not found in schema".to_string(),
        ];
        let warnings =
            vec!["specs/auth.spec.md: Export 'AuthService' is not documented in spec".to_string()];
        let output = render_check_comment(
            5,
            2,
            1,
            3,
            &errors,
            &warnings,
            &coverage,
            false,
            Some("owner/repo"),
            Some("feat/test"),
        );
        assert!(output.contains("SpecSync: Failed"));
        assert!(output.contains("### Errors"));
        assert!(output.contains("### Warnings"));
        assert!(output.contains("### Action Items"));
        assert!(output.contains("- [ ] Add a **Purpose** section to the spec"));
        assert!(output.contains("### Unspecced Files"));
        assert!(output.contains("`src/new.rs`"));
        // Check spec links
        assert!(output.contains("https://github.com/owner/repo/blob/feat/test/specs/auth.spec.md"));
    }

    #[test]
    fn test_render_check_comment_has_footer() {
        let coverage = CoverageReport {
            total_source_files: 0,
            specced_file_count: 0,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        };
        let output = render_check_comment(0, 0, 0, 0, &[], &[], &coverage, true, None, None);
        assert!(output.contains("Generated by [specsync]"));
        assert!(output.contains("--format github"));
    }

    #[test]
    fn test_render_check_comment_truncates_unspecced_files() {
        let files: Vec<String> = (0..20).map(|i| format!("src/file{i}.rs")).collect();
        let coverage = CoverageReport {
            total_source_files: 20,
            specced_file_count: 0,
            unspecced_files: files,
            unspecced_modules: vec![],
            coverage_percent: 0,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 0,
            unspecced_file_loc: vec![],
        };
        let output = render_check_comment(0, 0, 0, 0, &[], &[], &coverage, true, None, None);
        assert!(output.contains("...and 5 more"));
    }

    #[test]
    fn test_render_comment_body() {
        let violations = vec![SpecViolation {
            spec_path: "specs/auth.spec.md".to_string(),
            errors: vec!["Missing required section: Purpose".to_string()],
            warnings: vec![],
            fixes: vec![],
        }];
        let coverage = CoverageReport {
            total_source_files: 5,
            specced_file_count: 5,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 500,
            specced_loc: 500,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        };
        let body = render_comment_body(&violations, &coverage, Some("owner/repo"), Some("main"));
        assert!(body.contains("SpecSync: Failed"));
        assert!(body.contains("Action Items"));
    }

    #[test]
    fn test_render_comment_body_all_pass() {
        let violations = vec![SpecViolation {
            spec_path: "specs/auth.spec.md".to_string(),
            errors: vec![],
            warnings: vec![],
            fixes: vec![],
        }];
        let coverage = CoverageReport {
            total_source_files: 5,
            specced_file_count: 5,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 500,
            specced_loc: 500,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        };
        let body = render_comment_body(&violations, &coverage, Some("owner/repo"), Some("main"));
        assert!(body.contains("SpecSync: Passed"));
        assert!(!body.contains("Action Items"));
    }
}
