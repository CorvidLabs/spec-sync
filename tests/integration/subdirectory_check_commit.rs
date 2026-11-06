//! Audit D2 (#879). A SpecSync project rooted in `app/` below its Git
//! repository root must be able to run `change check --commit` on a change
//! whose source and materialized spec live in the project (REQ-change-128).

use crate::helpers::{SUBSTANTIVE_PLAN, specsync};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const ID: &str = "add-an-auth-probe";

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
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn run_ok(root: &Path, args: &[&str]) -> String {
    let output = specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "specsync {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn check_commit_completes_for_a_project_rooted_in_a_repository_subdirectory() {
    let temp = TempDir::new().unwrap();
    let repository = temp.path();
    git(repository, &["init", "-q", "-b", "main"]);
    git(repository, &["config", "user.email", "test@example.com"]);
    git(repository, &["config", "user.name", "Test"]);
    fs::write(repository.join("README.md"), "repository root\n").unwrap();

    let root = repository.join("app");
    for dir in [".specsync", "src", "specs/auth"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".specsync/config.toml"),
        "enforcement = \"strict\"\n",
    )
    .unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
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
    )
    .unwrap();
    git(repository, &["add", "--all"]);
    git(repository, &["commit", "-q", "-m", "base"]);
    assert_eq!(git(&root, &["rev-parse", "--show-prefix"]), "app/");

    run_ok(
        &root,
        &[
            "change",
            "new",
            "Add an auth probe",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "The auth probe is documented"),
        ("public_contract", "yes"),
        ("architecture_risk", "no"),
        ("constraints", "none"),
    ] {
        run_ok(&root, &["change", "answer", ID, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(ID);
    let record: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in record["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "tasks" => "# Tasks\n\n- [x] Add the probe.\n",
            "plan" => SUBSTANTIVE_PLAN,
            "testing" => "# Testing\n\nThe integration test proves the auth probe is documented.\n",
            _ => "# Complete\n\nReviewed probe evidence.\n",
        };
        fs::write(workspace.join(format!("{name}.md")), content).unwrap();
    }
    fs::write(
        workspace.join("deltas/auth.md"),
        "## MODIFIED\n\n### SPEC SECTION Invariants\n\nStable and probed.\n",
    )
    .unwrap();
    run_ok(&root, &["change", "approve", ID, "--actor", "Reviewer"]);
    fs::write(
        root.join("src/auth.rs"),
        "// Authentication module, probed.\n",
    )
    .unwrap();
    git(repository, &["add", "--all"]);
    git(repository, &["commit", "-q", "-m", "implement"]);
    let before = git(repository, &["rev-parse", "HEAD"]);

    run_ok(&root, &["change", "check", ID, "--commit"]);

    assert_ne!(
        git(repository, &["rev-parse", "HEAD"]),
        before,
        "nothing was committed"
    );
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec.contains("Stable and probed."),
        "delta not materialized:\n{spec}"
    );
    assert_eq!(
        git(repository, &["status", "--porcelain", "--", "app/specs"]),
        "",
        "materialized spec left uncommitted"
    );
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["state"], "verifying");
}
