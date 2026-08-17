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

    let output = specsync()
        .args(["watch", "--root"])
        .arg(root)
        .timeout(Duration::from_secs(5))
        .output()
        .expect("failed to run specsync watch");
    let stdout = String::from_utf8_lossy(&output.stdout);

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

    let output = specsync()
        .args(["watch", "--root"])
        .arg(root)
        .timeout(Duration::from_secs(5))
        .output()
        .expect("failed to run specsync watch");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("All checks passed!"),
        "a real, passing spec set must still report a pass; got: {stdout}"
    );
    assert!(
        !stdout.contains("No specs were examined"),
        "a real spec set was examined; got: {stdout}"
    );
}
