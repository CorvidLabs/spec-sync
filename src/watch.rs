use std::collections::{HashMap, HashSet};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use colored::Colorize;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, Debouncer, RecommendedCache, new_debouncer};
use sha2::{Digest, Sha256};

use crate::config::load_config;
use crate::types::OutputFormat;

#[derive(Clone, Debug, PartialEq)]
struct WatchTarget {
    path: PathBuf,
    configured: String,
    role: &'static str,
    mode: RecursiveMode,
    identity: (u64, u64),
}

#[derive(Clone, Debug, PartialEq)]
enum WatchPathIssue {
    Missing,
    Invalid(String),
}

#[derive(Debug, PartialEq)]
struct SkippedWatchPath {
    configured: String,
    role: &'static str,
    issue: WatchPathIssue,
}

impl SkippedWatchPath {
    fn message(&self) -> String {
        match &self.issue {
            WatchPathIssue::Missing => format!(
                "configured {} does not exist and will not be watched: {}",
                self.role, self.configured
            ),
            WatchPathIssue::Invalid(reason) => format!(
                "configured {} is invalid and will not be watched: {} ({reason})",
                self.role, self.configured
            ),
        }
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            // Keep the existing missing-path code for editor integrations.
            "warning": match self.issue {
                WatchPathIssue::Missing => "nonexistent_watch_directory",
                WatchPathIssue::Invalid(_) => "invalid_watch_path",
            },
            "path": self.configured,
            "role": self.role,
            "message": self.message(),
        })
    }
}

/// Configured directories and exact source files, plus truthful rejected-path diagnostics.
#[derive(Debug, PartialEq)]
struct WatchDirs {
    watched: Vec<WatchTarget>,
    skipped: Vec<SkippedWatchPath>,
}

fn watch_relative_path(configured: &str) -> Result<PathBuf, WatchPathIssue> {
    // Interpret portable separators consistently and reject drive-qualified paths on Unix too.
    let portable = configured.replace('\\', "/");
    if portable.is_empty() || portable.as_bytes().get(1) == Some(&b':') {
        return Err(WatchPathIssue::Invalid(
            "path must remain beneath the project root".to_string(),
        ));
    }
    let mut relative = PathBuf::new();
    for component in Path::new(&portable).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(name) => relative.push(name),
            _ => {
                return Err(WatchPathIssue::Invalid(
                    "path must remain beneath the project root".to_string(),
                ));
            }
        }
    }
    Ok(relative)
}

#[cfg(windows)]
fn watch_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;
    metadata.file_attributes() & 0x0000_0400 != 0
}

#[cfg(not(windows))]
fn watch_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(unix)]
fn watch_metadata_identity(metadata: &Metadata) -> Result<(u64, u64), WatchPathIssue> {
    use cap_std::fs::MetadataExt;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn watch_metadata_identity(metadata: &Metadata) -> Result<(u64, u64), WatchPathIssue> {
    use cap_primitives::fs::_WindowsByHandle;
    metadata
        .volume_serial_number()
        .zip(metadata.file_index())
        .map(|(volume, index)| (u64::from(volume), index))
        .ok_or_else(|| WatchPathIssue::Invalid("filesystem identity is unavailable".to_string()))
}

#[cfg(not(any(unix, windows)))]
fn watch_metadata_identity(_metadata: &Metadata) -> Result<(u64, u64), WatchPathIssue> {
    Err(WatchPathIssue::Invalid(
        "filesystem identity is unavailable".to_string(),
    ))
}

fn watch_io_issue(error: io::Error) -> WatchPathIssue {
    WatchPathIssue::Invalid(format!("cannot inspect or open path: {error}"))
}

/// Inspect each component relative to a retained root, without accepting link traversal.
/// The watcher API itself takes a path, so registration revalidates this identity too.
fn inspect_watch_target(
    root: &Path,
    configured: &str,
    role: &'static str,
    fingerprint: bool,
) -> Result<(WatchTarget, Option<[u8; 32]>), WatchPathIssue> {
    let relative = watch_relative_path(configured)?;
    let mut directory = Dir::open_ambient_dir(root, ambient_authority()).map_err(watch_io_issue)?;
    let mut metadata = directory.dir_metadata().map_err(watch_io_issue)?;
    let mut digest = None;
    let mut components = relative.components().peekable();
    while let Some(Component::Normal(name)) = components.next() {
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(WatchPathIssue::Missing);
            }
            Err(error) => return Err(watch_io_issue(error)),
        };
        if watch_metadata_is_link(&before) {
            return Err(WatchPathIssue::Invalid(
                "path must not traverse a symlink or reparse point".to_string(),
            ));
        }
        let identity = watch_metadata_identity(&before)?;
        let leaf = components.peek().is_none();
        let next = if before.is_dir() {
            let next = directory.open_dir(name).map_err(watch_io_issue)?;
            metadata = next.dir_metadata().map_err(watch_io_issue)?;
            Some(next)
        } else if leaf && role == "source_dirs" && before.is_file() {
            let mut options = OpenOptions::new();
            options
                .read(true)
                ._cap_fs_ext_nonblock(true)
                ._cap_fs_ext_follow(FollowSymlinks::No);
            let file = directory
                .open_with(name, &options)
                .map_err(watch_io_issue)?;
            metadata = file.metadata().map_err(watch_io_issue)?;
            if fingerprint {
                // Match the validator's existing source-file read cap. Never read siblings.
                const MAX_WATCH_SOURCE_BYTES: u64 = 8 * 1024 * 1024;
                if metadata.len() > MAX_WATCH_SOURCE_BYTES {
                    return Err(WatchPathIssue::Invalid(
                        "exact-source fingerprint exceeds 8 MiB".to_string(),
                    ));
                }
                let mut bytes = Vec::new();
                file.take(MAX_WATCH_SOURCE_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(watch_io_issue)?;
                let after_read = directory.symlink_metadata(name).map_err(watch_io_issue)?;
                if bytes.len() as u64 > MAX_WATCH_SOURCE_BYTES
                    || bytes.len() as u64 != metadata.len()
                    || after_read.len() != metadata.len()
                    || after_read.modified().ok() != metadata.modified().ok()
                {
                    return Err(WatchPathIssue::Invalid(
                        "exact source changed during bounded fingerprinting".to_string(),
                    ));
                }
                digest = Some(Sha256::digest(&bytes).into());
            }
            None
        } else {
            return Err(WatchPathIssue::Invalid(
                "expected a directory or an exact regular source file".to_string(),
            ));
        };
        let after = directory.symlink_metadata(name).map_err(watch_io_issue)?;
        if watch_metadata_is_link(&after)
            || identity != watch_metadata_identity(&metadata)?
            || identity != watch_metadata_identity(&after)?
        {
            return Err(WatchPathIssue::Invalid(
                "path changed during confined inspection".to_string(),
            ));
        }
        if let Some(next) = next {
            directory = next;
        }
    }
    Ok((
        WatchTarget {
            path: root.join(relative),
            configured: configured.to_string(),
            role,
            mode: if metadata.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            },
            identity: watch_metadata_identity(&metadata)?,
        },
        digest,
    ))
}

fn resolve_watch_target(
    root: &Path,
    configured: &str,
    role: &'static str,
) -> Result<WatchTarget, WatchPathIssue> {
    inspect_watch_target(root, configured, role, false).map(|(target, _)| target)
}

#[derive(Clone, Debug, PartialEq)]
enum ExactSourceSnapshot {
    Regular {
        identity: (u64, u64),
        digest: [u8; 32],
    },
    Missing,
    Invalid(String),
}

fn snapshot_exact_source(root: &Path, target: &WatchTarget) -> ExactSourceSnapshot {
    match inspect_watch_target(root, &target.configured, target.role, true) {
        Ok((current, Some(digest))) if current.mode == RecursiveMode::NonRecursive => {
            ExactSourceSnapshot::Regular {
                identity: current.identity,
                digest,
            }
        }
        Ok(_) => {
            ExactSourceSnapshot::Invalid("exact source is no longer a regular file".to_string())
        }
        Err(WatchPathIssue::Missing) => ExactSourceSnapshot::Missing,
        Err(WatchPathIssue::Invalid(reason)) => ExactSourceSnapshot::Invalid(reason),
    }
}

fn report_snapshot_issue(
    target: &WatchTarget,
    snapshot: &ExactSourceSnapshot,
    format: OutputFormat,
) {
    if let ExactSourceSnapshot::Invalid(reason) = snapshot {
        // Precise native events may still work; do not falsely claim all monitoring was dropped.
        let message = format!(
            "cannot fingerprint configured source_dirs for coarse events: {} ({reason})",
            target.configured
        );
        if matches!(format, OutputFormat::Json) {
            eprintln!(
                "{}",
                serde_json::json!({ "warning": "invalid_watch_path", "path": target.configured, "role": target.role, "message": message })
            );
        } else {
            eprintln!("{} {message}", "⊘ Warning:".yellow());
        }
    }
}

fn resolve_watch_dirs(root: &Path, specs_dir: &str, source_dirs: &[String]) -> WatchDirs {
    let mut watched = Vec::new();
    let mut skipped = Vec::new();
    for (configured, role) in std::iter::once((specs_dir, "specs_dir")).chain(
        source_dirs
            .iter()
            .map(|path| (path.as_str(), "source_dirs")),
    ) {
        match resolve_watch_target(root, configured, role) {
            Ok(target) => {
                if !watched
                    .iter()
                    .any(|existing: &WatchTarget| existing.path == target.path)
                {
                    watched.push(target);
                }
            }
            Err(issue) => skipped.push(SkippedWatchPath {
                configured: configured.to_string(),
                role,
                issue,
            }),
        }
    }
    WatchDirs { watched, skipped }
}

fn report_skipped_watch_path(skipped: &SkippedWatchPath, format: OutputFormat) {
    if matches!(format, OutputFormat::Json) {
        eprintln!("{}", skipped.json());
    } else {
        eprintln!("{} {}", "⊘ Warning:".yellow(), skipped.message());
    }
}

type NativeDebouncer = Debouncer<notify::RecommendedWatcher, RecommendedCache>;

fn register_watch_target(
    debouncer: &mut NativeDebouncer,
    root: &Path,
    target: &WatchTarget,
    mode: RecursiveMode,
) -> Result<(), WatchPathIssue> {
    let before = resolve_watch_target(root, &target.configured, target.role)?;
    if &before != target {
        return Err(WatchPathIssue::Invalid(
            "path changed before watch registration".to_string(),
        ));
    }
    debouncer
        .watch(&target.path, mode)
        .map_err(|error| WatchPathIssue::Invalid(format!("watch registration failed: {error}")))?;
    let after = resolve_watch_target(root, &target.configured, target.role);
    if after.as_ref().ok() != Some(target) {
        let _ = debouncer.unwatch(&target.path);
        return Err(after.err().unwrap_or_else(|| {
            WatchPathIssue::Invalid("path changed during watch registration".to_string())
        }));
    }
    Ok(())
}

fn exact_source_parent(root: &Path, target: &WatchTarget) -> Result<WatchTarget, WatchPathIssue> {
    let relative = watch_relative_path(&target.configured)?;
    let parent = relative
        .parent()
        .filter(|path| !path.as_os_str().is_empty());
    let parent = resolve_watch_target(
        root,
        &parent.unwrap_or(Path::new(".")).to_string_lossy(),
        "source_dirs",
    )?;
    if parent.mode != RecursiveMode::Recursive {
        return Err(WatchPathIssue::Invalid(
            "exact source parent must remain a directory".to_string(),
        ));
    }
    Ok(parent)
}

fn selected_event_path<'a>(event: &'a DebouncedEvent, targets: &[WatchTarget]) -> Option<&'a Path> {
    event.paths.iter().find_map(|path| {
        // Backends supply absolute paths; never treat lexical traversal as a selected event.
        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return None;
        }
        targets
            .iter()
            .any(|target| {
                if target.mode == RecursiveMode::Recursive {
                    path.starts_with(&target.path)
                } else {
                    path == &target.path
                }
            })
            .then_some(path.as_path())
    })
}

fn select_watch_event(
    root: &Path,
    mut event: DebouncedEvent,
    targets: &[WatchTarget],
    snapshots: &mut HashMap<PathBuf, ExactSourceSnapshot>,
    format: OutputFormat,
) -> Option<DebouncedEvent> {
    let mut selected = Vec::new();
    for path in &event.paths {
        let single = DebouncedEvent {
            event: notify::Event::new(event.kind).add_path(path.clone()),
            time: event.time,
        };
        if selected_event_path(&single, targets).is_some() {
            selected.push(path.clone());
        }
    }
    for target in targets
        .iter()
        .filter(|target| target.mode == RecursiveMode::NonRecursive)
    {
        let precise = event.paths.contains(&target.path);
        let coarse = event
            .paths
            .iter()
            .any(|path| Some(path.as_path()) == target.path.parent());
        if !precise && !coarse {
            continue;
        }
        // A native backend may coalesce a changed file to its immediate directory.
        // Compare only declared exact sources through confined no-follow inspection.
        let current = snapshot_exact_source(root, target);
        let previous = snapshots.insert(target.path.clone(), current.clone());
        if previous.as_ref() != Some(&current) {
            report_snapshot_issue(target, &current, format);
            if !selected.contains(&target.path) {
                selected.push(target.path.clone());
            }
        }
    }
    if selected.is_empty() {
        None
    } else {
        event.paths = selected;
        Some(event)
    }
}

fn rearm_exact_sources(
    debouncer: &mut NativeDebouncer,
    root: &Path,
    targets: &mut [WatchTarget],
    event: &DebouncedEvent,
    format: OutputFormat,
) {
    for target in targets.iter_mut().filter(|target| {
        target.mode == RecursiveMode::NonRecursive && event.paths.contains(&target.path)
    }) {
        match resolve_watch_target(root, &target.configured, target.role) {
            Ok(current) if current.mode != RecursiveMode::NonRecursive => {
                let _ = debouncer.unwatch(&target.path);
                report_skipped_watch_path(
                    &SkippedWatchPath {
                        configured: target.configured.clone(),
                        role: target.role,
                        issue: WatchPathIssue::Invalid(
                            "exact source is no longer a regular file".to_string(),
                        ),
                    },
                    format,
                );
            }
            Ok(current)
                if current.identity != target.identity
                    || matches!(
                        event.kind,
                        EventKind::Create(_)
                            | EventKind::Remove(_)
                            | EventKind::Modify(notify::event::ModifyKind::Name(_))
                    ) =>
            {
                // A file inode watch can disappear on atomic save/unlink. Its nonrecursive
                // parent watch remains, filters to this exact name, and permits safe rearming.
                let _ = debouncer.unwatch(&target.path);
                match register_watch_target(debouncer, root, &current, RecursiveMode::NonRecursive)
                {
                    Ok(()) => *target = current,
                    Err(issue) => report_skipped_watch_path(
                        &SkippedWatchPath {
                            configured: target.configured.clone(),
                            role: target.role,
                            issue,
                        },
                        format,
                    ),
                }
            }
            Ok(_) => {}
            Err(WatchPathIssue::Missing) => {
                // MOVE_SELF may retain the old inode subscription under its former name.
                // Keep only the safe parent lifecycle observation until the file reappears.
                let _ = debouncer.unwatch(&target.path);
            }
            Err(issue) => {
                let _ = debouncer.unwatch(&target.path);
                report_skipped_watch_path(
                    &SkippedWatchPath {
                        configured: target.configured.clone(),
                        role: target.role,
                        issue,
                    },
                    format,
                );
            }
        }
    }
}

/// Run the check command in watch mode, re-running on file changes.
/// Uses the hash cache to skip unchanged specs on subsequent runs.
pub fn run_watch(root: &Path, strict: bool, require_coverage: Option<usize>, format: OutputFormat) {
    let config = load_config(root);

    // Resolve configured directories and exact source files. A path that
    // cannot be watched is reported but does not stop the other selections:
    // watch is a long-running dev loop, and a typo in one of several paths
    // should not be fatal. It must never be invisible, though — silently
    // dropping a path makes the banner lie about what is being monitored.
    let resolved = resolve_watch_dirs(root, &config.specs_dir, &config.source_dirs);
    let watch_dirs = resolved.watched;

    // Report dropped paths on both human and machine-readable channels.
    // Even though watch is primarily interactive, CI and editor integrations
    // read its output and need a parseable signal that a configured path was
    // ignored.
    for skipped in &resolved.skipped {
        report_skipped_watch_path(skipped, format);
    }

    if watch_dirs.is_empty() {
        eprintln!(
            "{} No directories to watch and no exact source files (specs_dir={}, source_dirs={:?})",
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

    let mut registered = Vec::new();
    for target in watch_dirs {
        // Initial validation may have taken time. Do not register a replaced or linked path.
        match register_watch_target(&mut debouncer, root, &target, target.mode) {
            Ok(()) => registered.push(target),
            Err(issue) => report_skipped_watch_path(
                &SkippedWatchPath {
                    configured: target.configured,
                    role: target.role,
                    issue,
                },
                format,
            ),
        }
    }
    // Exact-name lifecycle observations keep inode-based backends alive across atomic saves.
    // These parents are never recursive and their unrelated sibling events are filtered out.
    let mut parents = HashSet::new();
    let mut durable = Vec::new();
    for target in &registered {
        let result = if target.mode == RecursiveMode::NonRecursive {
            exact_source_parent(root, target).and_then(|parent| {
                if parents.contains(&parent.path)
                    || registered.iter().any(|other| {
                        other.mode == RecursiveMode::Recursive
                            && parent.path.starts_with(&other.path)
                    })
                {
                    return Ok(());
                }
                register_watch_target(&mut debouncer, root, &parent, RecursiveMode::NonRecursive)?;
                parents.insert(parent.path);
                Ok(())
            })
        } else {
            Ok(())
        };
        match result {
            Ok(()) => durable.push(target.clone()),
            Err(issue) => {
                let _ = debouncer.unwatch(&target.path);
                report_skipped_watch_path(
                    &SkippedWatchPath {
                        configured: target.configured.clone(),
                        role: target.role,
                        issue: WatchPathIssue::Invalid(format!(
                            "exact-source lifecycle monitoring failed: {issue:?}"
                        )),
                    },
                    format,
                );
            }
        }
    }
    let mut registered = durable;
    if registered.is_empty() {
        eprintln!("{} No paths could be watched", "Error:".red());
        std::process::exit(1);
    }

    let mut snapshots = HashMap::new();
    for target in registered
        .iter()
        .filter(|target| target.mode == RecursiveMode::NonRecursive)
    {
        let snapshot = snapshot_exact_source(root, target);
        report_snapshot_issue(target, &snapshot, format);
        snapshots.insert(target.path.clone(), snapshot);
    }

    println!(
        "\n{} Watching for changes in: {}",
        ">>>".cyan(),
        registered
            .iter()
            .map(|target| target
                .path
                .strip_prefix(root)
                .unwrap_or(&target.path)
                .display()
                .to_string())
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

        let Some(event) = select_watch_event(root, event, &registered, &mut snapshots, format)
        else {
            continue;
        };
        let changed_path = event.paths[0].clone();
        rearm_exact_sources(&mut debouncer, root, &mut registered, &event, format);

        // Extra debounce: don't re-run if we just ran
        if last_run.elapsed() < Duration::from_millis(300) {
            continue;
        }

        let changed_file = changed_path
            .strip_prefix(root)
            .ok()
            .map(|path| path.display().to_string());

        // Coalesce checks, but rearm every selected replacement before draining its event.
        while let Ok(queued) = rx.try_recv() {
            if is_relevant_event(&queued)
                && let Some(queued) =
                    select_watch_event(root, queued, &registered, &mut snapshots, format)
            {
                rearm_exact_sources(&mut debouncer, root, &mut registered, &queued, format);
            }
        }

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
        assert!(resolved.skipped.contains(&SkippedWatchPath {
            configured: "specs".to_string(),
            role: "specs_dir",
            issue: WatchPathIssue::Missing
        }));
        assert!(resolved.skipped.contains(&SkippedWatchPath {
            configured: "src".to_string(),
            role: "source_dirs",
            issue: WatchPathIssue::Missing
        }));
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
        assert!(resolved.skipped.contains(&SkippedWatchPath {
            configured: "missing-specs".to_string(),
            role: "specs_dir",
            issue: WatchPathIssue::Missing
        }));
        assert!(resolved.skipped.contains(&SkippedWatchPath {
            configured: "also-missing".to_string(),
            role: "source_dirs",
            issue: WatchPathIssue::Missing
        }));
    }

    #[test]
    fn test_watch_exact_sources_are_nonrecursive_and_directories_remain_recursive() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("specs")).unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("lib.rs"), "pub fn library() {}\n").unwrap();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}\n").unwrap();
        let resolved = resolve_watch_dirs(
            tmp.path(),
            "specs",
            &[
                "lib.rs".into(),
                "main.rs".into(),
                "src".into(),
                "./lib.rs".into(),
            ],
        );
        assert!(resolved.skipped.is_empty());
        assert_eq!(
            resolved.watched.len(),
            4,
            "exact files must not be skipped or duplicated"
        );
        for target in resolved.watched {
            assert_eq!(
                target.mode,
                if target.configured.ends_with(".rs") {
                    RecursiveMode::NonRecursive
                } else {
                    RecursiveMode::Recursive
                }
            );
            assert_ne!(target.path, tmp.path(), "exact sources never widen to root");
        }
    }

    #[test]
    fn test_watch_rejects_escape_paths_even_when_target_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("lib.rs"), "pub fn library() {}\n").unwrap();
        for configured in [
            "../lib.rs",
            "src/../lib.rs",
            "/lib.rs",
            "C:\\lib.rs",
            "\\\\server\\lib.rs",
            "",
            tmp.path().join("lib.rs").to_str().unwrap(),
        ] {
            assert!(
                matches!(
                    resolve_watch_target(tmp.path(), configured, "source_dirs"),
                    Err(WatchPathIssue::Invalid(_))
                ),
                "{configured} must not be watched"
            );
        }
    }

    #[test]
    fn test_watch_distinguishes_missing_and_invalid_text_and_json() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("lib.rs"), "pub fn library() {}\n").unwrap();
        let resolved = resolve_watch_dirs(tmp.path(), "lib.rs", &["missing.rs".into()]);
        assert!(
            resolved.watched.is_empty(),
            "specs_dir must stay a directory"
        );
        let invalid = &resolved.skipped[0];
        assert!(matches!(invalid.issue, WatchPathIssue::Invalid(_)));
        assert_eq!(invalid.json()["warning"], "invalid_watch_path");
        assert_eq!(invalid.json()["path"], "lib.rs");
        assert_eq!(invalid.json()["role"], "specs_dir");
        assert!(invalid.message().contains("is invalid"));
        assert!(!invalid.message().contains("does not exist"));
        let missing = &resolved.skipped[1];
        assert_eq!(missing.issue, WatchPathIssue::Missing);
        assert_eq!(missing.json()["warning"], "nonexistent_watch_directory");
        assert_eq!(missing.json()["path"], "missing.rs");
        assert_eq!(missing.json()["role"], "source_dirs");
        assert!(missing.message().contains("does not exist"));
    }

    #[cfg(unix)]
    #[test]
    fn test_watch_rejects_symlink_files_directories_and_parent_components() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("real")).unwrap();
        std::fs::write(tmp.path().join("real/lib.rs"), "pub fn library() {}\n").unwrap();
        std::fs::write(outside.path().join("lib.rs"), "pub fn outside() {}\n").unwrap();
        symlink("real/lib.rs", tmp.path().join("inside.rs")).unwrap();
        symlink(outside.path().join("lib.rs"), tmp.path().join("outside.rs")).unwrap();
        symlink("missing.rs", tmp.path().join("broken.rs")).unwrap();
        symlink("real", tmp.path().join("linked")).unwrap();
        symlink(outside.path(), tmp.path().join("outside-dir")).unwrap();
        for configured in [
            "inside.rs",
            "outside.rs",
            "broken.rs",
            "linked",
            "linked/lib.rs",
            "outside-dir/lib.rs",
        ] {
            assert!(
                matches!(
                    resolve_watch_target(tmp.path(), configured, "source_dirs"),
                    Err(WatchPathIssue::Invalid(_))
                ),
                "link {configured} must not be accepted"
            );
        }
        assert!(matches!(
            resolve_watch_target(tmp.path(), "linked", "specs_dir"),
            Err(WatchPathIssue::Invalid(_))
        ));
        assert_eq!(
            resolve_watch_target(tmp.path(), "real/lib.rs", "source_dirs")
                .unwrap()
                .mode,
            RecursiveMode::NonRecursive
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_watch_rejects_nonregular_sources() {
        let tmp = TempDir::new().unwrap();
        let _socket =
            std::os::unix::net::UnixListener::bind(tmp.path().join("source.sock")).unwrap();
        assert!(matches!(
            resolve_watch_target(tmp.path(), "source.sock", "source_dirs"),
            Err(WatchPathIssue::Invalid(_))
        ));
    }

    #[test]
    fn test_watch_revalidation_detects_a_replaced_exact_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("lib.rs"), "pub fn library() {}\n").unwrap();
        let before = resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap();
        std::fs::write(tmp.path().join("replacement.rs"), "pub fn library() {}\n").unwrap();
        std::fs::rename(tmp.path().join("replacement.rs"), tmp.path().join("lib.rs")).unwrap();
        let after = resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap();
        assert_ne!(before.identity, after.identity);
        assert_ne!(before, after, "registration must reject a changed target");
    }

    #[test]
    fn test_watch_parent_events_filter_siblings_and_select_matching_rename_path() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("specs")).unwrap();
        std::fs::write(tmp.path().join("lib.rs"), "pub fn library() {}\n").unwrap();
        let targets = resolve_watch_dirs(tmp.path(), "specs", &["lib.rs".into()]).watched;
        let parent = exact_source_parent(tmp.path(), &targets[1]).unwrap();
        assert_eq!(
            parent.path,
            tmp.path(),
            "only the immediate parent is observed"
        );
        // The parent is not added to logical selection: its registration is NonRecursive.
        let sibling = tmp.path().join("sibling.rs");
        let mut event = make_event_with_path(EventKind::Modify(ModifyKind::Any), sibling);
        assert_eq!(selected_event_path(&event, &targets), None);
        event.paths.push(tmp.path().join("lib.rs"));
        assert_eq!(
            selected_event_path(&event, &targets),
            Some(tmp.path().join("lib.rs").as_path())
        );
        event.paths = vec![tmp.path().join("specs/nested/probe.spec.md")];
        assert!(
            selected_event_path(&event, &targets).is_some(),
            "directory recursion stays intact"
        );
        event.paths = vec![tmp.path().join("nested/../lib.rs")];
        assert_eq!(selected_event_path(&event, &targets), None);
    }

    #[test]
    fn test_watch_coarse_parent_fingerprints_only_selected_content_and_refreshes_precise_events() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("lib.rs");
        std::fs::write(&source, "fn before() {}\n").unwrap();
        let original_time = std::fs::metadata(&source).unwrap().modified().unwrap();
        let target = resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap();
        let targets = vec![target];
        let mut snapshots = HashMap::from([(
            source.clone(),
            snapshot_exact_source(tmp.path(), &targets[0]),
        )]);
        let parent_event =
            || make_event_with_path(EventKind::Modify(ModifyKind::Any), tmp.path().to_path_buf());
        std::fs::write(tmp.path().join("sibling.rs"), "pub fn sibling() {}\n").unwrap();
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_none()
        );
        std::fs::write(&source, "fn after_() {}\n").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&source)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(original_time))
            .unwrap();
        let selected = select_watch_event(
            tmp.path(),
            parent_event(),
            &targets,
            &mut snapshots,
            OutputFormat::Text,
        )
        .unwrap();
        assert_eq!(
            selected.paths,
            vec![source.clone()],
            "same-size/mtime source edits must still be noticed"
        );
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_none()
        );
        std::fs::write(&source, "fn precise() {}\n").unwrap();
        let precise = make_event_with_path(EventKind::Modify(ModifyKind::Any), source.clone());
        assert!(
            select_watch_event(
                tmp.path(),
                precise,
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_some()
        );
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_none(),
            "later sibling/parent event cannot re-report a precise edit"
        );
    }

    #[test]
    fn test_watch_coarse_parent_detects_atomic_replacement_removal_and_recreation() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("lib.rs");
        std::fs::write(&source, "fn before() {}\n").unwrap();
        let targets = vec![resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap()];
        let mut snapshots = HashMap::from([(
            source.clone(),
            snapshot_exact_source(tmp.path(), &targets[0]),
        )]);
        let parent_event =
            || make_event_with_path(EventKind::Modify(ModifyKind::Any), tmp.path().to_path_buf());
        std::fs::write(tmp.path().join("temporary.rs"), "fn before() {}\n").unwrap();
        std::fs::rename(tmp.path().join("temporary.rs"), &source).unwrap();
        assert_eq!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .unwrap()
            .paths,
            vec![source.clone()]
        );
        std::fs::remove_file(&source).unwrap();
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_some()
        );
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_none(),
            "missing selected file cannot turn every sibling event into a check"
        );
        std::fs::write(&source, "fn recreated() {}\n").unwrap();
        assert!(
            select_watch_event(
                tmp.path(),
                parent_event(),
                &targets,
                &mut snapshots,
                OutputFormat::Text
            )
            .is_some()
        );
    }

    #[test]
    fn test_watch_exact_fingerprint_bound_and_directory_replacement_are_explicit() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("lib.rs");
        std::fs::write(&source, "fn before() {}\n").unwrap();
        let target = resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&source)
            .unwrap()
            .set_len(8 * 1024 * 1024 + 1)
            .unwrap();
        assert!(
            matches!(snapshot_exact_source(tmp.path(), &target), ExactSourceSnapshot::Invalid(reason) if reason.contains("8 MiB"))
        );
        std::fs::remove_file(&source).unwrap();
        std::fs::create_dir(&source).unwrap();
        assert!(
            matches!(snapshot_exact_source(tmp.path(), &target), ExactSourceSnapshot::Invalid(reason) if reason.contains("regular file"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_watch_exact_fingerprint_never_follows_replacement_symlink() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let source = tmp.path().join("lib.rs");
        std::fs::write(&source, "fn before() {}\n").unwrap();
        let target = resolve_watch_target(tmp.path(), "lib.rs", "source_dirs").unwrap();
        std::fs::write(outside.path().join("outside.rs"), "must not be read\n").unwrap();
        std::fs::remove_file(&source).unwrap();
        symlink(outside.path().join("outside.rs"), &source).unwrap();
        assert!(
            matches!(snapshot_exact_source(tmp.path(), &target), ExactSourceSnapshot::Invalid(reason) if reason.contains("symlink"))
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
