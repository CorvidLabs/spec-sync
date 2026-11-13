//! Regression tests for `specsync watch`.

use crate::helpers::*;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

// ─── #577: watch silently drops nonexistent directories ────────────────────

#[test]
fn watch_warns_on_nonexistent_directory() {
    // Regression for #577: a configured directory that does not exist must be
    // reported, not silently dropped. This test fails on origin/main.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"missing-specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();

    let output = specsync()
        .args(["watch", "--root"])
        .arg(root)
        .timeout(Duration::from_secs(5))
        .output()
        .expect("failed to run specsync watch");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("missing-specs"),
        "stderr should report the nonexistent directory; got: {stderr}"
    );
    assert!(
        stderr.contains("will not be watched"),
        "stderr should explain the directory is skipped; got: {stderr}"
    );
}

#[test]
fn watch_warns_on_nonexistent_directory_json() {
    // The same disclosure must be machine-readable (#577).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"missing-specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();

    let output = specsync()
        .args(["watch", "--format", "json", "--root"])
        .arg(root)
        .timeout(Duration::from_secs(5))
        .output()
        .expect("failed to run specsync watch");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("nonexistent_watch_directory"),
        "stderr should contain a structured JSON warning; got: {stderr}"
    );
    assert!(
        stderr.contains("missing-specs"),
        "JSON warning should name the missing directory; got: {stderr}"
    );
}

#[test]
fn watch_errors_when_all_directories_missing() {
    // Vacuity control: if every configured directory is missing, watch must
    // still fail closed. Passing this on both binaries prevents a broken fix
    // that silently refuses every directory from looking correct.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"missing-specs\"\nsource_dirs = [\"missing-src\"]\n",
    )
    .unwrap();

    let output = specsync()
        .args(["watch", "--root"])
        .arg(root)
        .timeout(Duration::from_secs(5))
        .output()
        .expect("failed to run specsync watch");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    assert_eq!(
        output.status.code(),
        Some(1),
        "watch should exit 1 when no directories exist; got status {:?}. output: {combined}",
        output.status
    );
    assert!(
        combined.contains("No directories to watch"),
        "output should report no directories to watch; got: {combined}"
    );
}

/// Kills the child on every exit path — the assert/panic arms included — so a failing
/// watch test never leaves a `specsync watch` process behind (#602's leak class, #1055).
struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Whether `collected` holds a finished `watch` run: one of its verdict `markers`.
fn run_finished(collected: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| collected.contains(marker))
}

/// #1040: run `specsync watch` until one of `markers` appears in its stdout, then kill it.
/// The verdict is an event, not a duration — a fixed window flakes under load (#707's class).
/// The generous upper bound exists only to bound a hung child.
fn watch_until(root: &std::path::Path, markers: &[&str]) -> String {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    let mut child = KillOnDrop(
        specsync_std()
            .args(["watch", "--root"])
            .arg(root)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn specsync watch"),
    );
    let stdout = child.0.stdout.take().expect("stdout is piped");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { return };
            if tx.send(line).is_err() {
                return;
            }
        }
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    let mut collected = String::new();
    loop {
        if run_finished(&collected, markers) {
            break;
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                collected.push_str(&line);
                collected.push('\n');
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "watch printed none of {markers:?} within the generous bound; got: {collected}"
                );
                if child.0.try_wait().unwrap().is_some() {
                    panic!("watch exited before printing any of {markers:?}; got: {collected}");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("watch closed stdout before printing any of {markers:?}; got: {collected}");
            }
        }
    }
    drop(child);
    collected
}

#[test]
fn watch_does_not_claim_a_pass_over_zero_specs() {
    // Second half of #577: disclosing the dropped directory is not enough if
    // the run that examined nothing still reports `All checks passed!`. The
    // check child exits 0 with no specs, and reading that as success is the
    // same false all-clear one level down.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"missing-specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();

    let stdout = watch_until(root, &["No specs were examined", "All checks passed!"]);

    assert!(
        !stdout.contains("All checks passed!"),
        "watch must not claim a pass over a run that examined no specs; got: {stdout}"
    );
    assert!(
        stdout.contains("No specs were examined"),
        "watch should say nothing was checked; got: {stdout}"
    );
}

#[test]
fn watch_still_reports_a_pass_over_a_real_spec_set() {
    // Vacuity control for the assertion above: a tree whose specs exist and
    // pass must still print `All checks passed!`, so "never claim a pass"
    // cannot be satisfied by never passing.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/lib.spec.md"),
        concat!(
            "---\n",
            "module: lib\n",
            "version: 1\n",
            "status: stable\n",
            "files:\n",
            "  - src/lib.rs\n",
            "db_tables: []\n",
            "depends_on: []\n",
            "---\n",
            "\n# lib\n",
            "\n## Purpose\n",
            "\nControl fixture for the watch pass claim.\n",
            "\n## Public API\n",
            "\n| Export | Description |\n",
            "|--------|-------------|\n",
            "| `a` | Does a. |\n",
            "\n## Invariants\n",
            "\n- Pure helper; no I/O.\n",
            "\n## Behavioral Examples\n",
            "\n- `a()` does a.\n",
            "\n## Error Cases\n",
            "\nNone.\n",
            "\n## Dependencies\n",
            "\nNone.\n",
            "\n## Change Log\n",
            "\n| Change | Date | Version |\n",
            "|--------|------|---------|\n",
            "| Created | 2026-08-17 | 1 |\n",
        ),
    )
    .unwrap();

    let stdout = watch_until(root, &["All checks passed!", "No specs were examined"]);

    assert!(
        stdout.contains("All checks passed!"),
        "a real, passing spec set must still report a pass; got: {stdout}"
    );
    assert!(
        !stdout.contains("No specs were examined"),
        "a real spec set was examined; got: {stdout}"
    );
}

/// #1055: a run is finished only once its verdict is followed by the line `watch` prints after
/// every run, `Watching for changes in: ...` after the first and `Watching for changes...` after
/// each later one, so a later line of the same run, such as a contradicting warning, is in the
/// output the tests judge.
#[test]
fn a_watch_run_is_finished_only_at_the_line_after_its_verdict() {
    let markers = ["All checks passed!", "No specs were examined"];
    assert!(!run_finished("", &markers));
    assert!(
        !run_finished("All checks passed!\n", &markers),
        "the verdict alone does not end the run"
    );
    assert!(!run_finished(
        "All checks passed!\nwarning: a later line of the same run\n",
        &markers
    ));
    assert!(run_finished(
        "All checks passed!\nwarning: a later line\n\n>>> Watching for changes in: specs, src\n",
        &markers
    ));
    assert!(run_finished(
        "No specs were examined\n\n>>> Watching for changes... (Ctrl+C to stop)\n",
        &markers
    ));
    assert!(
        !run_finished(">>> Watching for changes in: specs\n", &markers),
        "the line without a verdict before it is not a finished run"
    );
}
