// A reader that closes early (`specsync … | head`) must not crash SpecSync or hide its verdict
// (#1036). Rust ignores SIGPIPE, so a write to a closed pipe fails with `BrokenPipe`, and the
// standard printing macros panic on that error, which turned a passing command into a crash. These
// definitions replace them crate-wide: a `macro_rules!` defined before the `mod` items below
// shadows the prelude macro of the same name in every module. Once the reader is gone, further
// output to that stream is dropped and the command runs to its own exit status, 0 or 1. Any other
// write error still panics, as the standard macros do.
macro_rules! println {
    () => {
        $crate::write_output_stream($crate::OutputStream::Stdout, format_args!(""), true)
    };
    ($($arg:tt)*) => {
        $crate::write_output_stream($crate::OutputStream::Stdout, format_args!($($arg)*), true)
    };
}

macro_rules! print {
    ($($arg:tt)*) => {
        $crate::write_output_stream($crate::OutputStream::Stdout, format_args!($($arg)*), false)
    };
}

macro_rules! eprintln {
    () => {
        $crate::write_output_stream($crate::OutputStream::Stderr, format_args!(""), true)
    };
    ($($arg:tt)*) => {
        $crate::write_output_stream($crate::OutputStream::Stderr, format_args!($($arg)*), true)
    };
}

// Unused today; defined so a future `eprint!` cannot bypass the closed-pipe handling.
#[allow(unused_macros)]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::write_output_stream($crate::OutputStream::Stderr, format_args!($($arg)*), false)
    };
}

mod agents;
mod archive;
#[cfg(test)]
mod build_fingerprint;
mod change;
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
mod mcp_tools_lock;
mod merge;
mod merge_driver;
mod output;
mod parser;
mod registry;
mod schema;
mod scoring;
mod types;
mod util;
mod validator;
mod view;
mod watch;

use clap::Parser;
use colored::Colorize;
use std::process;

use cli::{Cli, Command, LifecycleAction, McpAction};

const VERIFICATION_CONTEXT_ENV: &str = "SPECSYNC_VERIFICATION_CONTEXT";

/// Git environment that changes which history Git walks: replacement refs outside
/// `refs/replace/`, replacements switched off, and graft and shallow files outside the
/// repository. SpecSync's read-cache and lifecycle-index guards check only the repository's own
/// `refs/replace/`, `info/grafts` and shallow state, so SpecSync's Git must never see these
/// (REQ-cli-013, REQ-change-148).
const HISTORY_REWRITING_GIT_ENV: [&str; 4] = [
    "GIT_REPLACE_REF_BASE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_GRAFT_FILE",
    "GIT_SHALLOW_FILE",
];

/// The two standard streams the printing macros write to.
#[derive(Clone, Copy)]
enum OutputStream {
    Stdout,
    Stderr,
}

/// Set once a write to stdout found its reader gone (#1036).
static STDOUT_CLOSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Set once a write to stderr found its reader gone (#1036).
static STDERR_CLOSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether stdout's reader has gone, so a command that only reports (such as `watch`) can stop.
fn stdout_closed() -> bool {
    STDOUT_CLOSED.load(std::sync::atomic::Ordering::Relaxed)
}

/// What the printing macros above expand to: write `args` (and a newline) to `stream`, drop the
/// output once that stream's reader has gone, and panic on any other write error.
fn write_output_stream(stream: OutputStream, args: std::fmt::Arguments<'_>, newline: bool) {
    use std::io::Write;
    // Under `cargo test` the standard macros print through libtest's output capture, so a passing
    // unit test stays quiet; the closed-pipe handling matters only to the built binary, which the
    // integration tests run (#1048 review).
    if cfg!(test) {
        match (stream, newline) {
            (OutputStream::Stdout, true) => std::println!("{args}"),
            (OutputStream::Stdout, false) => std::print!("{args}"),
            (OutputStream::Stderr, true) => std::eprintln!("{args}"),
            (OutputStream::Stderr, false) => std::eprint!("{args}"),
        }
        return;
    }
    let (closed, name) = match stream {
        OutputStream::Stdout => (&STDOUT_CLOSED, "stdout"),
        OutputStream::Stderr => (&STDERR_CLOSED, "stderr"),
    };
    if closed.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    fn write_to(
        out: &mut dyn Write,
        args: std::fmt::Arguments<'_>,
        newline: bool,
    ) -> std::io::Result<()> {
        out.write_fmt(args)?;
        if newline {
            out.write_all(b"\n")?;
        }
        Ok(())
    }
    let result = match stream {
        OutputStream::Stdout => write_to(&mut std::io::stdout().lock(), args, newline),
        OutputStream::Stderr => write_to(&mut std::io::stderr().lock(), args, newline),
    };
    match result {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {
            closed.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Err(error) => panic!("failed printing to {name}: {error}"),
    }
}

fn main() {
    for key in HISTORY_REWRITING_GIT_ENV {
        // SAFETY: this is the first thing `main` does, before any thread is spawned and before
        // anything reads the environment, so no other thread can observe the change.
        unsafe { std::env::remove_var(key) };
    }
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
    if !root.is_dir() {
        eprintln!(
            "{} --root path does not exist or is not a directory: {}",
            "error:".red().bold(),
            root.display()
        );
        process::exit(2);
    }

    // --json flag is shorthand for --format json (backward compat)
    let format = if cli.json {
        types::OutputFormat::Json
    } else {
        cli.format
    };

    let command = cli.command.unwrap_or(Command::Check {
        fix: false,
        dry_run: false,
        backup: false,
        force: false,
        create_issues: false,
        explain: false,
        stale: None,
        spec: vec![],
        specs: vec![],
    });

    // Security-sensitive retained-capability commands must bind the user-requested directory
    // identity before canonicalizing any path. Canonicalizing a junction or symlink here would
    // erase the public pathname whose replacement the retained traversal must detect.
    let root = if matches!(
        &command,
        Command::Mcp { .. }
            | Command::Check { .. }
            | Command::Coverage
            | Command::Generate { .. }
            | Command::Score { .. }
            | Command::Report { .. }
            | Command::Comment { .. }
    ) {
        root
    } else {
        root.canonicalize().unwrap_or(root)
    };

    if matches!(
        &command,
        Command::Check { .. } | Command::Change { .. } | Command::Lifecycle { .. }
    ) {
        reject_recursive_lifecycle_dispatch(format);
    }

    match command {
        Command::Init { repair } => commands::init::cmd_init(&root, repair, format),
        Command::Check {
            fix,
            dry_run,
            backup,
            force,
            create_issues,
            explain,
            stale,
            spec,
            specs,
        } => {
            let mut filters = spec;
            filters.extend(specs);
            commands::check::cmd_check(
                &root,
                cli.strict,
                cli.enforcement,
                cli.require_coverage,
                format,
                fix,
                dry_run,
                backup,
                force,
                create_issues,
                explain,
                stale,
                &filters,
                &cli.exclude_status,
                &cli.only_status,
            )
        }
        Command::Coverage => commands::coverage::cmd_coverage(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
        ),
        Command::Generate { uncovered, batch } => commands::generate::cmd_generate(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
            uncovered,
            batch,
        ),
        Command::Score {
            explain,
            min_score,
            all,
            specs,
        } => commands::score::cmd_score(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            min_score,
            format,
            explain,
            all,
            &specs,
            &cli.exclude_status,
            &cli.only_status,
        ),
        Command::Watch => watch::run_watch(&root, cli.strict, cli.require_coverage, format),
        Command::Mcp {
            allow_write,
            action: None,
        } => {
            if let Err(message) = mcp::run_mcp_server(&root, allow_write) {
                eprintln!("{} {message}", "error:".red().bold());
                process::exit(2);
            }
        }
        Command::Mcp {
            action:
                Some(McpAction::Lock {
                    write,
                    allow_write: lock_allow_write,
                }),
            ..
        } => {
            let result = if write {
                mcp_tools_lock::cmd_lock_write(&root, lock_allow_write)
            } else {
                mcp_tools_lock::cmd_lock_print(lock_allow_write)
            };
            if let Err(message) = result {
                eprintln!("{} {message}", "error:".red().bold());
                process::exit(2);
            }
        }
        Command::Mcp {
            action:
                Some(McpAction::Diff {
                    allow_write: diff_allow_write,
                }),
            ..
        } => {
            if let Err(message) = mcp_tools_lock::cmd_diff(&root, diff_allow_write) {
                eprintln!("{} {message}", "error:".red().bold());
                process::exit(1);
            }
        }
        Command::AddSpec { name } => commands::scaffold::cmd_add_spec(&root, &name),
        Command::Scaffold {
            name,
            dir,
            template,
        } => commands::scaffold::cmd_scaffold(&root, &name, dir, template),
        Command::InitRegistry { name } => {
            commands::init_registry::cmd_init_registry(&root, name, format)
        }
        Command::Resolve {
            remote,
            verify,
            cache_ttl,
        } => commands::resolve::cmd_resolve(
            &root,
            remote || verify,
            verify,
            cache_ttl,
            format,
            cli.strict,
        ),
        Command::Diff { base } => {
            commands::diff::cmd_diff(&root, base.as_deref(), format, cli.strict)
        }
        Command::Hooks { action } => commands::hooks::cmd_hooks(&root, action),
        Command::Agents { action } => commands::agents::cmd_agents(&root, action),
        Command::Compact { keep, dry_run } => {
            commands::compact::cmd_compact(&root, keep, dry_run, format)
        }
        Command::ArchiveTasks { dry_run } => {
            commands::archive_tasks::cmd_archive_tasks(&root, dry_run, format)
        }
        Command::View { role, spec } => commands::view::cmd_view(&root, &role, spec.as_deref()),
        Command::Merge {
            dry_run,
            all,
            install,
        } => {
            if install {
                commands::merge::cmd_install_merge_driver(&root, format);
            } else {
                commands::merge::cmd_merge(&root, dry_run, all, format);
            }
        }
        Command::Doctor {
            fix,
            dry_run,
            close_late,
            actor,
            reason,
        } => commands::change::cmd_doctor(
            &root,
            change::DoctorRequest {
                fix,
                dry_run,
                close_late,
                actor,
                reason,
            },
            format,
        ),
        Command::MergeFile {
            ancestor,
            current,
            other,
            pathname,
        } => commands::merge::cmd_merge_file(&ancestor, &current, &other, pathname.as_deref()),
        Command::Issues { create } => commands::issues::cmd_issues(&root, format, create),
        Command::New { name, full } => commands::new::cmd_new(&root, &name, full),
        Command::Wizard => commands::wizard::cmd_wizard(&root),
        Command::Deps { mermaid, dot } => commands::deps::cmd_deps(
            &root,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
            format,
            mermaid,
            dot,
        ),
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
        Command::Stale { threshold } => commands::stale::cmd_stale(
            &root,
            format,
            threshold,
            &cli.exclude_status,
            &cli.only_status,
            cli.enforcement,
        ),
        Command::Report { stale_threshold } => commands::report::cmd_report(
            &root,
            format,
            stale_threshold,
            &cli.exclude_status,
            &cli.only_status,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
        ),
        Command::Comment { pr, base } => commands::comment::cmd_comment(
            &root,
            pr,
            &base,
            cli.strict,
            cli.enforcement,
            cli.require_coverage,
        ),
        Command::Rules => commands::rules::cmd_rules(&root),
        Command::Changelog { range } => commands::changelog::cmd_changelog(&root, &range, format),
        Command::Rehash => commands::rehash::cmd_rehash(&root),
        Command::Migrate {
            from,
            dry_run,
            no_backup,
        } => commands::migrate::cmd_migrate(&root, format, from, dry_run, no_backup),
        Command::Lifecycle { action } => match action {
            LifecycleAction::Promote { spec, force } => {
                commands::lifecycle::cmd_promote(&root, &spec, format, force)
            }
            LifecycleAction::Demote { spec, force } => {
                commands::lifecycle::cmd_demote(&root, &spec, format, force)
            }
            LifecycleAction::Set {
                spec,
                status,
                force,
            } => commands::lifecycle::cmd_set(&root, &spec, &status, format, force),
            LifecycleAction::Status { spec } => {
                commands::lifecycle::cmd_status(&root, spec.as_deref(), format)
            }
            LifecycleAction::History { spec } => {
                commands::lifecycle::cmd_history(&root, &spec, format)
            }
            LifecycleAction::Guard { spec, target } => {
                commands::lifecycle::cmd_guard(&root, &spec, target.as_deref(), format)
            }
            LifecycleAction::AutoPromote { dry_run } => {
                commands::lifecycle::cmd_auto_promote(&root, format, dry_run)
            }
            LifecycleAction::Enforce {
                require_status,
                max_age,
                allowed,
                all,
            } => commands::lifecycle::cmd_enforce(
                &root,
                format,
                require_status || all,
                max_age || all,
                allowed || all,
            ),
        },
        Command::Change { action } => {
            commands::change::cmd_change(&root, action, format, cli.strict)
        }
    }
}

fn reject_recursive_lifecycle_dispatch(format: types::OutputFormat) {
    let Some(error) = verification_recursion_error() else {
        return;
    };
    match format {
        types::OutputFormat::Json => println!("{}", serde_json::json!({ "error": error })),
        _ => eprintln!("{} {error}", "error:".red().bold()),
    }
    process::exit(1);
}

fn verification_recursion_error() -> Option<String> {
    let configured = std::env::var(VERIFICATION_CONTEXT_ENV).ok()?;
    let executable = std::env::current_exe().ok()?;
    let file_name = executable.file_name()?.to_string_lossy();
    if file_name != "specsync" && file_name != "specsync.exe" {
        return None;
    }
    Some(format!(
        "recursive lifecycle verification detected while executing `{configured}`; verification commands must not invoke specsync check or change verification"
    ))
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
            total_loc: 0,
            specced_loc: 0,
            unspecced_file_loc: vec![],
            missing_files: vec![],
            skipped_links: vec![],
            manifest_notices: vec![],
            manifest_module_paths: std::collections::HashMap::new(),
        }
    }

    /// A realistic coverage report at `percent`% over a non-empty source tree (100
    /// files). Use this for `--require-coverage` tests so they exercise the
    /// percentage gate — not the separate zero-source-files gate.
    fn coverage_pct(percent: usize) -> types::CoverageReport {
        types::CoverageReport {
            total_source_files: 100,
            specced_file_count: percent,
            ..empty_coverage()
        }
    }

    fn coverage_with_unspecced(files: Vec<&str>) -> types::CoverageReport {
        let total = files.len();
        types::CoverageReport {
            total_source_files: total,
            specced_file_count: 0,
            unspecced_files: files.iter().map(|s| s.to_string()).collect(),
            unspecced_modules: vec![],
            total_loc: 0,
            specced_loc: 0,
            unspecced_file_loc: vec![],
            missing_files: vec![],
            skipped_links: vec![],
            manifest_notices: vec![],
            manifest_module_paths: std::collections::HashMap::new(),
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
        let coverage = coverage_pct(50);
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
        let coverage = coverage_pct(70);
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
        let coverage = coverage_pct(85);
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

    #[test]
    fn require_coverage_fails_loud_on_zero_source_files() {
        // Regression: `--require-coverage` over 0 source files reports a vacuous 100%
        // and must NOT silently pass — it indicates an empty/misconfigured source tree.
        let coverage = empty_coverage(); // total_source_files: 0, so no percentage exists
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(100)
            ),
            1,
            "require-coverage over zero source files must fail loud"
        );
        // A require of 0 is a no-op requirement and still passes with no sources.
        assert_eq!(
            compute_exit_code(
                0,
                0,
                false,
                types::EnforcementMode::Strict,
                &coverage,
                Some(0)
            ),
            0
        );
        // No require-coverage flag → zero sources is fine.
        assert_eq!(
            compute_exit_code(0, 0, false, types::EnforcementMode::Strict, &coverage, None),
            0
        );
    }

    #[test]
    fn require_coverage_has_no_number_to_compare_against_an_unmeasured_tree() {
        // #582: the gate used to compare `100 < req` against a fabricated
        // percentage, and only a separate `total_source_files == 0` guard kept
        // it from passing. The percentage itself is now absent, so even at
        // every threshold from 1 upward there is nothing that could satisfy it.
        let coverage = empty_coverage();
        assert_eq!(coverage.file_coverage_percent(), None);
        for req in [1, 50, 80, 99, 100] {
            assert_eq!(
                compute_exit_code(
                    0,
                    0,
                    false,
                    types::EnforcementMode::Strict,
                    &coverage,
                    Some(req)
                ),
                1,
                "--require-coverage {req} must fail closed over an unmeasured tree"
            );
        }
    }

    #[test]
    fn require_coverage_still_gates_on_a_real_percentage() {
        // The other direction: the fix must not turn every gate into a failure.
        let coverage = coverage_pct(75);
        assert_eq!(coverage.file_coverage_percent(), Some(75));
        for (req, expected) in [(0, 0), (50, 0), (75, 0), (76, 1), (100, 1)] {
            assert_eq!(
                compute_exit_code(
                    0,
                    0,
                    false,
                    types::EnforcementMode::Strict,
                    &coverage,
                    Some(req)
                ),
                expected,
                "--require-coverage {req} against 75% coverage"
            );
        }
    }
}
