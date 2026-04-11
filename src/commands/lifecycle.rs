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
/// Only replaces `status:` within the YAML frontmatter (between `---` delimiters),
/// never in the body content.
/// Returns the new file content, or None if the status line wasn't found.
fn update_status_in_content(content: &str, new_status: &str) -> Option<String> {
    // Find frontmatter boundaries (first --- to second ---)
    let first = content.find("---\n")?;
    let rest = &content[first + 4..];
    let second = rest.find("\n---")?;
    let fm_end = first + 4 + second;

    let frontmatter = &content[first..fm_end];
    if STATUS_LINE_RE.is_match(frontmatter) {
        let new_fm = STATUS_LINE_RE
            .replace(frontmatter, format!("status: {new_status}"))
            .to_string();
        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..first]);
        result.push_str(&new_fm);
        result.push_str(&content[fm_end..]);
        Some(result)
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
            match std::fs::read_to_string(spec_path) {
                Ok(content) => {
                    let parsed = parser::parse_frontmatter(&content.replace("\r\n", "\n"));
                    match parsed {
                        Some(parsed) => {
                            let missing =
                                parser::get_missing_sections(&parsed.body, &guard.require_sections);
                            if !missing.is_empty() {
                                failures.push(format!(
                                    "guard: missing required sections: {}",
                                    missing.join(", ")
                                ));
                            }
                        }
                        None => {
                            failures.push(format!("guard: could not parse frontmatter for {rel}"));
                        }
                    }
                }
                Err(e) => {
                    failures.push(format!("guard: could not read spec {rel}: {e}"));
                }
            }
        }

        // Check staleness
        if guard.no_stale.unwrap_or(false) {
            let threshold = guard.stale_threshold.unwrap_or(5);
            match std::fs::read_to_string(spec_path) {
                Ok(content) => {
                    let parsed = parser::parse_frontmatter(&content.replace("\r\n", "\n"));
                    match parsed {
                        Some(parsed) => {
                            for source_file in &parsed.frontmatter.files {
                                let commits =
                                    git_utils::git_commits_between(root, &rel, source_file);
                                if commits >= threshold {
                                    failures.push(format!(
                                        "guard: stale — {source_file} has {commits} commits since spec was last updated (threshold: {threshold})"
                                    ));
                                }
                            }
                        }
                        None => {
                            failures.push(format!("guard: could not parse frontmatter for {rel}"));
                        }
                    }
                }
                Err(e) => {
                    failures.push(format!("guard: could not read spec {rel}: {e}"));
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

    // Use frontmatter lifecycle_log, or fall back to external JSON (post-migration)
    let log_owned;
    let log = if parsed.frontmatter.lifecycle_log.is_empty() {
        let module = parsed
            .frontmatter
            .module
            .clone()
            .unwrap_or_else(|| derive_module_from_path(&spec_path));
        log_owned = load_lifecycle_json(root, &module);
        &log_owned
    } else {
        &parsed.frontmatter.lifecycle_log
    };

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

/// `specsync lifecycle auto-promote` — scan all specs and promote any that pass guards.
pub fn cmd_auto_promote(root: &Path, format: OutputFormat, dry_run: bool) {
    let (config, spec_files) = load_and_discover(root, false);
    let mut promoted: Vec<(String, String, String)> = Vec::new(); // (rel, from, to)
    let mut skipped: Vec<(String, String, Vec<String>)> = Vec::new(); // (rel, reason, failures)

    for spec_path in &spec_files {
        let (content, current, rel) = read_spec_status(root, spec_path);

        let current = match current {
            Some(s) => s,
            None => {
                skipped.push((rel, "no valid status".to_string(), vec![]));
                continue;
            }
        };

        let next = match current.next() {
            Some(n) => n,
            None => continue, // Already at end of lifecycle — not an error
        };

        if !current.can_transition_to(&next) {
            continue; // Skip invalid transitions silently
        }

        let guard_result = evaluate_guards(root, spec_path, &config, &current, &next);
        if !guard_result.passed {
            skipped.push((
                rel,
                format!("{} → {}: guards failed", current.as_str(), next.as_str()),
                guard_result.failures,
            ));
            continue;
        }

        // Guards passed — promote
        if dry_run {
            promoted.push((rel, current.as_str().to_string(), next.as_str().to_string()));
        } else {
            let new_content = match update_status_in_content(&content, next.as_str()) {
                Some(c) => c,
                None => {
                    skipped.push((rel, "could not find status line".to_string(), vec![]));
                    continue;
                }
            };

            let final_content = if config.lifecycle.track_history {
                let today = chrono_today();
                let entry = format!(
                    "{today}: {} → {} (auto-promote)",
                    current.as_str(),
                    next.as_str()
                );
                append_lifecycle_log_entry(&new_content, &entry)
            } else {
                new_content
            };

            if let Err(e) = std::fs::write(spec_path, &final_content) {
                skipped.push((rel, format!("write failed: {e}"), vec![]));
                continue;
            }

            promoted.push((rel, current.as_str().to_string(), next.as_str().to_string()));
        }
    }

    match format {
        OutputFormat::Json => {
            let promoted_json: Vec<serde_json::Value> = promoted
                .iter()
                .map(|(rel, from, to)| {
                    serde_json::json!({
                        "spec": rel,
                        "from": from,
                        "to": to,
                    })
                })
                .collect();
            let skipped_json: Vec<serde_json::Value> = skipped
                .iter()
                .map(|(rel, reason, failures)| {
                    serde_json::json!({
                        "spec": rel,
                        "reason": reason,
                        "failures": failures,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "dry_run": dry_run,
                "promoted": promoted_json,
                "skipped": skipped_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            if dry_run {
                println!("{}", "Dry run — no files will be modified.\n".dimmed());
            }

            if promoted.is_empty() && skipped.is_empty() {
                println!("No specs eligible for auto-promotion.");
                return;
            }

            if !promoted.is_empty() {
                println!(
                    "{} {} spec(s) {}:\n",
                    "✓".green(),
                    promoted.len(),
                    if dry_run {
                        "would be promoted"
                    } else {
                        "promoted"
                    }
                );
                for (rel, from, to) in &promoted {
                    println!("  {} {} → {}", rel, from.dimmed(), to.green());
                }
            }

            if !skipped.is_empty() {
                println!("\n{} {} spec(s) skipped:\n", "⚠".yellow(), skipped.len());
                for (rel, reason, failures) in &skipped {
                    println!("  {} {rel}: {reason}", "—".dimmed());
                    for f in failures {
                        println!("      {}", f.dimmed());
                    }
                }
            }
        }
    }
}

/// `specsync lifecycle enforce` — CI enforcement: validate lifecycle rules, exit non-zero on violations.
pub fn cmd_enforce(
    root: &Path,
    format: OutputFormat,
    require_status: bool,
    check_max_age: bool,
    check_allowed: bool,
) {
    let (config, spec_files) = load_and_discover(root, false);
    let mut violations: Vec<(String, String)> = Vec::new(); // (spec_rel, message)

    let allowed_set: Vec<SpecStatus> = config
        .lifecycle
        .allowed_statuses
        .iter()
        .filter_map(|s| SpecStatus::from_str_loose(s))
        .collect();

    for spec_path in &spec_files {
        let rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .display()
            .to_string();

        let content = match std::fs::read_to_string(spec_path) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };

        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => {
                if require_status {
                    violations.push((rel, "could not parse frontmatter".to_string()));
                }
                continue;
            }
        };

        let status = parsed.frontmatter.parsed_status();

        // Check: require status field
        if require_status && status.is_none() {
            violations.push((rel.clone(), "missing status field".to_string()));
        }

        if let Some(status) = &status {
            // Check: allowed statuses
            if check_allowed && !allowed_set.is_empty() && !allowed_set.contains(status) {
                let allowed_str = config.lifecycle.allowed_statuses.join(", ");
                violations.push((
                    rel.clone(),
                    format!(
                        "status '{}' not in allowed list ({})",
                        status.as_str(),
                        allowed_str
                    ),
                ));
            }

            // Check: max age
            if check_max_age {
                if let Some(max_days) = config.lifecycle.max_age.get(status.as_str()) {
                    // Look at lifecycle_log (frontmatter or external JSON) for the most recent transition
                    let lifecycle_log = if parsed.frontmatter.lifecycle_log.is_empty() {
                        let module = parsed
                            .frontmatter
                            .module
                            .clone()
                            .unwrap_or_else(|| derive_module_from_path(spec_path));
                        load_lifecycle_json(root, &module)
                    } else {
                        parsed.frontmatter.lifecycle_log.clone()
                    };
                    let age_days = estimate_status_age(root, &rel, &lifecycle_log, status);
                    if let Some(age) = age_days {
                        if age > *max_days {
                            violations.push((
                                rel.clone(),
                                format!(
                                    "stuck in '{}' for ~{} days (max: {} days)",
                                    status.as_str(),
                                    age,
                                    max_days
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    let violation_count = violations.len();

    match format {
        OutputFormat::Json => {
            let items: Vec<serde_json::Value> = violations
                .iter()
                .map(|(spec, msg)| {
                    serde_json::json!({
                        "spec": spec,
                        "violation": msg,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "total_specs": spec_files.len(),
                "violations": violation_count,
                "details": items,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            if violations.is_empty() {
                println!(
                    "{} All {} specs pass lifecycle enforcement checks.",
                    "✓".green(),
                    spec_files.len()
                );
                return;
            }

            println!(
                "{} {} violation(s) across {} specs:\n",
                "✗".red().bold(),
                violation_count,
                spec_files.len()
            );

            for (spec, msg) in &violations {
                println!("  {} {spec}: {msg}", "✗".red());
            }

            println!(
                "\n{} Fix violations or adjust lifecycle config in .specsync/config.toml (run `specsync migrate` for older projects).",
                "Tip:".cyan()
            );
        }
    }

    if violation_count > 0 {
        process::exit(1);
    }
}

/// Estimate how many days a spec has been in its current status.
/// Uses lifecycle_log entries (format: "YYYY-MM-DD: from → to") or falls back to git.
fn estimate_status_age(
    root: &Path,
    spec_rel: &str,
    lifecycle_log: &[String],
    current_status: &SpecStatus,
) -> Option<u64> {
    // Try lifecycle_log first — look for the most recent entry that transitions INTO current status
    let target_suffix = format!("→ {}", current_status.as_str());
    let target_suffix_ascii = format!("-> {}", current_status.as_str());

    for entry in lifecycle_log.iter().rev() {
        if entry.contains(&target_suffix) || entry.contains(&target_suffix_ascii) {
            // Extract date from "YYYY-MM-DD: ..."
            if let Some(date_str) = entry.split(':').next() {
                let date_str = date_str.trim();
                if let Some(days) = days_since_date(date_str) {
                    return Some(days);
                }
            }
        }
    }

    // Fallback: use git to find last modification date of the spec file
    // Normalize path separators for git (backslashes on Windows break git path matching)
    let git_path = spec_rel.replace('\\', "/");
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%ct", "--", &git_path])
        .current_dir(root)
        .output()
        .ok()?;

    let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let timestamp: u64 = timestamp_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    Some((now - timestamp) / 86400)
}

/// Calculate days since a YYYY-MM-DD date string.
fn days_since_date(date_str: &str) -> Option<u64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;

    // Simple days-since calculation using Unix-like date math
    // Julian day number approximation
    let jdn = |y: i64, m: i64, d: i64| -> i64 {
        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
    };

    let then_jdn = jdn(year, month, day);

    // Get today's date
    let today_str = chrono_today();
    let today_parts: Vec<&str> = today_str.split('-').collect();
    if today_parts.len() != 3 {
        return None;
    }
    let ty: i64 = today_parts[0].parse().ok()?;
    let tm: i64 = today_parts[1].parse().ok()?;
    let td: i64 = today_parts[2].parse().ok()?;
    let today_jdn = jdn(ty, tm, td);

    let diff = today_jdn - then_jdn;
    if diff >= 0 {
        Some(diff as u64)
    } else {
        Some(0)
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
/// Uses pure Rust (no shell dependency) so it works cross-platform including Windows.
fn chrono_today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert Unix timestamp to date using civil calendar math
    let days = (secs / 86400) as i64;
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Load lifecycle log entries from the external JSON file (post-migration).
/// After `specsync migrate` extracts lifecycle_log from frontmatter to
/// `.specsync/lifecycle/{module}.json`, this function reads those entries
/// so history/enforce commands still work.
fn load_lifecycle_json(root: &Path, module: &str) -> Vec<String> {
    let path = root.join(format!(".specsync/lifecycle/{module}.json"));
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    data["entries"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e["raw"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Derive a module name from a spec file path (fallback when frontmatter has no module field).
fn derive_module_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.strip_suffix(".spec").unwrap_or(s).to_string())
        .unwrap_or_default()
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
            max_age: std::collections::HashMap::new(),
            allowed_statuses: vec![],
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
            max_age: std::collections::HashMap::new(),
            allowed_statuses: vec![],
        };

        let found = find_guards(&config, &SpecStatus::Draft, &SpecStatus::Review);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].min_score, Some(30));
    }

    #[test]
    fn days_since_date_same_day_is_zero() {
        let today = chrono_today();
        assert_eq!(days_since_date(&today), Some(0));
    }

    #[test]
    fn days_since_date_invalid_format_returns_none() {
        assert_eq!(days_since_date("not-a-date"), None);
        assert_eq!(days_since_date("2026"), None);
        assert_eq!(days_since_date(""), None);
    }

    #[test]
    fn days_since_date_past_date_is_positive() {
        // Use a date far in the past to ensure it's always positive
        let result = days_since_date("2020-01-01");
        assert!(result.is_some());
        assert!(result.unwrap() > 365);
    }

    #[test]
    fn estimate_status_age_from_lifecycle_log() {
        let today = chrono_today();
        let log = vec![format!("{today}: draft → review")];
        let age = estimate_status_age(
            Path::new("/tmp"),
            "specs/test.spec.md",
            &log,
            &SpecStatus::Review,
        );
        assert_eq!(age, Some(0));
    }

    #[test]
    fn estimate_status_age_picks_latest_entry() {
        let today = chrono_today();
        let log = vec![
            "2020-01-01: draft → review".to_string(),
            "2020-06-01: review → draft".to_string(),
            format!("{today}: draft → review"),
        ];
        let age = estimate_status_age(
            Path::new("/tmp"),
            "specs/test.spec.md",
            &log,
            &SpecStatus::Review,
        );
        // Should pick the most recent entry (today)
        assert_eq!(age, Some(0));
    }
}
