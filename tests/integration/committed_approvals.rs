// #428, REQ-change-166 and REQ-cmd-change-041: through the binary, `change finalize`,
// `change ship` and `change ship --dry-run` refuse an approvals ledger the working tree has
// changed since `HEAD`, with one message, and `change ship-status` lists that refusal as a
// blocker and does not call the change ready to finalize.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run(root: &Path, args: &[&str]) -> Output {
    specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn run_ok(root: &Path, args: &[&str]) -> String {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "specsync {} failed:\n{}{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Everything a command printed, stdout then stderr.
fn printed(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

const ID: &str = "document-the-greeter";

/// A no-spec documentation change, approved, checked with `--commit` and reviewed: ready to
/// finalize.
fn reviewed_project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "hello\n").unwrap();
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-q", "-m", "seed"]);
    run_ok(&root, &["init"]);
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "init specsync"]);
    run_ok(
        &root,
        &[
            "change",
            "new",
            "Document the greeter",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation only; no canonical spec changes",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "The README describes the greeter"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run_ok(&root, &["change", "answer", ID, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(ID);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let body = if name == "tasks" {
            "# Tasks\n\n- [x] Document the greeter.\n"
        } else {
            "# Complete\n\nThe README documents the greeter.\n"
        };
        fs::write(workspace.join(format!("{name}.md")), body).unwrap();
    }
    fs::write(root.join("README.md"), "hello\n\nThe greeter says hello.\n").unwrap();
    run_ok(&root, &["change", "approve", ID, "--actor", "Scope owner"]);
    run_ok(&root, &["change", "check", ID, "--commit"]);
    run_ok(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    (temp, root)
}

fn approvals(root: &Path) -> PathBuf {
    root.join(".specsync/changes")
        .join(ID)
        .join("approvals.json")
}

fn ship_status(root: &Path) -> Value {
    let output = run(root, &["--json", "change", "ship-status", ID]);
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "ship-status prints one document ({error}): {}",
            printed(&output)
        )
    })
}

#[test]
fn finalize_ship_and_ship_status_refuse_a_ledger_the_working_tree_changed() {
    let (_temp, root) = reviewed_project();
    let report = ship_status(&root);
    assert_eq!(
        report["ready_to_finalize"], true,
        "the fixture is ready: {report}"
    );

    let path = approvals(&root);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("Scope owner"), "{text}");
    fs::write(&path, text.replace("Scope owner", "Mallory")).unwrap();

    let report = ship_status(&root);
    assert_eq!(report["ready_to_finalize"], false, "{report}");
    let blockers: Vec<String> = report["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|blocker| blocker.as_str().unwrap().to_string())
        .collect();
    let message = blockers
        .iter()
        .find(|blocker| blocker.contains("approvals.json") && blocker.contains("HEAD"))
        .unwrap_or_else(|| panic!("ship-status lists the ledger as a blocker: {report}"))
        .clone();

    for args in [
        vec!["change", "ship", ID, "--dry-run"],
        vec!["change", "ship", ID],
        vec!["change", "finalize", ID],
    ] {
        let output = run(&root, &args);
        assert!(
            !output.status.success(),
            "{args:?} refuses: {}",
            printed(&output)
        );
        assert!(
            printed(&output).contains(&message),
            "{args:?} refuses with the same message:\n{message}\n---\n{}",
            printed(&output)
        );
    }
    assert!(
        root.join(".specsync/changes").join(ID).is_dir(),
        "nothing was archived"
    );

    git(
        &root,
        &[
            "checkout",
            "HEAD",
            "--",
            &format!(".specsync/changes/{ID}/approvals.json"),
        ],
    );
    let report = ship_status(&root);
    assert_eq!(
        report["ready_to_finalize"], true,
        "restored, it is ready again: {report}"
    );
}
