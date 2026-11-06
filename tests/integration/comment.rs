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
    let id = "harden-ci-evidence";
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
        } else if name == "plan" {
            SUBSTANTIVE_PLAN
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
fn ci_check_requires_persisted_verification_evidence() {
    let temp = TempDir::new().unwrap();
    let root = setup_minimal_project(&temp);
    let state_path = setup_active_change(&root, &[]);
    let change_dir = state_path.parent().unwrap();
    let mut state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["state"] = serde_json::Value::String("verifying".into());
    fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();

    // Project-health audit inspects persisted verifying evidence.
    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
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
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "latest verification evidence failed",
        ));
}

#[test]
fn neither_comment_nor_change_check_spawns_configured_project_commands() {
    // Honest label: DISCRIMINATOR for change check. The unfixed binary spawned
    // `rustc --version --verbose` and printed `binary: rustc`. Comment is the
    // control: it already stayed silent on both binaries.
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
        .env_remove("CI")
        .env_remove("GITHUB_ACTIONS")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("binary: rustc").not());
}

// Verifies REQ-cmd-comment-003 and REQ-cmd-comment-004 at the contract level (#848,
// SHIP-1.c.1, SHIP-1.c.2).
//
// Honest label: CONTRACT PIN. The test above pins the behavior; this one fails when a current
// cmd_comment contract goes back to saying that `comment` executes a configured verification
// command, folds lifecycle failures into its verdict, or that `change check` streams one.
#[test]
fn comment_contracts_say_no_configured_verification_command_runs() {
    let requirements = "specs/cmd_comment/requirements.md";
    let protocol =
        tracked_requirement(requirements, "REQ-cmd-comment-003").expect("REQ-cmd-comment-003");
    assert!(
        protocol.contains("executes no configured SDD verification command"),
        "REQ-cmd-comment-003 must say why nothing reaches stdout but the report: {protocol}"
    );
    let no_folding =
        tracked_requirement(requirements, "REQ-cmd-comment-004").expect("REQ-cmd-comment-004");
    for clause in [
        "SHALL report spec-check results only and SHALL NOT fold SDD lifecycle findings into them.",
        "Lifecycle errors and warnings are absent from the reported error and warning totals.",
        "Lifecycle state does not contribute to the command's exit status.",
    ] {
        assert!(
            no_folding.contains(clause),
            "REQ-cmd-comment-004 is the rule that replaced lifecycle folding and must keep \
             `{clause}`: {no_folding}"
        );
    }
    for id in ["REQ-cmd-comment-001", "REQ-cmd-comment-002"] {
        assert!(
            tracked_requirement(requirements, id).is_none(),
            "{id} folds SDD lifecycle failures into the comment, which REQ-cmd-comment-004 \
             reversed, so it stays retired"
        );
    }
    let retired = [
        "still execute and fail closed in comment mode",
        "verification command output is captured away from stdout",
        "the command still executes and affects lifecycle status",
        "while `change check` streams it",
        "while preserving execution and failure reporting",
        "lifecycle checking still streams configured command output",
    ];
    for relative in [
        "specs/cmd_comment/requirements.md",
        "specs/cmd_comment/cmd_comment.spec.md",
        "specs/cmd_comment/context.md",
        "specs/cmd_comment/cmd_comment.3md",
        "specs/cmd_comment/testing.md",
        "specs/cmd_comment/tasks.md",
    ] {
        let text = tracked_text(relative);
        for phrase in retired {
            assert!(
                !text.contains(phrase),
                "{relative} still says a configured command runs: `{phrase}`"
            );
        }
    }
}

// Verifies REQ-cmd-comment-003 and REQ-cmd-comment-004 through the shipped binary (#848).
//
// Honest label: DISCRIMINATOR by side effect. The test above only shows the configured command's
// output is absent, which a regression to the retired "execute but suppress child output" path
// would also satisfy. Here the configured command creates a file, so running it at all leaves
// evidence. The control runs the same probe directly and requires the file, so the absence below
// is not explained by a probe that never works.
#[cfg(unix)]
#[test]
fn comment_does_not_spawn_a_configured_command_even_silently() {
    let temp = TempDir::new().unwrap();
    let root = setup_minimal_project(&temp);
    let sentinel = root.join("configured-command-ran");
    let probe = format!("touch {}", sentinel.display());

    let control = std::process::Command::new("touch")
        .arg(&sentinel)
        .status()
        .unwrap();
    assert!(
        control.success() && sentinel.exists(),
        "control: the probe must create the sentinel when it runs"
    );
    fs::remove_file(&sentinel).unwrap();

    setup_active_change(&root, &[probe.as_str()]);
    specsync()
        .env("CI", "true")
        .env_remove("GITHUB_WORKSPACE")
        .args(["--root", root.to_str().unwrap(), "comment"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SpecSync:"));
    assert!(
        !sentinel.exists(),
        "comment must not execute a configured verification command, even with its output hidden"
    );
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
