use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::git_utils::{StaleInfo, git_commits_between, git_last_commit_hash, is_git_repo};
use crate::parser;
use crate::types;

use super::load_and_discover;

pub fn cmd_stale(root: &Path, format: types::OutputFormat, threshold: usize) {
    if !is_git_repo(root) {
        match format {
            types::OutputFormat::Json => {
                let output = serde_json::json!({
                    "error": "not a git repository",
                    "stale_specs": [],
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            _ => {
                eprintln!(
                    "{} Not a git repository — staleness detection requires git history.",
                    "Error:".red().bold()
                );
            }
        }
        std::process::exit(1);
    }

    let (_config, spec_files) = load_and_discover(root, false);

    let mut stale_specs: Vec<StaleInfo> = Vec::new();
    let mut fresh_count = 0;

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let fm = &parsed.frontmatter;
        let module_name = fm.module.clone().unwrap_or_else(|| {
            spec_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .strip_suffix(".spec")
                .unwrap_or("unknown")
                .to_string()
        });

        let rel_spec = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        if fm.files.is_empty() {
            fresh_count += 1;
            continue;
        }

        let spec_commit = git_last_commit_hash(root, &rel_spec);
        if spec_commit.is_none() {
            // Spec not yet tracked by git — skip
            fresh_count += 1;
            continue;
        }

        let mut max_behind: usize = 0;
        let mut source_details: Vec<(String, usize)> = Vec::new();

        for source_file in &fm.files {
            if !root.join(source_file).exists() {
                continue;
            }
            let behind = git_commits_between(root, &rel_spec, source_file);
            if behind > 0 {
                source_details.push((source_file.clone(), behind));
            }
            max_behind = max_behind.max(behind);
        }

        let is_stale = max_behind >= threshold;
        if is_stale {
            stale_specs.push(StaleInfo {
                spec_path: rel_spec,
                module_name,
                max_commits_behind: max_behind,
                source_details,
            });
        } else {
            fresh_count += 1;
        }
    }

    // Sort by most stale first
    stale_specs.sort_by(|a, b| b.max_commits_behind.cmp(&a.max_commits_behind));

    let total = spec_files.len();
    let stale_count = stale_specs.len();

    match format {
        types::OutputFormat::Json => {
            let specs_json: Vec<serde_json::Value> = stale_specs
                .iter()
                .map(|s| {
                    let details: Vec<serde_json::Value> = s
                        .source_details
                        .iter()
                        .map(|(file, behind)| {
                            serde_json::json!({
                                "file": file,
                                "commits_behind": behind,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "spec_path": s.spec_path,
                        "module": s.module_name,
                        "commits_behind": s.max_commits_behind,
                        "source_files": details,
                    })
                })
                .collect();

            let output = serde_json::json!({
                "total_specs": total,
                "stale_count": stale_count,
                "fresh_count": fresh_count,
                "threshold": threshold,
                "stale_specs": specs_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            println!("## Stale Spec Report\n");
            println!(
                "**{stale_count}** of **{total}** specs are stale (>{threshold} commits behind)\n"
            );

            if stale_specs.is_empty() {
                println!("All specs are up to date! :white_check_mark:");
            } else {
                println!("| Module | Spec | Commits Behind | Drifted Files |");
                println!("|--------|------|---------------|---------------|");
                for s in &stale_specs {
                    let drifted: Vec<String> = s
                        .source_details
                        .iter()
                        .map(|(f, n)| format!("`{f}` ({n})"))
                        .collect();
                    println!(
                        "| {} | {} | {} | {} |",
                        s.module_name,
                        s.spec_path,
                        s.max_commits_behind,
                        drifted.join(", "),
                    );
                }
                println!(
                    "\n> Run `specsync check` to validate these specs, or update them to match current source."
                );
            }
        }
        types::OutputFormat::Text => {
            println!(
                "\n--- {} ------------------------------------------------",
                "Stale Spec Detection".bold()
            );
            println!(
                "\n  Threshold: {} commit(s) behind source files",
                threshold.to_string().cyan()
            );
            println!(
                "  Result:    {}/{} specs are stale\n",
                if stale_count > 0 {
                    stale_count.to_string().yellow().bold().to_string()
                } else {
                    stale_count.to_string().green().to_string()
                },
                total
            );

            if stale_specs.is_empty() {
                println!(
                    "  {} All specs are up to date with their source files.",
                    "✓".green()
                );
            } else {
                for s in &stale_specs {
                    println!(
                        "  {} {} — {} commits behind",
                        "⚠".yellow(),
                        s.module_name.bold(),
                        s.max_commits_behind.to_string().yellow(),
                    );
                    println!("    spec: {}", s.spec_path.dimmed());
                    for (file, behind) in &s.source_details {
                        println!(
                            "      {} {file} ({behind} commit{})",
                            "→".dimmed(),
                            if *behind == 1 { "" } else { "s" },
                        );
                    }
                }

                println!(
                    "\n  {} Run {} to validate, or update specs to match source.",
                    "Tip:".cyan(),
                    "specsync check".bold(),
                );
            }
            println!();
        }
    }

    // Exit with non-zero if stale specs found
    if stale_count > 0 {
        std::process::exit(1);
    }
}
