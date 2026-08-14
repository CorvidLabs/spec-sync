use colored::Colorize;
use std::path::Path;
use std::process;

use crate::comment;
use crate::github;
use crate::ignore::IgnoreRules;
use crate::types;
use crate::validator::compute_coverage_checked;

use super::{compute_exit_code, load_and_discover, run_validation};

pub fn cmd_comment(
    root: &Path,
    pr: Option<u64>,
    _base: &str,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
) {
    let (config, spec_files) = load_and_discover(root, true);

    let ignore_rules = IgnoreRules::load(root);

    // CLI --enforcement flag overrides config; --strict implies strict enforcement.
    let enforcement = enforcement.unwrap_or(if strict {
        types::EnforcementMode::Strict
    } else {
        config.enforcement
    });

    // Use the same validation pipeline as `check` for consistent results
    let (total_errors, total_warnings, passed, total, all_errors, all_warnings, all_notices) =
        run_validation(
            root,
            &spec_files,
            &spec_files,
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
    // `comment` reports spec-check results only (REQ-cmd-comment-004). SDD lifecycle
    // state used to be folded in here — prefixed `.specsync/sdd.json:` — which put
    // trust-layer findings into a PR comment about spec drift and let them decide
    // whether the comment reported a pass. Lifecycle reporting belongs to the
    // `change` verbs and `specsync change audit`.
    let display_errors = total_errors;
    let display_warnings = total_warnings;
    let overall_passed = exit_code == 0;
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

    // Exit with the verdict this command just rendered. `exit_code` was computed
    // above precisely so the comment status would match CI — and then the
    // function returned normally, so the process exited 0 while the body it
    // posted said `## ❌ SpecSync: Failed` (#571).
    //
    // That made `specsync comment` a permanent pass as a CI step, and it was the
    // only command ignoring `--require-coverage`: `check`, `score`, `report` and
    // `deps` all exit 1 over a 0% tree while this one exited 0.
    process::exit(exit_code);
}
