use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::parser;
use crate::types;
use crate::validator::compute_coverage;

use super::load_and_discover;

pub fn cmd_report(root: &Path, format: types::OutputFormat, stale_threshold: usize) {
    let (config, spec_files) = load_and_discover(root, true);
    let coverage = compute_coverage(root, &spec_files, &config);

    // Build per-module stats from spec files
    struct ModuleInfo {
        spec_path: String,
        module_name: String,
        source_files: Vec<String>,
        coverage_pct: f64,
        stale: bool,
        stale_commits_behind: usize,
        incomplete: bool,
        missing_fields: Vec<String>,
        empty_sections: Vec<String>,
    }

    let mut modules: Vec<ModuleInfo> = Vec::new();

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
        let body = &parsed.body;

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

        // Coverage: how many of this spec's source files exist
        let existing: usize = fm.files.iter().filter(|f| root.join(f).exists()).count();
        let total_files = fm.files.len().max(1);
        let cov = (existing as f64 / total_files as f64) * 100.0;

        // Stale detection via git log
        let mut stale = false;
        let mut max_behind: usize = 0;
        if !fm.files.is_empty() {
            let spec_commit = git_last_commit_hash(root, &rel_spec);
            for source_file in &fm.files {
                if !root.join(source_file).exists() {
                    continue;
                }
                let behind = git_commits_between(root, &rel_spec, source_file);
                if behind >= stale_threshold {
                    stale = true;
                    max_behind = max_behind.max(behind);
                }
            }
            // If we couldn't get git info, skip stale
            if spec_commit.is_none() {
                stale = false;
            }
        }

        // Incomplete detection
        let mut missing_fields = Vec::new();
        let mut empty_sections = Vec::new();

        if fm.status.is_none() {
            missing_fields.push("status".to_string());
        }
        if fm.module.is_none() {
            missing_fields.push("module".to_string());
        }
        if fm.version.is_none() {
            missing_fields.push("version".to_string());
        }

        // Check required sections for empty/stub content
        for section_name in &["Public API", "Invariants"] {
            let header = format!("## {section_name}");
            if let Some(start) = body.find(&header) {
                let after = start + header.len();
                // Find next ## heading
                let section_body = if let Some(next) = body[after..].find("\n## ") {
                    &body[after..after + next]
                } else {
                    &body[after..]
                };
                let trimmed = section_body.trim();
                if trimmed.is_empty()
                    || trimmed == "TODO"
                    || trimmed == "TBD"
                    || trimmed == "N/A"
                    || trimmed.starts_with("<!-- ")
                {
                    empty_sections.push(section_name.to_string());
                }
            } else {
                empty_sections.push(format!("{section_name} (missing)"));
            }
        }

        let incomplete = !missing_fields.is_empty() || !empty_sections.is_empty();

        modules.push(ModuleInfo {
            spec_path: rel_spec,
            module_name,
            source_files: fm.files.clone(),
            coverage_pct: cov,
            stale,
            stale_commits_behind: max_behind,
            incomplete,
            missing_fields,
            empty_sections,
        });
    }

    // Sort by module name
    modules.sort_by(|a, b| a.module_name.cmp(&b.module_name));

    let total_modules = modules.len();
    let stale_count = modules.iter().filter(|m| m.stale).count();
    let incomplete_count = modules.iter().filter(|m| m.incomplete).count();
    let overall_coverage = if coverage.total_source_files == 0 {
        100.0
    } else {
        (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
    };

    match format {
        types::OutputFormat::Json => {
            let module_json: Vec<serde_json::Value> = modules
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "module": m.module_name,
                        "spec_path": m.spec_path,
                        "source_files": m.source_files,
                        "coverage_pct": (m.coverage_pct * 100.0).round() / 100.0,
                        "stale": m.stale,
                        "commits_behind": m.stale_commits_behind,
                        "incomplete": m.incomplete,
                        "missing_fields": m.missing_fields,
                        "empty_sections": m.empty_sections,
                    })
                })
                .collect();

            let output = serde_json::json!({
                "overall_coverage_pct": (overall_coverage * 100.0).round() / 100.0,
                "files_covered": coverage.specced_file_count,
                "files_total": coverage.total_source_files,
                "total_modules": total_modules,
                "stale_modules": stale_count,
                "incomplete_modules": incomplete_count,
                "stale_threshold": stale_threshold,
                "modules": module_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!(
                "\n--- {} ------------------------------------------------",
                "Spec Coverage Report".bold()
            );
            println!(
                "\n  Overall: {}/{} files covered ({:.1}%)",
                coverage.specced_file_count, coverage.total_source_files, overall_coverage,
            );
            println!(
                "  Modules: {} total, {} stale, {} incomplete\n",
                total_modules, stale_count, incomplete_count,
            );

            // Table header
            println!(
                "  {:<20} {:>8}  {:>7}  {:>10}",
                "Module", "Coverage", "Stale", "Incomplete"
            );
            println!("  {}", "-".repeat(52));

            for m in &modules {
                let cov_str = format!("{:.0}%", m.coverage_pct);
                let stale_str = if m.stale {
                    format!("{} behind", m.stale_commits_behind)
                        .yellow()
                        .to_string()
                } else {
                    "no".green().to_string()
                };
                let incomplete_str = if m.incomplete {
                    "yes".yellow().to_string()
                } else {
                    "no".green().to_string()
                };
                println!(
                    "  {:<20} {:>8}  {:>7}  {:>10}",
                    m.module_name, cov_str, stale_str, incomplete_str
                );
            }

            // Stale details
            let stale_modules: Vec<&ModuleInfo> = modules.iter().filter(|m| m.stale).collect();
            if !stale_modules.is_empty() {
                println!(
                    "\n  {} ({}) (>{} commits behind):",
                    "Stale modules".yellow().bold(),
                    stale_modules.len(),
                    stale_threshold,
                );
                for m in &stale_modules {
                    println!(
                        "    {} {} — {} commits behind source",
                        "⚠".yellow(),
                        m.module_name,
                        m.stale_commits_behind,
                    );
                }
            }

            // Incomplete details
            let incomplete_modules: Vec<&ModuleInfo> =
                modules.iter().filter(|m| m.incomplete).collect();
            if !incomplete_modules.is_empty() {
                println!(
                    "\n  {} ({}):",
                    "Incomplete modules".yellow().bold(),
                    incomplete_modules.len(),
                );
                for m in &incomplete_modules {
                    let mut reasons = Vec::new();
                    if !m.missing_fields.is_empty() {
                        reasons.push(format!("missing fields: {}", m.missing_fields.join(", ")));
                    }
                    if !m.empty_sections.is_empty() {
                        reasons.push(format!("empty sections: {}", m.empty_sections.join(", ")));
                    }
                    println!(
                        "    {} {} — {}",
                        "⚠".yellow(),
                        m.module_name,
                        reasons.join("; "),
                    );
                }
            }

            println!();
        }
    }
}

/// Get the last commit hash that touched a file.
fn git_last_commit_hash(root: &Path, file: &str) -> Option<String> {
    let output = process::Command::new("git")
        .args(["log", "-1", "--format=%H", "--", file])
        .current_dir(root)
        .output()
        .ok()?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() { None } else { Some(hash) }
}

/// Count commits that touched `source_file` since `spec_file` was last modified.
fn git_commits_between(root: &Path, spec_file: &str, source_file: &str) -> usize {
    // Get the last commit that touched the spec
    let spec_commit = match git_last_commit_hash(root, spec_file) {
        Some(h) => h,
        None => return 0,
    };

    // Count commits to source_file since that spec commit
    let output = match process::Command::new("git")
        .args([
            "rev-list",
            "--count",
            &format!("{spec_commit}..HEAD"),
            "--",
            source_file,
        ])
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(_) => return 0,
    };

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
}
