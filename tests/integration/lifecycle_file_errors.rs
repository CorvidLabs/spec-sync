//! #434 (REQ-change-164, REQ-change-165): an error about one of a change's lifecycle files names
//! that file, and a file a newer SpecSync wrote is refused with one message wherever it is read,
//! `change audit` included.

use crate::helpers::specsync_std;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const ID: &str = "probe-lifecycle-file-errors";

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// Everything a command printed, stdout and stderr together, and whether it succeeded.
fn run(root: &Path, args: &[&str]) -> (bool, String) {
    let output = specsync_std()
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

/// An adopted project with one draft change, committed.
fn draft_project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn probe() {}\n").unwrap();
    assert!(run(&root, &["init"]).0);
    assert!(run(&root, &["change", "adopt"]).0);
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "adopt"]);
    let (created, text) = run(
        &root,
        &[
            "change",
            "new",
            "probe lifecycle file errors",
            "--kind",
            "bug-fix",
            "--no-spec-change",
            "--rationale",
            "probe",
            "--path",
            "src/lib.rs",
        ],
    );
    assert!(created, "{text}");
    (temp, root)
}

fn change_file(root: &Path, name: &str) -> PathBuf {
    root.join(".specsync/changes").join(ID).join(name)
}

fn edit_json(path: &Path, edit: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

#[test]
fn a_type_error_in_approvals_names_the_file() {
    let (_temp, root) = draft_project();
    edit_json(&change_file(&root, "approvals.json"), |value| {
        value["approvals"] = Value::String("soon".into());
    });
    let named = format!(".specsync/changes/{ID}/approvals.json");
    for args in [&["change", "status", ID][..], &["change", "audit"][..]] {
        let (_, text) = run(&root, args);
        assert!(
            text.contains("invalid type"),
            "{args:?} did not report the error: {text}"
        );
        assert!(
            text.contains(&named),
            "{args:?} does not name {named}: {text}"
        );
    }
}

#[test]
fn a_state_file_from_a_newer_specsync_is_refused_with_the_newer_specsync_message() {
    let (_temp, root) = draft_project();
    edit_json(&change_file(&root, "state.json"), |value| {
        value["schema_version"] = Value::from(99);
    });
    let (passed, text) = run(&root, &["change", "status", ID]);
    assert!(!passed, "a newer state.json was read silently: {text}");
    assert!(
        text.contains("was written by a newer SpecSync (schema version 99)")
            && text.contains("state.json"),
        "{text}"
    );
}

#[test]
fn a_verification_attempt_history_from_a_newer_specsync_is_refused_by_audit_and_status() {
    let (_temp, root) = draft_project();
    fs::write(
        change_file(&root, "verification-attempts.json"),
        format!(r#"{{"schema_version": 99, "change_id": "{ID}", "attempts": []}}"#),
    )
    .unwrap();
    for args in [&["change", "audit"][..], &["change", "status", ID][..]] {
        let (passed, text) = run(&root, args);
        assert!(!passed, "{args:?} passed a newer attempt history: {text}");
        assert!(
            text.contains("was written by a newer SpecSync (schema version 99)")
                && text.contains("verification-attempts.json"),
            "{args:?}: {text}"
        );
    }
}
