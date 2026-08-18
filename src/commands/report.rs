use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

use crate::git_utils::{MissingHistory, SpecBaseline, git_commits_since, spec_baseline};
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

/// A staleness verdict for a human-readable column. `n/a` is deliberately not
/// `no`: an unmeasured module must not read like a module that was checked and
/// found current (#572).
fn stale_cell(stale: Option<bool>) -> &'static str {
    match stale {
        Some(true) => "yes",
        Some(false) => "no",
        None => "n/a",
    }
}

/// A commit distance for a human-readable column, or `n/a` when git had no
/// history to measure it against.
fn behind_cell(behind: Option<usize>) -> String {
    match behind {
        Some(value) => value.to_string(),
        None => "n/a".to_string(),
    }
}

/// A CSV field for a value that was never measured: empty, like the coverage
/// fields. Writing `false` or `0` there would state a finding nobody made.
fn csv_field<T: std::fmt::Display>(value: Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

/// The one `Modules: …` summary, so text, markdown, github and table cannot
/// drift apart about how many modules were unmeasured.
fn module_summary_line(total: usize, stale: usize, unmeasured: usize, incomplete: usize) -> String {
    // When every module's staleness was unmeasurable there is no stale COUNT to
    // report, and printing `0 stale` beside `N staleness unmeasured` is the
    // whole defect in miniature: a dashboard scraping "N stale" reads zero
    // drift from a run that measured none. Say "unknown" instead, and only
    // print a number when at least one module was actually measured.
    let mut line = if unmeasured > 0 && stale == 0 && unmeasured == total {
        format!("{total} total, stale unknown")
    } else {
        format!("{total} total, {stale} stale")
    };
    if unmeasured > 0 {
        line.push_str(&format!(", {unmeasured} staleness unmeasured"));
    }
    line.push_str(&format!(", {incomplete} incomplete"));
    line
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
        /// `None` when git had no history to measure staleness against, so the
        /// module is neither stale nor known-current (#572).
        stale: Option<bool>,
        /// `None` for the same reason `stale` is.
        stale_commits_behind: Option<usize>,
        incomplete: bool,
        missing_fields: Vec<String>,
        empty_sections: Vec<String>,
    }

    let mut modules: Vec<ModuleInfo> = Vec::new();
    // Set the first time a module actually needs a staleness answer git cannot
    // give. Deliberately NOT probed before the loop: a project whose specs list
    // no source files never asks git anything, and must keep getting its full
    // coverage report (that is the tree the #582 suite measures). The guard
    // fires where the fabricated `false`/`0` used to be written, not earlier.
    let mut history_missing: Option<MissingHistory> = None;

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

        // Stale detection via git log. `None` means UNMEASURED, and only a
        // measured run may say `false`/`0`.
        let mut stale: Option<bool> = Some(false);
        let mut max_behind: Option<usize> = Some(0);
        if !fm.files.is_empty() {
            // Resolve the spec commit once, then count per source file.
            match spec_baseline(root, &rel_spec) {
                SpecBaseline::Commit(spec_commit) => {
                    let mut behind_max = 0usize;
                    let mut is_stale = false;
                    let mut measured_any = false;
                    for source_file in &fm.files {
                        let absolute = root.join(source_file);
                        // A cited file that is absent, or that names a
                        // directory, cannot be compared against anything. It
                        // used to be skipped, which left `behind_max` at 0 and
                        // reported the module current — the same mistake as
                        // #572 one level down: there the missing input was the
                        // history, here it is the file, and both were answered
                        // with a confident `false`/`0`.
                        if !absolute.exists() || crate::exports::files_entry_is_directory(&absolute)
                        {
                            continue;
                        }
                        measured_any = true;
                        let behind = git_commits_since(root, &spec_commit, source_file);
                        // Always track the real drift: `commits_behind` must
                        // reflect sub-threshold drift too, not only once the
                        // module is stale.
                        behind_max = behind_max.max(behind);
                        if behind >= stale_threshold {
                            is_stale = true;
                        }
                    }
                    if measured_any {
                        stale = Some(is_stale);
                        max_behind = Some(behind_max);
                    } else {
                        // Every cited file was unreadable as a git subject, so
                        // nothing was measured. `None` is what the existing
                        // `unmeasured_stale_modules` / `staleness_inconclusive`
                        // machinery is for; it was simply never wired here.
                        stale = None;
                        max_behind = None;
                    }
                }
                // History exists; git simply has no record of this spec yet, so
                // there is nothing for it to be behind. A measured zero.
                SpecBaseline::Untracked => {}
                // No history at all: the distance is unknown. Reporting `false`
                // and `0` here is the bug (#572) — a tree with `.git` removed
                // reported every module current and exited 0.
                SpecBaseline::Missing(missing) => {
                    history_missing.get_or_insert(missing);
                    stale = None;
                    max_behind = None;
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
    let stale_count = modules.iter().filter(|m| m.stale == Some(true)).count();
    // Modules whose staleness git could not be asked about. Counted SEPARATELY
    // from `stale_count`: they are not known-stale, and they are emphatically
    // not known-current. Folding them into either number is the defect (#572).
    let unmeasured_stale_count = modules.iter().filter(|m| m.stale.is_none()).count();
    let incomplete_count = modules.iter().filter(|m| m.incomplete).count();
    // The staleness half of the report is inconclusive, exactly the way
    // `compute_coverage_checked` above makes the coverage half inconclusive.
    // The report still renders in full — an early exit would take the coverage
    // figures with it — but it must not exit 0 certifying "0 stale" over a
    // question git was never able to answer. Threaded into the finding count so
    // the project's own `enforcement` decides: a `warn` project still exits 0
    // with honest `n/a` cells, a gating project fails closed.
    // One flag for one concept. `history_missing` is only ONE reason staleness
    // can be unmeasurable — a module whose cited files are all absent is just
    // as unmeasurable, and reporting `staleness_inconclusive: false` beside
    // `unmeasured_stale_modules: 1` tells a consumer the run was conclusive
    // when it was not. Keyed off the count so any future reason is covered by
    // construction rather than by remembering to add it here.
    let staleness_note = if let Some(missing) = history_missing {
        Some(format!(
            "Staleness inconclusive: {} — {unmeasured_stale_count} module(s) could not be checked \
             for drift against their source files",
            missing.reason()
        ))
    } else if unmeasured_stale_count > 0 {
        Some(format!(
            "Staleness inconclusive: {unmeasured_stale_count} module(s) cite no file that could be \
             measured, so their drift is unknown rather than zero"
        ))
    } else {
        None
    };
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
                // `null`, not `0`, when nothing could be measured: a consumer
                // must be able to tell "no module is stale" from "no module's
                // staleness is knowable". Zero is an answer; this is not one.
                "stale_modules": if unmeasured_stale_count > 0 && stale_count == 0 {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::from(stale_count)
                },
                "unmeasured_stale_modules": unmeasured_stale_count,
                "staleness_inconclusive": staleness_note.is_some(),
                "staleness_error": staleness_note,
                "incomplete_modules": incomplete_count,
                "stale_threshold": stale_threshold,
                "modules": module_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            // Machine consumers get the gate via the exit code alone (printing
            // the human gate message would corrupt the JSON on stdout).
            std::process::exit(compute_exit_code(
                stale_count + unmeasured_stale_count + incomplete_count,
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
                    csv_field(m.stale),
                    csv_field(m.stale_commits_behind),
                    m.incomplete,
                );
            }
            println!(
                "SUMMARY,,,{},{stale_count},,{incomplete_count}",
                percent_csv(overall_coverage, 1)
            );
            if let Some(note) = &staleness_note {
                eprintln!("{note}");
            }
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            println!("## Spec Coverage Report\n");
            println!("**Overall:** {}  ", overall_line(1));
            println!(
                "**Modules:** {}\n",
                module_summary_line(
                    total_modules,
                    stale_count,
                    unmeasured_stale_count,
                    incomplete_count,
                )
            );
            if let Some(note) = &staleness_note {
                println!("> **{note}**\n");
            }
            println!("| Module | Status | Coverage | Stale | Commits Behind | Incomplete |");
            println!("|--------|--------|----------|-------|----------------|------------|");
            for m in &modules {
                println!(
                    "| {} | {} | {} | {} | {} | {} |",
                    m.module_name,
                    m.status.as_deref().unwrap_or(""),
                    percent_cell(m.coverage_pct),
                    stale_cell(m.stale),
                    behind_cell(m.stale_commits_behind),
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
                "  Modules: {}\n",
                module_summary_line(
                    total_modules,
                    stale_count,
                    unmeasured_stale_count,
                    incomplete_count,
                )
            );
            if let Some(note) = &staleness_note {
                println!("  {} {note}\n", "!".yellow().bold());
            }

            // Table header
            println!(
                "  {:<20} {:>8}  {:>7}  {:>10}",
                "Module", "Coverage", "Stale", "Incomplete"
            );
            println!("  {}", "-".repeat(52));

            for m in &modules {
                let cov_str = percent_cell(m.coverage_pct);
                let stale_str = match (m.stale, m.stale_commits_behind) {
                    (Some(true), Some(behind)) => format!("{behind} behind").yellow().to_string(),
                    (Some(_), _) => "no".green().to_string(),
                    // Never green: nothing was measured here.
                    (None, _) => "n/a".yellow().to_string(),
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
            let stale_modules: Vec<&ModuleInfo> =
                modules.iter().filter(|m| m.stale == Some(true)).collect();
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
                        behind_cell(m.stale_commits_behind),
                    );
                }
            }

            // Unmeasured details: named individually so nobody has to infer
            // which modules the summary's `n/a` refers to.
            let unmeasured_modules: Vec<&ModuleInfo> =
                modules.iter().filter(|m| m.stale.is_none()).collect();
            if !unmeasured_modules.is_empty() {
                println!(
                    "\n  {} ({}):",
                    "Modules with unmeasured staleness".yellow().bold(),
                    unmeasured_modules.len(),
                );
                for m in &unmeasured_modules {
                    println!(
                        "    {} {} — no git history to measure drift against",
                        "?".yellow(),
                        m.module_name,
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
    // Modules whose staleness could not be measured count too: `report` must
    // not exit 0 attesting to drift it never looked for (#572).
    let failures = stale_count + unmeasured_stale_count + incomplete_count;
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
