use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::comment;
use crate::git_utils;
use crate::github;
use crate::hash_cache;
use crate::ignore::IgnoreRules;
use crate::output::{print_check_markdown, print_coverage_line, print_summary};
use crate::parser;
use crate::types;
use crate::validator::compute_coverage_checked;

use crate::config::is_legacy_layout;

use super::{
    compute_exit_code, create_drift_issues, exit_with_status, filter_by_status, filter_specs,
    load_and_discover, run_validation_with_suppressions,
};

fn suppressed_warning_summary(detail: &serde_json::Value) -> String {
    let clean = |field: &str| {
        detail[field]
            .as_str()
            .unwrap_or("unknown")
            .replace(['\r', '\n'], " ")
    };
    format!(
        "{} [{}; {}]: {}",
        clean("spec"),
        clean("category"),
        clean("source"),
        clean("warning")
    )
}

fn print_suppressed_markdown(details: &[serde_json::Value]) {
    if details.is_empty() {
        return;
    }
    println!("\n### Suppressed warnings\n");
    for detail in details {
        println!("- {}", suppressed_warning_summary(detail));
    }
}

fn append_suppressed_markdown(body: &mut String, details: &[serde_json::Value]) {
    if details.is_empty() {
        return;
    }
    body.push_str("\n### Suppressed warnings\n\n");
    for detail in details {
        body.push_str("- ");
        body.push_str(&suppressed_warning_summary(detail));
        body.push('\n');
    }
}

fn checked_coverage_or_exit(
    root: &Path,
    spec_files: &[PathBuf],
    config: &types::SpecSyncConfig,
    format: types::OutputFormat,
) -> types::CoverageReport {
    compute_coverage_checked(root, spec_files, config).unwrap_or_else(|error| {
        let message = format!("Coverage inconclusive: {error}");
        match format {
            types::OutputFormat::Json => {
                let output = serde_json::json!({
                    "passed": false,
                    "valid": false,
                    "inconclusive": true,
                    "error": message,
                    "errors": [message],
                    "warnings": [],
                    "notices": [],
                    "stale": [],
                    "specs_checked": 0,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            _ => eprintln!("{} {message}", "error:".red().bold()),
        }
        process::exit(1);
    })
}

/// Files whose contents affect validation globally rather than through a single
/// spec's frontmatter: the resolved config file and every file in the schema
/// directory. A change to any of these must invalidate the unchanged-skip —
/// otherwise a migration that drops a documented column, or a newly-added
/// custom rule, is silently skipped on the default incremental `check` path
/// (a false-PASS) and only surfaces under `--force`.
fn global_validation_inputs(root: &Path, config: &types::SpecSyncConfig) -> Vec<String> {
    let normalize = |p: &Path| -> String {
        p.strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let mut inputs: Vec<String> = Vec::new();
    if let Some(cfg) = &config.config_path {
        inputs.push(normalize(cfg));
    }
    if let Some(dir) = &config.schema_dir
        && let Ok(entries) = fs::read_dir(root.join(dir))
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                inputs.push(normalize(&path));
            }
        }
    }
    inputs.sort();
    inputs
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_check(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    fix: bool,
    dry_run: bool,
    backup: bool,
    force: bool,
    create_issues: bool,
    explain: bool,
    stale: Option<Option<usize>>,
    spec_filters: &[String],
    exclude_status: &[String],
    only_status: &[String],
) {
    use hash_cache::{ChangeClassification, ChangeKind};
    use types::OutputFormat::*;

    // Auto-detect legacy 3.x layout and suggest migration
    if is_legacy_layout(root) && matches!(format, Text) {
        eprintln!(
            "{} Legacy 3.x layout detected (config files at project root).",
            "⚠".yellow()
        );
        eprintln!(
            "  Run {} to upgrade to v4.0.0 (.specsync/ directory structure, TOML config).",
            "specsync migrate".cyan()
        );
        eprintln!(
            "  Use {} to preview changes without modifying files.\n",
            "specsync migrate --dry-run".dimmed()
        );
    }

    // Always allow the empty-specs case through: `check` handles it itself below,
    // where a requested coverage/enforcement gate is still evaluated (the shared
    // early-exit would `exit(0)` and silently pass the gate — and emit a non-JSON
    // message under --format json). Default warn mode still exits 0 there.
    let (config, all_spec_files) = load_and_discover(root, true);
    // Active workspaces + living specs only. Archives are history; full archive
    // integrity is not part of `specsync check` (use `change audit` / internal
    // check_project when a full historical walk is intentionally required).
    // Lifecycle state is reported here, never enforced (REQ-cmd-check-004).
    //
    // `specsync check` is the bi-directional spec<->code drift check. Gating it on
    // lifecycle state made every trust-layer failure — squash orphaning, ledger
    // divergence, a stale evidence commit — present to the user as "the drift check
    // is broken", and made the lifecycle a *stricter* gate than the specs it
    // guarded: default enforcement is `warn`, which always exits 0, while this gate
    // exited 1 unconditionally. Gating belongs to the `change` verbs and
    // `specsync change audit`.
    //
    // Findings go to stderr in every format so machine consumers keep a signal
    // without the stdout protocol being disturbed; the summary line is text-only.
    let sdd_report = crate::change::audit_project(root);
    if sdd_report.enabled {
        for finding in sdd_report.warnings.iter().chain(sdd_report.errors.iter()) {
            eprintln!("{} {finding}", "warning:".yellow().bold());
        }
        if matches!(format, Text) {
            println!(
                "{} {} active change(s)\n",
                "•".dimmed(),
                sdd_report.checked_changes
            );
        }
    }
    let spec_files = filter_specs(root, &all_spec_files, spec_filters);
    let spec_files = filter_by_status(&spec_files, exclude_status, only_status);
    // CLI --enforcement flag overrides config; --strict implies strict enforcement.
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });

    // Spec name filters that matched nothing are an error — don't fall through
    // to the misleading "No spec files found" message when specs do exist.
    if spec_files.is_empty() && !spec_filters.is_empty() && !all_spec_files.is_empty() {
        match format {
            Json => {
                let output = serde_json::json!({
                    "passed": false,
                    "errors": [format!("No specs matched: {}", spec_filters.join(", "))],
                    "warnings": [],
                    "notices": [],
                    "stale": [],
                    "specs_checked": 0,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            _ => {
                // filter_specs already printed the "No specs matched: ..." warning
                eprintln!(
                    "{} no specs matched the given filter(s) ({} spec file(s) exist)",
                    "error:".red().bold(),
                    all_spec_files.len()
                );
            }
        }
        process::exit(1);
    }

    if spec_files.is_empty() {
        // No specs to validate — but a requested gate must still be evaluated
        // against source coverage. Otherwise `check --require-coverage N`,
        // `--enforcement enforce-new`, or `--strict` silently PASS in exactly the
        // state they exist to catch: a project with source code but no specs (the
        // default state right after `specsync init`). Default mode (warn, no gate)
        // still exits 0 with the informational message.
        let coverage = checked_coverage_or_exit(root, &spec_files, &config, format);
        let mut exit_code =
            compute_exit_code(0, 0, strict, enforcement, &coverage, require_coverage);
        // The comment above states the intent; `--strict` did not deliver it.
        // `compute_exit_code` escalates WARNINGS under `--strict`, and a project
        // with no specs produces none — so `check --strict` exited 0 over real
        // source with zero coverage, printing no number at all, while `coverage`
        // on the same tree reported 0%. A caller who asked for strict validation
        // of a tree that was never measured should not be told it is clean.
        //
        // Confined to a tree that actually has source: an empty project, or one
        // whose specs simply have not been generated yet and has nothing to
        // measure, still exits 0.
        if strict && coverage.total_source_files > 0 {
            exit_code = 1;
        }
        match format {
            Json => {
                let output = serde_json::json!({
                    "passed": exit_code == 0,
                    "errors": [],
                    "warnings": [],
                    "notices": [],
                    "stale": [],
                    "specs_checked": 0,
                    // Without these a consumer sees `specs_checked: 0` and cannot
                    // tell an empty project from one with source and no specs.
                    // `coverage_percent` is null when there was nothing to
                    // measure — a machine consumer must not read that as 100
                    // (#582).
                    "total_source_files": coverage.total_source_files,
                    "coverage_percent": coverage.file_coverage_percent(),
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            Markdown | Github => {
                println!("## SpecSync Check Results\n");
                println!("No spec files found. Run `specsync generate` to scaffold specs.");
            }
            Text | Table | Csv => {
                let abs_specs = root.join(&config.specs_dir);
                println!(
                    "No spec files found in {}/. Run `specsync generate` to scaffold specs.",
                    abs_specs.display()
                );
                // Always show the number. Printing it only on failure is how a
                // tree with source and zero specs produced a log containing
                // nothing to notice.
                print_coverage_line(&coverage);
            }
        }
        process::exit(exit_code);
    }

    // Load hash cache and classify changes for each spec.
    let mut cache = hash_cache::HashCache::load(root);
    // Config + schema files affect validation globally (not via one spec's
    // frontmatter), so track them separately from per-spec hashes.
    let global_inputs = global_validation_inputs(root, &config);
    let (specs_to_validate, change_classifications) = if force || strict || !spec_filters.is_empty()
    {
        (spec_files.clone(), Vec::new())
    } else if fix {
        // --fix bypasses the unchanged-skip: an explicit fix request must
        // never be a silent no-op because a previous (failing or warning) run
        // recorded the hashes.
        let classifications = hash_cache::classify_all_changes(root, &spec_files, &cache);
        (spec_files.clone(), classifications)
    } else {
        let classifications = hash_cache::classify_all_changes(root, &spec_files, &cache);
        let global_changed = global_inputs.iter().any(|p| cache.is_changed(root, p));
        let changed: Vec<PathBuf> = if global_changed {
            // A schema or config file changed since the cache was written — a
            // spec whose own files are unchanged may still now be stale (e.g. a
            // migration dropped a documented column, or a new custom rule was
            // added). Re-validate everything rather than trust the stale skip.
            spec_files.clone()
        } else {
            classifications
                .iter()
                .map(|c| c.spec_path.clone())
                .collect()
        };
        (changed, classifications)
    };

    let skipped = spec_files.len() - specs_to_validate.len();
    if skipped > 0 && matches!(format, Text) {
        let cache_path = root.join(".specsync").join("hashes.json");
        println!(
            "{} Skipped {skipped} unchanged spec(s) (use --force/--no-cache to re-validate all)",
            "⊘".cyan()
        );
        println!("  {} Cache: {}\n", "ℹ".dimmed(), cache_path.display());
    }

    if specs_to_validate.is_empty() && matches!(format, Text) {
        println!("{}", "All specs unchanged — nothing to validate.".green());
        let coverage = checked_coverage_or_exit(root, &spec_files, &config, format);
        print_coverage_line(&coverage);
        // A warm cache skips spec RE-validation, but a requested coverage/
        // enforcement gate must still be evaluated against freshly computed
        // coverage — otherwise an unchanged run silently flips a failing
        // --require-coverage / --enforcement gate to exit 0. Cached specs had no
        // errors (that's why they're cached), so 0 errors/warnings is correct here.
        let exit_code = compute_exit_code(0, 0, strict, enforcement, &coverage, require_coverage);
        process::exit(exit_code);
    }

    // Report staleness from change classifications
    let mut stale_entries: Vec<serde_json::Value> = Vec::new();
    let mut staleness_warnings: usize = 0;
    let mut requirements_stale_specs: Vec<ChangeClassification> = Vec::new();

    for classification in &change_classifications {
        let spec_rel = classification
            .spec_path
            .strip_prefix(root)
            .unwrap_or(&classification.spec_path)
            .to_string_lossy()
            .to_string();

        // Reported only against a real baseline: a cold cache classifies every
        // companion as changed so that everything is re-validated, but saying
        // so out loud would put one warning per spec in every CI run, where the
        // cache is always cold. Selection is unaffected — the spec is still
        // re-validated either way.
        if classification.reportable(&ChangeKind::Requirements) {
            if matches!(format, Text) {
                println!(
                    "  {} {spec_rel}: requirements changed — spec may need re-validation",
                    "⚠".yellow()
                );
            }
            stale_entries.push(serde_json::json!({
                "spec": spec_rel,
                "reason": "requirements_changed",
                "message": "requirements changed — spec may need re-validation"
            }));
            staleness_warnings += 1;
            requirements_stale_specs.push(classification.clone());
        }

        if classification.reportable(&ChangeKind::Companion) && matches!(format, Text) {
            println!(
                "  {} {spec_rel}: companion file updated (hash refreshed)",
                "ℹ".cyan()
            );
        }
    }

    if staleness_warnings > 0 && matches!(format, Text) {
        println!(); // spacing after staleness messages
    }

    // Requirements drift requires human/agent review; spec-sync never invokes a
    // provider or executes a configured command.
    if !requirements_stale_specs.is_empty() && matches!(format, Text) && !fix {
        println!("  Review the affected specs directly or ask your coding agent to update them.\n");
    }

    let ignore_rules = IgnoreRules::load(root);

    if dry_run && !fix {
        eprintln!(
            "{} --dry-run has no effect without --fix (nothing to preview)",
            "⚠".yellow()
        );
    }

    // If --fix is requested, auto-add undocumented exports to specs
    if fix {
        if backup && !dry_run {
            let backup_dir = root.join(".specsync/backup-fix");
            if let Err(e) = fs::create_dir_all(&backup_dir) {
                eprintln!(
                    "{} Failed to create backup directory {}: {e}",
                    "✗".red(),
                    backup_dir.display()
                );
                eprintln!("  Aborting --fix to avoid data loss. Fix the backup path and retry.");
                process::exit(1);
            }
            let mut backed_up = 0usize;
            for spec_file in &specs_to_validate {
                let rel = match spec_file.strip_prefix(root) {
                    Ok(r) => r,
                    Err(_) => {
                        eprintln!(
                            "{} Cannot backup {}: path is not under project root",
                            "✗".red(),
                            spec_file.display()
                        );
                        process::exit(1);
                    }
                };
                let dest = backup_dir.join(rel);
                if let Some(parent) = dest.parent()
                    && let Err(e) = fs::create_dir_all(parent)
                {
                    eprintln!(
                        "{} Failed to create backup subdirectory {}: {e}",
                        "✗".red(),
                        parent.display()
                    );
                    process::exit(1);
                }
                if let Err(e) = fs::copy(spec_file, &dest) {
                    eprintln!(
                        "{} Failed to backup {}: {e}",
                        "✗".red(),
                        spec_file.display()
                    );
                    eprintln!("  Aborting --fix to avoid data loss.");
                    process::exit(1);
                }
                backed_up += 1;
            }
            if matches!(format, Text) {
                println!(
                    "{} Backed up {} spec(s) to {}\n",
                    "✓".green(),
                    backed_up,
                    backup_dir.display()
                );
            }
        }

        let outcome = auto_fix_specs(root, &specs_to_validate, &config, dry_run);
        if outcome.fixed > 0 && matches!(format, Text) {
            let verb = if dry_run {
                "Would auto-add"
            } else {
                "Auto-added"
            };
            println!(
                "{} {verb} exports to {} spec(s)\n",
                "✓".green(),
                outcome.fixed
            );
        }
        // Reported on stderr in every format — a machine consumer asked for a
        // mutation too, and `passed: true` must not be the only thing it sees.
        if !outcome.failures.is_empty() {
            for failure in &outcome.failures {
                eprintln!("{} {failure}", "error:".red().bold());
            }
            eprintln!(
                "{} --fix could not repair {} spec(s)",
                "error:".red().bold(),
                outcome.failures.len()
            );
            process::exit(1);
        }
    }

    let collect = !matches!(format, Text);
    let (
        total_errors,
        total_warnings,
        passed,
        total,
        all_errors,
        all_warnings,
        all_notices,
        suppressed_warnings,
    ) = run_validation_with_suppressions(
        root,
        &specs_to_validate,
        &all_spec_files,
        &config,
        collect,
        explain,
        &ignore_rules,
    );
    // Git-based staleness detection (--stale flag)
    let stale_threshold = stale.map(|opt| opt.unwrap_or(5));
    let mut git_stale_warnings: usize = 0;
    let mut git_stale_entries: Vec<serde_json::Value> = Vec::new();

    // `--stale` is an explicit request for a git-history answer. When there is
    // no history, the honest reply is "could not check", not silence — the old
    // `is_git_repo`-only guard skipped the whole block on an unborn HEAD and
    // printed nothing, which reads exactly like "checked, found nothing" (#572).
    // Reported as a warning, the same weight a real drift finding carries here,
    // so `--strict` fails and a plain `check --stale` still exits 0 as it does
    // for genuinely drifted specs.
    let history_missing = stale_threshold.and_then(|_| git_utils::missing_history(root));
    if let Some(missing) = history_missing {
        git_stale_warnings += 1;
        if matches!(format, types::OutputFormat::Text) {
            println!(
                "  {} staleness not checked: {} — `--stale` needs git history",
                "⚠".yellow(),
                missing.reason(),
            );
            println!();
        }
        git_stale_entries.push(serde_json::json!({
            "spec": serde_json::Value::Null,
            "reason": "history_unavailable",
            "detail": missing.reason(),
            "commits_behind": serde_json::Value::Null,
            "drifted_files": [],
        }));
    }

    if let Some(threshold) = stale_threshold
        && history_missing.is_none()
    {
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
            if fm.files.is_empty() {
                continue;
            }

            let rel_spec = spec_file
                .strip_prefix(root)
                .unwrap_or(spec_file)
                .to_string_lossy()
                .to_string();

            let spec_commit = match git_utils::spec_baseline(root, &rel_spec) {
                git_utils::SpecBaseline::Commit(commit) => commit,
                // History exists; this spec is just not in it yet, so there is
                // nothing for it to be behind.
                git_utils::SpecBaseline::Untracked => continue,
                // Guarded above; only reachable if history vanished mid-run.
                // Warn rather than skip — that skip was the bug.
                git_utils::SpecBaseline::Missing(missing) => {
                    git_stale_warnings += 1;
                    git_stale_entries.push(serde_json::json!({
                        "spec": rel_spec,
                        "reason": "history_unavailable",
                        "detail": missing.reason(),
                        "commits_behind": serde_json::Value::Null,
                        "drifted_files": [],
                    }));
                    continue;
                }
            };

            let mut max_behind: usize = 0;
            let mut drifted_files: Vec<(String, usize)> = Vec::new();
            for source_file in &fm.files {
                if !root.join(source_file).exists() {
                    continue;
                }
                let behind = git_utils::git_commits_since(root, &spec_commit, source_file);
                if behind >= threshold {
                    drifted_files.push((source_file.clone(), behind));
                }
                max_behind = max_behind.max(behind);
            }

            if max_behind >= threshold {
                git_stale_warnings += 1;
                if matches!(format, types::OutputFormat::Text) {
                    let module = fm.module.as_deref().unwrap_or(&rel_spec);
                    println!(
                        "  {} {module}: spec is {max_behind} commits behind source files",
                        "⚠".yellow()
                    );
                    for (file, behind) in &drifted_files {
                        println!(
                            "      {} {file} ({behind} commit{})",
                            "→".dimmed(),
                            if *behind == 1 { "" } else { "s" },
                        );
                    }
                }
                let details: Vec<serde_json::Value> = drifted_files
                    .iter()
                    .map(|(f, n)| serde_json::json!({"file": f, "commits_behind": n}))
                    .collect();
                git_stale_entries.push(serde_json::json!({
                    "spec": rel_spec,
                    "reason": "git_drift",
                    "commits_behind": max_behind,
                    "drifted_files": details,
                }));
            }
        }

        if git_stale_warnings > 0 && matches!(format, types::OutputFormat::Text) {
            println!();
        }
    }
    stale_entries.extend(git_stale_entries);

    // Include staleness warnings in total when --strict
    let effective_warnings = total_warnings + staleness_warnings + git_stale_warnings;
    let coverage = checked_coverage_or_exit(root, &spec_files, &config, format);

    // Update hash cache after validation (only when no errors).
    // Specs with warnings are still cached, which is why --fix, --strict, and
    // --force all bypass the unchanged-skip above — an explicit fix or strict
    // run must never trust hashes recorded by a run that had findings.
    if total_errors == 0 {
        hash_cache::update_cache(root, &specs_to_validate, &mut cache);
        // Record global inputs (config + schema files) so a later unchanged run
        // can skip, and a future schema/config edit is detected as a change.
        for input in &global_inputs {
            cache.update(root, input);
        }
        let _ = cache.save(root);
    }

    // --create-issues: create GitHub issues for specs with validation errors
    if create_issues && total_errors > 0 {
        create_drift_issues(root, &config, &all_errors, format);
    }

    match format {
        Json => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
            let output = serde_json::json!({
                "passed": exit_code == 0,
                "errors": all_errors,
                "warnings": all_warnings,
                "suppressed_warnings": suppressed_warnings,
                "notices": all_notices,
                "stale": stale_entries,
                "specs_checked": total,
                // Machine consumers are exactly who cannot see the text
                // disclosure, and they are the ones acting on `passed` (#546).
                "skipped_links": coverage.skipped_links,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            process::exit(exit_code);
        }
        Markdown => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
            print_check_markdown(
                total,
                passed,
                effective_warnings,
                total_errors,
                &all_errors,
                &all_warnings,
                &all_notices,
                &coverage,
                exit_code == 0,
            );
            print_suppressed_markdown(&suppressed_warnings);
            process::exit(exit_code);
        }
        Github => {
            let exit_code = compute_exit_code(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
            let repo = github::detect_repo(root);
            let branch = comment::detect_branch(root);
            let mut body = comment::render_check_comment(
                total,
                passed,
                effective_warnings,
                total_errors,
                &all_errors,
                &all_warnings,
                &all_notices,
                &coverage,
                exit_code == 0,
                repo.as_deref(),
                branch.as_deref(),
            );
            append_suppressed_markdown(&mut body, &suppressed_warnings);
            print!("{body}");
            process::exit(exit_code);
        }
        Text | Table | Csv => {
            if !suppressed_warnings.is_empty() {
                println!(
                    "{} {} warning(s) suppressed by explicit ignore rules",
                    "ℹ".cyan(),
                    suppressed_warnings.len()
                );
                for detail in &suppressed_warnings {
                    println!("  - {}", suppressed_warning_summary(detail));
                }
            }
            print_summary(total, passed, effective_warnings, total_errors);
            print_coverage_line(&coverage);
            exit_with_status(
                total_errors,
                effective_warnings,
                strict,
                enforcement,
                &coverage,
                require_coverage,
            );
        }
    }
}

// ─── Auto-fix: add undocumented exports to spec ─────────────────────────

use crate::util::levenshtein;

/// Build "### Exported Functions"/"### Exported Types" subsection blocks for
/// auto-added export rows, omitting empty groups.
fn build_export_subsections(value_rows: &str, type_rows: &str) -> String {
    let mut block = String::new();
    if !value_rows.is_empty() {
        block.push_str(&format!(
            "\n\n### Exported Functions\n\n| Export | Description |\n|--------|-------------|\n{value_rows}"
        ));
    }
    if !type_rows.is_empty() {
        block.push_str(&format!(
            "\n\n### Exported Types\n\n| Type | Description |\n|------|-------------|\n{type_rows}"
        ));
    }
    block
}

/// Markdown row for one auto-added export. When the target table's column
/// count is known, the symbol goes in the first cell, the guidance text in the
/// last, and any middle cells get a `TODO` placeholder.
fn build_fix_row(
    name: &str,
    columns: Option<usize>,
    primary_lang: Option<types::Language>,
) -> String {
    const GUIDANCE: &str = "Document caller-visible behavior and constraints.";
    match columns {
        Some(n) if n > 2 => {
            let middle = "TODO | ".repeat(n - 2);
            format!("| `{name}` | {middle}{GUIDANCE} |")
        }
        Some(_) => format!("| `{name}` | {GUIDANCE} |"),
        None => match primary_lang {
            Some(types::Language::Swift)
            | Some(types::Language::Kotlin)
            | Some(types::Language::Java) => {
                format!("| `{name}` | Type or member kind | {GUIDANCE} |")
            }
            _ => format!("| `{name}` | {GUIDANCE} |"),
        },
    }
}

/// Column count of the first markdown table row found in `section`, if any.
fn table_column_count(section: &str) -> Option<usize> {
    section.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.len() > 2 && trimmed.starts_with('|') && trimmed.ends_with('|') {
            Some(trimmed[1..trimmed.len() - 1].split('|').count())
        } else {
            None
        }
    })
}

/// Normalize near-miss export headers within ## Public API.
/// E.g., "### Exportd Functions" → "### Exported Functions"
/// Uses Levenshtein distance ≤ 2 against a canonical list to catch typos,
/// singular/plural mismatches, and uncommon variations.
/// Returns true if the content was modified.
fn fix_near_miss_headers(content: &mut String) -> bool {
    use regex::Regex;
    let re = Regex::new(r"(?m)^(### )(.+)$").unwrap();

    // Find the Public API section bounds
    let api_start = match content.find("## Public API") {
        Some(pos) => pos,
        None => return false,
    };
    let after = &content[api_start..];
    let api_end = after[1..]
        .find("\n## ")
        .map(|p| api_start + 1 + p)
        .unwrap_or(content.len());

    let api_section = content[api_start..api_end].to_string();
    let mut modified = false;

    // Canonical export subsection names. Levenshtein distance ≤ 2 triggers a rename.
    let canonicals: &[&str] = &[
        "Exported Functions",
        "Exported Types",
        "Exported Classes",
        "Exported Constants",
        "Exported Components",
        "Exported Hooks",
        "Exported Interfaces",
        "Exported Enums",
    ];

    let mut new_section = api_section.clone();
    for cap in re.captures_iter(&api_section) {
        let header_text = cap.get(2).unwrap().as_str();
        let lower = header_text.to_ascii_lowercase();

        // Skip headers that already pass is_export_header
        if crate::parser::is_export_header(&format!("### {header_text}")) {
            continue;
        }

        // Find closest canonical by edit distance; fix if within 2 edits
        if let Some((&canonical, _)) = canonicals
            .iter()
            .map(|c| (c, levenshtein(&lower, &c.to_ascii_lowercase())))
            .min_by_key(|(_, d)| *d)
            .filter(|(_, d)| *d > 0 && *d <= 2)
        {
            let old = format!("### {header_text}");
            let new = format!("### {canonical}");
            new_section = new_section.replacen(&old, &new, 1);
            modified = true;
            continue;
        }

        // Bare API-kind headings ("### Functions", "### Methods", "### Types", …)
        // describe export tables but fail is_export_header, so their rows are
        // informational-only and --fix would append a duplicate export table
        // for the same symbols. Promote them to "### Exported <Kind>".
        let bare_kinds: &[&str] = &[
            "functions",
            "methods",
            "types",
            "classes",
            "constants",
            "components",
            "hooks",
            "interfaces",
            "enums",
            "structs",
            "traits",
            "protocols",
        ];
        if bare_kinds.contains(&lower.trim()) {
            let old = format!("### {header_text}");
            let new = format!("### Exported {}", header_text.trim());
            new_section = new_section.replacen(&old, &new, 1);
            modified = true;
        }
    }

    if modified {
        content.replace_range(api_start..api_end, &new_section);
    }

    modified
}

/// Rename near-miss `## Required Section` headings in the spec body.
/// Uses the same Levenshtein ≤ 2 approach as export-subsection fixing,
/// applied to the top-level required sections from config.
/// Returns true if the content was modified.
fn fix_near_miss_required_headers(content: &mut String, required_sections: &[String]) -> bool {
    let near_misses = crate::parser::get_near_miss_sections(content, required_sections);
    if near_misses.is_empty() {
        return false;
    }
    let mut modified = false;
    for (canonical, found) in &near_misses {
        let old = format!("## {found}");
        let new = format!("## {canonical}");
        if content.contains(&old) {
            *content = content.replacen(&old, &new, 1);
            modified = true;
        }
    }
    modified
}

/// What `--fix` actually did, including what it could not do.
struct AutoFixOutcome {
    fixed: usize,
    /// Specs `--fix` was asked to repair and could not. Never empty silently:
    /// a mutation request that failed must not be reported as success.
    failures: Vec<String>,
}

fn auto_fix_specs(
    root: &Path,
    spec_files: &[PathBuf],
    config: &types::SpecSyncConfig,
    dry_run: bool,
) -> AutoFixOutcome {
    use crate::exports::get_exported_symbols_full;
    use crate::parser::{get_all_api_table_symbols, get_spec_symbols, parse_frontmatter};

    let mut fixed_count = 0;
    let mut failures: Vec<String> = Vec::new();
    let sub_re = regex::Regex::new(r"(?m)^### ").unwrap();

    for spec_file in spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(error) => {
                // `--fix` is a mutation request. A spec it could not even read
                // is one it definitely did not fix, and skipping silently
                // reported success for work that never happened (#549).
                failures.push(format!(
                    "{}: could not be read, so it was not fixed: {error}",
                    spec_file.strip_prefix(root).unwrap_or(spec_file).display()
                ));
                continue;
            }
        };

        // First pass: fix near-miss required section headers (## level)
        let mut content = content;
        if fix_near_miss_required_headers(&mut content, &config.required_sections) {
            let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
            let verb = if dry_run { "would rename" } else { "renamed" };
            println!(
                "  {} {rel}: {verb} near-miss required section header(s) to canonical form",
                "✓".green()
            );
            if !dry_run {
                let _ = fs::write(spec_file, &content);
            }
        }

        // Second pass: fix near-miss export subsection headers (### level)
        if fix_near_miss_headers(&mut content) {
            let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
            let verb = if dry_run { "would rename" } else { "renamed" };
            println!(
                "  {} {rel}: {verb} near-miss export header(s) to canonical form",
                "✓".green()
            );
            if !dry_run {
                let _ = fs::write(spec_file, &content);
            }
        }

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        if parsed.frontmatter.files.is_empty() {
            continue;
        }

        // Collect all exports from source files
        let mut all_exports: Vec<String> = Vec::new();
        for file in &parsed.frontmatter.files {
            // Never read (or --fix persist) exports from a `files:` entry that
            // escapes the project root — it would write arbitrary host-file
            // identifiers into the spec. `check` reports such entries as errors.
            if !crate::validator::source_within_root(root, file) {
                continue;
            }
            let full_path = root.join(file);
            all_exports.extend(get_exported_symbols_full(
                &full_path,
                config.export_level,
                config.parse_mode,
            ));
        }
        let mut seen = std::collections::HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        // Find which exports are already documented — in recognized export
        // tables AND in any other table within ## Public API. A symbol that a
        // human already documented under an informational heading must not be
        // appended a second time.
        let mut documented: std::collections::HashSet<String> =
            get_spec_symbols(&parsed.body).into_iter().collect();
        documented.extend(get_all_api_table_symbols(&parsed.body));

        let undocumented: Vec<&str> = all_exports
            .iter()
            .filter(|s| !documented.contains(s.as_str()))
            .map(|s| s.as_str())
            .collect();

        if undocumented.is_empty() {
            continue;
        }

        // Detect primary language for context-aware row format
        let primary_lang = parsed
            .frontmatter
            .files
            .iter()
            .filter_map(|f| {
                std::path::Path::new(f)
                    .extension()
                    .and_then(|e| e.to_str())
                    .and_then(types::Language::from_extension)
            })
            .next();

        // Classify undocumented exports as types vs functions/values so new
        // rows land in the matching table — functions must not be appended to
        // an "Exported Types" table.
        let mut type_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for file in &parsed.frontmatter.files {
            if !crate::validator::source_within_root(root, file) {
                continue;
            }
            let full_path = root.join(file);
            type_names.extend(get_exported_symbols_full(
                &full_path,
                types::ExportLevel::Type,
                config.parse_mode,
            ));
        }
        let (type_syms, value_syms): (Vec<&str>, Vec<&str>) = undocumented
            .iter()
            .copied()
            .partition(|s| type_names.contains(*s));

        let build_rows = |syms: &[&str], columns: Option<usize>| -> String {
            syms.iter()
                .map(|name| build_fix_row(name, columns, primary_lang))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Find insertion points: type exports go to the last "… Types" export
        // subsection within ## Public API, everything else to the last
        // "… Functions"/"… Methods" subsection (falling back to the last
        // recognized export subsection), so new rows land where
        // get_spec_symbols will find them. Inserting at the end of the whole
        // section causes duplicates when non-export subsections
        // (e.g. ### API Endpoints) come after the export table.
        let mut new_content = content.clone();
        if let Some(api_start) = content.find("## Public API") {
            let after = &content[api_start..];
            let api_end = after[1..]
                .find("\n## ")
                .map(|p| api_start + 1 + p)
                .unwrap_or(content.len());
            let api_section = &content[api_start..api_end];

            // Collect start offsets (relative to api_section) of every ### subsection
            let sub_positions: Vec<usize> =
                sub_re.find_iter(api_section).map(|m| m.start()).collect();

            // Recognized export subsections: (lowercased header, absolute start, absolute end)
            let export_subs: Vec<(String, usize, usize)> = sub_positions
                .iter()
                .enumerate()
                .filter_map(|(i, &rel_pos)| {
                    let header_line = api_section[rel_pos..].lines().next().unwrap_or("");
                    if crate::parser::is_export_header(header_line) {
                        let rel_end = sub_positions
                            .get(i + 1)
                            .copied()
                            .unwrap_or(api_section.len());
                        Some((
                            header_line.to_ascii_lowercase(),
                            api_start + rel_pos,
                            api_start + rel_end,
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            if !export_subs.is_empty() {
                let type_target = export_subs
                    .iter()
                    .rposition(|(header, _, _)| header.contains("type"));
                let value_target = export_subs
                    .iter()
                    .rposition(|(header, _, _)| {
                        header.contains("function") || header.contains("method")
                    })
                    .or_else(|| {
                        // No functions table — use the last export subsection
                        // that is not the types table
                        export_subs
                            .iter()
                            .enumerate()
                            .rev()
                            .find(|(i, _)| Some(*i) != type_target)
                            .map(|(i, _)| i)
                    })
                    .unwrap_or(export_subs.len() - 1);
                let type_target = type_target.unwrap_or(value_target);

                // Group symbols per target subsection (a group may be empty)
                let mut syms_by_target: std::collections::BTreeMap<usize, Vec<&str>> =
                    std::collections::BTreeMap::new();
                for sym in &value_syms {
                    syms_by_target.entry(value_target).or_default().push(sym);
                }
                for sym in &type_syms {
                    syms_by_target.entry(type_target).or_default().push(sym);
                }

                // Apply insertions in descending offset order so earlier
                // offsets remain valid as the content grows
                for (&target, syms) in syms_by_target.iter().rev() {
                    let (_, start, end) = export_subs[target];
                    let columns = table_column_count(&content[start..end]);
                    let rows = build_rows(syms, columns);
                    new_content = format!(
                        "{}\n{}\n{}",
                        new_content[..end].trim_end(),
                        rows,
                        &new_content[end..]
                    );
                }
            } else if sub_positions.is_empty() {
                // No ### subsections — flat table or empty body; insert at section end
                let rows = build_rows(&undocumented, table_column_count(api_section));
                new_content = format!(
                    "{}\n{}\n{}",
                    content[..api_end].trim_end(),
                    rows,
                    &content[api_end..]
                );
            } else {
                // Has subsections but none are recognized export headers;
                // create new export subsections at the top of the section
                let api_header_end =
                    api_start + api_section.find('\n').unwrap_or(api_section.len());
                let header_block = build_export_subsections(
                    &build_rows(&value_syms, Some(2)),
                    &build_rows(&type_syms, Some(2)),
                );
                new_content = format!(
                    "{}{}{}",
                    &content[..api_header_end],
                    header_block,
                    &content[api_header_end..]
                );
            }
        } else {
            // No Public API section — append one
            let section = format!(
                "\n## Public API{}\n",
                build_export_subsections(
                    &build_rows(&value_syms, Some(2)),
                    &build_rows(&type_syms, Some(2)),
                )
            );
            new_content.push_str(&section);
        }

        if dry_run {
            fixed_count += 1;
            let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
            println!(
                "  {} {rel}: would add {} export(s)",
                "✓".green(),
                undocumented.len()
            );
        } else {
            let rel = spec_file.strip_prefix(root).unwrap_or(spec_file).display();
            match fs::write(spec_file, &new_content) {
                Ok(()) => {
                    fixed_count += 1;
                    println!(
                        "  {} {rel}: added {} export(s)",
                        "✓".green(),
                        undocumented.len()
                    );
                }
                // The one outcome `--fix` must never hide. Discarding this error
                // meant a read-only spec produced exit 0 and a clean report,
                // while the identical writable spec reported `added 1
                // export(s)` — the user asked for a mutation and was told
                // everything was fine (#549).
                Err(error) => failures.push(format!(
                    "{rel}: could not be written, so the fix was not applied: {error}"
                )),
            }
        }
    }

    AutoFixOutcome {
        fixed: fixed_count,
        failures,
    }
}
