use colored::Colorize;
use std::path::Path;
use std::process;

use crate::change;
use crate::comment;
use crate::github;
use crate::ignore::IgnoreRules;
use crate::types;
use crate::validator::{compute_coverage_checked, get_schema_table_names};

use super::{build_schema_columns, compute_exit_code, load_and_discover, run_validation};

pub fn cmd_comment(
    root: &Path,
    pr: Option<u64>,
    _base: &str,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
) {
    let (config, spec_files) = load_and_discover(root, true);

    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);
    let ignore_rules = IgnoreRules::load(root);

    // CLI --enforcement flag overrides config; --strict implies strict enforcement.
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });

    // Use the same validation pipeline as `check` for consistent results
    let (
        total_errors,
        total_warnings,
        passed,
        total,
        mut all_errors,
        mut all_warnings,
        all_notices,
    ) = run_validation(
        root,
        &spec_files,
        &spec_files,
        &schema_tables,
        &schema_columns,
        &config,
        true, // collect mode
        false,
        &ignore_rules,
    );

    let coverage = compute_coverage_checked(root, &spec_files, &config).unwrap_or_else(|error| {
        eprintln!("{} Coverage inconclusive: {error}", "error:".red().bold());
        process::exit(1);
    });

    // Use the same exit-code logic as `check` so the comment status matches CI
    let exit_code = compute_exit_code(
        total_errors,
        total_warnings,
        strict,
        enforcement,
        &coverage,
        require_coverage,
    );
    // Configured verification commands still execute and fail closed, but their
    // child output must not contaminate the markdown-only stdout protocol.
    let sdd_report = change::check_project_quiet(root);
    let sdd_error_count = sdd_report.errors.len();
    let sdd_warning_count = sdd_report.warnings.len();
    all_errors.extend(
        sdd_report
            .errors
            .into_iter()
            .map(|error| format!(".specsync/sdd.json: {error}")),
    );
    all_warnings.extend(
        sdd_report
            .warnings
            .into_iter()
            .map(|warning| format!(".specsync/sdd.json: {warning}")),
    );
    let display_errors = total_errors + sdd_error_count;
    let display_warnings = total_warnings + sdd_warning_count;
    let overall_passed = exit_code == 0 && sdd_error_count == 0;
    let repo = github::detect_repo(root);
    let branch = comment::detect_branch(root);

    let body = comment::render_check_comment(
        total,
        passed,
        display_warnings,
        display_errors,
        &all_errors,
        &all_warnings,
        &all_notices,
        &coverage,
        overall_passed,
        repo.as_deref(),
        branch.as_deref(),
    );

    if let Some(pr_number) = pr {
        // Post as a PR comment via `gh`
        let repo_name = match github::resolve_repo(
            config.github.as_ref().and_then(|g| g.repo.as_deref()),
            root,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{} {e}", "error:".red().bold());
                process::exit(1);
            }
        };

        let status = std::process::Command::new("gh")
            .args([
                "pr",
                "comment",
                &pr_number.to_string(),
                "--repo",
                &repo_name,
                "--body",
                &body,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Posted spec-sync comment on PR #{pr_number}");
            }
            Ok(s) => {
                eprintln!(
                    "{} gh pr comment exited with {}",
                    "error:".red().bold(),
                    s.code().unwrap_or(-1)
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("{} Failed to run gh CLI: {e}", "error:".red().bold());
                eprintln!("Install the GitHub CLI: https://cli.github.com/");
                process::exit(1);
            }
        }
    } else {
        // Just print the comment body to stdout for piping
        print!("{body}");
    }
}
