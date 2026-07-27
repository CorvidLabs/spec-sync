use super::helpers::{specsync, valid_spec};
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn verification_freshness_status_and_check_are_environment_independent() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::write(root.join("seed.txt"), "seed\n").unwrap();
    git(&["add", "seed.txt"]);
    git(&["commit", "-m", "seed"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/lifecycle")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    fs::write(
        root.join("specs/lifecycle/lifecycle.spec.md"),
        valid_spec("lifecycle", &["src/lib.rs"]),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "enabled": true,
            "require_change_for_meaningful_files": false,
            "meaningful_paths": ["src/", ".specsync/sdd.json"],
            "ignored_paths": [".specsync/"],
            "verification_commands": [],
            "custom_artifacts": {},
            "principles_file": null
        }))
        .unwrap(),
    )
    .unwrap();
    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Harden verification freshness",
            "--kind",
            "bug-fix",
            "--spec",
            "lifecycle",
            "--path",
            "src/lib.rs",
            "--no-spec-change",
            "--rationale",
            "Internal lifecycle behavior only",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();
    for (question, answer) in [
        ("acceptance_criteria", "Verification freshness is portable"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let shown = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "show",
            id,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let shown: Value = serde_json::from_slice(&shown).unwrap();
    for artifact in shown["change"]["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = if name == "tasks" {
            "# Tasks\n\n- [x] Complete verification preparation.\n"
        } else {
            "# Complete\n\nReviewed lifecycle evidence.\n"
        };
        fs::write(
            root.join(format!(".specsync/changes/{id}/{name}.md")),
            content,
        )
        .unwrap();
    }
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Reviewer",
        ])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "start", id])
        .assert()
        .success();
    git(&["add", "--all"]);
    git(&["commit", "-m", "implement"]);
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", id])
        .assert()
        .success();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Reviewer",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "invalidates the prior verification",
        ));
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", id])
        .assert()
        .success();
    for name in [
        "state.json",
        "verification.json",
        "verification-attempts.json",
    ] {
        git(&["add", &format!(".specsync/changes/{id}/{name}")]);
    }
    git(&["commit", "-m", "persist verification"]);

    let assert_surfaces = |expected_action: &str, check_succeeds: bool, environment: &str| {
        let configure = |command: &mut assert_cmd::Command| {
            command
                .env_remove("CI")
                .env_remove("GITHUB_ACTIONS")
                .env_remove("GITHUB_WORKSPACE");
            match environment {
                "ci" => {
                    command.env("CI", "true");
                }
                "github" => {
                    command
                        .env("GITHUB_ACTIONS", "true")
                        .env("GITHUB_WORKSPACE", root);
                }
                _ => {}
            }
        };
        let mut status = specsync();
        configure(&mut status);
        let output = status
            .args([
                "--root",
                root.to_str().unwrap(),
                "--json",
                "change",
                "status",
                id,
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["summary"]["next_action"], expected_action);
        let mut check = specsync();
        configure(&mut check);
        let assertion = check
            .args(["--root", root.to_str().unwrap(), "change", "check"])
            .assert();
        if check_succeeds {
            assertion.success();
        } else {
            assertion.failure();
        }
    };
    for environment in ["local", "ci", "github"] {
        assert_surfaces("accept", true, environment);
    }
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "change governed input"]);
    for environment in ["local", "ci", "github"] {
        assert_surfaces("verify", false, environment);
    }
}

#[test]
fn change_new_json_returns_state_and_interview() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("specs/auth")).unwrap();
    fs::write(
        temp.path().join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles: []\n---\n",
    )
    .unwrap();
    let output = specsync()
        .args([
            "--root",
            temp.path().to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Add passkeys",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["change"]["state"], "draft");
    assert_eq!(value["change"]["id"], "CHG-0001-add-passkeys");
    assert!(value["questions"].as_array().unwrap().len() >= 3);
    assert!(
        temp.path()
            .join(".specsync/changes/CHG-0001-add-passkeys/change.md")
            .is_file()
    );
}

#[test]
fn change_new_rejects_a_nonexistent_spec_before_allocating_state() {
    let temp = TempDir::new().unwrap();
    specsync()
        .args([
            "--root",
            temp.path().to_str().unwrap(),
            "change",
            "new",
            "Reference a missing module",
            "--spec",
            "missing",
            "--path",
            "src/missing.rs",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
    assert!(!temp.path().join(".specsync/changes").exists());
    assert!(!temp.path().join(".specsync/change-sequence.json").exists());
}

#[test]
fn change_without_specs_requires_rationale() {
    let temp = TempDir::new().unwrap();
    specsync()
        .args([
            "--root",
            temp.path().to_str().unwrap(),
            "change",
            "new",
            "Update CI",
            "--no-spec-change",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --rationale"));
}

#[test]
fn adopt_dry_run_does_not_enable_policy() {
    let temp = TempDir::new().unwrap();
    specsync()
        .args([
            "--root",
            temp.path().to_str().unwrap(),
            "change",
            "adopt",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("enable SDD policy"));
    assert!(!temp.path().join(".specsync/sdd.json").exists());
}

#[test]
fn adopt_existing_4x_project_is_idempotent() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(root.join(".specsync/version"), "4.3.2\n").unwrap();
    fs::write(
        root.join("specs/auth/requirements.md"),
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\nUsers can sign in.\n",
    )
    .unwrap();
    for _ in 0..2 {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", "adopt"])
            .assert()
            .success();
    }
    assert!(root.join(".specsync/sdd.json").is_file());
    let report: Value = serde_json::from_str(
        &fs::read_to_string(root.join(".specsync/adoption-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        report["requirements_needing_ids"][0]["suggested_first_id"],
        "REQ-auth-001"
    );
    assert_eq!(
        fs::read_to_string(root.join(".specsync/config.toml")).unwrap(),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n"
    );
}

#[test]
fn adopt_openspec_imports_canonical_and_active_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
    fs::create_dir_all(root.join("openspec/changes/add-passkeys")).unwrap();
    fs::write(
        root.join("openspec/specs/auth/spec.md"),
        "# Authentication\n\nCanonical OpenSpec contract.\n",
    )
    .unwrap();
    fs::write(
        root.join("openspec/changes/add-passkeys/proposal.md"),
        "# Add passkeys\n\nActive proposal.\n",
    )
    .unwrap();
    for _ in 0..2 {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "adopt",
                "--source",
                "openspec",
            ])
            .assert()
            .success();
    }
    assert!(
        root.join(".specsync/imports/openspec/canonical/auth/spec.md")
            .is_file()
    );
    let changes: Value = serde_json::from_slice(
        &specsync()
            .args(["--root", root.to_str().unwrap(), "--json", "change", "list"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(changes.as_array().unwrap().len(), 1);
}

#[test]
fn adopt_speckit_imports_constitution_and_active_feature_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specify/memory")).unwrap();
    fs::create_dir_all(root.join("specs/001-passkeys")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join(".specify/memory/constitution.md"),
        "# Constitution\n\nSafety first.\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/001-passkeys/spec.md"),
        "# Passkeys\n\nActive Spec Kit feature.\n",
    )
    .unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), "# Native auth spec\n").unwrap();
    fs::write(root.join("specs/auth/tasks.md"), "# Native auth tasks\n").unwrap();
    for _ in 0..2 {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "adopt",
                "--source",
                "speckit",
            ])
            .assert()
            .success();
    }
    assert!(
        root.join(".specsync/imports/speckit/constitution.md")
            .is_file()
    );
    assert_eq!(
        root.join(".specsync/changes")
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .count(),
        1
    );
}

#[test]
fn init_enables_sdd_for_new_projects() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    specsync()
        .args(["--root", temp.path().to_str().unwrap(), "init"])
        .assert()
        .success();
    let policy: Value =
        serde_json::from_str(&fs::read_to_string(temp.path().join(".specsync/sdd.json")).unwrap())
            .unwrap();
    assert_eq!(policy["enabled"], true);
    assert!(temp.path().join(".specsync/archive/changes").is_dir());
}

#[test]
fn no_spec_change_completes_full_cli_lifecycle() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["commit", "--allow-empty", "-m", "base"]);
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Update contributor documentation",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only wording does not alter a technical contract",
        ])
        .assert()
        .success();
    let id = "CHG-0001-update-contributor-documentation";
    for (question, answer) in [
        (
            "acceptance_criteria",
            "Contributors can follow the updated workflow",
        ),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let dir = root.join(".specsync/changes").join(id);
    fs::write(
        dir.join("context.md"),
        "# Context\n\nDocumentation update.\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs.md"),
        "# Docs\n\nReviewed contributor copy.\n",
    )
    .unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .success();
    }
    git(&["add", "."]);
    git(&["commit", "-m", "record accepted lifecycle evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": ["cargo metadata --manifest-path definitely-missing/Cargo.toml"],
  "custom_artifacts": {},
  "principles_file": null
}

"#,
    )
    .unwrap();
    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "CI verification command `cargo metadata --manifest-path definitely-missing/Cargo.toml` failed",
        ));
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "archive", id])
        .assert()
        .success();
    assert!(!dir.exists());
    assert!(
        root.join(".specsync/archive/changes")
            .read_dir()
            .unwrap()
            .any(|entry| entry.unwrap().file_name().to_string_lossy().ends_with(id))
    );
}

#[test]
fn change_list_surfaces_malformed_workspaces_and_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("README.md"), "Docs.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Update docs",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only change",
        ])
        .assert()
        .success();
    let bogus = root.join(".specsync/changes/CHG-9999-bogus");
    fs::create_dir_all(&bogus).unwrap();
    fs::write(bogus.join("state.json"), b"{ not json\n").unwrap();
    // The healthy change is still listed, the corrupt workspace is named on
    // stderr with a remediation, and the exit code is non-zero.
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "list"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("CHG-0001-update-docs"))
        .stderr(predicate::str::contains("CHG-9999-bogus"))
        .stderr(predicate::str::contains("repair or remove"));
    let output = specsync()
        .args(["--root", root.to_str().unwrap(), "--json", "change", "list"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["valid"], false);
    assert_eq!(value["changes"].as_array().unwrap().len(), 1);
    assert_eq!(value["errors"].as_array().unwrap().len(), 1);
}

#[test]
fn change_status_prints_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("README.md"), "Docs.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Update docs",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only change",
        ])
        .assert()
        .success();
    let id = "CHG-0001-update-docs";
    let state_path = root.join(".specsync/changes").join(id).join("state.json");
    let mut state: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["dependencies"] = serde_json::json!(["CHG-0000-predecessor"]);
    fs::write(&state_path, serde_json::to_string_pretty(&state).unwrap()).unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "status", id])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dependencies: CHG-0000-predecessor",
        ));
}

#[test]
fn change_status_fails_on_corrupt_archived_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["commit", "--allow-empty", "-m", "base"]);
    fs::write(root.join("README.md"), "Docs.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Update docs",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only change",
        ])
        .assert()
        .success();
    let id = "CHG-0001-update-docs";
    for (question, answer) in [
        ("acceptance_criteria", "Docs stay accurate"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let dir = root.join(".specsync/changes").join(id);
    fs::write(dir.join("context.md"), "# Context\n\nDocs.\n").unwrap();
    fs::write(dir.join("docs.md"), "# Docs\n\nDocs.\n").unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .success();
    }
    git(&["add", "."]);
    git(&["commit", "-m", "record accepted lifecycle evidence"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "archive", id])
        .assert()
        .success();
    let archive_dir = root
        .join(".specsync/archive/changes")
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.file_name().unwrap().to_string_lossy().ends_with(id))
        .unwrap();
    // Corrupt the authenticated accepted-state snapshot: status must fail
    // loudly instead of rendering the archive as healthy.
    fs::write(archive_dir.join("accepted-state.json"), b"{}\n").unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "status", id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("corrupt archived evidence"));
}

#[test]
fn change_supersede_persists_an_exact_predecessor_obligation_through_the_cli() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/auth.rs"),
        "pub fn authenticate() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n| `authenticate` | Return whether authentication succeeds |\n\n## Invariants\n\nAuthentication is available.\n\n## Behavioral Examples\n\nValid users authenticate.\n\n## Error Cases\n\nInvalid users fail.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/requirements.md"),
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nAuthentication SHALL remain available.\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Govern authentication",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success();
    let predecessor = "CHG-0001-govern-authentication";
    for (question, answer) in [
        ("acceptance_criteria", "Authentication remains governed"),
        ("public_contract", "yes"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                predecessor,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let predecessor_dir = root.join(".specsync/changes").join(predecessor);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(predecessor_dir.join("state.json")).unwrap())
            .unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = if name == "tasks" {
            "# Tasks\n\n- [x] Complete predecessor preparation.\n"
        } else {
            "# Complete\n\nReviewed predecessor evidence.\n"
        };
        fs::write(predecessor_dir.join(format!("{name}.md")), content).unwrap();
    }
    fs::write(
        predecessor_dir.join("deltas/auth.md"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n",
    )
    .unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                command,
                predecessor,
            ])
            .assert()
            .success();
    }
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let verification: Value = serde_json::from_str(
        &fs::read_to_string(predecessor_dir.join("verification.json")).unwrap(),
    )
    .unwrap();
    let digest = verification["acceptance_manifest"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "src/auth.rs")
        .unwrap()["entry_digest"]
        .as_str()
        .unwrap()
        .to_string();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Evolve authentication",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success();
    let successor = "CHG-0002-evolve-authentication";
    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "supersede",
            successor,
            predecessor,
            "--path",
            "src/auth.rs",
            "--spec",
            "auth",
            "--digest",
            &digest,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let persisted: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        persisted["change"]["supersedes"][0]["predecessor_id"],
        predecessor
    );
    assert_eq!(
        persisted["change"]["supersedes"][0]["obligations"][0]["path"],
        "src/auth.rs"
    );
    assert_eq!(
        persisted["change"]["supersedes"][0]["obligations"][0]["module"],
        "auth"
    );
    assert_eq!(
        persisted["change"]["supersedes"][0]["obligations"][0]["predecessor_entry_digest"],
        digest
    );
}

#[test]
fn stale_accepted_change_reopens_through_cli_with_deterministic_audit_json() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("README.md"), "Initial review instructions.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Update review instructions",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only review guidance",
        ])
        .assert()
        .success();
    let id = "CHG-0001-update-review-instructions";
    for (question, answer) in [
        (
            "acceptance_criteria",
            "Reviewers can follow the release workflow",
        ),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let dir = root.join(".specsync/changes").join(id);
    fs::write(dir.join("context.md"), "# Context\n\nRelease review.\n").unwrap();
    fs::write(dir.join("docs.md"), "# Docs\n\nReview instructions.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Definition reviewer",
        ])
        .assert()
        .success();
    for command in ["start", "verify"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .success();
    }
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "accept",
            id,
            "--actor",
            "Closing reviewer",
        ])
        .assert()
        .success();

    fs::write(root.join("README.md"), "Final review instructions.\n").unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "accepted change verification is stale for current delivery inputs",
        ))
        .stderr(predicate::str::contains(
            "exact-only delivery input `README.md` changed after acceptance and requires an audited reopen; run `specsync change reopen CHG-0001-update-review-instructions --actor <name> --reason <why>` to re-verify the accepted change",
        ));
    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "reopen",
            id,
            "--actor",
            "Release reviewer",
            "--reason",
            "Final review changed governed delivery inputs",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["change"]["state"], "verifying");
    assert_eq!(value["audit"]["schema_version"], 1);
    assert_eq!(value["audit"]["from_state"], "accepted");
    assert_eq!(value["audit"]["to_state"], "verifying");
    assert_eq!(
        value["audit"]["superseded_approval"]["actor"],
        "Closing reviewer"
    );
    assert_eq!(
        value["audit"]["reason"],
        "Final review changed governed delivery inputs"
    );
    let ledger: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("approvals.json")).unwrap()).unwrap();
    assert_eq!(ledger["approvals"].as_array().unwrap().len(), 2);
    assert_eq!(ledger["reopenings"].as_array().unwrap().len(), 1);
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("verification evidence is stale"));

    let docs_path = dir.join("docs.md");
    let accepted_docs = fs::read_to_string(&docs_path).unwrap();
    fs::write(
        &docs_path,
        "# Docs\n\nA modified definition cannot be silently ignored.\n",
    )
    .unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Definition reviewer",
        ])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", id])
        .assert()
        .success();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "accept",
            id,
            "--actor",
            "Closing reviewer",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "perform further spec changes in a new change workspace",
        ));
    let rejected_ledger: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("approvals.json")).unwrap()).unwrap();
    assert_eq!(rejected_ledger["approvals"].as_array().unwrap().len(), 3);

    fs::write(&docs_path, accepted_docs).unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Definition reviewer",
        ])
        .assert()
        .success();

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", id])
        .assert()
        .success();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "accept",
            id,
            "--actor",
            "Closing reviewer",
        ])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .success();
    let ledger: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("approvals.json")).unwrap()).unwrap();
    assert_eq!(ledger["approvals"].as_array().unwrap().len(), 5);
    assert_eq!(ledger["reopenings"].as_array().unwrap().len(), 1);
}

// Verifies REQ-change-033, REQ-cli-args-001, and REQ-cmd-change-001.
#[test]
fn reopened_owner_correction_is_deterministic_through_json_cli() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": ["src/"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    let spec = |module: &str| {
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
        )
    };
    fs::write(root.join("specs/legacy/legacy.spec.md"), spec("legacy")).unwrap();
    fs::write(root.join("specs/current/current.spec.md"), spec("current")).unwrap();

    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Preserve historical input ownership",
            "--kind",
            "bug-fix",
            "--spec",
            "legacy",
            "--path",
            "src/lib.rs",
            "--no-spec-change",
            "--rationale",
            "Internal ownership evidence only",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();
    for (question, answer) in [
        ("acceptance_criteria", "Exact ownership is signed"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let dir = root.join(".specsync/changes").join(id);
    fs::write(dir.join("context.md"), "# Context\n\nComplete.\n").unwrap();
    fs::write(dir.join("testing.md"), "# Testing\n\nComplete.\n").unwrap();
    fs::write(dir.join("tasks.md"), "# Tasks\n\n- [x] Complete.\n").unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        let mut args = vec!["--root", root.to_str().unwrap(), "change", command, id];
        if matches!(command, "approve" | "accept") {
            args.extend(["--actor", "Lifecycle reviewer"]);
        }
        specsync().args(args).assert().success();
    }

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "reopen",
            id,
            "--actor",
            "Release reviewer",
            "--reason",
            "The accepted source changed during release review",
        ])
        .assert()
        .success();
    let corrected = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "correct-owner",
            id,
            "--path",
            "src/lib.rs",
            "--spec",
            "current",
            "--actor",
            "Release reviewer",
            "--reason",
            "The historical definition omitted the current canonical owner",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let corrected: Value = serde_json::from_slice(&corrected).unwrap();
    assert_eq!(corrected["state"], "verifying");
    assert_eq!(corrected["acceptance_owner_corrections"][0]["sequence"], 1);
    assert_eq!(
        corrected["acceptance_owner_corrections"][0]["module"],
        "current"
    );

    for command in ["approve", "verify", "accept"] {
        let mut args = vec!["--root", root.to_str().unwrap(), "change", command, id];
        if matches!(command, "approve" | "accept") {
            args.extend(["--actor", "Lifecycle reviewer"]);
        }
        specsync().args(args).assert().success();
    }
    let verification: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("verification.json")).unwrap()).unwrap();
    let source = verification["acceptance_manifest"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "src/lib.rs")
        .unwrap();
    assert_eq!(source["owners"], serde_json::json!(["current", "legacy"]));
}

// Verifies REQ-change-039, REQ-cli-args-006, and REQ-cmd-change-004.
#[test]
fn batch_correct_owner_through_cli_is_transactional() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/legacy")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": ["src/"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() {}\n").unwrap();
    fs::write(root.join("src/b.rs"), "pub fn b() {}\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    let owned = |module: &str| {
        format!(
            "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/a.rs\n  - src/b.rs\n  - src/lib.rs\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
        )
    };
    fs::write(root.join("specs/legacy/legacy.spec.md"), owned("legacy")).unwrap();
    fs::write(root.join("specs/current/current.spec.md"), owned("current")).unwrap();
    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Batch owner correction fixture",
            "--kind",
            "bug-fix",
            "--spec",
            "legacy",
            "--path",
            "src/a.rs",
            "--path",
            "src/b.rs",
            "--path",
            "src/lib.rs",
            "--no-spec-change",
            "--rationale",
            "ownership evidence only",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();
    let dir = root.join(".specsync/changes").join(id);
    for (question, answer) in [
        ("acceptance_criteria", "Batch correct-owner works"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    fs::write(dir.join("context.md"), "# Context\n\nComplete.\n").unwrap();
    fs::write(dir.join("testing.md"), "# Testing\n\nComplete.\n").unwrap();
    fs::write(dir.join("tasks.md"), "# Tasks\n\n- [x] Complete.\n").unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        let mut args = vec!["--root", root.to_str().unwrap(), "change", command, id];
        if matches!(command, "approve" | "accept") {
            args.extend(["--actor", "Lifecycle reviewer"]);
        }
        specsync().args(args).assert().success();
    }

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "reopen",
            id,
            "--actor",
            "Release reviewer",
            "--reason",
            "The accepted source changed during release review",
        ])
        .assert()
        .success();

    let before = fs::read(dir.join("state.json")).unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "correct-owner",
            id,
            "--path",
            "src/a.rs",
            "--path",
            "outside.rs",
            "--spec",
            "current",
            "--actor",
            "Release reviewer",
            "--reason",
            "Partial batch must not apply",
        ])
        .assert()
        .failure();
    assert_eq!(fs::read(dir.join("state.json")).unwrap(), before);

    let corrected = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "correct-owner",
            id,
            "--path",
            "src/a.rs",
            "--path",
            "src/b.rs",
            "--path",
            "src/lib.rs",
            "--spec",
            "current",
            "--actor",
            "Release reviewer",
            "--reason",
            "Batch repair omitted current owners",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let corrected: Value = serde_json::from_slice(&corrected).unwrap();
    assert_eq!(
        corrected["acceptance_owner_corrections"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

// Verifies REQ-change-032, REQ-cli-args-004, and REQ-cmd-change-002.
#[test]
fn accepted_metadata_corrects_through_cli_with_effective_text_and_json_views() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(root.join("README.md"), "Lifecycle guidance.\n").unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Correct lifecycle classification",
            "--kind",
            "bug-fix",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "The accepted implementation is unchanged",
        ])
        .assert()
        .success();
    let id = "CHG-0001-correct-lifecycle-classification";
    for (question, answer) in [
        ("acceptance_criteria", "Correction evidence is inspectable"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let dir = root.join(".specsync/changes").join(id);
    fs::write(dir.join("context.md"), "# Context\n\nComplete.\n").unwrap();
    fs::write(
        dir.join("testing.md"),
        "# Testing\n\nCorrection evidence is inspectable.\n",
    )
    .unwrap();
    fs::write(dir.join("tasks.md"), "# Tasks\n\n- [x] Complete.\n").unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        let mut args = vec!["--root", root.to_str().unwrap(), "change", command, id];
        if matches!(command, "approve" | "accept") {
            args.extend(["--actor", "Lifecycle reviewer"]);
        }
        specsync().args(args).assert().success();
    }

    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "correct",
            id,
            "architecture_risk",
            "yes",
            "--actor",
            "Release reviewer",
            "--reason",
            "The lifecycle implementation affects persisted architecture",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["change"]["state"], "verifying");
    assert_eq!(value["change"]["answers"]["architecture_risk"], "no");
    assert_eq!(value["change"]["correction_count"], 1);
    assert_eq!(value["correction"]["original_value"], "no");
    assert_eq!(value["correction"]["prior_effective_value"], "no");
    assert_eq!(value["correction"]["corrected_value"], "yes");
    assert_eq!(value["correction"]["actor"], "Release reviewer");
    assert!(
        !value["correction"]["prior_view_digest"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert!(
        !value["correction"]["corrected_view_digest"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        value["effective_definition"]["answers"]["architecture_risk"],
        "yes"
    );
    assert_eq!(value["summary"]["next_action"], "complete artifacts");
    assert_eq!(value["corrections"].as_array().unwrap().len(), 1);

    for artifact in ["research.md", "design.md", "plan.md"] {
        fs::write(
            dir.join(artifact),
            format!("# {}\n\nComplete.\n", artifact.trim_end_matches(".md")),
        )
        .unwrap();
    }
    let show_output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "show",
            id,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show: Value = serde_json::from_slice(&show_output).unwrap();
    assert_eq!(
        show["effective_definition"]["answers"]["architecture_risk"],
        "yes"
    );
    assert_eq!(
        show["corrections"][0]["reason"],
        "The lifecycle implementation affects persisted architecture"
    );
    assert_eq!(show["summary"]["next_action"], "approve");

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "Definition reviewer",
        ])
        .assert()
        .success();
    let status = specsync()
        .args(["--root", root.to_str().unwrap(), "change", "status", id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status = String::from_utf8(status).unwrap();
    assert!(status.contains("architecture_risk: no → yes by Release reviewer"));
    assert!(status.contains("architecture_risk=yes"));
    assert!(status.contains("Next: verify"));

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", id])
        .assert()
        .success();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "accept",
            id,
            "--actor",
            "Closing reviewer",
        ])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .success();

    let state: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["answers"]["architecture_risk"], "no");
    assert_eq!(state["correction_count"], 1);
    let corrections: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("corrections.json")).unwrap()).unwrap();
    assert_eq!(corrections["corrections"].as_array().unwrap().len(), 1);
}

// Verifies REQ-cli-004.
#[test]
fn indirect_recursive_lifecycle_check_fails_once_with_context() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("specsync.json"), "{}\n").unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();

    let stderr = specsync()
        .env("SPECSYNC_VERIFICATION_CONTEXT", "fledge lanes run verify")
        .args(["--root", root.to_str().unwrap(), "check", "--strict"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(stderr).unwrap();
    assert_eq!(
        stderr.matches("recursive lifecycle verification").count(),
        1
    );
    assert!(stderr.contains("fledge lanes run verify"));
    assert!(!stderr.contains("Legacy 3.x layout detected"));
}

#[test]
fn indirect_recursive_lifecycle_subcommands_fail_once_with_context() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    for args in [
        vec!["--root", root.to_str().unwrap(), "change", "list"],
        vec!["--root", root.to_str().unwrap(), "lifecycle", "enforce"],
    ] {
        let stderr = specsync()
            .env("SPECSYNC_VERIFICATION_CONTEXT", "fledge run verify")
            .args(args)
            .assert()
            .failure()
            .get_output()
            .stderr
            .clone();
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(
            stderr.matches("recursive lifecycle verification").count(),
            1,
            "{stderr}"
        );
        assert!(stderr.contains("fledge run verify"));
    }
}

// Verifies REQ-cmd-migrate-002 and REQ-cli-args-007.
#[test]
fn migrate_5_0_backfills_reopening_digest_fields_idempotently() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let workspace = root.join(".specsync/changes/CHG-0001-adopt-trust");
    fs::create_dir_all(&workspace).unwrap();
    let reopening = serde_json::json!({
        "schema_version": 1,
        "change_id": "CHG-0001-adopt-trust",
        "actor": "0xLeif",
        "reason": "5.0.1-era reopen",
        "timestamp": 100,
        "from_state": "accepted",
        "to_state": "verifying",
        "superseded_approval": {
            "gate": "acceptance",
            "actor": "0xLeif",
            "timestamp": 90,
            "digest": "aaa",
            "note": null
        },
        "prior_verification": {
            "timestamp": 80,
            "commit": null,
            "contract_digest": "ccc",
            "workspace_digest": "www",
            "acceptance_input_digest": "stale-digest-aaa",
            "passed": true,
            "commands": [],
            "requirement_ids": []
        }
    });
    let ledger = serde_json::json!({
        "approvals": [],
        "reopenings": [reopening]
    });
    let approvals_path = workspace.join("approvals.json");
    fs::write(
        &approvals_path,
        format!("{}\n", serde_json::to_string_pretty(&ledger).unwrap()),
    )
    .unwrap();
    fs::write(
        workspace.join("verification.json"),
        "{\n  \"timestamp\": 200,\n  \"commit\": null,\n  \"contract_digest\": \"ccc\",\n  \"workspace_digest\": \"www\",\n  \"acceptance_input_digest\": \"current-digest-bbb\",\n  \"passed\": true,\n  \"commands\": [],\n  \"requirement_ids\": []\n}\n",
    )
    .unwrap();

    // Unknown source families fail through deterministic Clap validation before any mutation.
    specsync()
        .args(["migrate", "9.9"])
        .current_dir(root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));

    // Dry run reports the repair without writing.
    let before = fs::read(&approvals_path).unwrap();
    specsync()
        .args(["migrate", "5.0", "--dry-run"])
        .current_dir(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("would be backfilled"));
    assert_eq!(fs::read(&approvals_path).unwrap(), before);

    // The write restores exactly the recorded evidence.
    specsync()
        .args(["migrate", "5.0"])
        .current_dir(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("backfilled"));
    let repaired: Value =
        serde_json::from_str(&fs::read_to_string(&approvals_path).unwrap()).unwrap();
    assert_eq!(
        repaired["reopenings"][0]["stale_acceptance_input_digest"],
        "stale-digest-aaa"
    );
    assert_eq!(
        repaired["reopenings"][0]["current_acceptance_input_digest"],
        "current-digest-bbb"
    );

    // A second run is a no-op.
    let before = fs::read(&approvals_path).unwrap();
    specsync()
        .args(["migrate", "5.0"])
        .current_dir(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to migrate"));
    assert_eq!(fs::read(&approvals_path).unwrap(), before);
}

#[test]
fn change_supersede_resolves_the_entry_digest_automatically() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/auth.rs"),
        "pub fn authenticate() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n| `authenticate` | Return whether authentication succeeds |\n\n## Invariants\n\nAuthentication is available.\n\n## Behavioral Examples\n\nValid users authenticate.\n\n## Error Cases\n\nInvalid users fail.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/requirements.md"),
        "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nAuthentication SHALL remain available.\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Govern authentication",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success();
    let predecessor = "CHG-0001-govern-authentication";
    for (question, answer) in [
        ("acceptance_criteria", "Authentication remains governed"),
        ("public_contract", "yes"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                predecessor,
                question,
                answer,
            ])
            .assert()
            .success();
    }
    let predecessor_dir = root.join(".specsync/changes").join(predecessor);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(predecessor_dir.join("state.json")).unwrap())
            .unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = if name == "tasks" {
            "# Tasks\n\n- [x] Complete predecessor preparation.\n"
        } else {
            "# Complete\n\nReviewed predecessor evidence.\n"
        };
        fs::write(predecessor_dir.join(format!("{name}.md")), content).unwrap();
    }
    fs::write(
        predecessor_dir.join("deltas/auth.md"),
        "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n",
    )
    .unwrap();
    for command in ["approve", "start", "verify", "accept"] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                command,
                predecessor,
            ])
            .assert()
            .success();
    }
    git(&["add", "."]);
    git(&["commit", "-m", "accept predecessor"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let verification: Value = serde_json::from_str(
        &fs::read_to_string(predecessor_dir.join("verification.json")).unwrap(),
    )
    .unwrap();
    let digest = verification["acceptance_manifest"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "src/auth.rs")
        .unwrap()["entry_digest"]
        .as_str()
        .unwrap()
        .to_string();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Evolve authentication",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success();
    let successor = "CHG-0002-evolve-authentication";
    // No --digest: the exact entry digest is resolved from the predecessor.
    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "supersede",
            successor,
            predecessor,
            "--path",
            "src/auth.rs",
            "--spec",
            "auth",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let persisted: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        persisted["change"]["supersedes"][0]["obligations"][0]["predecessor_entry_digest"],
        digest
    );
}
