use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::commands::{build_schema_columns, run_validation};
use crate::config::load_config;
use crate::github;
use crate::parser;
use crate::types;
use crate::validator::{find_spec_files, get_schema_table_names};

pub(crate) fn cmd_issues(root: &Path, format: types::OutputFormat, create: bool) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_spec_files(&specs_dir);

    if spec_files.is_empty() {
        println!("No spec files found.");
        return;
    }

    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match github::resolve_repo(repo_config, root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            process::exit(1);
        }
    };

    if matches!(format, types::OutputFormat::Text) {
        println!("Verifying issue references against {repo}...\n");
    }

    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;
    let mut total_errors = 0usize;
    let mut json_results: Vec<serde_json::Value> = Vec::new();

    for spec_path in &spec_files {
        let content = match fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let fm = &parsed.frontmatter;
        if fm.implements.is_empty() && fm.tracks.is_empty() {
            continue;
        }

        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();

        let verification = github::verify_spec_issues(&repo, &rel_path, &fm.implements, &fm.tracks);

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();
        total_errors += verification.errors.len();

        match format {
            types::OutputFormat::Text => {
                if !verification.valid.is_empty()
                    || !verification.closed.is_empty()
                    || !verification.not_found.is_empty()
                    || !verification.errors.is_empty()
                {
                    println!("  {}", rel_path.bold());

                    for issue in &verification.valid {
                        println!(
                            "    {} #{} — {} (open)",
                            "✓".green(),
                            issue.number,
                            issue.title
                        );
                    }
                    for issue in &verification.closed {
                        println!(
                            "    {} #{} — {} (closed — spec may need updating)",
                            "⚠".yellow(),
                            issue.number,
                            issue.title
                        );
                    }
                    for num in &verification.not_found {
                        println!("    {} #{num} — not found", "✗".red());
                    }
                    for err in &verification.errors {
                        println!("    {} {err}", "✗".red());
                    }
                    println!();
                }
            }
            types::OutputFormat::Json | types::OutputFormat::Markdown => {
                json_results.push(serde_json::json!({
                    "spec": rel_path,
                    "valid": verification.valid.iter().map(|i| serde_json::json!({
                        "number": i.number,
                        "title": i.title,
                        "state": i.state,
                    })).collect::<Vec<_>>(),
                    "closed": verification.closed.iter().map(|i| serde_json::json!({
                        "number": i.number,
                        "title": i.title,
                    })).collect::<Vec<_>>(),
                    "not_found": verification.not_found,
                    "errors": verification.errors,
                }));
            }
        }
    }

    // If --create, also run validation and create issues for drift
    if create {
        let schema_tables = get_schema_table_names(root, &config);
        let schema_columns = build_schema_columns(root, &config);
        let (_, _, _, _, all_errors, _) = run_validation(
            root,
            &spec_files,
            &schema_tables,
            &schema_columns,
            &config,
            true,
        );
        if !all_errors.is_empty() {
            create_drift_issues(root, &config, &all_errors, format);
        }
    }

    match format {
        types::OutputFormat::Json => {
            let output = serde_json::json!({
                "repo": repo,
                "valid": total_valid,
                "closed": total_closed,
                "not_found": total_not_found,
                "errors": total_errors,
                "specs": json_results,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown => {
            println!("## Issue Verification — {repo}\n");
            println!("| Metric | Count |");
            println!("|--------|-------|");
            println!("| Valid (open) | {total_valid} |");
            println!("| Closed | {total_closed} |");
            println!("| Not found | {total_not_found} |");
            println!("| Errors | {total_errors} |");
        }
        types::OutputFormat::Text => {
            let total_refs = total_valid + total_closed + total_not_found;
            if total_refs == 0 {
                println!(
                    "{}",
                    "No issue references found in spec frontmatter.".cyan()
                );
                println!(
                    "Add `implements: [42]` or `tracks: [10]` to spec frontmatter to link issues."
                );
            } else {
                println!(
                    "Issue references: {} valid, {} closed, {} not found",
                    total_valid.to_string().green(),
                    total_closed.to_string().yellow(),
                    total_not_found.to_string().red(),
                );
            }
        }
    }

    if total_not_found > 0 || total_errors > 0 {
        process::exit(1);
    }
}

/// Create GitHub issues for specs with validation errors.
/// `all_errors` contains strings in the format `"spec/path: error message"`.
pub(crate) fn create_drift_issues(
    root: &Path,
    config: &types::SpecSyncConfig,
    all_errors: &[String],
    format: types::OutputFormat,
) {
    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = match github::resolve_repo(repo_config, root) {
        Ok(r) => r,
        Err(e) => {
            if matches!(format, types::OutputFormat::Text) {
                eprintln!("{} Cannot create issues: {e}", "error:".red().bold());
            }
            return;
        }
    };

    let labels = config
        .github
        .as_ref()
        .map(|g| g.drift_labels.clone())
        .unwrap_or_else(|| vec!["spec-drift".to_string()]);

    // Group errors by spec path (format: "spec/path: error message")
    let mut errors_by_spec: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for entry in all_errors {
        if let Some((spec, error)) = entry.split_once(": ") {
            errors_by_spec
                .entry(spec.to_string())
                .or_default()
                .push(error.to_string());
        }
    }

    if matches!(format, types::OutputFormat::Text) {
        println!(
            "\n{} Creating GitHub issues for {} spec(s) with errors...",
            "⟳".cyan(),
            errors_by_spec.len()
        );
    }

    for (spec_path, errors) in &errors_by_spec {
        match github::create_drift_issue(&repo, spec_path, errors, &labels) {
            Ok(issue) => {
                if matches!(format, types::OutputFormat::Text) {
                    println!(
                        "  {} Created issue #{} for {spec_path}: {}",
                        "✓".green(),
                        issue.number,
                        issue.url
                    );
                }
            }
            Err(e) => {
                if matches!(format, types::OutputFormat::Text) {
                    eprintln!(
                        "  {} Failed to create issue for {spec_path}: {e}",
                        "✗".red()
                    );
                }
            }
        }
    }
}
