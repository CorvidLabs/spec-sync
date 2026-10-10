//! Git-based export diffs and staleness reporting.

use crate::helpers::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── Diff Command Tests ─────────────────────────────────────────────────

#[test]
fn diff_shows_changes_since_base_ref() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    // Initial commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Add a new export after the commit
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Stage but don't commit — diff should detect changes
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    // Run diff with --json
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "diff command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    let changes = json["changes"].as_array().unwrap();
    assert!(!changes.is_empty(), "Expected at least one changed spec");
    assert!(
        changes[0]["new_exports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str() == Some("logout")),
        "Expected 'logout' in new_exports"
    );
}

#[test]
fn diff_fails_loud_on_unreadable_source_file() {
    // Regression: a changed source file whose exports can't be read (non-UTF-8)
    // silently contributed zero exports, so real new API was dropped and diff
    // reported "no drift" with exit 0. It must now surface the file and fail loud.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "t@t.com"],
        vec!["config", "user.name", "T"],
    ] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
    }
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/api.ts"), "export function apiFn() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        valid_spec("m", &["src/api.ts"]),
    )
    .unwrap();
    for args in [vec!["add", "."], vec!["commit", "-m", "init"]] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
    }

    // Rewrite api.ts with a genuinely-new export plus an invalid UTF-8 byte, then stage.
    let mut bad = b"export function apiFn() {}\nexport function brandNew() {}\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("src/api.ts"), bad).unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    specsync()
        .args(["diff", "--base", "HEAD", "--root", root.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("inconclusive"));
}

#[test]
fn diff_no_changes_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    // Commit everything — no changes after commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Run diff — nothing changed since HEAD
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        json["changes"].as_array().unwrap().is_empty(),
        "Expected no changes"
    );
}

#[test]
fn diff_bad_base_ref_fails_loud() {
    // Regression: `git diff` exits non-zero on a bad base ref with empty stdout.
    // The command must NOT report "no files changed" and exit 0 (that would silently
    // mask a failed comparison and green-light CI); it must fail loud (exit != 0).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let output = specsync()
        .args([
            "diff",
            "--base",
            "no-such-ref-xyz",
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "diff against a bogus base ref must fail loud, not report 'no changes' and exit 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no-such-ref-xyz"),
        "error should name the bad base ref; stderr was: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("No files changed"),
        "must not print the no-drift message on a failed diff; stdout was: {stdout}"
    );
}

#[test]
fn diff_detects_removed_exports() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec documents both login and logout
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = r#"---
module: auth
version: 1
status: active
files:
  - src/auth/service.ts
db_tables: []
depends_on: []
---

# Auth

## Purpose

Auth module.

## Public API

| Function | Description |
|----------|-------------|
| `login` | Log in |
| `logout` | Log out |

## Invariants

1. Always valid.

## Behavioral Examples

### Scenario: Basic

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

None

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    // Commit with both exports
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Remove logout export
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    let changes = json["changes"].as_array().unwrap();
    assert!(!changes.is_empty());
    assert!(
        changes[0]["removed_exports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str() == Some("logout")),
        "Expected 'logout' in removed_exports"
    );
}

#[test]
fn diff_human_readable_output() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Add new export
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function signup() {}\n",
    )
    .unwrap();

    // Run without --json for human-readable output
    specsync()
        .args(["diff", "--base", "HEAD", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("signup"));
}

#[test]
fn diff_detects_spec_file_only_changes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Modify ONLY the spec file — no source file changes
    let updated_spec = valid_spec("auth", &["src/auth/service.ts"]).replace(
        "This module does something.",
        "Updated auth module description.",
    );
    fs::write(root.join("specs/auth/auth.spec.md"), &updated_spec).unwrap();

    // diff should detect the spec was modified even though no source files changed
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("auth"),
        "Expected diff to report the auth spec when only the spec file changed. Got:\n{stdout}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let changes = json["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "Expected exactly 1 change entry");
    assert_eq!(changes[0]["spec_modified"], true);
    assert!(
        changes[0]["changed_files"].as_array().unwrap().is_empty(),
        "No source files should have changed"
    );
}

// ─── specsync stale ──────────────────────────────────────────────────────

#[test]
fn stale_outside_git_repo_fails_with_message() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    // No `git init` — staleness detection requires git history.

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));
}

#[test]
fn stale_outside_git_repo_json_reports_error() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("not a git repository"))
        .stdout(predicate::str::contains("\"stale_specs\""));
}

#[test]
fn stale_in_fresh_repo_reports_all_up_to_date() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Initialize a repo and commit everything so source and spec share history.
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&root)
            .assert()
            .success();
    };
    git(&["init"]);
    git(&["config", "user.email", "test@test.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    git(&["add", "-A"]);
    git(&["commit", "-m", "initial"]);

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));
}
