//! MERGE-1..MERGE-4 (#862). Two finalized changes to one module must merge
//! cleanly under `git merge-tree`, and a requirements file that kept the same
//! requirement ID twice must be refused by `check` and `change audit`.

use crate::helpers::{setup_minimal_project, specsync};
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("git {args:?} failed to start: {error}"))
}

fn git_ok(root: &Path, args: &[&str]) {
    let output = git(root, args);
    assert!(
        output.status.success(),
        "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn spec_text(version: u64, extra_file: Option<&str>, changelog_row: Option<&str>) -> String {
    let mut files = "  - src/cli.rs\n".to_string();
    if let Some(extra) = extra_file {
        files.push_str(&format!("  - {extra}\n"));
    }
    let mut rows = "| 2026-01-01 | base: initial |\n".to_string();
    if let Some(row) = changelog_row {
        rows.push_str(row);
        if !row.ends_with('\n') {
            rows.push('\n');
        }
    }
    format!(
        r#"---
module: cli
version: {version}
status: stable
files:
{files}db_tables: []
depends_on: []
---

# CLI

## Purpose

The command line.

## Public API

| Export | Description |
|--------|-------------|

## Invariants

1. The command line stays the command line.

## Behavioral Examples

### Scenario: Run

- **Given** a command
- **When** it runs
- **Then** it finishes

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

None.

## Change Log

| Date | Change |
|------|--------|
{rows}"#
    )
}

fn requirements_text(extra: Option<(&str, &str)>) -> String {
    let mut body = r#"---
spec: cli.spec.md
---

# Requirements

### REQ-cli-001

The system SHALL start.

Acceptance Criteria
- It starts.
"#
    .to_string();
    if let Some((id, sentence)) = extra {
        body.push_str(&format!(
            "\n### {id}\n\nThe system SHALL {sentence}.\n\nAcceptance Criteria\n- It holds.\n"
        ));
    }
    body
}

fn commit_all(root: &Path, message: &str) {
    git_ok(root, &["add", "-A"]);
    git_ok(
        root,
        &[
            "-c",
            "user.email=merge-fixture@example.com",
            "-c",
            "user.name=Merge Fixture",
            "commit",
            "-m",
            message,
        ],
    );
}

fn write_branch(
    root: &Path,
    branch: &str,
    version: u64,
    file: &str,
    row: &str,
    req: &str,
    title: &str,
) {
    git_ok(root, &["checkout", "-q", "-B", branch, "main"]);
    fs::write(
        root.join("specs/cli/cli.spec.md"),
        spec_text(version, Some(file), Some(row)),
    )
    .unwrap();
    fs::write(
        root.join("specs/cli/requirements.md"),
        requirements_text(Some((req, title))),
    )
    .unwrap();
    commit_all(root, branch);
}

fn merged_file(root: &Path, tree: &str, path: &str) -> String {
    let output = git(root, &["show", &format!("{tree}:{path}")]);
    assert!(
        output.status.success(),
        "git show {tree}:{path} failed\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn merge_tree(root: &Path, into: &str, from: &str) -> (bool, String, String) {
    let output = git(root, &["merge-tree", "--write-tree", into, from]);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stdout, stderr)
}

fn assert_clean_contract(root: &Path, stdout: &str) {
    let tree = stdout.lines().next().unwrap_or("").trim().to_string();
    assert!(!tree.is_empty(), "merge-tree returned no tree\n{stdout}");
    let spec = merged_file(root, &tree, "specs/cli/cli.spec.md");
    let requirements = merged_file(root, &tree, "specs/cli/requirements.md");
    let combined = format!("{stdout}\n{spec}\n{requirements}");
    assert!(
        !combined.contains("<<<<<<<"),
        "merged contract still has conflict markers:\n{combined}"
    );
    assert!(
        spec.contains("version: 7\n"),
        "version should count both bumps (5 + 1 + 1):\n{spec}"
    );
    assert!(spec.contains("src/a.rs"), "missing branch A file\n{spec}");
    assert!(spec.contains("src/b.rs"), "missing branch B file\n{spec}");
    assert!(
        spec.contains("change-a: add a"),
        "missing branch A change-log row\n{spec}"
    );
    assert!(
        spec.contains("change-b: add b"),
        "missing branch B change-log row\n{spec}"
    );
    assert!(
        requirements.contains("### REQ-cli-002"),
        "missing REQ-cli-002\n{requirements}"
    );
    assert!(
        requirements.contains("### REQ-cli-003"),
        "missing REQ-cli-003\n{requirements}"
    );
    assert_eq!(
        requirements.matches("### REQ-cli-001").count(),
        1,
        "base requirement was duplicated\n{requirements}"
    );
}

/// MERGE-1 and MERGE-2. The two branches are the shape `change check --commit`
/// writes: each bumps `version:` once, appends one change-log row, adds one
/// `files:` entry, and appends one requirement.
#[test]
fn parallel_finalized_module_changes_merge_cleanly_in_either_order() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();
    git_ok(root, &["init", "-q", "-b", "main"]);
    fs::create_dir_all(root.join("specs/cli")).unwrap();
    fs::write(root.join("specs/cli/cli.spec.md"), spec_text(5, None, None)).unwrap();
    fs::write(
        root.join("specs/cli/requirements.md"),
        requirements_text(None),
    )
    .unwrap();
    specsync()
        .args(["merge", "--install"])
        .arg("--root")
        .arg(root)
        .assert()
        .success();
    commit_all(root, "base");

    write_branch(
        root,
        "change-a",
        6,
        "src/a.rs",
        "| 2026-11-05 | change-a: add a |",
        "REQ-cli-002",
        "run A",
    );
    write_branch(
        root,
        "change-b",
        6,
        "src/b.rs",
        "| 2026-11-05 | change-b: add b |",
        "REQ-cli-003",
        "run B",
    );

    let (ok_ab, out_ab, err_ab) = merge_tree(root, "change-a", "change-b");
    assert!(
        ok_ab,
        "merge-tree change-a change-b conflicted\nstdout:\n{out_ab}\nstderr:\n{err_ab}"
    );
    assert_clean_contract(root, &out_ab);

    let (ok_ba, out_ba, err_ba) = merge_tree(root, "change-b", "change-a");
    assert!(
        ok_ba,
        "merge-tree change-b change-a conflicted\nstdout:\n{out_ba}\nstderr:\n{err_ba}"
    );
    assert_clean_contract(root, &out_ba);

    let tree_ab = out_ab.lines().next().unwrap().trim();
    let tree_ba = out_ba.lines().next().unwrap().trim();
    let spec_ab = merged_file(root, tree_ab, "specs/cli/cli.spec.md");
    let spec_ba = merged_file(root, tree_ba, "specs/cli/cli.spec.md");
    let req_ab = merged_file(root, tree_ab, "specs/cli/requirements.md");
    let req_ba = merged_file(root, tree_ba, "specs/cli/requirements.md");
    assert_eq!(
        spec_ab, spec_ba,
        "spec merge result depends on which branch is ours"
    );
    assert_eq!(
        req_ab, req_ba,
        "requirements merge result depends on which branch is ours"
    );
}

/// MERGE-3. A requirements file that contains one ID twice is refused, and the
/// refusal names that ID.
#[test]
fn duplicate_requirement_id_is_refused_by_check_and_change_audit() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    git_ok(&root, &["init", "-q", "-b", "main"]);
    fs::write(
        root.join("specs/auth/requirements.md"),
        r#"---
spec: auth.spec.md
---

# Requirements

### REQ-auth-002

The system SHALL keep one session.

Acceptance Criteria
- One session.

### REQ-auth-002

The system SHALL keep a different session.

Acceptance Criteria
- A different session.
"#,
    )
    .unwrap();

    let check = specsync()
        .args(["check", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let check_text = format!(
        "{}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(
        !check.status.success(),
        "check accepted a duplicated requirement ID\n{check_text}"
    );
    assert!(
        check_text.contains("REQ-auth-002"),
        "check did not name the duplicate ID\n{check_text}"
    );

    let audit = specsync()
        .args(["change", "audit", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let audit_text = format!(
        "{}{}",
        String::from_utf8_lossy(&audit.stdout),
        String::from_utf8_lossy(&audit.stderr)
    );
    assert!(
        !audit.status.success(),
        "change audit accepted a duplicated requirement ID\n{audit_text}"
    );
    assert!(
        audit_text.contains("REQ-auth-002"),
        "change audit did not name the duplicate ID\n{audit_text}"
    );
}

/// REQ-cmd-merge-003. `merge --install` refuses a symlinked `.gitattributes`:
/// it fails, leaves the link target untouched, and records no driver.
#[cfg(unix)]
#[test]
fn merge_install_refuses_a_symlinked_gitattributes() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();
    git_ok(root, &["init", "-q", "-b", "main"]);
    fs::create_dir_all(root.join("specs")).unwrap();
    let target = root.join("elsewhere.attributes");
    fs::write(&target, "*.txt text\n").unwrap();
    std::os::unix::fs::symlink(&target, root.join(".gitattributes")).unwrap();

    let output = specsync()
        .args(["merge", "--install"])
        .arg("--root")
        .arg(root)
        .output()
        .unwrap();
    assert!(!output.status.success(), "install accepted a symlink");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to update symlinked .gitattributes"),
        "stderr did not name the refusal:\n{stderr}"
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "*.txt text\n");
    let driver = git(root, &["config", "--local", "--get", "merge.specsync.driver"]);
    assert!(
        !driver.status.success(),
        "a driver was recorded despite the refusal: {}",
        String::from_utf8_lossy(&driver.stdout)
    );
}
