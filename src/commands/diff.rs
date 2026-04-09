use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::exports::get_exported_symbols;
use crate::output::print_diff_markdown;
use crate::parser::parse_frontmatter;
use crate::types;

use super::load_and_discover;

pub fn cmd_diff(root: &Path, base: &str, format: types::OutputFormat) {
    let (config, spec_files) = load_and_discover(root, false);

    // Auto-detect PR context: when base is default "HEAD" and we're in a GitHub
    // Actions pull_request event, compare against the PR's base branch instead.
    let effective_base = if base == "HEAD" {
        detect_pr_base().unwrap_or_else(|| base.to_string())
    } else {
        base.to_string()
    };
    let base = effective_base.as_str();

    // Get list of files changed since base ref
    let output = match std::process::Command::new("git")
        .args(["diff", "--name-only", base])
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to run git diff: {e}");
            process::exit(1);
        }
    };

    let changed_files: std::collections::HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    if changed_files.is_empty() {
        match format {
            types::OutputFormat::Json => println!("{{\"changes\":[]}}"),
            types::OutputFormat::Markdown | types::OutputFormat::Github => {
                println!("## SpecSync Drift Report\n");
                println!("No files changed since `{base}`.");
            }
            types::OutputFormat::Text => println!("No files changed since {base}"),
        }
        return;
    }

    // Collect structured diff data for all specs
    struct DiffEntry {
        spec: String,
        changed_files: Vec<String>,
        new_exports: Vec<String>,
        removed_exports: Vec<String>,
    }

    let mut entries: Vec<DiffEntry> = Vec::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let spec_rel = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .replace('\\', "/");

        let affected_files: Vec<String> = parsed
            .frontmatter
            .files
            .iter()
            .filter(|f| changed_files.contains(*f))
            .cloned()
            .collect();

        if affected_files.is_empty() {
            continue;
        }

        // Get current exports from changed files
        let mut current_exports: Vec<String> = Vec::new();
        for file in &parsed.frontmatter.files {
            let full_path = root.join(file);
            current_exports.extend(get_exported_symbols(&full_path));
        }
        let mut seen = std::collections::HashSet::new();
        current_exports.retain(|s| seen.insert(s.clone()));

        // Get spec-documented symbols
        let spec_symbols = crate::parser::get_spec_symbols(&parsed.body);
        let spec_set: std::collections::HashSet<&str> =
            spec_symbols.iter().map(|s| s.as_str()).collect();
        let export_set: std::collections::HashSet<&str> =
            current_exports.iter().map(|s| s.as_str()).collect();

        let new_exports: Vec<String> = current_exports
            .iter()
            .filter(|s| !spec_set.contains(s.as_str()))
            .cloned()
            .collect();

        let removed_exports: Vec<String> = spec_symbols
            .iter()
            .filter(|s| !export_set.contains(s.as_str()))
            .cloned()
            .collect();

        entries.push(DiffEntry {
            spec: spec_rel,
            changed_files: affected_files,
            new_exports,
            removed_exports,
        });
    }

    match format {
        types::OutputFormat::Json => {
            let changes: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "spec": e.spec,
                        "changed_files": e.changed_files,
                        "new_exports": e.new_exports,
                        "removed_exports": e.removed_exports,
                    })
                })
                .collect();
            let output = serde_json::json!({ "changes": changes });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            #[allow(clippy::type_complexity)]
            let tuples: Vec<(String, Vec<String>, Vec<String>, Vec<String>)> = entries
                .iter()
                .map(|e| {
                    (
                        e.spec.clone(),
                        e.changed_files.clone(),
                        e.new_exports.clone(),
                        e.removed_exports.clone(),
                    )
                })
                .collect();
            print_diff_markdown(&tuples, &changed_files, &spec_files, root, &config, base);
        }
        types::OutputFormat::Text => {
            for entry in &entries {
                println!("\n{}", entry.spec.bold());
                println!("  Changed files: {}", entry.changed_files.join(", "));
                if !entry.new_exports.is_empty() {
                    println!(
                        "  {} New exports (not in spec): {}",
                        "+".green(),
                        entry.new_exports.join(", ")
                    );
                }
                if !entry.removed_exports.is_empty() {
                    println!(
                        "  {} Removed exports (still in spec): {}",
                        "-".red(),
                        entry.removed_exports.join(", ")
                    );
                }
                if entry.new_exports.is_empty() && entry.removed_exports.is_empty() {
                    println!("  {} Spec is up to date", "✓".green());
                }
            }

            if entries.is_empty() {
                // Check if any changed files are NOT covered by specs
                let specced_files: std::collections::HashSet<String> = spec_files
                    .iter()
                    .filter_map(|f| fs::read_to_string(f).ok())
                    .filter_map(|c| parse_frontmatter(&c.replace("\r\n", "\n")))
                    .flat_map(|p| p.frontmatter.files)
                    .collect();

                let untracked: Vec<&String> = changed_files
                    .iter()
                    .filter(|f| {
                        let path = std::path::Path::new(f.as_str());
                        crate::exports::has_extension(path, &config.source_extensions)
                            && !specced_files.contains(*f)
                    })
                    .collect();

                if untracked.is_empty() {
                    println!("No spec-tracked source files changed since {base}.");
                } else {
                    println!("Changed files not covered by any spec:");
                    for f in &untracked {
                        println!("  {} {f}", "?".yellow());
                    }
                }
            }
        }
    }
}

/// Detect the PR base ref from GitHub Actions environment variables.
/// Returns `Some("origin/<base_branch>")` when running in a pull_request event,
/// or `None` otherwise.
fn detect_pr_base() -> Option<String> {
    let event = std::env::var("GITHUB_EVENT_NAME").ok()?;
    if event != "pull_request" && event != "pull_request_target" {
        return None;
    }
    let base_ref = std::env::var("GITHUB_BASE_REF").ok()?;
    if base_ref.is_empty() {
        return None;
    }
    eprintln!(
        "specsync: detected PR context (GITHUB_BASE_REF={base_ref}), comparing against origin/{base_ref}"
    );
    Some(format!("origin/{base_ref}"))
}
