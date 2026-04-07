use colored::Colorize;
use std::path::Path;
use std::process;

use crate::comment;
use crate::github;
use crate::validator::{compute_coverage, get_schema_table_names};

use super::{build_schema_columns, load_and_discover};

pub fn cmd_comment(root: &Path, pr: Option<u64>, _base: &str) {
    let (config, spec_files) = load_and_discover(root, false);

    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = build_schema_columns(root, &config);

    // Run validation, collecting all results
    let mut violations: Vec<comment::SpecViolation> = Vec::new();
    for spec_file in &spec_files {
        let result = crate::validator::validate_spec(
            spec_file,
            root,
            &schema_tables,
            &schema_columns,
            &config,
        );
        violations.push(comment::SpecViolation::from_result(&result));
    }

    let coverage = compute_coverage(root, &spec_files, &config);
    let repo = github::detect_repo(root);
    let branch = comment::detect_branch(root);

    let body =
        comment::render_comment_body(&violations, &coverage, repo.as_deref(), branch.as_deref());

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
