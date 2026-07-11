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
  "verification_commands": ["false"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    specsync()
        .env("CI", "true")
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "CI verification command `false` failed",
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
