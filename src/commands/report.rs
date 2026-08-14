use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::git_utils::{git_commits_since, git_last_commit_hash};
use crate::output::{NO_FILES_MEASURED, percent_json};
use crate::parser;
use crate::types;
use crate::validator::compute_coverage_checked;

use super::{compute_exit_code, default_enforcement, filter_by_status, load_and_discover};

/// A percentage for a human-readable column, or a marker that there was
/// nothing to measure. `n/a` is deliberately not a number: a reader scanning
/// the column must not mistake an unmeasured module for a scored one.
fn percent_cell(percent: Option<f64>) -> String {
    match percent {
        Some(value) => format!("{value:.0}%"),
        None => "n/a".to_string(),
    }
}

/// A percentage for a CSV field. An empty field is how CSV spells "no value";
/// writing `0` or `100` there would report a measurement nobody took.
fn percent_csv(percent: Option<f64>, precision: usize) -> String {
    match percent {
        Some(value) => format!("{value:.precision$}"),
        None => String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_report(
    root: &Path,
    format: types::OutputFormat,
    stale_threshold: usize,
    exclude_status: &[String],
    only_status: &[String],
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
) {
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        let config = crate::config::load_config(root);
        default_enforcement(&config)
    });
    let (config, all_spec_files) = load_and_discover(root, true);
    let spec_files = filter_by_status(&all_spec_files, exclude_status, only_status);
    let coverage = compute_coverage_checked(root, &spec_files, &config).unwrap_or_else(|error| {
        if matches!(format, types::OutputFormat::Json) {
            let output = serde_json::json!({
                "valid": false,
                "inconclusive": true,
                "error": format!("Coverage inconclusive: {error}"),
                "overall_coverage_pct": serde_json::Value::Null,
                "files_covered": 0,
                "files_total": 0,
                "total_modules": 0,
                "stale_modules": 0,
                "incomplete_modules": 0,
                "stale_threshold": stale_threshold,
                "modules": [],
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            eprintln!("Coverage inconclusive: {error}");
        }
        process::exit(1);
    });

    // Build per-module stats from spec files
    struct ModuleInfo {
        spec_path: String,
        module_name: String,
        source_files: Vec<String>,
        /// `None` when the spec lists no files, so there is nothing to measure.
        coverage_pct: Option<f64>,
        status: Option<String>,
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

        // Coverage: how many of this spec's source files exist. A spec that
        // lists no files has nothing to divide by; `.max(1)` used to turn that
        // into a fabricated 0%, which reads as a measured result rather than as
        // the absence of one (#582).
        let existing: usize = fm.files.iter().filter(|f| root.join(f).exists()).count();
        let cov = (!fm.files.is_empty()).then(|| (existing as f64 / fm.files.len() as f64) * 100.0);

        // Stale detection via git log
        let mut stale = false;
        let mut max_behind: usize = 0;
        if !fm.files.is_empty() {
            // Resolve the spec commit once; if git has no record of the spec we
            // can't determine staleness, so leave `stale` false.
            if let Some(spec_commit) = git_last_commit_hash(root, &rel_spec) {
                for source_file in &fm.files {
                    if !root.join(source_file).exists() {
                        continue;
                    }
                    let behind = git_commits_since(root, &spec_commit, source_file);
                    // Always track the real drift: `commits_behind` must reflect
                    // sub-threshold drift too, not only once the module is stale.
                    max_behind = max_behind.max(behind);
                    if behind >= stale_threshold {
                        stale = true;
                    }
                }
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
            status: fm.status.clone(),
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
    // Re-derived here with a hardcoded 100.0 fallback until #582; now the one
    // implementation on `CoverageReport`, which has no percentage to give when
    // the denominator is zero.
    let overall_coverage = coverage.file_coverage();
    // Every human-readable rendering of the overall figure, so text, markdown,
    // github and table cannot drift apart again.
    let overall_line = |precision: usize| match overall_coverage {
        Some(pct) => format!(
            "{}/{} files covered ({pct:.precision$}%)",
            coverage.specced_file_count,
            coverage.measured_file_total(),
        ),
        None => format!("0/0 files covered ({NO_FILES_MEASURED})"),
    };

    match format {
        types::OutputFormat::Json => {
            let module_json: Vec<serde_json::Value> = modules
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "module": m.module_name,
                        "spec_path": m.spec_path,
                        "status": m.status,
                        "source_files": m.source_files,
                        "coverage_pct": percent_json(m.coverage_pct),
                        "stale": m.stale,
                        "commits_behind": m.stale_commits_behind,
                        "incomplete": m.incomplete,
                        "missing_fields": m.missing_fields,
                        "empty_sections": m.empty_sections,
                    })
                })
                .collect();

            let output = serde_json::json!({
                "overall_coverage_pct": percent_json(overall_coverage),
                "files_covered": coverage.specced_file_count,
                "files_total": coverage.measured_file_total(),
                "total_modules": total_modules,
                "stale_modules": stale_count,
                "incomplete_modules": incomplete_count,
                "stale_threshold": stale_threshold,
                "modules": module_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            // Machine consumers get the gate via the exit code alone (printing
            // the human gate message would corrupt the JSON on stdout).
            std::process::exit(compute_exit_code(
                stale_count + incomplete_count,
                0,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            ));
        }
        types::OutputFormat::Csv => {
            println!("module,spec_path,status,coverage_pct,stale,commits_behind,incomplete");
            for m in &modules {
                println!(
                    "{},{},{},{},{},{},{}",
                    m.module_name,
                    m.spec_path,
                    m.status.as_deref().unwrap_or(""),
                    percent_csv(m.coverage_pct, 0),
                    m.stale,
                    m.stale_commits_behind,
                    m.incomplete,
                );
            }
            println!(
                "SUMMARY,,,{},{stale_count},,{incomplete_count}",
                percent_csv(overall_coverage, 1)
            );
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            println!("## Spec Coverage Report\n");
            println!("**Overall:** {}  ", overall_line(1));
            println!(
                "**Modules:** {total_modules} total, {stale_count} stale, {incomplete_count} incomplete\n"
            );
            println!("| Module | Status | Coverage | Stale | Commits Behind | Incomplete |");
            println!("|--------|--------|----------|-------|----------------|------------|");
            for m in &modules {
                println!(
                    "| {} | {} | {} | {} | {} | {} |",
                    m.module_name,
                    m.status.as_deref().unwrap_or(""),
                    percent_cell(m.coverage_pct),
                    if m.stale { "yes" } else { "no" },
                    m.stale_commits_behind,
                    if m.incomplete { "yes" } else { "no" },
                );
            }
        }
        _ => {
            println!(
                "\n--- {} ------------------------------------------------",
                "Spec Coverage Report".bold()
            );
            println!("\n  Overall: {}", overall_line(1));
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
                let cov_str = percent_cell(m.coverage_pct);
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

    // CI gating (#430): every enforcement path must be able to fail the
    // process. Stale and incomplete modules count as failures; `--enforcement
    // warn` still exits 0, `enforce-new` gates on unspecced files, and
    // `--require-coverage` gates on real file coverage. CSV is a machine
    // format — gate silently via the exit code like JSON (handled above).
    let failures = stale_count + incomplete_count;
    if matches!(format, types::OutputFormat::Csv) {
        std::process::exit(compute_exit_code(
            failures,
            0,
            strict,
            enforcement,
            &coverage,
            require_coverage,
        ));
    }
    super::exit_with_status(
        failures,
        0,
        strict,
        enforcement,
        &coverage,
        require_coverage,
    );
}
