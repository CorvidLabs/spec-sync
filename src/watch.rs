use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use colored::Colorize;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

use crate::config::load_config;
use crate::types::OutputFormat;

/// The result of resolving configured watch directories.
#[derive(Debug, PartialEq)]
struct WatchDirs {
    /// Directories that exist and should be watched.
    watched: Vec<PathBuf>,
    /// Configured paths that do not exist, with their role (`specs_dir` or `source_dirs`).
    skipped: Vec<(String, String)>,
}

/// Resolve the configured `specs_dir` and `source_dirs` into the set of
/// directories that actually exist. Nonexistent configured paths are recorded
/// in `skipped` instead of silently dropped.
fn resolve_watch_dirs(root: &Path, specs_dir: &str, source_dirs: &[String]) -> WatchDirs {
    let abs_specs = root.join(specs_dir);
    let abs_sources: Vec<PathBuf> = source_dirs.iter().map(|d| root.join(d)).collect();

    let mut watched = Vec::new();
    let mut skipped = Vec::new();

    if abs_specs.is_dir() {
        watched.push(abs_specs);
    } else {
        skipped.push((specs_dir.to_string(), "specs_dir".to_string()));
    }

    for (configured, abs) in source_dirs.iter().zip(abs_sources.iter()) {
        if abs.is_dir() {
            watched.push(abs.clone());
        } else {
            skipped.push((configured.clone(), "source_dirs".to_string()));
        }
    }

    WatchDirs { watched, skipped }
}

/// Run the check command in watch mode, re-running on file changes.
/// Uses the hash cache to skip unchanged specs on subsequent runs.
pub fn run_watch(root: &Path, strict: bool, require_coverage: Option<usize>, format: OutputFormat) {
    let config = load_config(root);

    // Resolve the configured directories into the set to watch. A configured
    // directory that does not exist is reported but does not stop the watch:
    // watch is a long-running dev loop, and a typo in one of several paths
    // should not be fatal. It must never be invisible, though — silently
    // dropping a path makes the banner lie about what is being monitored.
    let resolved = resolve_watch_dirs(root, &config.specs_dir, &config.source_dirs);
    let watch_dirs = resolved.watched;

    // Report dropped directories on both human and machine-readable channels.
    // Even though watch is primarily interactive, CI and editor integrations
    // read its output and need a parseable signal that a configured path was
    // ignored.
    for (configured_path, role) in &resolved.skipped {
        match format {
            OutputFormat::Json => {
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "warning": "nonexistent_watch_directory",
                        "path": configured_path,
                        "role": role,
                        "message": format!(
                            "configured {role} does not exist and will not be watched: {configured_path}"
                        )
                    })
                );
            }
            _ => {
                eprintln!(
                    "{} configured {} does not exist and will not be watched: {}",
                    "⊘ Warning:".yellow(),
                    role,
                    configured_path
                );
            }
        }
    }

    if watch_dirs.is_empty() {
        eprintln!(
            "{} No directories to watch (specs_dir={}, source_dirs={:?})",
            "Error:".red(),
            config.specs_dir,
            config.source_dirs
        );
        std::process::exit(1);
    }

    // Initial run with --force to validate everything
    print_separator(None);
    run_check(root, strict, require_coverage, true);

    // Set up debounced file watcher
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |events| match events {
            Ok(evts) => {
                for evt in evts {
                    let _ = tx.send(evt);
                }
            }
            Err(errs) => {
                for e in errs {
                    eprintln!("{} watcher error: {e}", "Error:".red());
                }
            }
        },
    )
    .expect("Failed to create file watcher");

    for dir in &watch_dirs {
        debouncer
            .watch(dir, RecursiveMode::Recursive)
            .unwrap_or_else(|e| {
                eprintln!("{} Failed to watch {}: {e}", "Error:".red(), dir.display());
            });
    }

    println!(
        "\n{} Watching for changes in: {}",
        ">>>".cyan(),
        watch_dirs
            .iter()
            .map(|d| d.strip_prefix(root).unwrap_or(d).display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if strict {
        println!(
            "{} Strict mode active — all specs will be re-validated on each run",
            ">>>".cyan()
        );
    } else {
        println!(
            "{} Hash cache active — only changed specs will be re-validated",
            ">>>".cyan()
        );
    }
    println!("{} Press Ctrl+C to stop\n", ">>>".cyan());

    // Event loop
    let mut last_run = Instant::now();
    while let Ok(event) = rx.recv() {
        // Skip non-modify events
        if !is_relevant_event(&event) {
            continue;
        }

        // Extra debounce: don't re-run if we just ran
        if last_run.elapsed() < Duration::from_millis(300) {
            continue;
        }

        let changed_file: Option<String> = event
            .paths
            .first()
            .and_then(|p: &PathBuf| p.strip_prefix(root).ok())
            .map(|p: &Path| p.display().to_string());

        // Drain any remaining queued events
        while rx.try_recv().is_ok() {}

        print_separator(changed_file.as_deref());
        // Subsequent runs use hash cache (no --force), only re-validating changed specs
        run_check(root, strict, require_coverage, false);
        last_run = Instant::now();

        println!(
            "\n{} Watching for changes... (Ctrl+C to stop)",
            ">>>".cyan()
        );
    }
}

fn is_relevant_event(event: &DebouncedEvent) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn print_separator(changed_file: Option<&str>) {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!(
        "{}",
        "════════════════════════════════════════════════════════════".cyan()
    );
    if let Some(file) = changed_file {
        println!("{} Changed: {}", ">>>".cyan(), file.bold());
    } else {
        println!("{} Initial run (full validation)", ">>>".cyan());
    }
    println!(
        "{}",
        "════════════════════════════════════════════════════════════".cyan()
    );
}

fn build_check_args(
    root: &Path,
    strict: bool,
    require_coverage: Option<usize>,
    force: bool,
) -> Vec<std::ffi::OsString> {
    let mut args: Vec<std::ffi::OsString> = Vec::new();
    args.push("check".into());
    args.push("--root".into());
    args.push(root.as_os_str().to_owned());
    if strict {
        args.push("--strict".into());
    }
    if force {
        args.push("--force".into());
    }
    if let Some(cov) = require_coverage {
        args.push("--require-coverage".into());
        args.push(cov.to_string().into());
    }
    args
}

fn run_check(root: &Path, strict: bool, require_coverage: Option<usize>, force: bool) {
    // Fork a child process to isolate exit calls from the check command.
    use std::io::{BufRead, BufReader, IsTerminal};
    use std::process::{Command, Stdio};

    let start = Instant::now();
    let args = build_check_args(root, strict, require_coverage, force);
    let mut cmd = Command::new(std::env::current_exe().expect("Cannot find current executable"));
    for arg in &args {
        cmd.arg(arg);
    }

    // Pipe the child's stdout so the summary line can be inspected — the exit
    // code alone is not enough because `enforcement = warn` (the default)
    // exits 0 even when specs failed. Keep colors on when we're on a TTY.
    cmd.stdout(Stdio::piped());
    if std::io::stdout().is_terminal() {
        cmd.env("CLICOLOR_FORCE", "1");
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to run check: {e}", "Error:".red());
            return;
        }
    };

    // Stream the child's output through while scanning for the summary line.
    let mut failed_specs: Option<usize> = None;
    let mut examined_nothing = false;
    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            println!("{line}");
            if let Some(failed) = parse_failed_count(&line) {
                failed_specs = Some(failed);
            }
            if reports_no_specs(&line) {
                examined_nothing = true;
            }
        }
    }

    match child.wait() {
        Ok(status) => {
            let elapsed = start.elapsed();
            let failed = !status.success() || failed_specs.unwrap_or(0) > 0;
            if failed {
                println!(
                    "\n{} ({}ms)",
                    "Some checks failed.".red().bold(),
                    elapsed.as_millis()
                );
            } else if examined_nothing {
                // A check that found no specs exits zero, and reading that as
                // success is how watch printed a green `All checks passed!`
                // over a run that examined nothing — the same false all-clear
                // #577 produces on the watch set itself. Claim a pass only on
                // positive evidence that specs were examined.
                println!(
                    "\n{} ({}ms)",
                    "No specs were examined — nothing was checked."
                        .yellow()
                        .bold(),
                    elapsed.as_millis()
                );
            } else {
                println!(
                    "\n{} ({}ms)",
                    "All checks passed!".green().bold(),
                    elapsed.as_millis()
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to run check: {e}", "Error:".red());
        }
    }
}

/// Recognise the check command's "there was nothing to examine" line.
///
/// The child prints this and exits zero, so the exit status alone cannot
/// distinguish a clean run from an empty one.
fn reports_no_specs(line: &str) -> bool {
    strip_ansi(line)
        .trim_start()
        .starts_with("No spec files found in ")
}

/// Parse the failed-spec count from a check summary line like
/// `"3 specs checked: 2 passed, 1 warning(s), 1 failed"`.
/// Returns `None` for lines that are not the summary.
fn parse_failed_count(line: &str) -> Option<usize> {
    let plain = strip_ansi(line);
    let (_, rest) = plain.split_once(" specs checked: ")?;
    let failed_part = rest.rsplit(',').next()?.trim();
    let count = failed_part.strip_suffix(" failed")?.trim();
    count.parse().ok()
}

/// Remove ANSI escape sequences (colors) from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip a CSI sequence like `\x1b[31m` up to its terminating letter
            if chars.peek() == Some(&'[') {
                chars.next();
                for esc in chars.by_ref() {
                    if esc.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_event(kind: EventKind) -> DebouncedEvent {
        DebouncedEvent {
            event: notify::Event {
                kind,
                paths: vec![],
                attrs: Default::default(),
            },
            time: Instant::now(),
        }
    }

    fn make_event_with_path(kind: EventKind, path: PathBuf) -> DebouncedEvent {
        DebouncedEvent {
            event: notify::Event {
                kind,
                paths: vec![path],
                attrs: Default::default(),
            },
            time: Instant::now(),
        }
    }

    // --- is_relevant_event ---

    #[test]
    fn test_is_relevant_event_create() {
        let event = make_event(EventKind::Create(CreateKind::File));
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn test_is_relevant_event_modify() {
        let event = make_event(EventKind::Modify(ModifyKind::Data(
            notify::event::DataChange::Content,
        )));
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn test_is_relevant_event_remove() {
        let event = make_event(EventKind::Remove(RemoveKind::File));
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn test_is_relevant_event_rejects_access() {
        let event = make_event(EventKind::Access(AccessKind::Read));
        assert!(!is_relevant_event(&event));
    }

    #[test]
    fn test_is_relevant_event_rejects_other() {
        let event = make_event(EventKind::Other);
        assert!(!is_relevant_event(&event));
    }

    #[test]
    fn test_is_relevant_event_create_any() {
        let event = make_event(EventKind::Create(CreateKind::Any));
        assert!(is_relevant_event(&event));
    }

    // --- build_check_args ---

    #[test]
    fn test_build_check_args_basic() {
        let tmp = TempDir::new().unwrap();
        let args = build_check_args(tmp.path(), false, None, false);
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert_eq!(strs[0], "check");
        assert_eq!(strs[1], "--root");
        assert_eq!(strs[2], tmp.path().to_string_lossy());
        assert_eq!(strs.len(), 3);
    }

    #[test]
    fn test_build_check_args_strict() {
        let tmp = TempDir::new().unwrap();
        let args = build_check_args(tmp.path(), true, None, false);
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(strs.contains(&"--strict".to_string()));
        assert!(!strs.contains(&"--force".to_string()));
    }

    #[test]
    fn test_build_check_args_force() {
        let tmp = TempDir::new().unwrap();
        let args = build_check_args(tmp.path(), false, None, true);
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(strs.contains(&"--force".to_string()));
        assert!(!strs.contains(&"--strict".to_string()));
    }

    #[test]
    fn test_build_check_args_require_coverage() {
        let tmp = TempDir::new().unwrap();
        let args = build_check_args(tmp.path(), false, Some(80), false);
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(strs.contains(&"--require-coverage".to_string()));
        assert!(strs.contains(&"80".to_string()));
    }

    #[test]
    fn test_build_check_args_all_flags() {
        let tmp = TempDir::new().unwrap();
        let args = build_check_args(tmp.path(), true, Some(95), true);
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(strs.contains(&"--strict".to_string()));
        assert!(strs.contains(&"--force".to_string()));
        assert!(strs.contains(&"--require-coverage".to_string()));
        assert!(strs.contains(&"95".to_string()));
        assert_eq!(strs.len(), 7); // check --root <path> --strict --force --require-coverage 95
    }

    // --- run_watch empty directories ---

    #[test]
    fn test_run_watch_collects_watch_dirs() {
        // Verify that the watch directory collection logic works correctly
        let tmp = TempDir::new().unwrap();
        let specs_dir = tmp.path().join("specs");
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();

        let resolved = resolve_watch_dirs(tmp.path(), "specs", &["src".to_string()]);

        assert_eq!(resolved.watched.len(), 2);
        assert!(resolved.skipped.is_empty());
    }

    #[test]
    fn test_run_watch_empty_dirs_detected() {
        // Verify that empty watch dirs are detected
        let tmp = TempDir::new().unwrap();
        // No specs or source dirs exist

        let resolved = resolve_watch_dirs(tmp.path(), "specs", &["src".to_string()]);

        assert!(resolved.watched.is_empty());
        assert_eq!(resolved.skipped.len(), 2);
        assert!(
            resolved
                .skipped
                .contains(&("specs".to_string(), "specs_dir".to_string()))
        );
        assert!(
            resolved
                .skipped
                .contains(&("src".to_string(), "source_dirs".to_string()))
        );
    }

    #[test]
    fn test_resolve_watch_dirs_reports_partially_missing_dirs() {
        // One configured directory exists and one does not; both fates are recorded.
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();

        let resolved = resolve_watch_dirs(
            tmp.path(),
            "missing-specs",
            &["src".to_string(), "also-missing".to_string()],
        );

        assert_eq!(resolved.watched.len(), 1);
        assert_eq!(resolved.skipped.len(), 2);
        assert!(
            resolved
                .skipped
                .contains(&("missing-specs".to_string(), "specs_dir".to_string()))
        );
        assert!(
            resolved
                .skipped
                .contains(&("also-missing".to_string(), "source_dirs".to_string()))
        );
    }

    // --- event path extraction ---

    #[test]
    fn test_event_path_extraction() {
        let root = PathBuf::from("/project");
        let event = make_event_with_path(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            PathBuf::from("/project/specs/auth/auth.spec.md"),
        );

        let changed_file: Option<String> = event
            .paths
            .first()
            .and_then(|p| p.strip_prefix(&root).ok())
            .map(|p| p.display().to_string());

        assert_eq!(changed_file, Some("specs/auth/auth.spec.md".to_string()));
    }

    #[test]
    fn test_event_path_extraction_no_paths() {
        let root = PathBuf::from("/project");
        let event = make_event(EventKind::Create(CreateKind::File));

        let changed_file: Option<String> = event
            .paths
            .first()
            .and_then(|p| p.strip_prefix(&root).ok())
            .map(|p| p.display().to_string());

        assert_eq!(changed_file, None);
    }

    // --- summary line parsing (watch footer) ---

    #[test]
    fn test_parse_failed_count_with_failures() {
        let line = "3 specs checked: 2 passed, 1 warning(s), 1 failed";
        assert_eq!(parse_failed_count(line), Some(1));
    }

    #[test]
    fn test_parse_failed_count_all_passed() {
        let line = "13 specs checked: 13 passed, 0 warning(s), 0 failed";
        assert_eq!(parse_failed_count(line), Some(0));
    }

    #[test]
    fn test_parse_failed_count_ignores_other_lines() {
        assert_eq!(parse_failed_count("All checks passed!"), None);
        assert_eq!(parse_failed_count("  ✗ Frontmatter invalid"), None);
        assert_eq!(parse_failed_count(""), None);
    }

    #[test]
    fn test_parse_failed_count_strips_colors() {
        // print_summary colors the failed count red when non-zero
        let line = "1 specs checked: \u{1b}[32m0\u{1b}[0m passed, \u{1b}[33m0\u{1b}[0m warning(s), \u{1b}[31m1\u{1b}[0m failed";
        assert_eq!(parse_failed_count(line), Some(1));
    }

    #[test]
    fn test_reports_no_specs_recognises_the_empty_run() {
        assert!(reports_no_specs(
            "No spec files found in /tmp/proj/nope/. Run `specsync generate` to scaffold specs."
        ));
    }

    #[test]
    fn test_reports_no_specs_ignores_a_real_run() {
        // The control: lines from a run that did examine specs must not be
        // mistaken for the empty case, or watch would never report a pass.
        assert!(!reports_no_specs(
            "1 specs checked: 1 passed, 0 warning(s), 0 failed"
        ));
        assert!(!reports_no_specs(
            "  ✓ specs/mod.md — 1/1 exports documented"
        ));
        assert!(!reports_no_specs("File coverage: 0/1 (0%)"));
        assert!(!reports_no_specs(""));
    }

    #[test]
    fn test_strip_ansi_removes_escape_sequences() {
        assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m"), "red");
        assert_eq!(strip_ansi("plain"), "plain");
    }
}
