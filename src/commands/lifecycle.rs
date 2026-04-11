use colored::Colorize;
use regex::Regex;
use std::path::Path;
use std::process;
use std::sync::LazyLock;

use crate::parser;
use crate::types::{OutputFormat, SpecStatus};

use super::{filter_specs, load_and_discover};

static STATUS_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^status:\s*\S+").unwrap());

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

/// Resolve a single spec from user input (module name, path, etc.)
fn resolve_spec(root: &Path, spec_filter: &str) -> std::path::PathBuf {
    let (_, spec_files) = load_and_discover(root, false);
    let matched = filter_specs(root, &spec_files, &[spec_filter.to_string()]);
    if matched.is_empty() {
        eprintln!(
            "{} No spec matched: {}",
            "error:".red().bold(),
            spec_filter
        );
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
            eprintln!(
                "  {}",
                m.strip_prefix(root).unwrap_or(m).display()
            );
        }
        process::exit(1);
    }
    matched.into_iter().next().unwrap()
}

/// Read a spec file and return its current status, content, and relative path.
fn read_spec_status(
    root: &Path,
    spec_path: &Path,
) -> (String, Option<SpecStatus>, String) {
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

    let status = parser::parse_frontmatter(&content)
        .and_then(|p| p.frontmatter.parsed_status());

    (content, status, rel)
}

/// `specsync lifecycle promote <spec>`
pub fn cmd_promote(root: &Path, spec_filter: &str, format: OutputFormat, force: bool) {
    let spec_path = resolve_spec(root, spec_filter);
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

    write_status(&spec_path, &content, current, next, &rel, format);
}

/// `specsync lifecycle demote <spec>`
pub fn cmd_demote(root: &Path, spec_filter: &str, format: OutputFormat, force: bool) {
    let spec_path = resolve_spec(root, spec_filter);
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

    write_status(&spec_path, &content, current, prev, &rel, format);
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

    write_status(&spec_path, &content, current, target, &rel, format);
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

                println!(
                    "\n{} ({})",
                    colored_label,
                    paths.len()
                );
                for path in paths {
                    println!("  {path}");
                }
            }

            // Summary line
            println!();
            let summary: Vec<String> = SpecStatus::all()
                .iter()
                .filter_map(|s| {
                    counts.get(s.as_str()).map(|c| format!("{}: {c}", s.as_str()))
                })
                .collect();
            println!("{} specs — {}", entries.len(), summary.join(", "));
        }
    }
}

/// Write the updated status to disk and print the result.
fn write_status(
    spec_path: &Path,
    content: &str,
    from: SpecStatus,
    to: SpecStatus,
    rel: &str,
    format: OutputFormat,
) {
    let new_content = match update_status_in_content(content, to.as_str()) {
        Some(c) => c,
        None => {
            eprintln!(
                "{} {rel}: could not find status line in frontmatter",
                "error:".red().bold()
            );
            process::exit(1);
        }
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpecStatus;

    #[test]
    fn update_status_in_content_replaces_status_line() {
        let content = "---\nmodule: foo\nversion: 1\nstatus: draft\nfiles:\n  - src/foo.rs\n---\n# Foo\n";
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
}
