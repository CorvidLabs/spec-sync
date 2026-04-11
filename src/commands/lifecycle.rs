use colored::Colorize;
use regex::Regex;
use std::path::Path;
use std::process;
use std::sync::LazyLock;

use crate::git_utils;
use crate::parser;
use crate::scoring;
use crate::types::{LifecycleConfig, OutputFormat, SpecStatus, SpecSyncConfig, TransitionGuard};

use super::{filter_specs, load_and_discover};

static STATUS_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^status:\s*\S+").unwrap());

static LIFECYCLE_LOG_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^lifecycle_log:\n(?:  - [^\n]+\n)*").unwrap());

/// Update the status field in a spec file's frontmatter.
/// Returns the new file content, or None if the status line wasn't found.
fn update_status_in_content(content: &str, new_status: &str) -> Option<String> {
    if STATUS_LINE_RE.is_match(content) {
        Some(
            STATUS_LINE_RE
                .replace(content, format!("status: {new_status}"))
                .to_string(),
        )
    } else {
        None
    }
}

/// Append a lifecycle_log entry to spec frontmatter content.
/// If lifecycle_log already exists, append to it. Otherwise, insert before the closing ---.
fn append_lifecycle_log_entry(content: &str, entry: &str) -> String {
    let log_line = format!("  - {entry}\n");

    if LIFECYCLE_LOG_BLOCK_RE.is_match(content) {
        // Append to existing block
        LIFECYCLE_LOG_BLOCK_RE
            .replace(content, |caps: &regex::Captures| {
                format!("{}{log_line}", &caps[0])
            })
            .to_string()
    } else {
        // Insert before closing ---
        // Find the second --- in frontmatter
        if let Some(first) = content.find("---\n") {
            let rest = &content[first + 4..];
            if let Some(second) = rest.find("\n---\n") {
                let insert_pos = first + 4 + second;
                let mut result = String::with_capacity(content.len() + 50);
                result.push_str(&content[..insert_pos]);
                result.push_str("\nlifecycle_log:\n");
                result.push_str(&log_line);
                result.push_str(&content[insert_pos..]);
                return result;
            }
        }
        // Fallback: return unchanged
        content.to_string()
    }
}

/// Resolve a single spec from user input (module name, path, etc.)
fn resolve_spec(root: &Path, spec_filter: &str) -> std::path::PathBuf {
    let (_, spec_files) = load_and_discover(root, false);
    let matched = filter_specs(root, &spec_files, &[spec_filter.to_string()]);
    if matched.is_empty() {
        eprintln!("{} No spec matched: {}", "error:".red().bold(), spec_filter);
        process::exit(1);
    }
    if matched.len() > 1 {
        eprintln!(
            "{} Ambiguous — {} specs matched '{}'. Be more specific.",
            "error:".red().bold(),
            matched.len(),
            spec_filter
        );
        for m in &matched {
            eprintln!("  {}", m.strip_prefix(root).unwrap_or(m).display());
        }
        process::exit(1);
    }
    matched.into_iter().next().unwrap()
}

/// Read a spec file and return its current status, content, and relative path.
fn read_spec_status(root: &Path, spec_path: &Path) -> (String, Option<SpecStatus>, String) {
    let rel = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .display()
        .to_string();

    let content = match std::fs::read_to_string(spec_path) {
        Ok(c) => c.replace("\r\n", "\n"),
        Err(e) => {
            eprintln!("{} Cannot read {rel}: {e}", "error:".red().bold());
            process::exit(1);
        }
    };

    let status = parser::parse_frontmatter(&content).and_then(|p| p.frontmatter.parsed_status());

    (content, status, rel)
}

/// Result of evaluating transition guards.
#[derive(Debug)]
pub struct GuardResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

/// Look up guards that apply to a specific transition.
fn find_guards<'a>(
    config: &'a LifecycleConfig,
    from: &SpecStatus,
    to: &SpecStatus,
) -> Vec<&'a TransitionGuard> {
    let specific_key = format!("{}→{}", from.as_str(), to.as_str());
    let wildcard_key = format!("*→{}", to.as_str());
    let specific_ascii = format!("{}->{}", from.as_str(), to.as_str());
    let wildcard_ascii = format!("*->{}", to.as_str());

    let keys = [specific_key, specific_ascii, wildcard_key, wildcard_ascii];
    let mut guards = Vec::new();
    for key in &keys {
        if let Some(g) = config.guards.get(key) {
            guards.push(g);
        }
    }
    guards
}

/// Evaluate all guards for a transition. Returns a GuardResult.
pub fn evaluate_guards(
    root: &Path,
    spec_path: &Path,
    config: &SpecSyncConfig,
    from: &SpecStatus,
    to: &SpecStatus,
) -> GuardResult {
    let guards = find_guards(&config.lifecycle, from, to);
    let mut failures: Vec<String> = Vec::new();

    let rel = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .display()
        .to_string();

    for guard in &guards {
        // Check minimum score
        if let Some(min_score) = guard.min_score {
            let score = scoring::score_spec(spec_path, root, config);
            if score.total < min_score {
                let msg = guard.message.as_deref().unwrap_or("score too low");
                failures.push(format!(
                    "guard: score {} < required {} — {msg}",
                    score.total, min_score
                ));
            }
        }

        // Check required sections
        if !guard.require_sections.is_empty() {
            let content = std::fs::read_to_string(spec_path).unwrap_or_default();
            let parsed = parser::parse_frontmatter(&content.replace("\r\n", "\n"));
            if let Some(parsed) = &parsed {
                let missing = parser::get_missing_sections(&parsed.body, &guard.require_sections);
                if !missing.is_empty() {
                    failures.push(format!(
                        "guard: missing required sections: {}",
                        missing.join(", ")
                    ));
                }
            }
        }

        // Check staleness
        if guard.no_stale.unwrap_or(false) {
            let threshold = guard.stale_threshold.unwrap_or(5);
            let content = std::fs::read_to_string(spec_path).unwrap_or_default();
            let parsed = parser::parse_frontmatter(&content.replace("\r\n", "\n"));
            if let Some(parsed) = &parsed {
                for source_file in &parsed.frontmatter.files {
                    let commits = git_utils::git_commits_between(root, &rel, source_file);
                    if commits >= threshold {
                        failures.push(format!(
                            "guard: stale — {source_file} has {commits} commits since spec was last updated (threshold: {threshold})"
                        ));
                    }
                }
            }
        }
    }

    GuardResult {
        passed: failures.is_empty(),
        failures,
    }
}

/// `specsync lifecycle promote <spec>`
pub fn cmd_promote(root: &Path, spec_filter: &str, format: OutputFormat, force: bool) {
    let spec_path = resolve_spec(root, spec_filter);
    let (config, _) = load_and_discover(root, false);
    let (content, current, rel) = read_spec_status(root, &spec_path);

    let current = match current {
        Some(s) => s,
        None => {
            eprintln!(
                "{} {rel}: no valid status in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    let next = match current.next() {
        Some(n) => n,
        None => {
            eprintln!(
                "{} {rel}: already at {} — cannot promote further",
                "error:".red().bold(),
                current.as_str()
            );
            process::exit(1);
        }
    };

    if !force && !current.can_transition_to(&next) {
        eprintln!(
            "{} {rel}: cannot promote {} → {} (use --force to override)",
            "error:".red().bold(),
            current.as_str(),
            next.as_str()
        );
        process::exit(1);
    }

    // Evaluate guards
    if !force {
        let guard_result = evaluate_guards(root, &spec_path, &config, &current, &next);
        if !guard_result.passed {
            eprintln!(
                "{} {rel}: transition {} → {} blocked by guards:",
                "error:".red().bold(),
                current.as_str(),
                next.as_str()
            );
            for f in &guard_result.failures {
                eprintln!("  {}", f.red());
            }
            eprintln!("\nUse --force to override guards.");
            process::exit(1);
        }
    }

    write_status(
        &spec_path,
        &content,
        current,
        next,
        &rel,
        format,
        config.lifecycle.track_history,
    );
}

/// `specsync lifecycle demote <spec>`
pub fn cmd_demote(root: &Path, spec_filter: &str, format: OutputFormat, force: bool) {
    let spec_path = resolve_spec(root, spec_filter);
    let (config, _) = load_and_discover(root, false);
    let (content, current, rel) = read_spec_status(root, &spec_path);

    let current = match current {
        Some(s) => s,
        None => {
            eprintln!(
                "{} {rel}: no valid status in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    let prev = match current.prev() {
        Some(p) => p,
        None => {
            eprintln!(
                "{} {rel}: already at {} — cannot demote further",
                "error:".red().bold(),
                current.as_str()
            );
            process::exit(1);
        }
    };

    if !force && !current.can_transition_to(&prev) {
        eprintln!(
            "{} {rel}: cannot demote {} → {} (use --force to override)",
            "error:".red().bold(),
            current.as_str(),
            prev.as_str()
        );
        process::exit(1);
    }

    // Guards apply to demotions too
    if !force {
        let guard_result = evaluate_guards(root, &spec_path, &config, &current, &prev);
        if !guard_result.passed {
            eprintln!(
                "{} {rel}: transition {} → {} blocked by guards:",
                "error:".red().bold(),
                current.as_str(),
                prev.as_str()
            );
            for f in &guard_result.failures {
                eprintln!("  {}", f.red());
            }
            eprintln!("\nUse --force to override guards.");
            process::exit(1);
        }
    }

    write_status(
        &spec_path,
        &content,
        current,
        prev,
        &rel,
        format,
        config.lifecycle.track_history,
    );
}

/// `specsync lifecycle set <spec> <status>`
pub fn cmd_set(
    root: &Path,
    spec_filter: &str,
    target_str: &str,
    format: OutputFormat,
    force: bool,
) {
    let target = match SpecStatus::from_str_loose(target_str) {
        Some(s) => s,
        None => {
            eprintln!(
                "{} Unknown status: '{}'. Valid: {}",
                "error:".red().bold(),
                target_str,
                SpecStatus::all()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            process::exit(1);
        }
    };

    let spec_path = resolve_spec(root, spec_filter);
    let (config, _) = load_and_discover(root, false);
    let (content, current, rel) = read_spec_status(root, &spec_path);

    let current = match current {
        Some(s) => s,
        None => {
            eprintln!(
                "{} {rel}: no valid status in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    if current == target {
        if matches!(format, OutputFormat::Text) {
            println!("{rel}: already {}", target.as_str());
        }
        return;
    }

    if !force && !current.can_transition_to(&target) {
        let valid = current
            .valid_transitions()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!(
            "{} {rel}: cannot transition {} → {} (valid: {valid}; use --force to override)",
            "error:".red().bold(),
            current.as_str(),
            target.as_str()
        );
        process::exit(1);
    }

    // Evaluate guards
    if !force {
        let guard_result = evaluate_guards(root, &spec_path, &config, &current, &target);
        if !guard_result.passed {
            eprintln!(
                "{} {rel}: transition {} → {} blocked by guards:",
                "error:".red().bold(),
                current.as_str(),
                target.as_str()
            );
            for f in &guard_result.failures {
                eprintln!("  {}", f.red());
            }
            eprintln!("\nUse --force to override guards.");
            process::exit(1);
        }
    }

    write_status(
        &spec_path,
        &content,
        current,
        target,
        &rel,
        format,
        config.lifecycle.track_history,
    );
}

/// `specsync lifecycle status [spec]` — show status of one or all specs.
pub fn cmd_status(root: &Path, spec_filter: Option<&str>, format: OutputFormat) {
    let (_, spec_files) = load_and_discover(root, false);

    let specs: Vec<std::path::PathBuf> = if let Some(filter) = spec_filter {
        filter_specs(root, &spec_files, &[filter.to_string()])
    } else {
        spec_files
    };

    if specs.is_empty() {
        if matches!(format, OutputFormat::Text) {
            println!("No specs found.");
        }
        return;
    }

    // Collect status info
    let mut entries: Vec<(String, String, usize)> = Vec::new();
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for spec_path in &specs {
        let rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .display()
            .to_string();

        let status = std::fs::read_to_string(spec_path)
            .ok()
            .and_then(|c| parser::parse_frontmatter(&c.replace("\r\n", "\n")))
            .and_then(|p| p.frontmatter.parsed_status());

        let status_str = status
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let ordinal = status.map(|s| s.ordinal()).unwrap_or(99);

        *counts.entry(status_str.clone()).or_insert(0) += 1;
        entries.push((rel, status_str, ordinal));
    }

    match format {
        OutputFormat::Json => {
            let items: Vec<serde_json::Value> = entries
                .iter()
                .map(|(path, status, _)| {
                    serde_json::json!({
                        "spec": path,
                        "status": status,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "specs": items,
                "summary": counts,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            // Group by status in lifecycle order
            let mut by_status: std::collections::BTreeMap<usize, Vec<&str>> =
                std::collections::BTreeMap::new();
            for (path, _, ordinal) in &entries {
                by_status.entry(*ordinal).or_default().push(path);
            }

            for (ordinal, paths) in &by_status {
                let label = if *ordinal == 99 {
                    "unknown".to_string()
                } else {
                    SpecStatus::all()
                        .get(*ordinal)
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_else(|| "?".to_string())
                };

                let colored_label = match label.as_str() {
                    "draft" => label.dimmed().to_string(),
                    "review" => label.yellow().to_string(),
                    "active" => label.green().to_string(),
                    "stable" => label.green().bold().to_string(),
                    "deprecated" => label.red().to_string(),
                    "archived" => label.dimmed().italic().to_string(),
                    _ => label.red().bold().to_string(),
                };

                println!("\n{} ({})", colored_label, paths.len());
                for path in paths {
                    println!("  {path}");
                }
            }

            // Summary line
            println!();
            let summary: Vec<String> = SpecStatus::all()
                .iter()
                .filter_map(|s| {
                    counts
                        .get(s.as_str())
                        .map(|c| format!("{}: {c}", s.as_str()))
                })
                .collect();
            println!("{} specs — {}", entries.len(), summary.join(", "));
        }
    }
}

/// `specsync lifecycle history <spec>` — show transition history for a spec.
pub fn cmd_history(root: &Path, spec_filter: &str, format: OutputFormat) {
    let spec_path = resolve_spec(root, spec_filter);
    let (content, _, rel) = read_spec_status(root, &spec_path);

    let parsed = match parser::parse_frontmatter(&content) {
        Some(p) => p,
        None => {
            eprintln!(
                "{} {rel}: could not parse frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    let log = &parsed.frontmatter.lifecycle_log;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "spec": rel,
                "status": parsed.frontmatter.status,
                "history": log,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!("{}", rel.bold());
            let status_str = parsed
                .frontmatter
                .parsed_status()
                .map(|s| s.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            println!("Current status: {status_str}");

            if log.is_empty() {
                println!("\nNo transition history recorded.");
            } else {
                println!("\n{}", "Transition history:".dimmed());
                for entry in log {
                    println!("  {entry}");
                }
            }
        }
    }
}

/// `specsync lifecycle guard <spec> [target]` — dry-run guard evaluation.
pub fn cmd_guard(root: &Path, spec_filter: &str, target_str: Option<&str>, format: OutputFormat) {
    let spec_path = resolve_spec(root, spec_filter);
    let (config, _) = load_and_discover(root, false);
    let (_, current, rel) = read_spec_status(root, &spec_path);

    let current = match current {
        Some(s) => s,
        None => {
            eprintln!(
                "{} {rel}: no valid status in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    // If target is specified, check just that transition. Otherwise check the next status.
    let targets: Vec<SpecStatus> = if let Some(t) = target_str {
        match SpecStatus::from_str_loose(t) {
            Some(s) => vec![s],
            None => {
                eprintln!(
                    "{} Unknown status: '{t}'. Valid: {}",
                    "error:".red().bold(),
                    SpecStatus::all()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                process::exit(1);
            }
        }
    } else {
        current.valid_transitions()
    };

    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut any_failed = false;

    for target in &targets {
        let guard_result = evaluate_guards(root, &spec_path, &config, &current, target);

        match format {
            OutputFormat::Json => {
                results.push(serde_json::json!({
                    "from": current.as_str(),
                    "to": target.as_str(),
                    "passed": guard_result.passed,
                    "failures": guard_result.failures,
                }));
            }
            _ => {
                let arrow = format!("{} → {}", current.as_str(), target.as_str());
                if guard_result.passed {
                    println!("{} {arrow}: {}", "✓".green(), "all guards pass".green());
                } else {
                    any_failed = true;
                    println!("{} {arrow}: {}", "✗".red(), "blocked".red());
                    for f in &guard_result.failures {
                        println!("    {}", f.dimmed());
                    }
                }
            }
        }

        if !guard_result.passed {
            any_failed = true;
        }
    }

    if matches!(format, OutputFormat::Json) {
        let output = serde_json::json!({
            "spec": rel,
            "current_status": current.as_str(),
            "transitions": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    if any_failed {
        process::exit(1);
    }
}

/// Write the updated status to disk, optionally recording in lifecycle_log, and print the result.
fn write_status(
    spec_path: &Path,
    content: &str,
    from: SpecStatus,
    to: SpecStatus,
    rel: &str,
    format: OutputFormat,
    track_history: bool,
) {
    let mut new_content = match update_status_in_content(content, to.as_str()) {
        Some(c) => c,
        None => {
            eprintln!(
                "{} {rel}: could not find status line in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

    // Append to lifecycle_log if history tracking is enabled
    if track_history {
        let today = chrono_today();
        let entry = format!("{today}: {} → {}", from.as_str(), to.as_str());
        new_content = append_lifecycle_log_entry(&new_content, &entry);
    }

    if let Err(e) = std::fs::write(spec_path, &new_content) {
        eprintln!("{} {rel}: failed to write: {e}", "error:".red().bold());
        process::exit(1);
    }

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "spec": rel,
                "from": from.as_str(),
                "to": to.as_str(),
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            let arrow = "→".bold();
            let from_colored = match from {
                SpecStatus::Draft => from.as_str().dimmed().to_string(),
                _ => from.as_str().yellow().to_string(),
            };
            let to_colored = match to {
                SpecStatus::Active | SpecStatus::Stable => to.as_str().green().to_string(),
                SpecStatus::Deprecated | SpecStatus::Archived => to.as_str().red().to_string(),
                _ => to.as_str().yellow().to_string(),
            };
            println!(
                "{} {} {from_colored} {arrow} {to_colored}",
                "✓".green(),
                rel,
            );
        }
    }
}

/// Get today's date as YYYY-MM-DD string.
fn chrono_today() -> String {
    // Use std::time to avoid a chrono dependency
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown-date".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpecStatus;

    #[test]
    fn update_status_in_content_replaces_status_line() {
        let content =
            "---\nmodule: foo\nversion: 1\nstatus: draft\nfiles:\n  - src/foo.rs\n---\n# Foo\n";
        let result = update_status_in_content(content, "review").unwrap();
        assert!(result.contains("status: review"));
        assert!(!result.contains("status: draft"));
    }

    #[test]
    fn update_status_preserves_rest_of_frontmatter() {
        let content = "---\nmodule: bar\nversion: 2\nstatus: active\nfiles:\n  - src/bar.rs\n---\n# Bar\nBody text.";
        let result = update_status_in_content(content, "stable").unwrap();
        assert!(result.contains("module: bar"));
        assert!(result.contains("version: 2"));
        assert!(result.contains("# Bar\nBody text."));
        assert!(result.contains("status: stable"));
    }

    #[test]
    fn update_status_returns_none_when_no_status_line() {
        let content = "---\nmodule: baz\nversion: 1\n---\n# Baz\n";
        assert!(update_status_in_content(content, "active").is_none());
    }

    #[test]
    fn spec_status_next() {
        assert_eq!(SpecStatus::Draft.next(), Some(SpecStatus::Review));
        assert_eq!(SpecStatus::Review.next(), Some(SpecStatus::Active));
        assert_eq!(SpecStatus::Active.next(), Some(SpecStatus::Stable));
        assert_eq!(SpecStatus::Stable.next(), Some(SpecStatus::Deprecated));
        assert_eq!(SpecStatus::Deprecated.next(), Some(SpecStatus::Archived));
        assert_eq!(SpecStatus::Archived.next(), None);
    }

    #[test]
    fn spec_status_prev() {
        assert_eq!(SpecStatus::Draft.prev(), None);
        assert_eq!(SpecStatus::Review.prev(), Some(SpecStatus::Draft));
        assert_eq!(SpecStatus::Active.prev(), Some(SpecStatus::Review));
        assert_eq!(SpecStatus::Archived.prev(), Some(SpecStatus::Deprecated));
    }

    #[test]
    fn spec_status_valid_transitions() {
        // Draft can go to review or deprecated
        let draft_transitions = SpecStatus::Draft.valid_transitions();
        assert!(draft_transitions.contains(&SpecStatus::Review));
        assert!(draft_transitions.contains(&SpecStatus::Deprecated));
        assert!(!draft_transitions.contains(&SpecStatus::Active));

        // Active can go to stable, review, or deprecated
        let active_transitions = SpecStatus::Active.valid_transitions();
        assert!(active_transitions.contains(&SpecStatus::Stable));
        assert!(active_transitions.contains(&SpecStatus::Review));
        assert!(active_transitions.contains(&SpecStatus::Deprecated));

        // Deprecated can go to archived or stable (prev)
        let dep_transitions = SpecStatus::Deprecated.valid_transitions();
        assert!(dep_transitions.contains(&SpecStatus::Archived));
        assert!(dep_transitions.contains(&SpecStatus::Stable));

        // Archived can only go to deprecated (prev)
        let arch_transitions = SpecStatus::Archived.valid_transitions();
        assert!(arch_transitions.contains(&SpecStatus::Deprecated));
        assert_eq!(arch_transitions.len(), 1);
    }

    #[test]
    fn spec_status_can_transition_to() {
        assert!(SpecStatus::Draft.can_transition_to(&SpecStatus::Review));
        assert!(SpecStatus::Draft.can_transition_to(&SpecStatus::Deprecated));
        assert!(!SpecStatus::Draft.can_transition_to(&SpecStatus::Active));
        assert!(!SpecStatus::Draft.can_transition_to(&SpecStatus::Archived));
    }

    #[test]
    fn append_lifecycle_log_new() {
        let content = "---\nmodule: foo\nstatus: draft\nfiles:\n  - src/foo.rs\n---\n# Foo\n";
        let result = append_lifecycle_log_entry(content, "2026-04-11: draft → review");
        assert!(result.contains("lifecycle_log:\n  - 2026-04-11: draft → review\n"));
        assert!(result.contains("status: draft"));
        assert!(result.contains("# Foo"));
    }

    #[test]
    fn append_lifecycle_log_existing() {
        let content = "---\nmodule: foo\nstatus: review\nlifecycle_log:\n  - 2026-04-10: draft → review\n---\n# Foo\n";
        let result = append_lifecycle_log_entry(content, "2026-04-11: review → active");
        assert!(result.contains("  - 2026-04-10: draft → review\n"));
        assert!(result.contains("  - 2026-04-11: review → active\n"));
    }

    #[test]
    fn find_guards_specific_and_wildcard() {
        let mut guards = std::collections::HashMap::new();
        guards.insert(
            "review→active".to_string(),
            TransitionGuard {
                min_score: Some(70),
                require_sections: vec![],
                no_stale: None,
                stale_threshold: None,
                message: None,
            },
        );
        guards.insert(
            "*→stable".to_string(),
            TransitionGuard {
                min_score: Some(85),
                require_sections: vec!["Public API".to_string()],
                no_stale: Some(true),
                stale_threshold: None,
                message: None,
            },
        );

        let config = LifecycleConfig {
            guards,
            track_history: true,
        };

        // Specific match
        let found = find_guards(&config, &SpecStatus::Review, &SpecStatus::Active);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].min_score, Some(70));

        // Wildcard match
        let found = find_guards(&config, &SpecStatus::Active, &SpecStatus::Stable);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].min_score, Some(85));

        // No match
        let found = find_guards(&config, &SpecStatus::Draft, &SpecStatus::Review);
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn find_guards_ascii_arrow() {
        let mut guards = std::collections::HashMap::new();
        guards.insert(
            "draft->review".to_string(),
            TransitionGuard {
                min_score: Some(30),
                require_sections: vec![],
                no_stale: None,
                stale_threshold: None,
                message: None,
            },
        );

        let config = LifecycleConfig {
            guards,
            track_history: true,
        };

        let found = find_guards(&config, &SpecStatus::Draft, &SpecStatus::Review);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].min_score, Some(30));
    }
}
