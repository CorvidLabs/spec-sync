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
    use std::process::Command;

    let start = Instant::now();
    let args = build_check_args(root, strict, require_coverage, force);
    let mut cmd = Command::new(std::env::current_exe().expect("Cannot find current executable"));
    for arg in &args {
        cmd.arg(arg);
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

        // Write a basic config
        let config_content = r#"{"specsDir": "specs", "sourceDirs": ["src"]}"#;
        std::fs::write(tmp.path().join("specsync.json"), config_content).unwrap();

        let config = load_config(tmp.path());
        let specs = tmp.path().join(&config.specs_dir);
        let source_dirs: Vec<PathBuf> = config
            .source_dirs
            .iter()
            .map(|d| tmp.path().join(d))
            .collect();

        let mut watch_dirs: Vec<PathBuf> = Vec::new();
        if specs.is_dir() {
            watch_dirs.push(specs);
        }
        for dir in &source_dirs {
            if dir.is_dir() {
                watch_dirs.push(dir.clone());
            }
        }

        assert_eq!(watch_dirs.len(), 2);
    }

    #[test]
    fn test_run_watch_empty_dirs_detected() {
        // Verify that empty watch dirs are detected
        let tmp = TempDir::new().unwrap();
        // No specs or source dirs exist
        let config_content = r#"{"specsDir": "specs", "sourceDirs": ["src"]}"#;
        std::fs::write(tmp.path().join("specsync.json"), config_content).unwrap();

        let config = load_config(tmp.path());
        let specs = tmp.path().join(&config.specs_dir);
        let source_dirs: Vec<PathBuf> = config
            .source_dirs
            .iter()
            .map(|d| tmp.path().join(d))
            .collect();

        let mut watch_dirs: Vec<PathBuf> = Vec::new();
        if specs.is_dir() {
            watch_dirs.push(specs);
        }
        for dir in &source_dirs {
            if dir.is_dir() {
                watch_dirs.push(dir.clone());
            }
        }

        assert!(watch_dirs.is_empty());
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
}
