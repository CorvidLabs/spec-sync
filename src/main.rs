mod ai;
mod archive;
mod cli;
mod compact;
mod commands;
mod config;
mod exports;
mod generator;
mod github;
mod hash_cache;
mod hooks;
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
    });

    match command {
        Command::Init => commands::init::cmd_init(&root),
        Command::Check {
            fix,
            force,
            create_issues,
        } => commands::check::cmd_check(
            &root,
            cli.strict,
            cli.require_coverage,
            format,
            fix,
            force,
            create_issues,
        ),
        Command::Coverage => {
            commands::coverage::cmd_coverage(&root, cli.strict, cli.require_coverage, format)
        }
        Command::Generate { provider } => {
            commands::generate::cmd_generate(&root, cli.strict, cli.require_coverage, format, provider)
        }
        Command::Score => commands::score::cmd_score(&root, format),
        Command::Watch => watch::run_watch(&root, cli.strict, cli.require_coverage),
        Command::Mcp => mcp::run_mcp_server(&root),
        Command::AddSpec { name } => commands::init::cmd_add_spec(&root, &name),
        Command::InitRegistry { name } => commands::init::cmd_init_registry(&root, name),
        Command::Resolve { remote } => commands::resolve::cmd_resolve(&root, remote),
        Command::Diff { base } => commands::diff::cmd_diff(&root, &base, format),
        Command::Hooks { action } => commands::hooks::cmd_hooks(&root, action),
        Command::Compact { keep, dry_run } => {
            commands::compact::cmd_compact(&root, keep, dry_run)
        }
        Command::ArchiveTasks { dry_run } => commands::compact::cmd_archive_tasks(&root, dry_run),
        Command::View { role, spec } => commands::view::cmd_view(&root, &role, spec.as_deref()),
        Command::Merge { dry_run, all } => commands::merge::cmd_merge(&root, dry_run, all, format),
        Command::Issues { create } => commands::issues::cmd_issues(&root, format, create),
    }
}
