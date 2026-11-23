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
        Self::with_workflow_version(2)
    }

    fn with_workflow_version(version: u32) -> Self {
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
                "version": version,
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
    let ledger_path = format!(".specsync/changes/{id}/verification-attempts.json");
    let attempts_before = project.read(&ledger_path);

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
    assert_eq!(project.read(&ledger_path), attempts_before);
    let tasks_path = format!(".specsync/changes/{id}/tasks.md");
    let tasks = project.read(&tasks_path);
    assert!(tasks.contains("- [x]"));
    project.write(&tasks_path, &tasks.replacen("- [x]", "- [ ]", 1));
    let ship = project.invoke(&["change", "ship", &id]);
    let text = combined(&ship);
    assert!(
        !ship.status.success() && text.contains("tasks.md contains incomplete tasks"),
        "{text}"
    );
    assert_eq!(project.read(&ledger_path), attempts_before);
    project.write(&tasks_path, &tasks);

    let ship = project.invoke(&["change", "ship", &id]);
    let text = combined(&ship);
    assert!(
        !ship.status.success() && text.contains("REQ-auth-002"),
        "{text}"
    );
    assert_appended_refusal(&project, &ledger_path, &attempts_before, "REQ-auth-002");
    assert_refusal_preserves_audit_error(&project, &ledger_path, &["change", "ship", &id]);

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
    project.run(&["change", "check", &id, "--commit"]);
    project.run(&["change", "review", &id, "--reviewer", "Scoped reviewer"]);
    let attempts = project.read(&ledger_path);
    let head = project.git(&["rev-parse", "HEAD"]);
    project.run(&["change", "ship", &id, "--dry-run"]);
    assert_eq!(project.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(project.read(&ledger_path), attempts);
    project.run(&["change", "ship", &id]);
    assert!(
        !project
            .root
            .join(format!(".specsync/changes/{id}"))
            .exists()
    );
}

fn assert_appended_refusal(project: &Project, ledger_path: &str, before: &str, expected: &str) {
    let before: Value = serde_json::from_str(before).unwrap();
    let after: Value = serde_json::from_str(&project.read(ledger_path)).unwrap();
    let before = before["attempts"].as_array().unwrap();
    let after = after["attempts"].as_array().unwrap();
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(&after[..before.len()], before);
    let refusal = after.last().unwrap();
    assert_eq!(refusal["passed"], false);
    assert!(refusal["refusal"].as_str().unwrap().contains(expected));
}

fn assert_refusal_preserves_audit_error(project: &Project, ledger_path: &str, args: &[&str]) {
    let attempts = project.read(ledger_path);
    let broken = "{";
    project.write(ledger_path, broken);
    let refusal = project.invoke(args);
    let text = combined(&refusal);
    assert!(!refusal.status.success(), "{text}");
    assert!(text.contains("REQ-auth-002"), "{text}");
    assert!(text.contains("could not record refused attempt"), "{text}");
    assert!(
        text.contains("invalid verification attempt history"),
        "{text}"
    );
    assert_eq!(project.read(ledger_path), broken);
    project.write(ledger_path, &attempts);
}

#[test]
fn accept_records_a_lost_requirement_and_can_retry_after_restore() {
    let project = Project::with_workflow_version(1);
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.run(&["change", "check", &id]);
    let path = "specs/auth/requirements.md";
    let requirements = project.read(path);
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(path, &requirements[..start]);
    let workspace = format!(".specsync/changes/{id}");
    let ledger_path = format!("{workspace}/verification-attempts.json");
    let state_path = format!("{workspace}/state.json");
    let verification_path = format!("{workspace}/verification.json");
    let attempts = project.read(&ledger_path);
    let state = project.read(&state_path);
    let verification = project.read(&verification_path);
    let accept = ["change", "accept", &id, "--actor", "Closing reviewer"];

    let refusal = project.invoke(&accept);
    let text = combined(&refusal);
    assert!(
        !refusal.status.success() && text.contains("REQ-auth-002"),
        "{text}"
    );
    assert_appended_refusal(&project, &ledger_path, &attempts, "REQ-auth-002");
    assert_eq!(project.read(&state_path), state);
    assert_eq!(project.read(&verification_path), verification);
    assert_refusal_preserves_audit_error(&project, &ledger_path, &accept);

    project.write(path, &requirements);
    project.git(&["add", "--", path]);
    let refused_history: Value = serde_json::from_str(&project.read(&ledger_path)).unwrap();
    project.run(&["change", "check", &id]);
    project.run(&accept);
    assert_eq!(
        serde_json::from_str::<Value>(&project.read(&state_path)).unwrap()["state"],
        "accepted"
    );
    let accepted_history: Value = serde_json::from_str(&project.read(&ledger_path)).unwrap();
    let before = refused_history["attempts"].as_array().unwrap();
    let after = accepted_history["attempts"].as_array().unwrap();
    assert_eq!(after.len(), before.len() + 2);
    assert_eq!(&after[..before.len()], before);
    assert_eq!(after[before.len()]["passed"], true);
    assert!(after[before.len()]["acceptance_input_digest"].is_null());
    let closing_attempt = after.last().unwrap();
    assert_eq!(closing_attempt["passed"], true);
    assert!(closing_attempt["acceptance_input_digest"].is_string());
    assert_eq!(
        closing_attempt,
        &serde_json::from_str::<Value>(&project.read(&verification_path)).unwrap()
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

/// The tester's run on #1108 at 4ced5a26 (rule 3): with `specs_dir` outside the project, the
/// check never sends Git a pathspec outside the repository, so it never says it cannot read the
/// repository and never names a fetch. It refuses naming the directory, and once `specs_dir` is
/// inside the project again the check passes. Since 6.4.0 the confined configuration load refuses
/// it first, in its own words, before this check runs; `specsync check` still says the directory
/// "must remain beneath the project root".
#[test]
fn a_specs_directory_outside_the_project_is_named_and_never_sent_to_git() {
    let project = Project::new();
    let id = project.approved_change(
        "## ADDED\n\n### REQUIREMENT REQ-auth-003\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n",
    );
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(
        ".specsync/config.toml",
        "specs_dir = \"../outside/specs\"\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "specs outside the project"]);

    let refused = project.invoke(&["change", "check", &id]);
    let text = combined(&refused);

    assert!(!refused.status.success(), "{text}");
    assert!(
        text.contains("../outside/specs")
            && (text.contains("must remain beneath the project root")
                || text.contains("must be a relative path inside the project")),
        "{text}"
    );
    assert!(
        !text.contains("git fetch") && !text.contains("cannot tell"),
        "{text}"
    );
    fs::remove_file(project.root.join(".specsync/config.toml")).unwrap();
    project.git(&["commit", "-q", "-am", "specs back inside the project"]);
    project.run(&["change", "check", &id]);
}

// ---- The lost-requirements follow-ups: #1112, #1113, #1116, #1117 ----

/// The branch drops REQ-auth-002 from the canonical requirements file and commits.
fn lose_auth_002(project: &Project) -> String {
    let requirements = project.read("specs/auth/requirements.md");
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(
        "specs/auth/requirements.md",
        requirements[..start].trim_end(),
    );
    project.git(&["commit", "-q", "-am", "lose REQ-auth-002"]);
    requirements
}

/// #1112: the files read at the reference come only from what was canonical there. A branch
/// that registers a spec beside a requirements file that was already there, and rewrites that
/// file, is not refused for the ids it held before it was canonical.
#[test]
fn a_file_the_branch_makes_canonical_is_not_read_at_the_reference() {
    let project = Project::new();
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-note-001\n\nA note kept beside the docs.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "notes"]);
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(
        ".specsync/registry.toml",
        "[registry]\nname = \"notes\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nnotes = \"docs/notes/notes.spec.md\"\n",
    );
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-notes-001\n\nThe docs SHALL keep one page per module.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "register the notes"]);

    let text = combined(&project.invoke(&["change", "ship-status", &id]));

    assert!(!text.contains("REQ-note-001"), "{text}");
}

/// #1113: an archived change cannot be checked, finalized or shipped again, so `change
/// ship-status` lists no lost-requirements blocker for it.
#[test]
fn ship_status_of_an_archived_change_lists_no_lost_requirement() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.run(&["change", "review", &id, "--reviewer", "Reviewer"]);
    project.run(&["change", "finalize", &id]);
    // An active change on the same branch (REQ-cmd-change-045's second criterion).
    let created: Value = serde_json::from_str(&project.run(&[
        "--json",
        "change",
        "new",
        "Log every sign-in",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ]))
    .unwrap();
    let active = created["change"]["id"].as_str().unwrap().to_string();
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "a draft change"]);
    lose_auth_002(&project);

    let text = combined(&project.invoke(&["change", "ship-status", &id]));
    let active_text = combined(&project.invoke(&["change", "ship-status", &active]));

    assert!(
        !text.contains("that no approved `## REMOVED` delta names"),
        "{text}"
    );
    assert!(
        active_text.contains("that no approved `## REMOVED` delta names")
            && active_text.contains("REQ-auth-002"),
        "{active_text}"
    );
}

/// #1116: `change audit` reports the branch's lost requirements for an active change whose
/// own delta does not validate, beside that error.
#[test]
fn the_audit_reports_a_lost_requirement_beside_a_malformed_delta() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(
        &format!(".specsync/changes/{id}/deltas/auth.md"),
        "This is not a delta.\n",
    );
    lose_auth_002(&project);

    let text = combined(&project.invoke(&["change", "audit"]));

    assert!(
        text.contains("that no approved `## REMOVED` delta names") && text.contains("REQ-auth-002"),
        "{text}"
    );
}

/// #1117: a requirements directory the check cannot read is named, and the requirement it may
/// hold is not reported lost.
#[cfg(unix)]
#[test]
fn an_unreadable_requirements_directory_is_named_instead_of_a_loss() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    // The branch moves REQ-auth-002 into a file of its own in a subdirectory.
    let requirements = lose_auth_002(&project);
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(
        "specs/auth/more/requirements.md",
        &format!("# Requirements\n\n{}", &requirements[start..]),
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "move REQ-auth-002"]);
    let more = project.root.join("specs/auth/more");
    fs::set_permissions(&more, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&more).is_ok() {
        // This user reads every directory (root), so nothing here is unreadable.
        fs::set_permissions(&more, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }

    let ledger_path = format!(".specsync/changes/{id}/verification-attempts.json");
    let attempts = project.read(&ledger_path);
    let status = project.invoke(&["change", "ship-status", &id]);
    let dry_run = project.invoke(&["change", "ship", &id, "--dry-run"]);
    let audit = project.invoke(&["change", "audit", "--strict"]);
    let after_read_only = project.read(&ledger_path);
    let check = project.invoke(&["change", "check", &id]);
    fs::set_permissions(&more, fs::Permissions::from_mode(0o755)).unwrap();

    let diagnostic = "cannot tell whether this branch lost canonical requirements";
    for output in [status, dry_run, audit, check] {
        let text = combined(&output);
        assert!(!output.status.success(), "{text}");
        assert!(
            text.contains(diagnostic) && text.contains("specs/auth/more"),
            "{text}"
        );
        assert!(
            !text.contains("that no approved `## REMOVED` delta names"),
            "{text}"
        );
    }
    assert_eq!(after_read_only, attempts);
    assert_appended_refusal(&project, &ledger_path, &attempts, diagnostic);
    // Once the directory is readable again, nothing is lost (REQ-change-177's second criterion).
    let again = combined(&project.invoke(&["change", "ship-status", &id]));
    assert!(!again.contains("lost canonical requirements"), "{again}");
}

#[cfg(unix)]
#[test]
fn an_unreadable_directory_with_no_tracked_requirements_does_not_hide_a_loss() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    let historical_path = "specs/auth/scratch/more.req.md";
    let requirements = "# Requirements\n\n### REQ-auth-003\n\nKeep a recovery record.\n";
    project.write(historical_path, requirements);
    project.write("specs/auth/scratch/notes.txt", "Working notes.\n");
    project.git(&["add", "--", "specs/auth/scratch"]);
    project.git(&["commit", "-q", "-m", "track scratch requirements and notes"]);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.git(&["rm", "--", historical_path]);
    project.git(&["commit", "-q", "-m", "lose REQ-auth-003"]);
    project.write("specs/auth/scratch/untracked.req.md", requirements);

    let scratch = project.root.join("specs/auth/scratch");
    fs::set_permissions(&scratch, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&scratch).is_ok() {
        fs::set_permissions(&scratch, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let output = project.invoke(&["change", "ship-status", &id]);
    fs::set_permissions(&scratch, fs::Permissions::from_mode(0o755)).unwrap();

    let text = combined(&output);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("REQ-auth-003"), "{text}");
    assert!(
        text.contains("that no approved `## REMOVED` delta names"),
        "{text}"
    );
    assert!(
        !text.contains("cannot tell whether this branch lost canonical requirements"),
        "{text}"
    );
}

/// #1112, with `specs_dir`: a branch that moves its specs directory onto one that already held a
/// `requirements.md` is not refused for that file's old ids.
#[test]
fn a_specs_directory_the_branch_moves_onto_is_not_read_at_the_reference() {
    let project = Project::new();
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-note-001\n\nA note kept beside the docs.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "notes"]);
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(".specsync/config.toml", "specs_dir = \"docs\"\n");
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-notes-001\n\nThe docs SHALL keep one page per module.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "move the specs directory"]);

    let text = combined(&project.invoke(&["change", "ship-status", &id]));

    assert!(!text.contains("REQ-note-001"), "{text}");
}

/// #1116, with an approval that does not read: the audit still reports the branch's loss.
#[test]
fn the_audit_reports_a_lost_requirement_beside_an_unreadable_approval() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(
        &format!(".specsync/changes/{id}/approvals.json"),
        "not an approval\n",
    );
    lose_auth_002(&project);

    let text = combined(&project.invoke(&["change", "audit"]));

    assert!(
        text.contains("that no approved `## REMOVED` delta names") && text.contains("REQ-auth-002"),
        "{text}"
    );
}

/// #1117, with a file: a requirements file the check cannot read is named, and the requirement it
/// holds is not reported lost.
#[cfg(unix)]
#[test]
fn an_unreadable_requirements_file_is_named_instead_of_a_loss() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    let requirements = lose_auth_002(&project);
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(
        "specs/auth/more.req.md",
        &format!("# Requirements\n\n{}", &requirements[start..]),
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "move REQ-auth-002"]);
    let file = project.root.join("specs/auth/more.req.md");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_to_string(&file).is_ok() {
        // This user reads every file (root), so nothing here is unreadable.
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        return;
    }

    let status = project.invoke(&["change", "ship-status", &id]);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    let text = combined(&status);

    assert!(
        text.contains("cannot tell whether this branch lost canonical requirements"),
        "{text}"
    );
    assert!(text.contains("specs/auth/more.req.md"), "{text}");
    assert!(
        !text.contains("that no approved `## REMOVED` delta names"),
        "{text}"
    );
}

#[test]
fn an_unreadable_untracked_requirements_file_does_not_hide_a_loss() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    lose_auth_002(&project);

    let path = "specs/auth/more.req.md";
    fs::write(project.root.join(path), [0xff, 0xfe]).unwrap();
    let indeterminate = "cannot tell whether this branch lost canonical requirements";
    let definitive = "that no approved `## REMOVED` delta names";
    let untracked = project.invoke(&["change", "ship-status", &id]);
    let text = combined(&untracked);
    assert!(!untracked.status.success());
    assert!(text.contains("REQ-auth-002"), "{text}");
    assert!(text.contains(definitive), "{text}");
    assert!(!text.contains(indeterminate), "{text}");

    project.git(&["add", "--", path]);
    project.git(&[
        "commit",
        "-q",
        "-m",
        "track the unreadable requirements file",
    ]);
    let tracked = project.invoke(&["change", "ship-status", &id]);
    let text = combined(&tracked);
    assert!(!tracked.status.success());
    assert!(text.contains(indeterminate), "{text}");
    assert!(text.contains(path), "{text}");
    assert!(!text.contains(definitive), "{text}");
}

/// #1117, the guard: on a branch that lost nothing, an unreadable requirements directory changes
/// nothing (REQ-change-177).
#[cfg(unix)]
#[test]
fn an_unreadable_directory_with_nothing_lost_changes_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.write(
        "specs/auth/more/requirements.md",
        "# Requirements\n\n### REQ-auth-003\n\nThe system SHALL log each sign-in.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "a requirement of its own"]);
    let more = project.root.join("specs/auth/more");
    fs::set_permissions(&more, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&more).is_ok() {
        fs::set_permissions(&more, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }

    let status = project.invoke(&["change", "ship-status", &id]);
    fs::set_permissions(&more, fs::Permissions::from_mode(0o755)).unwrap();
    let text = combined(&status);

    assert!(!text.contains("lost canonical requirements"), "{text}");
}

// ---- corvid-agent's review and Codex's first round on #1134 ----

/// Main holds `docs/notes/requirements.md` and a configuration `config`; the branch drops the
/// configuration and rewrites the notes. Returns `change ship-status`'s output.
fn nothing_canonical_at_the_reference(config: &str) -> String {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-note-001\n\nA note kept beside the docs.\n",
    );
    project.write(".specsync/config.toml", config);
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "notes and a configuration"]);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.git(&["rm", "-q", ".specsync/config.toml"]);
    project.write(
        "docs/notes/requirements.md",
        "# Requirements\n\n### REQ-notes-001\n\nThe docs SHALL keep one page per module.\n",
    );
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "drop the configuration"]);
    combined(&project.invoke(&["change", "ship-status", &id]))
}

/// corvid-agent and Codex on #1134 (REQ-change-175): when nothing was canonical at the reference,
/// because its `specs_dir` leaves the project, nothing is read there, not every path.
#[test]
fn a_reference_whose_specs_dir_leaves_the_project_is_not_read_whole() {
    let text = nothing_canonical_at_the_reference("specs_dir = \"../elsewhere\"\n");
    assert!(!text.contains("REQ-note-001"), "{text}");
}

/// The same when the reference's configuration does not parse.
#[test]
fn a_reference_whose_configuration_does_not_parse_is_not_read_whole() {
    let text = nothing_canonical_at_the_reference("specs_dir = [\n");
    assert!(!text.contains("REQ-note-001"), "{text}");
}

/// The branch registers a spec in `docs/notes` and moves REQ-auth-002 into its requirements file.
fn move_auth_002_into_registered_notes(project: &Project) -> String {
    let requirements = lose_auth_002(project);
    let start = requirements.find("### REQ-auth-002").unwrap();
    project.write(
        ".specsync/registry.toml",
        "[registry]\nname = \"notes\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\nnotes = \"docs/notes/notes.spec.md\"\n",
    );
    project.write(
        "docs/notes/requirements.md",
        &format!("# Requirements\n\n{}", &requirements[start..]),
    );
    project.git(&["add", "--all"]);
    project.git(&[
        "commit",
        "-q",
        "-m",
        "move REQ-auth-002 beside a registered spec",
    ]);
    requirements
}

/// REQ-change-175's third criterion (corvid-agent on #1134): a canonical requirement moved into a
/// newly canonical file is found there, and once dropped from it, it is reported.
#[test]
fn a_requirement_moved_into_a_newly_canonical_file_is_found_and_reported_once_dropped() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    move_auth_002_into_registered_notes(&project);

    let moved = combined(&project.invoke(&["change", "ship-status", &id]));
    assert!(!moved.contains("lost canonical requirements"), "{moved}");

    project.write("docs/notes/requirements.md", "# Requirements\n");
    project.git(&["commit", "-q", "-am", "drop REQ-auth-002"]);
    let dropped = combined(&project.invoke(&["change", "ship-status", &id]));
    assert!(
        dropped.contains("that no approved `## REMOVED` delta names")
            && dropped.contains("REQ-auth-002"),
        "{dropped}"
    );
}

/// REQ-change-176's second criterion (corvid-agent on #1134): a delta body edited after approval
/// does not keep the audit from reporting the branch's loss.
#[test]
fn the_audit_reports_a_lost_requirement_beside_an_edited_delta_body() {
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    project.write(
        &format!(".specsync/changes/{id}/deltas/auth.md"),
        &REWORD_DELTA.replace("exactly one session", "exactly one active session"),
    );
    lose_auth_002(&project);

    let text = combined(&project.invoke(&["change", "audit"]));

    assert!(
        text.contains("that no approved `## REMOVED` delta names") && text.contains("REQ-auth-002"),
        "{text}"
    );
}

/// Codex and corvid-agent on #1134 (REQ-change-177): a registered spec's directory the scan
/// cannot look at, because the directory above it cannot be read, is named too.
#[cfg(unix)]
#[test]
fn a_scope_directory_below_an_unreadable_directory_is_named_instead_of_a_loss() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    let id = project.approved_change(REWORD_DELTA);
    project.git(&["push", "-q", "origin", "main"]);
    project.git(&["checkout", "-q", "-b", "change"]);
    project.run(&["change", "check", &id, "--commit"]);
    move_auth_002_into_registered_notes(&project);
    let docs = project.root.join("docs");
    fs::set_permissions(&docs, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::symlink_metadata(docs.join("notes")).is_ok() {
        // This user reads every directory (root), so nothing here is unreadable.
        fs::set_permissions(&docs, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }

    let status = project.invoke(&["change", "ship-status", &id]);
    fs::set_permissions(&docs, fs::Permissions::from_mode(0o755)).unwrap();
    let text = combined(&status);

    assert!(
        text.contains("cannot tell whether this branch lost canonical requirements"),
        "{text}"
    );
    assert!(text.contains("docs/notes"), "{text}");
    assert!(
        !text.contains("that no approved `## REMOVED` delta names"),
        "{text}"
    );
}
