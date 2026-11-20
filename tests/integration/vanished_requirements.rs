// #1081 (REQ-change-169, REQ-cmd-change-042) through the binary: a branch that lost a canonical
// requirement no approved REMOVED delta names is listed as a blocker by `change ship-status`,
// refused by `change ship --dry-run`, `change check` and `change finalize`, and cleared by
// restoring the block.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

const BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n- A session expires after one day.\n\n### REQ-auth-002\n\nThe system SHALL sign a user out on request.\n\nAcceptance Criteria\n- Signing out ends the session.\n";

/// The change rewords REQ-auth-001's first sentence.
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
    /// A change to REQ-auth-001 with `delta`, approved against `BASE_REQUIREMENTS` and committed.
    fn approved_change(&self, delta: &str) -> String {
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
        // The evidence names each requirement the delta touches.
        let rows: String = delta
            .split("### REQUIREMENT ")
            .skip(1)
            .filter_map(|rest| rest.split_whitespace().next())
            .map(|requirement| {
                format!("| {requirement} | SpecSync structural check of the module |\n")
            })
            .collect();
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Reword the requirement.\n".to_string(),
                "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
                "testing" => format!(
                    "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n{rows}"
                ),
                _ => "# Complete\n\nReviewed requirement wording.\n".to_string(),
            };
            fs::write(workspace.join(format!("{name}.md")), content).unwrap();
        }
        fs::write(workspace.join("deltas/auth.md"), delta).unwrap();
        self.run(&["change", "approve", &id, "--actor", "Scope owner"]);
        self.git(&["add", "--all"]);
        self.git(&["commit", "-q", "-m", "approve"]);
        id
    }
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn ship_status_lists_a_lost_requirement_and_ship_and_finalize_refuse_it() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    // The branch loses REQ-auth-002, which no REMOVED delta names.
    let requirements = project.read("specs/auth/requirements.md");
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(
        "specs/auth/requirements.md",
        requirements[..start].trim_end(),
    );
    project.git(&["commit", "-q", "-am", "truncate"]);

    let status = project.invoke(&["change", "ship-status", &id]);
    let text = combined(&status);
    assert!(
        text.contains("lost canonical requirements") && text.contains("REQ-auth-002"),
        "{text}"
    );
    // REQ-cmd-change-042: ship-status exits 1 while the blocker holds (with #654 on main).
    assert_eq!(status.status.code(), Some(1), "{text}");
    let json: Value = serde_json::from_slice(
        &project
            .invoke(&["--json", "change", "ship-status", &id])
            .stdout,
    )
    .unwrap();
    assert_eq!(json["ready_to_finalize"], false, "{json}");
    let dry_run = project.invoke(&["change", "ship", &id, "--dry-run"]);
    assert!(!dry_run.status.success(), "{}", combined(&dry_run));
    assert!(
        combined(&dry_run).contains("REQ-auth-002"),
        "{}",
        combined(&dry_run)
    );
    // Finalize refuses and archives nothing. A loss committed after a passing check stales that
    // check first, so finalize names the stale verification, and the check it asks for names the
    // lost requirement; finalize's own gate is the second line behind it.
    let finalize = project.invoke(&["change", "finalize", &id]);
    assert!(!finalize.status.success(), "{}", combined(&finalize));
    assert!(
        !project
            .root
            .join(".specsync/archive/changes")
            .read_dir()
            .is_ok_and(|mut entries| entries.next().is_some()),
        "finalize archived nothing"
    );
    let check = project.invoke(&["change", "check", &id, "--commit"]);
    assert!(!check.status.success(), "{}", combined(&check));
    assert!(
        combined(&check).contains("REQ-auth-002"),
        "{}",
        combined(&check)
    );

    // Restoring the block clears the blocker.
    project.write("specs/auth/requirements.md", &requirements);
    project.git(&["commit", "-q", "-am", "restore"]);
    let status = project.invoke(&["change", "ship-status", &id]);
    assert!(
        !combined(&status).contains("lost canonical requirements"),
        "{}",
        combined(&status)
    );
}

/// Codex on #1108, third round (P1), with ss-lead's rule-3 point: a change whose approval
/// records no delta digests, as one made before they existed, binds no delta wording, so its
/// own `## REMOVED` item accounts for nothing and the check refuses. The approval the refusal
/// prints, run as printed, binds the delta, and the check after it passes.
#[test]
fn an_unbound_removal_is_refused_and_its_printed_way_out_ends_in_a_passing_check() {
    let project = Project::new();
    let id = project.approved_change(&format!(
        "{REWORD_DELTA}\n## REMOVED\n\n### REQUIREMENT REQ-auth-002\n"
    ));
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    assert!(
        !project
            .read("specs/auth/requirements.md")
            .contains("REQ-auth-002")
    );
    // The approval predates delta digests: such a ledger records none.
    let ledger = format!(".specsync/changes/{id}/approvals.json");
    let mut approvals: Value = serde_json::from_str(&project.read(&ledger)).unwrap();
    let definition = approvals["approvals"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|approval| approval["gate"] == "definition")
        .unwrap();
    assert!(
        definition
            .as_object_mut()
            .unwrap()
            .remove("approved_delta_digests")
            .is_some()
    );
    project.write(&ledger, &serde_json::to_string_pretty(&approvals).unwrap());
    project.git(&[
        "commit",
        "-q",
        "-am",
        "an approval from before delta digests",
    ]);

    let refused = project.invoke(&["change", "check", &id]);
    let text = combined(&refused);
    assert!(!refused.status.success(), "{text}");
    assert!(
        text.contains("lost canonical requirements") && text.contains("REQ-auth-002"),
        "{text}"
    );
    let printed = text
        .split('`')
        .find(|part| part.starts_with("specsync change approve "))
        .unwrap_or_else(|| panic!("no approval command in the refusal: {text}"))
        .replace("<name>", "Scope owner");
    let words: Vec<&str> = printed.split_whitespace().collect();
    project.run(&words[1..]);

    project.run(&["change", "check", &id]);
}
