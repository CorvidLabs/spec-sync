use super::helpers::specsync;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
fn change_new_json_returns_state_and_interview() {
    let temp = TempDir::new().unwrap();
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
