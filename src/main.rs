mod ai;
mod archive;
mod changelog;
mod cli;
mod commands;
mod comment;
mod compact;
mod config;
mod deps;
mod exports;
mod generator;
mod git_utils;
mod github;
mod hash_cache;
mod hooks;
mod ignore;
mod importer;
mod manifest;
mod mcp;
mod merge;
mod output;
mod parser;
mod registry;
mod schema;
mod scoring;
mod types;
mod validator;
mod view;
mod watch;

use clap::Parser;
use colored::Colorize;
use std::process;

use cli::{Cli, Command};

fn main() {
    let result = std::panic::catch_unwind(run);
    match result {
        Ok(()) => {}
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown error".to_string()
            };
            eprintln!(
                "{} specsync panicked: {msg}\n\nThis is a bug — please report it at https://github.com/CorvidLabs/spec-sync/issues",
                "Error:".red().bold()
            );
            process::exit(1);
        }
    }
}

fn run() {
    let cli = Cli::parse();
    let root = cli
        .root
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot determine cwd"));
    let root = root.canonicalize().unwrap_or(root);

    // --json flag is shorthand for --format json (backward compat)
    let format = if cli.json {
        types::OutputFormat::Json
    } else {
        cli.format
    };

    let command = cli.command.unwrap_or(Command::Check {
        fix: false,
        force: false,
        create_issues: false,
        explain: false,
        stale: None,
        specs: vec![],
    });

    match command {
        Command::Init => commands::init::cmd_init(&root),
        Command::Check {
            fix,
            force,
            create_issues,
            explain,
            stale,
            specs,
        } => commands::check::cmd_check(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
            fix,
            force,
            create_issues,
            explain,
            stale,
            &specs,
        ),
        Command::Coverage => commands::coverage::cmd_coverage(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
        ),
        Command::Generate {
            provider,
            uncovered,
            batch,
        } => commands::generate::cmd_generate(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
            provider,
            uncovered,
            batch,
        ),
        Command::Score {
            explain,
            all,
            specs,
        } => commands::score::cmd_score(&root, format, explain, all, &specs),
        Command::Watch => watch::run_watch(&root, cli.strict, cli.require_coverage),
        Command::Mcp => mcp::run_mcp_server(&root),
        Command::AddSpec { name } => commands::scaffold::cmd_add_spec(&root, &name),
        Command::Scaffold {
            name,
            dir,
            template,
        } => commands::scaffold::cmd_scaffold(&root, &name, dir, template),
        Command::InitRegistry { name } => commands::init_registry::cmd_init_registry(&root, name),
        Command::Resolve {
            remote,
            verify,
            cache_ttl,
        } => commands::resolve::cmd_resolve(&root, remote || verify, verify, cache_ttl),
        Command::Diff { base } => commands::diff::cmd_diff(&root, &base, format),
        Command::Hooks { action } => commands::hooks::cmd_hooks(&root, action),
        Command::Compact { keep, dry_run } => commands::compact::cmd_compact(&root, keep, dry_run),
        Command::ArchiveTasks { dry_run } => {
            commands::archive_tasks::cmd_archive_tasks(&root, dry_run)
        }
        Command::View { role, spec } => commands::view::cmd_view(&root, &role, spec.as_deref()),
        Command::Merge { dry_run, all } => commands::merge::cmd_merge(&root, dry_run, all, format),
        Command::Issues { create } => commands::issues::cmd_issues(&root, format, create),
        Command::New { name, full } => commands::new::cmd_new(&root, &name, full),
        Command::Wizard => commands::wizard::cmd_wizard(&root),
        Command::Deps { mermaid, dot } => commands::deps::cmd_deps(&root, format, mermaid, dot),
        Command::Import {
            source,
            id,
            repo,
            all_issues,
            label,
            from_dir,
        } => commands::import::cmd_import(
            &root,
            source.as_deref(),
            id.as_deref(),
            repo.as_deref(),
            all_issues,
            label.as_deref(),
            from_dir.as_deref(),
        ),
        Command::Stale { threshold } => commands::stale::cmd_stale(&root, format, threshold),
        Command::Report { stale_threshold } => {
            commands::report::cmd_report(&root, format, stale_threshold)
        }
        Command::Comment { pr, base } => commands::comment::cmd_comment(&root, pr, &base),
        Command::Rules => commands::rules::cmd_rules(&root),
        Command::Changelog { range } => commands::changelog::cmd_changelog(&root, &range, format),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::compute_exit_code;
    use crate::types;

    fn empty_coverage() -> types::CoverageReport {
        types::CoverageReport {
            total_source_files: 0,
            specced_file_count: 0,
            unspecced_files: vec![],
            unspecced_modules: vec![],
            coverage_percent: 100,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 100,
            unspecced_file_loc: vec![],
        }
    }

    fn coverage_with_unspecced(files: Vec<&str>) -> types::CoverageReport {
        let total = files.len();
        types::CoverageReport {
            total_source_files: total,
            specced_file_count: 0,
            unspecced_files: files.iter().map(|s| s.to_string()).collect(),
            unspecced_modules: vec![],
            coverage_percent: 0,
            total_loc: 0,
            specced_loc: 0,
            loc_coverage_percent: 0,
            unspecced_file_loc: vec![],
        }
    }

    // ─── Warn mode ───────────────────────────────────────────────────────────

    #[test]
    fn warn_mode_exits_0_with_no_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 0, false, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_exits_0_even_with_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(5, 3, false, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_exits_0_even_with_strict_flag() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, true, types::EnforcementMode::Warn, &coverage, None),
            0
        );
    }

    #[test]
    fn warn_mode_respects_require_coverage() {
        let coverage = types::CoverageReport {
            coverage_percent: 50,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Warn,
                &coverage,
                Some(80)
            ),
            1
        );
    }

    // ─── EnforceNew mode ─────────────────────────────────────────────────────

    #[test]
    fn enforce_new_exits_0_when_all_files_specced() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            0
        );
    }

    #[test]
    fn enforce_new_exits_0_with_errors_if_all_specced() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(
                3,
                2,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            0
        );
    }

    #[test]
    fn enforce_new_exits_1_when_unspecced_files_exist() {
        let coverage = coverage_with_unspecced(vec!["src/foo.rs"]);
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            1
        );
    }

    #[test]
    fn enforce_new_exits_1_with_multiple_unspecced_files() {
        let coverage = coverage_with_unspecced(vec!["src/foo.rs", "src/bar.rs"]);
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::EnforceNew,
                &coverage,
                None
            ),
            1
        );
    }

    // ─── Strict mode ─────────────────────────────────────────────────────────

    #[test]
    fn strict_mode_exits_0_with_no_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 0, false, types::EnforcementMode::Strict, &coverage, None),
            0
        );
    }

    #[test]
    fn strict_mode_exits_1_with_errors() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(1, 0, false, types::EnforcementMode::Strict, &coverage, None),
            1
        );
    }

    #[test]
    fn strict_mode_exits_0_with_warnings_only() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, false, types::EnforcementMode::Strict, &coverage, None),
            0
        );
    }

    #[test]
    fn strict_mode_exits_1_with_warnings_and_strict_flag() {
        let coverage = empty_coverage();
        assert_eq!(
            compute_exit_code(0, 3, true, types::EnforcementMode::Strict, &coverage, None),
            1
        );
    }

    #[test]
    fn strict_mode_respects_require_coverage() {
        let coverage = types::CoverageReport {
            coverage_percent: 70,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(80)
            ),
            1
        );
    }

    #[test]
    fn strict_mode_exits_0_when_coverage_meets_threshold() {
        let coverage = types::CoverageReport {
            coverage_percent: 85,
            ..empty_coverage()
        };
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(80)
            ),
            0
        );
    }
}
