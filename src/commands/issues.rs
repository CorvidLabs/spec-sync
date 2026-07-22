use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::github;
use crate::parser;
use crate::types;
use crate::validator::{find_spec_files, get_schema_table_names};

use super::{build_schema_columns, create_drift_issues, run_validation};

fn issue_text_summary(
    reference_specs: usize,
    valid: usize,
    closed: usize,
    not_found: usize,
    errors: usize,
) -> Option<String> {
    (reference_specs > 0).then(|| {
        format!(
            "Issue references: {valid} valid, {closed} closed, {not_found} not found, {errors} errors"
        )
    })
}

pub fn cmd_issues(root: &Path, format: types::OutputFormat, create: bool) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_spec_files(&specs_dir);

    if spec_files.is_empty() {
        println!("No spec files found.");
        return;
    }

    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;
    let mut total_errors = 0usize;
    let mut json_results: Vec<serde_json::Value> = Vec::new();
    let mut references = Vec::new();

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

        if !fm.implements.is_empty() || !fm.tracks.is_empty() {
            references.push((rel_path, fm.implements.clone(), fm.tracks.clone()));
        }
    }

    let repo_config = config.github.as_ref().and_then(|g| g.repo.as_deref());
    let repo = if references.is_empty() {
        repo_config.map(str::to_owned)
    } else {
        match github::resolve_repo(repo_config, root) {
            Ok(repo) => Some(repo),
            Err(error) => {
                eprintln!("{} {error}", "error:".red().bold());
                process::exit(1);
            }
        }
    };

    if matches!(format, types::OutputFormat::Text)
        && let Some(repo) = repo.as_deref()
        && !references.is_empty()
    {
        println!("Verifying issue references against {repo}...\n");
    }

    let verifications = repo
        .as_deref()
        .filter(|_| !references.is_empty())
        .map(|repo| github::verify_issue_batch(repo, &references))
        .unwrap_or_default();

    for verification in verifications {
        let rel_path = verification.spec_path.clone();

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();
        total_errors += verification.errors.len();

        match format {
            types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
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
            types::OutputFormat::Json
            | types::OutputFormat::Markdown
            | types::OutputFormat::Github => {
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
        let ignore_rules = crate::ignore::IgnoreRules::default();
        let (_, _, _, _, all_errors, _, _) = run_validation(
            root,
            &spec_files,
            &spec_files,
            &schema_tables,
            &schema_columns,
            &config,
            true,
            false,
            &ignore_rules,
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
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            if let Some(repo) = repo.as_deref() {
                println!("## Issue Verification — {repo}\n");
            } else {
                println!("## Issue Verification\n");
            }
            println!("| Metric | Count |");
            println!("|--------|-------|");
            println!("| Valid (open) | {total_valid} |");
            println!("| Closed | {total_closed} |");
            println!("| Not found | {total_not_found} |");
            println!("| Errors | {total_errors} |");
        }
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            if let Some(summary) = issue_text_summary(
                references.len(),
                total_valid,
                total_closed,
                total_not_found,
                total_errors,
            ) {
                println!("{summary}");
            } else {
                println!(
                    "{}",
                    "No issue references found in spec frontmatter.".cyan()
                );
                println!(
                    "Add `implements: [42]` or `tracks: [10]` to spec frontmatter to link issues."
                );
            }
        }
    }

    if total_not_found > 0 || total_errors > 0 {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::issue_text_summary;

    #[test]
    fn all_error_batches_report_errors_instead_of_no_reference_guidance() {
        let summary = issue_text_summary(1, 0, 0, 0, 2)
            .expect("a batch with references must produce a summary");

        assert_eq!(
            summary,
            "Issue references: 0 valid, 0 closed, 0 not found, 2 errors"
        );
        assert!(issue_text_summary(0, 0, 0, 0, 0).is_none());
    }
}
