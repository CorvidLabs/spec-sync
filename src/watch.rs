use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use colored::Colorize;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

use crate::config::load_config;

/// Run the check command in watch mode, re-running on file changes.
/// Uses the hash cache to skip unchanged specs on subsequent runs.
pub fn run_watch(root: &Path, strict: bool, require_coverage: Option<usize>) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let source_dirs: Vec<PathBuf> = config.source_dirs.iter().map(|d| root.join(d)).collect();

    // Collect directories to watch
    let mut watch_dirs: Vec<PathBuf> = Vec::new();
    if specs_dir.is_dir() {
        watch_dirs.push(specs_dir.clone());
    }
    for dir in &source_dirs {
        if dir.is_dir() {
            watch_dirs.push(dir.clone());
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
    println!(
        "{} Hash cache active — only changed specs will be re-validated",
        ">>>".cyan()
    );
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

fn run_check(root: &Path, strict: bool, require_coverage: Option<usize>, force: bool) {
    // Fork a child process to isolate exit calls from the check command.
    use std::process::Command;

    let start = Instant::now();
    let mut cmd = Command::new(std::env::current_exe().expect("Cannot find current executable"));
    cmd.arg("check");
    cmd.arg("--root").arg(root);
    if strict {
        cmd.arg("--strict");
    }
    if force {
        cmd.arg("--force");
    }
    if let Some(cov) = require_coverage {
        cmd.arg("--require-coverage").arg(cov.to_string());
    }

    match cmd.status() {
        Ok(status) => {
            let elapsed = start.elapsed();
            if status.success() {
                println!(
                    "\n{} ({}ms)",
                    "All checks passed!".green().bold(),
                    elapsed.as_millis()
                );
            } else {
                println!(
                    "\n{} ({}ms)",
                    "Some checks failed.".red().bold(),
                    elapsed.as_millis()
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to run check: {e}", "Error:".red());
        }
    }
}
