use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn setup_active_change(root: &Path, verification_commands: &[&str]) -> PathBuf {
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "enabled": true,
            "require_change_for_meaningful_files": false,
            "meaningful_paths": [],
            "ignored_paths": [],
            "verification_commands": verification_commands,
            "custom_artifacts": {},
            "principles_file": null
        }))
        .unwrap(),
    )
    .unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Harden CI evidence",
            "--kind",
            "bug-fix",
            "--path",
            "src/lib.rs",
            "--no-spec-change",
            "--rationale",
            "Internal verification hardening",
        ])
        .assert()
        .success();
    let id = "CHG-0001-harden-ci-evidence";
    for (question, answer) in [
        ("acceptance_criteria", "CI rejects missing evidence"),
        ("public_contract", "no"),
        ("architecture_risk", "yes"),
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
    let change_dir = root.join(".specsync/changes").join(id);
    let state_path = change_dir.join("state.json");
    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let body = if name == "tasks" {
            "# Tasks\n\n- [x] Complete\n"
        } else {
            "# Complete\n\nReviewed.\n"
        };
        fs::write(change_dir.join(format!("{name}.md")), body).unwrap();
    }
    for command in ["approve", "start"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .success();
    }
    state_path
}

#[test]
fn comment_reports_sdd_only_failures() {
    let temp = TempDir::new().unwrap();
    let root = setup_minimal_project(&temp);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();

    specsync()
        .arg("comment")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("SpecSync: Failed"))
        .stdout(predicate::str::contains(".specsync/sdd.json"))
        .stdout(predicate::str::contains("invalid SDD policy"));
}

#[test]
fn comment_reports_sdd_failures_when_no_specs_exist() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();

    specsync()
        .arg("comment")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("SpecSync: Failed"))
        .stdout(predicate::str::contains("invalid SDD policy"))
        .stdout(predicate::str::contains("No spec files found").not());
}

#[test]
fn ci_check_requires_persisted_verification_evidence() {
    let temp = TempDir::new().unwrap();
    let root = setup_minimal_project(&temp);
    let state_path = setup_active_change(&root, &[]);
    let change_dir = state_path.parent().unwrap();
    let mut state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["state"] = serde_json::Value::String("verifying".into());
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();

    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("verification evidence is missing"));

    fs::write(
        change_dir.join("verification.json"),
        r#"{
  "timestamp": 0,
  "commit": null,
  "contract_digest": "stale",
  "workspace_digest": "stale",
  "passed": false,
  "commands": [],
  "requirement_ids": []
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
            "latest verification evidence failed",
        ));
}

#[test]
fn comment_suppresses_configured_command_output_but_check_streams_it() {
    let temp = TempDir::new().unwrap();
    let root = setup_minimal_project(&temp);
    setup_active_change(&root, &["rustc --version --verbose"]);

    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "comment"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SpecSync:"))
        .stdout(predicate::str::contains("binary: rustc").not());

    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("binary: rustc"));
}

#[test]
fn ci_captures_comment_with_quiet_cargo_and_a_defensive_byte_cap() {
    let workflow =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml"))
            .unwrap();

    assert!(workflow.contains("BODY=$(cargo run --quiet -- comment 2>/dev/null) || true"));
    assert!(workflow.contains("limit = 49152"));
    assert!(workflow.contains("decode(\"utf-8\", \"ignore\")"));
    assert!(workflow.contains("secrets.token_hex(16)"));
    assert!(workflow.contains("grep -Fqx \"$delimiter\""));
    assert!(!workflow.contains("body<<EOF"));
    assert!(!workflow.contains("context<<CONTEXT_EOF"));
}
