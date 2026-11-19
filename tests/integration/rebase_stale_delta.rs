// Issue #1085: a MODIFIED requirement delta that went stale only because main edited the same
// requirement is rebased by `change check` when the two edits do not overlap word by word, keeping
// the approval and recording the rebase (REQ-change-161, REQ-cmd-change-038). Overlapping edits
// are refused as before (REQ-change-122).
//
// These drive the built binary in a real repository, because the approval-time text of the
// requirement is recovered from Git history.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

const BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

/// The change rewords the first sentence; main later edits the last criterion.
const REWORD_DELTA: &str = "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep exactly one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n";

struct Project {
    _temp: TempDir,
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let project = Self { _temp: temp, root };
        project.git(&["init", "-b", "main"]);
        project.git(&["config", "user.email", "test@example.com"]);
        project.git(&["config", "user.name", "Test"]);
        project.git(&["config", "core.autocrlf", "false"]);
        project.write("src/auth.rs", "// Authentication module.\n");
        project.write(
            "specs/auth/auth.spec.md",
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        );
        project.write("specs/auth/requirements.md", BASE_REQUIREMENTS);
        project.write(".gitignore", ".specsync/hashes.json\n");
        project.write(
            ".specsync/sdd.json",
            &serde_json::to_string_pretty(&serde_json::json!({
                "version": 2,
                "enabled": true,
                "require_change_for_meaningful_files": false,
                "meaningful_paths": ["src/", ".specsync/sdd.json"],
                "ignored_paths": [".specsync/"],
                "verification_commands": ["true"],
                "custom_artifacts": {},
                "principles_file": null
            }))
            .unwrap(),
        );
        project.git(&["add", "--all"]);
        project.git(&["commit", "-q", "-m", "base"]);
        // A bare origin inside .git, so the working tree never sees it: `origin/main` is the
        // default branch the rebase reads main's text from.
        let origin = project.root.join(".git/test-origin.git");
        project.git(&[
            "init",
            "-q",
            "--bare",
            "-b",
            "main",
            origin.to_str().unwrap(),
        ]);
        project.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        project.git(&["push", "-q", "origin", "main"]);
        project
    }

    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.root)
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

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.root.join(relative)).unwrap()
    }

    fn invoke(&self, args: &[&str]) -> Output {
        specsync()
            .args(["--root", self.root.to_str().unwrap()])
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    /// A change to REQ-auth-001, approved against `BASE_REQUIREMENTS` and committed; then main
    /// replaces `from` with `to` in that requirement and commits.
    fn approved_change_then_main_edits(&self, delta: &str, from: &str, to: &str) -> String {
        self.approved_change_then_edits(delta, from, to, true)
    }

    /// As `approved_change_then_main_edits`, but the edit stays on the change's own branch when
    /// `push` is false: it is never on the default branch.
    fn approved_change_then_edits(&self, delta: &str, from: &str, to: &str, push: bool) -> String {
        let created: Value = serde_json::from_str(&self.run(&[
            "--json",
            "change",
            "new",
            "Say a user holds exactly one session",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ]))
        .unwrap();
        let id = created["change"]["id"].as_str().unwrap().to_string();
        for (question, answer) in [
            (
                "acceptance_criteria",
                "The requirement says exactly one session",
            ),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            self.run(&["change", "answer", &id, question, answer]);
        }
        let workspace = self.root.join(".specsync/changes").join(&id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Reword the requirement.\n".to_string(),
                "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
                "testing" => "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-001 | SpecSync structural check of the module |\n".to_string(),
                _ => "# Complete\n\nReviewed requirement wording.\n".to_string(),
            };
            fs::write(workspace.join(format!("{name}.md")), content).unwrap();
        }
        fs::write(workspace.join("deltas/auth.md"), delta).unwrap();
        self.run(&["change", "approve", &id, "--actor", "Scope owner"]);
        self.git(&["add", "--all"]);
        self.git(&["commit", "-q", "-m", "approve"]);
        let moved = self.read("specs/auth/requirements.md").replace(from, to);
        assert_ne!(moved, self.read("specs/auth/requirements.md"));
        self.write("specs/auth/requirements.md", &moved);
        self.git(&["add", "--all"]);
        self.git(&["commit", "-q", "-m", "main edits REQ-auth-001"]);
        if push {
            self.git(&["push", "-q", "origin", "main"]);
        }
        id
    }

    fn ledger(&self, id: &str) -> Value {
        serde_json::from_str(&self.read(&format!(".specsync/changes/{id}/approvals.json"))).unwrap()
    }
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// DISCRIMINATOR for #1085: before it, `change check` refused this delta as stale and the human
/// had to approve a hand-rebased body. The edits are words apart, so the check merges them.
#[test]
fn a_non_overlapping_main_edit_is_rebased_by_change_check_and_shown_in_status() {
    let project = Project::new();
    let id = project.approved_change_then_main_edits(
        REWORD_DELTA,
        "after one day.",
        "after twelve hours.",
    );
    let approvals_before = project.ledger(&id)["approvals"].as_array().unwrap().len();

    project.run(&["change", "check", &id]);

    let requirements = project.read("specs/auth/requirements.md");
    assert!(
        requirements.contains("The system SHALL keep exactly one session per user."),
        "the change's edit lands: {requirements}"
    );
    assert!(
        requirements.contains("- A session expires after twelve hours."),
        "main's edit survives: {requirements}"
    );
    let delta = project.read(&format!(".specsync/changes/{id}/deltas/auth.md"));
    assert!(
        delta.contains("exactly one session") && delta.contains("twelve hours"),
        "the delta is rebased onto main's text: {delta}"
    );
    let ledger = project.ledger(&id);
    assert_eq!(
        ledger["approvals"].as_array().unwrap().len(),
        approvals_before,
        "no approval is recorded on anyone's behalf"
    );
    let rebases = ledger["delta_rebases"].as_array().expect("a rebase record");
    assert_eq!(rebases.len(), 1, "{ledger}");
    assert_eq!(rebases[0]["module"], "auth");

    let status: Value =
        serde_json::from_str(&project.run(&["--json", "change", "status", &id])).unwrap();
    assert_eq!(status["summary"]["approval_valid"], true, "{status}");
    assert!(
        status["summary"]["changed_approved_deltas"]
            .as_array()
            .is_none_or(Vec::is_empty),
        "the rebased delta is the approved delta: {status}"
    );
    let listed = status["summary"]["delta_rebases"].as_array().unwrap();
    assert_eq!(listed.len(), 1, "{status}");
    assert_eq!(listed[0]["module"], "auth");
    assert_eq!(listed[0]["requirement"], "REQ-auth-001");
    let text = project.run(&["change", "status", &id]);
    assert!(
        text.contains("Rebased") && text.contains("REQ-auth-001"),
        "text status names the rebase:\n{text}"
    );

    // Checking again changes nothing and records no second rebase.
    project.run(&["change", "check", &id]);
    assert_eq!(
        project.ledger(&id)["delta_rebases"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

/// REQ-change-122 still holds when the edits overlap: the same words changed on both sides.
#[test]
fn an_overlapping_main_edit_is_still_refused_and_writes_nothing() {
    let project = Project::new();
    let id = project.approved_change_then_main_edits(
        REWORD_DELTA,
        "keep one session per user.",
        "keep a single session per user.",
    );
    let requirements = project.read("specs/auth/requirements.md");
    let delta = project.read(&format!(".specsync/changes/{id}/deltas/auth.md"));
    let ledger = project.read(&format!(".specsync/changes/{id}/approvals.json"));

    let output = project.invoke(&["change", "check", &id]);

    let text = combined(&output);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("REQ-auth-001"), "{text}");
    assert!(
        text.contains(&format!("specsync change approve {id}")),
        "{text}"
    );
    assert_eq!(project.read("specs/auth/requirements.md"), requirements);
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/deltas/auth.md")),
        delta
    );
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/approvals.json")),
        ledger
    );
}

/// REQ-change-161: CI never writes, so a CI run of `change check` does not rebase; the delta stays
/// refused there until it is checked locally.
#[test]
fn a_ci_run_does_not_rebase() {
    let project = Project::new();
    let id = project.approved_change_then_main_edits(
        REWORD_DELTA,
        "after one day.",
        "after twelve hours.",
    );
    let delta = project.read(&format!(".specsync/changes/{id}/deltas/auth.md"));
    let ledger = project.read(&format!(".specsync/changes/{id}/approvals.json"));

    let output = specsync()
        .env("CI", "true")
        .env("GITHUB_WORKSPACE", &project.root)
        .args([
            "--root",
            project.root.to_str().unwrap(),
            "change",
            "check",
            &id,
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", combined(&output));
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/deltas/auth.md")),
        delta
    );
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/approvals.json")),
        ledger
    );
}

/// fable-lead's #1089 review, end to end: an edit to the requirement that only the change's own
/// branch holds is never merged into the approved delta. `change check` refuses it as stale, as
/// it always has, and writes nothing.
#[test]
fn a_branch_side_edit_to_the_requirement_is_refused_not_rebased() {
    let project = Project::new();
    let id = project.approved_change_then_edits(
        REWORD_DELTA,
        "after one day.",
        "after one day, or at sign-out.",
        false,
    );
    let delta = project.read(&format!(".specsync/changes/{id}/deltas/auth.md"));
    let ledger = project.read(&format!(".specsync/changes/{id}/approvals.json"));

    let output = project.invoke(&["change", "check", &id]);

    let text = combined(&output);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("has changed since"), "{text}");
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/deltas/auth.md")),
        delta
    );
    assert_eq!(
        project.read(&format!(".specsync/changes/{id}/approvals.json")),
        ledger
    );
}
