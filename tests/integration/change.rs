use super::helpers::{specsync, tracked_requirement, tracked_text};
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// Convert a host path into a form Git Bash accepts on Windows (`/c/Users/...`).
fn path_for_bash(path: &Path) -> String {
    let raw = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let displayed = raw.to_string_lossy();
    let stripped = displayed
        .strip_prefix(r"\\?\")
        .or_else(|| displayed.strip_prefix("//?/"))
        .unwrap_or(&displayed);
    let forward = stripped.replace('\\', "/");
    if let Some((drive, rest)) = forward.split_once(':')
        && drive.len() == 1
        && drive
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
    {
        return format!("/{}{}", drive.to_ascii_lowercase(), rest);
    }
    forward
}

/// Resolve a real bash suitable for shell scripts.
///
/// On Windows GitHub runners, bare `bash` often resolves to the WSL launcher
/// (`System32\bash.exe`), which fails with "no installed distributions" when WSL
/// is not provisioned. Prefer an explicit override, then Git for Windows bash.
fn resolve_bash() -> PathBuf {
    if let Some(explicit) = std::env::var_os("SPECSYNC_BASH") {
        let path = PathBuf::from(explicit);
        if path.as_os_str().is_empty() {
            // fall through
        } else {
            return path;
        }
    }

    #[cfg(windows)]
    {
        const CANDIDATES: &[&str] = &[
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files\Git\usr\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for candidate in CANDIDATES {
            let path = Path::new(candidate);
            if path.is_file() {
                return path.to_path_buf();
            }
        }

        if let Ok(output) = Command::new("where.exe").arg("bash").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut fallback: Option<PathBuf> = None;
                for line in text.lines() {
                    let candidate = line.trim();
                    if candidate.is_empty() {
                        continue;
                    }
                    let lower = candidate.to_ascii_lowercase();
                    // Skip the Windows Store / WSL stub — it is not a POSIX shell.
                    if lower.ends_with(r"\system32\bash.exe")
                        || lower.ends_with(r"\sysnative\bash.exe")
                        || lower.contains(r"\windowsapps\")
                    {
                        continue;
                    }
                    if lower.contains(r"\git\bin\bash") || lower.contains(r"\git\usr\bin\bash") {
                        return PathBuf::from(candidate);
                    }
                    if fallback.is_none() {
                        fallback = Some(PathBuf::from(candidate));
                    }
                }
                if let Some(path) = fallback {
                    return path;
                }
            }
        }
    }

    PathBuf::from("bash")
}

fn bash_command() -> Command {
    Command::new(resolve_bash())
}

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
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    // Production source has to belong to some spec. A file owned by nothing is
    // the state that used to pass approve and every check, then fail at finalize
    // with no way forward, so a fixture that omits ownership no longer describes
    // a project a change can be completed in.
    fs::create_dir_all(root.join("specs/lib")).unwrap();
    fs::write(
        root.join("specs/lib/lib.spec.md"),
        "---\nmodule: lib\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Lib\n\n## Purpose\n\nLibrary entry point.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
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
            "verification_commands": ["true"],
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
            // `lib` owns src/lib.rs; approve refuses owned production source no declared module
            // owns, even with no spec change (#986, #956).
            "--spec",
            "lib",
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
        } else if name == "plan" {
            super::helpers::SUBSTANTIVE_PLAN
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
    for name in [
        "state.json",
        "verification.json",
        "verification-attempts.json",
    ] {
        git(&["add", &format!(".specsync/changes/{id}/{name}")]);
    }
    git(&["commit", "-m", "persist verification"]);

    let assert_surfaces =
        |expected_action: &str, check_succeeds: Option<bool>, environment: &str| {
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
            if let Some(check_succeeds) = check_succeeds {
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
            }
        };
    for environment in ["local", "ci", "github"] {
        assert_surfaces(
            "run `specsync change verify harden-verification-freshness`, then `specsync change accept harden-verification-freshness`, then `specsync change archive harden-verification-freshness`",
            Some(true),
            environment,
        );
    }
    fs::write(
        root.join("src/lib.rs"),
        "pub fn ready() -> bool { false }\n",
    )
    .unwrap();
    git(&["add", "src/lib.rs"]);
    git(&["commit", "-m", "change governed input"]);
    for environment in ["local", "ci", "github"] {
        assert_surfaces(
            "run `specsync change verify harden-verification-freshness`, then `specsync change accept harden-verification-freshness`, then `specsync change archive harden-verification-freshness`",
            None,
            environment,
        );
    }
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check"])
        .assert()
        .success();
    for environment in ["local", "ci", "github"] {
        assert_surfaces(
            "run `specsync change verify harden-verification-freshness`, then `specsync change accept harden-verification-freshness`, then `specsync change archive harden-verification-freshness`",
            None,
            environment,
        );
    }
}

#[test]
fn change_answer_rejects_unknown_question_key_and_persists_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
    )
    .unwrap();
    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
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
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "answer",
            id,
            "nonexistent_question",
            "x",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown interview question"));

    // A hyphen/underscore typo fails the same way (#426).
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "answer",
            id,
            "acceptance-criteria",
            "typo",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown interview question"));

    // Nothing was persisted.
    let state: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".specsync/changes/{id}/state.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(state["answers"], serde_json::json!({}));
}

#[test]
fn change_answer_rejects_empty_and_whitespace_answers() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
    )
    .unwrap();
    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
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
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();

    for question in ["acceptance_criteria", "affected_paths", "public_contract"] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                id,
                question,
                "   ",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("must not be empty"));
    }

    // A real answer still works.
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "answer",
            id,
            "acceptance_criteria",
            "Passkeys enroll",
        ])
        .assert()
        .success();
}

#[test]
fn change_new_and_answer_reject_unregistered_affected_spec() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
    )
    .unwrap();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "new",
            "Add passkeys",
            "--spec",
            "nosuch",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("has no spec at").and(predicate::str::contains(
                "scaffold it first with `specsync new",
            )),
        );

    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
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
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "answer",
            id,
            "affected_specs",
            "nosuch",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("has no spec at").and(predicate::str::contains(
                "scaffold it first with `specsync new",
            )),
        )
        .stderr(predicate::str::contains("auth"));
}

#[test]
fn change_approve_trims_and_validates_actor_identity() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success());
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-m", "seed"]);

    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
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
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();
    for (question, answer) in [
        ("acceptance_criteria", "Passkeys enroll"),
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
        } else if name == "plan" {
            super::helpers::SUBSTANTIVE_PLAN
        } else {
            "# Complete\n\nReviewed lifecycle evidence.\n"
        };
        fs::write(
            root.join(format!(".specsync/changes/{id}/{name}.md")),
            content,
        )
        .unwrap();
    }
    fs::create_dir_all(root.join(format!(".specsync/changes/{id}/deltas"))).unwrap();
    fs::write(
        root.join(format!(".specsync/changes/{id}/deltas/auth.md")),
        "## ADDED\n\n### REQUIREMENT REQ-auth-900\n\nThe system SHALL hold.\n\nAcceptance Criteria\n- holds.\n",
    )
    .unwrap();

    // An actor taken from Git is held to the same rule and says where it came from (#1026 review).
    git(&["config", "user.name", "Ada\u{202E}x"]);
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "approve", id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("taken from Git `user.name`"));
    git(&["config", "user.name", "Test"]);
    // A newline-bearing actor is rejected before any ledger write (#426).
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "\n  padded\nactor  ",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("single line"));
    // A Unicode line separator or a bidirectional override reorders or breaks the printed line
    // just as a newline does, so it is refused the same way (#1026 review).
    for actor in [
        "Ada\u{2028}Evil",
        "Ada\u{202E}live",
        "Ada\u{200B}",
        "\u{FEFF}Ada",
        // Every format and default-ignorable character, not only the bidi ones (#1026 review).
        "Ada\u{00AD}X",
        "Ada\u{E0041}\u{E0042}",
        "Ada\u{3164}",
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "approve",
                id,
                "--actor",
                actor,
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("single line"));
    }
    let ledger: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".specsync/changes/{id}/approvals.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(ledger["approvals"], serde_json::json!([]));

    // Padding is trimmed before it reaches the append-only ledger.
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "approve",
            id,
            "--actor",
            "  Reviewer  ",
        ])
        .assert()
        .success();
    let ledger: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".specsync/changes/{id}/approvals.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(ledger["approvals"][0]["actor"], "Reviewer");

    // Every actor-accepting verb applies the same rule: a newline actor is
    // rejected before any ledger write, here through send-back (#426 round 2).
    let before = fs::read(root.join(format!(".specsync/changes/{id}/approvals.json"))).unwrap();
    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "send-back",
            id,
            "--actor",
            "Ada\nInjected",
            "--reason",
            "probe",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("single line"));
    let after = fs::read(root.join(format!(".specsync/changes/{id}/approvals.json"))).unwrap();
    assert_eq!(before, after, "the ledger is untouched by a rejected actor");
}

#[test]
fn change_new_json_returns_state_and_interview() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("specs/auth")).unwrap();
    fs::write(
        temp.path().join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
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
    assert_eq!(value["change"]["id"], "add-passkeys");
    assert!(value["questions"].as_array().unwrap().len() >= 3);
    assert!(
        temp.path()
            .join(".specsync/changes/add-passkeys/change.md")
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
fn init_then_adopt_enables_sdd() {
    // Honest label: DISCRIMINATOR. `init` writes the policy OFF and prints
    // "Enable with `specsync change adopt`". On the unfixed binary `adopt`
    // wrote a policy only when the file was MISSING — which it never is after
    // `init` — so the advertised on-switch left `enabled: false` and the only
    // way in was hand-editing JSON.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

    specsync()
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();
    let policy_path = root.join(".specsync/sdd.json");
    let after_init: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    assert_eq!(
        after_init["enabled"], false,
        "CONTROL: init still writes SDD off"
    );

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "adopt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("enable SDD policy"));

    let after_adopt: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    assert_eq!(
        after_adopt["enabled"], true,
        "adopt is the on-switch the init hint promises"
    );
    assert_eq!(
        after_adopt["require_change_for_meaningful_files"], false,
        "adopt flips `enabled` only; path coverage stays where the author left it"
    );

    // The switch is live, not just a JSON edit: the lifecycle gate now bites.
    let broken = root.join(".specsync/changes/broken-sibling");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("state.json"), "{ this is not json\n").unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
        .assert()
        .stderr(
            predicate::str::contains("state.json")
                .or(predicate::str::contains("unreadable"))
                .or(predicate::str::contains("invalid")),
        );
}

#[test]
fn adopt_is_idempotent_on_an_enabled_policy() {
    // CONTROL for the on-switch: adopting an already-adopted project must not
    // rewrite the author's policy, and must never reset a customized field to
    // the fresh-project default.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "adopt"])
        .assert()
        .success();

    let policy_path = root.join(".specsync/sdd.json");
    let mut policy: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    policy["meaningful_paths"] = serde_json::json!(["src/", "private/"]);
    policy["require_change_for_meaningful_files"] = serde_json::json!(true);
    fs::write(&policy_path, serde_json::to_string_pretty(&policy).unwrap()).unwrap();
    let before = fs::read(&policy_path).unwrap();

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "adopt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already enabled"));

    assert_eq!(
        fs::read(&policy_path).unwrap(),
        before,
        "adopting an enabled policy must leave it byte-identical"
    );
    let after: Value = serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    assert_eq!(after["enabled"], true);
    assert_eq!(
        after["meaningful_paths"],
        serde_json::json!(["src/", "private/"])
    );
    assert_eq!(after["require_change_for_meaningful_files"], true);
}

#[test]
fn adopt_fails_closed_on_a_policy_it_cannot_parse() {
    // A corrupt policy must not be silently replaced by a fresh default that
    // discards whatever the author had written.
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{ not json\n").unwrap();

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "adopt"])
        .assert()
        .failure();
    assert_eq!(
        fs::read_to_string(root.join(".specsync/sdd.json")).unwrap(),
        "{ not json\n"
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
fn init_leaves_sdd_off_for_new_projects() {
    // Honest label: DISCRIMINATOR for SDD-off-by-default. On the unfixed binary
    // init wrote `"enabled": true` and CI/check walked active changes.
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
    assert_eq!(policy["version"], 2);
    assert_eq!(policy["enabled"], false);
    assert_eq!(policy["require_change_for_meaningful_files"], false);
    assert!(temp.path().join(".specsync/archive/changes").is_dir());
}

// Regression for the doubled review/CI cost demonstrated by CorvidLabs/rune PR #23.
#[test]
fn eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci() {
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
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    // Workflow-v2 baseline is byte-compared. Disable autocrlf so Windows runners
    // do not rewrite committed LF JSON into CRLF working-tree bytes.
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

    let classifier = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github/scripts/classify-ci-paths.sh");
    let mut archived_ids = Vec::new();

    for index in 1..=8 {
        let branch = format!("rune-originating-pr-{index}");
        git(&["switch", "-c", &branch]);
        let affected_path = format!("docs/rune-change-{index}.md");
        let description = format!("Finalize Rune change {index}");
        let created = specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "--json",
                "change",
                "new",
                &description,
                "--kind",
                "documentation",
                "--path",
                &affected_path,
                "--no-spec-change",
                "--rationale",
                "Documentation-only behavior does not alter a technical contract",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let created: Value = serde_json::from_slice(&created).unwrap();
        let id = created["change"]["id"].as_str().unwrap().to_string();
        archived_ids.push(id.clone());

        for (question, answer) in [
            (
                "acceptance_criteria",
                "The originating PR contains its own compact archive finalization",
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
                    &id,
                    question,
                    answer,
                ])
                .assert()
                .success();
        }

        let active_dir = root.join(".specsync/changes").join(&id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap())
                .unwrap();
        assert_eq!(state["workflow_version"], 2);
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let artifact = artifact.as_str().unwrap();
            let content = if artifact == "tasks" {
                "# Tasks\n\n- [x] Complete the originating PR.\n".to_string()
            } else {
                format!(
                    "# {}\n\nComplete evidence for the originating PR.\n",
                    artifact.replace('-', " ")
                )
            };
            fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
        }
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "approve",
                &id,
                "--actor",
                "Scope owner",
            ])
            .assert()
            .success();

        fs::write(
            root.join(&affected_path),
            format!("# Rune change {index}\n\nDelivered in its originating PR.\n"),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", &format!("implement Rune change {index}")]);
        let implementation_commit = git(&["rev-parse", "HEAD"]);
        let implementation_tree = git(&["rev-parse", "HEAD^{tree}"]);
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", "check", &id])
            .assert()
            .success();

        let status = specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "--json",
                "change",
                "status",
                &id,
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let status: Value = serde_json::from_slice(&status).unwrap();
        assert_eq!(
            status["summary"]["next_action"],
            format!(
                "run `specsync change review {id} --reviewer <human>` after the PR's scoped review passes"
            )
        );
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "review",
                &id,
                "--reviewer",
                "Independent reviewer",
            ])
            .assert()
            .success();
        let finalized = specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "--json",
                "change",
                "finalize",
                &id,
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let finalized: Value = serde_json::from_slice(&finalized).unwrap();
        assert_eq!(finalized["ready_for_github_merge"], true);
        assert_eq!(
            finalized["next_action"],
            "Push this archive, wait until the checks are green, then merge the pull request."
        );
        assert_eq!(finalized["implementation_commit"], implementation_commit);
        assert_eq!(finalized["implementation_tree"], implementation_tree);

        let archive_dir = std::path::PathBuf::from(finalized["archived"].as_str().unwrap());
        assert!(!active_dir.exists());
        assert!(archive_dir.is_dir());
        let archived_state: Value =
            serde_json::from_str(&fs::read_to_string(archive_dir.join("state.json")).unwrap())
                .unwrap();
        let accepted_state: Value = serde_json::from_str(
            &fs::read_to_string(archive_dir.join("accepted-state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(archived_state["workflow_version"], 2);
        assert_eq!(archived_state["state"], "archived");
        assert_eq!(accepted_state["state"], "accepted");

        let verification: Value = serde_json::from_str(
            &fs::read_to_string(archive_dir.join("verification.json")).unwrap(),
        )
        .unwrap();
        let review: Value =
            serde_json::from_str(&fs::read_to_string(archive_dir.join("review.json")).unwrap())
                .unwrap();
        let finalization: Value = serde_json::from_str(
            &fs::read_to_string(archive_dir.join("finalization.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(review["schema_version"], 2);
        assert_eq!(review["verdict"], "pass");
        assert_eq!(review["execution_digest"], verification["execution_digest"]);
        assert_eq!(
            finalization["contract_digest"],
            verification["contract_digest"]
        );
        assert_eq!(
            finalization["workspace_digest"],
            verification["workspace_digest"]
        );
        assert_eq!(finalization["implementation_commit"], implementation_commit);
        assert_eq!(finalization["implementation_tree"], implementation_tree);
        for field in ["closing_digest", "review_digest", "finalization_digest"] {
            assert_eq!(finalization[field].as_str().unwrap().len(), 64);
        }
        let approvals: Value =
            serde_json::from_str(&fs::read_to_string(archive_dir.join("approvals.json")).unwrap())
                .unwrap();
        let approvals = approvals["approvals"].as_array().unwrap();
        // definition + terminal finalization closing approval (for reopen recovery)
        assert_eq!(approvals.len(), 2);
        assert_eq!(approvals[0]["gate"], "definition");
        assert_eq!(approvals[1]["gate"], "finalization");
        assert_eq!(approvals[1]["actor"], "specsync:finalization");

        // Finalize commits the archive move, so the index matches HEAD. Classify
        // that commit. An empty diff against HEAD is not an archive.
        let archive_diff = Command::new("git")
            .args([
                "diff",
                "--name-status",
                "-z",
                "--find-renames",
                "HEAD^",
                "HEAD",
                "--",
                ".specsync",
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(archive_diff.status.success());
        // Always invoke through bash: the classifier is a shell script, and Windows cannot
        // execute #! scripts directly (Win32 error 193). Resolve Git Bash explicitly —
        // bare `bash` on GHA Windows often hits the WSL stub instead of Git for Windows.
        // Pass bash-native absolute paths so Git Bash does not mis-parse `C:\...` roots.
        let classifier_bash = path_for_bash(Path::new(&classifier));
        let root_bash = path_for_bash(root);
        let bash = resolve_bash();
        let mut classifier_child = bash_command()
            .arg(&classifier_bash)
            .arg(&root_bash)
            .arg("false")
            .arg("name-status")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|error| {
                panic!(
                    "failed to spawn bash classifier via {}: {error}",
                    bash.display()
                )
            });
        classifier_child
            .stdin
            .take()
            .unwrap()
            .write_all(&archive_diff.stdout)
            .unwrap();
        let classification = classifier_child.wait_with_output().unwrap();
        let classifier_stdout = String::from_utf8_lossy(&classification.stdout);
        let classifier_stderr = String::from_utf8_lossy(&classification.stderr);
        assert!(
            classification.status.success(),
            "classifier failed (status={:?})\nstdout:\n{classifier_stdout}\nstderr:\n{classifier_stderr}",
            classification.status.code()
        );
        let classification = classifier_stdout.into_owned();
        assert!(
            classification.contains("archive_only=true\n")
                || classification.contains("archive_only=true\r\n"),
            "{classification}"
        );
        assert!(
            classification.contains("full=false\n") || classification.contains("full=false\r\n"),
            "{classification}"
        );
        assert!(
            classification.contains("site=false\n") || classification.contains("site=false\r\n"),
            "{classification}"
        );
        assert!(
            classification.contains("vscode=false\n")
                || classification.contains("vscode=false\r\n"),
            "{classification}"
        );

        assert_eq!(git(&["rev-parse", "HEAD^"]), implementation_commit);
        assert_eq!(
            git(&["log", "-1", "--format=%s"]),
            format!("chore(lifecycle): archive {id}")
        );
        git(&["switch", "main"]);
        git(&["merge", "--ff-only", &branch]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git(&["branch", "-d", &branch]);
    }

    let archived = root
        .join(".specsync/archive/changes")
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .count();
    assert_eq!(archived, archived_ids.len());
    let active = root
        .join(".specsync/changes")
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .count();
    assert_eq!(active, 0);
    assert_eq!(git(&["branch", "--format=%(refname:short)"]), "main");
}

#[test]
fn workflow_v2_cannot_downgrade_by_omitting_workflow_version() {
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
    let id = "update-contributor-documentation";
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "accept", id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uses the single 6.0 workflow"));
    let state_path = root.join(".specsync/changes").join(id).join("state.json");
    let mut legacy_state: Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    legacy_state
        .as_object_mut()
        .unwrap()
        .remove("workflow_version");
    fs::write(
        &state_path,
        format!("{}\n", serde_json::to_string_pretty(&legacy_state).unwrap()),
    )
    .unwrap();
    for command in ["status", "approve", "start", "finalize"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow version 1 conflicts with immutable origin 2",
            ));
    }
    // Project integrity surfaces workflow conflicts; scoped check only verifies one open change.
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "workflow version 1 conflicts with immutable origin 2",
        ));
}

#[test]
fn first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff() {
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
    git(&["commit", "--allow-empty", "-m", "trusted pre-v2 cutoff"]);
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
    let id = "update-contributor-documentation";
    let state_path = root.join(".specsync/changes").join(id).join("state.json");
    let mut legacy_state: Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let object = legacy_state.as_object_mut().unwrap();
    object.remove("workflow_version");
    object.remove("workflow_origin_version");
    fs::write(
        &state_path,
        format!("{}\n", serde_json::to_string_pretty(&legacy_state).unwrap()),
    )
    .unwrap();
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "attempt first-reachable workflow-v1 downgrade",
    ]);

    for command in ["status", "accept", "archive"] {
        specsync()
            .args(["--root", root.to_str().unwrap(), "change", command, id])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "was not present at the trusted pre-v2 cutoff",
            ));
    }
    // Project integrity surfaces cutoff violations; scoped check only verifies one open change.
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "was not present at the trusted pre-v2 cutoff",
        ));
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
  "verification_commands": ["true"],
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
    let predecessor = "govern-authentication";
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
        } else if name == "plan" {
            super::helpers::SUBSTANTIVE_PLAN
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
    let successor = "evolve-authentication";
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

// Verifies REQ-cmd-change-010.
#[test]
fn invalid_correction_ledger_blocks_mutating_commands_before_persistence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("README.md"), "Mutation ordering fixture.\n").unwrap();

    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Protect lifecycle mutation ordering",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "Command ordering regression fixture",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap();
    let change_dir = root.join(".specsync/changes").join(id);
    fs::write(
        change_dir.join("corrections.json"),
        "{ malformed correction ledger\n",
    )
    .unwrap();

    let snapshot = || {
        let mut files = change_dir
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| {
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    fs::read(entry.path()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files
    };
    let digest = "0".repeat(64);
    let commands = [
        vec!["change", "answer", id, "acceptance_criteria", "No mutation"],
        vec!["change", "depend", id, id],
        vec![
            "change",
            "supersede",
            id,
            id,
            "--path",
            "README.md",
            "--spec",
            "docs",
            "--digest",
            &digest,
        ],
    ];

    for command in commands {
        let before = snapshot();
        specsync()
            .args(["--root", root.to_str().unwrap()])
            .args(command)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "correction ledger integrity is invalid",
            ));
        assert_eq!(snapshot(), before, "failed command mutated lifecycle files");
    }
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
  "verification_commands": ["true"],
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
    let id = "update-review-instructions";
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
    // Stale accepted evidence is project health (`change audit`), not scoped `change check`.
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "audit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "accepted change verification is stale for current delivery inputs",
        ))
        .stderr(predicate::str::contains(
            "exact-only delivery input `README.md` changed after acceptance and requires an audited reopen; run `specsync change reopen update-review-instructions --actor <name> --reason <text>` to re-verify the accepted change",
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
        .success()
        .stdout(predicate::str::contains("verified"));

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
  "verification_commands": ["true"],
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
  "verification_commands": ["true"],
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
  "verification_commands": ["true"],
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
    let id = "correct-lifecycle-classification";
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
    let next_action = value["summary"]["next_action"].as_str().unwrap();
    for artifact in ["research.md", "design.md", "plan.md"] {
        assert!(next_action.contains(artifact));
    }
    assert_eq!(value["corrections"].as_array().unwrap().len(), 1);

    for artifact in ["research.md", "design.md", "plan.md"] {
        let body = if artifact == "plan.md" {
            super::helpers::SUBSTANTIVE_PLAN.to_string()
        } else {
            format!("# {}\n\nComplete.\n", artifact.trim_end_matches(".md"))
        };
        fs::write(dir.join(artifact), body).unwrap();
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
    assert_eq!(
        show["summary"]["next_action"],
        "run `specsync change approve correct-lifecycle-classification --actor <name>`"
    );

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
    // Text mode summarizes corrections without printing digest-bearing audit lines;
    // effective answers stay in --json (asserted above via change show).
    assert!(
        status.contains("Corrections: 1 recorded"),
        "expected correction count summary in text status, got:\n{status}"
    );
    assert!(
        status.contains(&format!("specsync change check {id}"))
            && (status.contains("ship-status") || status.contains("ship ")),
        "expected verifying next_action to name check and ship guidance, got:\n{status}"
    );
    let status_json = specsync()
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
    let status_json: Value = serde_json::from_slice(&status_json).unwrap();
    // Original interview answer remains no; effective definition is yes after correction.
    assert_eq!(status_json["change"]["answers"]["architecture_risk"], "no");
    assert_eq!(
        status_json["effective_definition"]["answers"]["architecture_risk"],
        "yes"
    );
    assert_eq!(status_json["corrections"].as_array().unwrap().len(), 1);

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
  "verification_commands": ["true"],
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

/// #997 (REQ-cli-003): `doctor --fix`, which writes lifecycle state, refuses under an inherited
/// verification context like every other lifecycle writer, with or without `--dry-run` or
/// `--close-late`, with one diagnostic. Read-only `doctor` still reports there, exactly as it
/// does without the marker (Leif's decision, 2026-11-06).
#[test]
fn indirect_recursive_lifecycle_doctor_fix_fails_once_and_plain_doctor_reports() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();

    for args in [
        vec!["--root", root_arg, "doctor", "--fix"],
        vec!["--root", root_arg, "doctor", "--fix", "--dry-run"],
        vec![
            "--root",
            root_arg,
            "doctor",
            "--fix",
            "--close-late",
            "some-change",
            "--actor",
            "Ada",
            "--reason",
            "merged long ago",
        ],
    ] {
        let stderr = specsync()
            .env("SPECSYNC_VERIFICATION_CONTEXT", "fledge run verify")
            .args(&args)
            .assert()
            .failure()
            .get_output()
            .stderr
            .clone();
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(
            stderr.matches("recursive lifecycle verification").count(),
            1,
            "{args:?}: {stderr}"
        );
        assert!(stderr.contains("fledge run verify"), "{args:?}: {stderr}");
    }

    let plain = |marked: bool| {
        let mut command = specsync();
        if marked {
            command.env("SPECSYNC_VERIFICATION_CONTEXT", "fledge run verify");
        } else {
            command.env_remove("SPECSYNC_VERIFICATION_CONTEXT");
        }
        command
            .args(["--root", root_arg, "doctor"])
            .output()
            .unwrap()
    };
    let (marked, unmarked) = (plain(true), plain(false));
    let stderr = String::from_utf8_lossy(&marked.stderr);
    assert!(
        !stderr.contains("recursive lifecycle verification"),
        "read-only doctor must still report: {stderr}"
    );
    assert_eq!(marked.status.code(), unmarked.status.code(), "{stderr}");
    assert_eq!(marked.stdout, unmarked.stdout);
}

// Verifies REQ-cli-003 at the contract level (#911, SHIP-1.c.1, SHIP-1.c.2).
//
// Honest label: CONTRACT PIN. The two tests above pin the guard's behavior; this fails when the cli
// contracts go back to describing a verification child that SpecSync makes. SpecSync sets
// `SPECSYNC_VERIFICATION_CONTEXT` on no child of its own, because verification executes no
// configured command (REQ-change-023), so the guard only ever sees a context an outside caller set.
#[test]
fn cli_contracts_describe_an_inherited_verification_context_not_a_child() {
    let requirement =
        tracked_requirement("specs/cli/requirements.md", "REQ-cli-003").expect("REQ-cli-003");
    assert!(
        requirement.contains("inherited `SPECSYNC_VERIFICATION_CONTEXT`")
            && requirement.contains("REQ-change-023"),
        "REQ-cli-003 must name the inherited marker and cite REQ-change-023: {requirement}"
    );
    let retired = [
        "configured verification child",
        "configured verification children",
        "a verification child calling",
    ];
    for relative in [
        "specs/cli/requirements.md",
        "specs/cli/cli.spec.md",
        "specs/cli/cli.3md",
        "specs/cli/context.md",
        "specs/cli/testing.md",
        "specs/cli/tasks.md",
        "docs/HLD.md",
    ] {
        let text = tracked_text(relative);
        for phrase in retired {
            assert!(
                !text.contains(phrase),
                "{relative} still describes a SpecSync-made verification child: `{phrase}`"
            );
        }
    }
}

// Verifies REQ-cmd-migrate-002 and REQ-cli-args-007.
#[test]
fn migrate_5_0_backfills_reopening_digest_fields_idempotently() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let workspace = root.join(".specsync/changes/adopt-trust");
    fs::create_dir_all(&workspace).unwrap();
    let reopening = serde_json::json!({
        "schema_version": 1,
        "change_id": "adopt-trust",
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

/// Builds a project whose one change is implementing and ready for `change verify`.
/// Policy may still list `verification_commands`; `change check` does not execute them.
#[cfg(unix)]
fn change_ready_to_verify(root: &Path, verification_commands: &[String]) -> String {
    change_ready_to_verify_with(root, verification_commands, serde_json::json!({}))
}

/// [`change_ready_to_verify`] with `extra` merged into `.specsync/sdd.json` before the change is
/// created, for the component and strict command lists.
#[cfg(unix)]
fn change_ready_to_verify_with(
    root: &Path,
    verification_commands: &[String],
    extra: Value,
) -> String {
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
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    fs::create_dir_all(root.join("specs/lib")).unwrap();
    fs::write(
        root.join("specs/lib/lib.spec.md"),
        "---\nmodule: lib\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Lib\n\n## Purpose\n\nLibrary entry point.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    let mut policy = serde_json::json!({
        "version": 1,
        "enabled": true,
        "require_change_for_meaningful_files": false,
        "meaningful_paths": ["src/", ".specsync/sdd.json"],
        "ignored_paths": [".specsync/"],
        "verification_commands": verification_commands,
        "custom_artifacts": {},
        "principles_file": null
    });
    if let (Some(policy), Some(extra)) = (policy.as_object_mut(), extra.as_object()) {
        policy.extend(extra.clone());
    }
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&policy).unwrap(),
    )
    .unwrap();
    let created = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "new",
            "Reap an interrupted verification child",
            "--kind",
            "bug-fix",
            // `lib` owns src/lib.rs; approve refuses owned production source no declared module
            // owns, even with no spec change (#986, #956).
            "--spec",
            "lib",
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
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The verification child is reaped"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        specsync()
            .args([
                "--root",
                root.to_str().unwrap(),
                "change",
                "answer",
                &id,
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
            &id,
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
        } else if name == "plan" {
            super::helpers::SUBSTANTIVE_PLAN
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
            &id,
            "--actor",
            "Reviewer",
        ])
        .assert()
        .success();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "start", &id])
        .assert()
        .success();
    git(&["add", "--all"]);
    git(&["commit", "-m", "implement"]);
    id
}

/// Writes an executable POSIX shell script and returns its path.
#[cfg(unix)]
fn executable_script(path: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    path.to_path_buf()
}

/// Holds an exclusive lock on `<root>/target/<profile>/.cargo-lock`.
#[cfg(unix)]
fn hold_cargo_build_lock(root: &Path, profile: &str) -> fs::File {
    use fs2::FileExt;

    let build = root.join("target").join(profile);
    fs::create_dir_all(&build).unwrap();
    let path = build.join(".cargo-lock");
    fs::write(&path, b"").unwrap();
    let holder = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    holder.lock_exclusive().unwrap();
    holder
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR, end to end through the shipped binary. The
// unfixed binary spawned `verification_commands` and named a held Cargo lock
// before that child blocked. This binary must not spawn, so both locks can be
// held and stderr stays silent about them.
#[cfg(unix)]
#[test]
fn change_check_does_not_wait_on_a_held_cargo_build_lock() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    let debug = hold_cargo_build_lock(root, "debug");
    let release = hold_cargo_build_lock(root, "release");

    let stderr = specsync()
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        // An isolated, empty CARGO_HOME so a real `~/.cargo/config.toml` on the
        // host cannot decide whether the notice fires.
        .env("CARGO_HOME", root.join("isolated-cargo-home"))
        .args(["--root", root.to_str().unwrap(), "change", "verify", &id])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(stderr).unwrap();

    assert!(
        !stderr.contains("waiting on"),
        "change check must not spawn cargo, so a held lock is not named: {stderr}"
    );

    drop(debug);
    drop(release);
}

// Verifies REQ-change-091.
//
// Honest label: CONTROL for the test above, and the one that makes its silence
// readable. A four-to-five minute compile with nothing held must print nothing,
// or the notice would fire on every healthy run and mean nothing.
#[cfg(unix)]
#[test]
fn an_unheld_cargo_build_lock_is_not_reported_during_verification() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    let build = root.join("target/debug");
    fs::create_dir_all(&build).unwrap();
    fs::write(build.join(".cargo-lock"), b"").unwrap();

    let stderr = specsync()
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        // An isolated, empty CARGO_HOME so a real `~/.cargo/config.toml` on the
        // host cannot decide whether the notice fires.
        .env("CARGO_HOME", root.join("isolated-cargo-home"))
        .args(["--root", root.to_str().unwrap(), "change", "verify", &id])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(stderr).unwrap();

    assert!(
        !stderr.contains("waiting on"),
        "an unheld lock must not be reported: {stderr}"
    );
}

// Verifies REQ-change-091.
//
// Honest label: DISCRIMINATOR. The unfixed binary spawned the configured
// reporter and wrote group.txt. This binary must leave that file absent.
#[cfg(unix)]
#[test]
fn change_check_does_not_spawn_a_configured_verification_child() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let record = root.join("group.txt");
    let reporter = executable_script(
        &root.join("stand-in/report-group"),
        &format!(
            "#!/bin/sh\nprintf '%s %s\\n' \"$$\" \"$(ps -o pgid= -p $$)\" > '{}'\nexit 0\n",
            record.display()
        ),
    );
    let id = change_ready_to_verify(root, &[reporter.display().to_string()]);

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "verify", &id])
        .assert()
        .success();

    assert!(
        !record.exists(),
        "configured verification_commands must not be spawned"
    );
}

// Verifies REQ-cmd-change-unrun-commands-001 (#893, SHIP-1.c.3).
//
// Honest label: DISCRIMINATOR. The unfixed binary passes this check in silence while the policy
// configures three commands it never runs. This binary says so on stderr in text, lists them in
// JSON, and still spawns none of them.
#[cfg(unix)]
#[test]
fn change_check_notes_the_configured_commands_it_did_not_run() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let marker = root.join("ran.txt");
    let reporter = executable_script(
        &root.join("stand-in/report"),
        &format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    );
    let reporter = reporter.display().to_string();
    let id = change_ready_to_verify_with(
        root,
        std::slice::from_ref(&reporter),
        serde_json::json!({
            "component_verification_commands": { "lib": ["cargo test lib::"] },
            "strict_verification_commands": ["cargo test lib::", "cargo test"],
        }),
    );
    let root_arg = root.to_str().unwrap();

    let text = specsync()
        .args(["--root", root_arg, "change", "check", &id])
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&text.stderr);
    assert!(
        stderr.contains(
            "note: .specsync/sdd.json configures 3 verification commands that SpecSync did not run"
        ),
        "a passing check must say it ran none of the configured commands: {stderr}"
    );
    assert!(
        stderr.contains("to stop this note"),
        "the note says how to silence it: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&text.stdout).contains("configures 3 verification commands"),
        "the note goes to stderr, not stdout"
    );

    let json = specsync()
        .args(["--root", root_arg, "--json", "change", "check", &id])
        .assert()
        .success()
        .get_output()
        .clone();
    let document: Value =
        serde_json::from_slice(&json.stdout).expect("JSON check prints one document on stdout");
    assert_eq!(
        document["configured_commands_not_run"],
        serde_json::json!([reporter, "cargo test lib::", "cargo test"]),
        "the configured commands, each once, in policy order: {document}"
    );
    assert!(!marker.exists(), "the note never runs a configured command");
}

// Verifies REQ-cmd-change-unrun-commands-001 (#893, SHIP-1.c.3).
//
// Honest label: DISCRIMINATOR. The committed check path (`change check --commit`) prints the
// note too; the unfixed binary prints only its own progress lines.
#[cfg(unix)]
#[test]
fn change_check_commit_notes_the_configured_commands_it_did_not_run() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &["cargo test".to_string()]);
    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "check",
            &id,
            "--commit",
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "note: .specsync/sdd.json configures 1 verification command that SpecSync did not run"
        ),
        "{stderr}"
    );
}

// Verifies REQ-cmd-change-unrun-commands-001 (#893, SHIP-1.c.3).
//
// Honest label: DISCRIMINATOR. `change audit` passes in silence on the unfixed binary.
#[cfg(unix)]
#[test]
fn change_audit_notes_the_configured_commands_it_did_not_run() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    change_ready_to_verify(
        root,
        &["cargo test".to_string(), "cargo clippy".to_string()],
    );
    let root_arg = root.to_str().unwrap();
    let text = specsync()
        .args(["--root", root_arg, "change", "audit"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&text.stderr);
    assert!(
        stderr.contains(
            "note: .specsync/sdd.json configures 2 verification commands that SpecSync did not run"
        ),
        "{stderr}"
    );
    let json = specsync()
        .args(["--root", root_arg, "--json", "change", "audit"])
        .assert()
        .success()
        .get_output()
        .clone();
    let document: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(
        document["configured_commands_not_run"],
        serde_json::json!(["cargo test", "cargo clippy"]),
        "{document}"
    );
}

// Verifies REQ-cmd-change-unrun-commands-001 (#893). Honest label: CONTROL. With nothing
// configured there is no note, and the JSON list is empty rather than missing.
#[cfg(unix)]
#[test]
fn an_empty_policy_gets_no_note_about_unrun_commands() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &[]);
    let root_arg = root.to_str().unwrap();
    for args in [
        vec!["change", "check", id.as_str()],
        vec!["change", "audit"],
    ] {
        let output = specsync()
            .args(["--root", root_arg])
            .args(&args)
            .assert()
            .success()
            .get_output()
            .clone();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("did not run"), "{args:?}: {stderr}");
        let json = specsync()
            .args(["--root", root_arg, "--json"])
            .args(&args)
            .assert()
            .success()
            .get_output()
            .clone();
        let document: Value = serde_json::from_slice(&json.stdout).unwrap();
        assert_eq!(
            document["configured_commands_not_run"],
            serde_json::json!([]),
            "{args:?}: {document}"
        );
    }
}

// Verifies SHIP-1.c.3 for this repository (#893, Leif's narrowed 3b decision). Honest label:
// CONTRACT PIN. Only the top-level list is empty; the component and strict lists stay configured,
// so the new note still discloses those commands instead of being silenced here.
#[test]
fn this_repository_configures_no_verification_command() {
    let policy: Value = serde_json::from_str(&tracked_text(".specsync/sdd.json")).unwrap();
    assert!(
        policy["verification_commands"]
            .as_array()
            .is_none_or(|list| list.is_empty()),
        "`verification_commands` lists commands `change check` does not run: {}",
        policy["verification_commands"]
    );
    assert_eq!(
        policy["component_verification_commands"]
            .as_object()
            .map_or(0, |lists| lists.len()),
        13,
        "the component verification commands stay configured"
    );
    assert_eq!(
        policy["strict_verification_commands"]
            .as_array()
            .map_or(0, |list| list.len()),
        3,
        "the strict verification commands stay configured"
    );
}

// Verifies REQ-change-023 at the contract level (#848, SHIP-1.c.1, SHIP-1.c.2).
//
// Honest label: CONTRACT PIN. It reads the tracked contracts, not the binary; the behavior is
// pinned by the REQ-change-091 discriminators above. It fails when a current change contract goes
// back to promising that verification executes the configured commands, which is how
// REQ-change-015 stood beside REQ-change-023 until #848.
#[test]
fn change_contracts_give_one_answer_on_configured_verification_commands() {
    let requirements = "specs/change/requirements.md";
    assert!(
        tracked_requirement(requirements, "REQ-change-015").is_none(),
        "REQ-change-015 required a reporting mode that executes every configured verification \
         command; REQ-change-058 says that mode SHALL NOT exist, so it stays retired"
    );
    let rule = tracked_requirement(requirements, "REQ-change-023").expect("REQ-change-023");
    assert!(
        rule.contains(
            "`change check` does not execute `.specsync/sdd.json` `verification_commands`."
        ),
        "REQ-change-023 is the rule the other clauses defer to: {rule}"
    );
    // A requirement that speaks about configured verification commands without being the rule
    // defers to it by ID, so a reader meets the answer where the question is asked.
    for id in [
        "REQ-change-019",
        "REQ-change-027",
        "REQ-change-030",
        "REQ-change-061",
        "REQ-change-069",
    ] {
        let block = tracked_requirement(requirements, id).unwrap_or_else(|| panic!("{id}"));
        assert!(
            block.contains("REQ-change-023"),
            "{id} mentions configured verification commands and must cite REQ-change-023: {block}"
        );
    }
    let lessons = tracked_text("specs/change/change.3md");
    assert!(
        lessons.contains(
            "The rest of this section records how those unwired helpers were designed while \
             `change check` still spawned the commands; none of it runs on a lifecycle path today \
             (REQ-change-023, REQ-change-091)."
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .as_str()
        ),
        "change.3md must mark its wait-notice and own-group child design notes as historical"
    );
    let retired = [
        "still executes every configured verification command",
        "remain unaffected and execute once",
        "A failed configured command",
        "as initialization instructs",
        "and native verification commands while",
        "rejection before lifecycle state or attempt history mutates",
        // change.3md lesson text that described the unwired helpers as running (#899 review).
        "printed before the child starts rather than after it has blocked",
        "Ctrl-C still reaches the child",
    ];
    for relative in [
        "specs/change/requirements.md",
        "specs/change/change.spec.md",
        "specs/change/testing.md",
        "specs/change/context.md",
        "specs/change/change.3md",
        "specs/change/tasks.md",
    ] {
        let text = tracked_text(relative);
        for phrase in retired {
            assert!(
                !text.contains(phrase),
                "{relative} still promises execution of configured commands: `{phrase}`"
            );
        }
    }
}

// Verifies REQ-change-023 in the public comparison pages (#848, SHIP-1.c.1, SHIP-1.c.2).
//
// Honest label: CONTRACT PIN over published guidance. The comparison tables credited SpecSync with
// executing configured tests and isolating its verification shell, but `change check` spawns no
// configured command. The pages must say the project's CI runs the tests.
#[test]
fn comparison_pages_do_not_credit_specsync_with_running_configured_tests() {
    let why = tracked_text("site/src/content/docs/why-specsync.md");
    let proof = tracked_text("site/src/content/docs/comparisons/adversarial-proof.md");
    for (relative, text, phrase) in [
        (
            "why-specsync.md",
            &why,
            "Configured tests executed by CI gate",
        ),
        (
            "why-specsync.md",
            &why,
            "Verification command shell isolation",
        ),
        (
            "adversarial-proof.md",
            &proof,
            "**Block** when configured for verification/CI",
        ),
        // The project's CI blocks only when it runs the tests and fails on them; SpecSync
        // cannot promise that for every project (Codex review on #899).
        ("adversarial-proof.md", &proof, "**Your CI** blocks;"),
    ] {
        assert!(
            !text.contains(phrase),
            "{relative} overstates who runs the tests or what blocks them: `{phrase}`"
        );
    }
    for (relative, text) in [("why-specsync.md", &why), ("adversarial-proof.md", &proof)] {
        assert!(
            text.contains("runs no configured command"),
            "{relative} must say that verification runs no configured command"
        );
    }
}

// Verifies REQ-cmd-change-005 and REQ-change-093: every text result ends with exactly one
// `Handoff:` line and JSON carries the same verdict under `summary.handoff`.
#[test]
fn status_prints_a_handoff_line_and_json_carries_it() {
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
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    fs::create_dir_all(root.join("specs/lib")).unwrap();
    fs::write(
        root.join("specs/lib/lib.spec.md"),
        "---\nmodule: lib\nversion: 1\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Lib\n\n## Purpose\n\nLibrary entry point.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
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
            "verification_commands": ["true"],
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
            "Tell agents when to clear",
            "--kind",
            "bug-fix",
            // `lib` owns src/lib.rs; approve refuses owned production source no declared module
            // owns, even with no spec change (#986, #956).
            "--spec",
            "lib",
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

    let handoff_lines = |text: &str| -> Vec<String> {
        text.lines()
            .filter(|line| line.trim_start().starts_with("Handoff:"))
            .map(str::to_string)
            .collect()
    };
    let status_text = |root: &Path| -> String {
        let output = specsync()
            .args(["--root", root.to_str().unwrap(), "change", "status", id])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(output).unwrap()
    };

    // Draft: interview open, so the verdict is conditional and names the answer command.
    let draft = status_text(root);
    let lines = handoff_lines(&draft);
    assert_eq!(lines.len(), 1, "exactly one Handoff line, got:\n{draft}");
    assert!(
        lines[0].contains("Handoff: conditional —"),
        "draft is never safe, got:\n{draft}"
    );
    assert!(lines[0].contains("Before clearing:"));
    let next_index = draft.find("Next:").expect("status prints Next:");
    let handoff_index = draft.find("Handoff:").unwrap();
    assert!(
        next_index < handoff_index,
        "Handoff follows Next:, got:\n{draft}"
    );
    let draft_json = specsync()
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
    let draft_json: Value = serde_json::from_slice(&draft_json).unwrap();
    assert_eq!(draft_json["summary"]["handoff"]["readiness"], "conditional");
    assert_eq!(
        draft_json["summary"]["handoff"]["resume"],
        format!("specsync change status {id}")
    );
    assert!(
        draft_json["summary"]["handoff"]["before_clearing"]
            .as_array()
            .is_some_and(|steps| !steps.is_empty())
    );

    for (question, answer) in [
        ("acceptance_criteria", "The handoff line is printed"),
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
            "# Tasks\n\n- [x] Print the handoff line.\n"
        } else if name == "plan" {
            super::helpers::SUBSTANTIVE_PLAN
        } else {
            "# Complete\n\nReviewed lifecycle evidence.\n"
        };
        fs::write(
            root.join(format!(".specsync/changes/{id}/{name}.md")),
            content,
        )
        .unwrap();
    }

    // Approve with src/lib.rs still untracked: the transition JSON carries the verdict and
    // the uncommitted scoped work makes it conditional.
    let approved = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "change",
            "approve",
            id,
            "--actor",
            "Reviewer",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let approved: Value = serde_json::from_slice(&approved).unwrap();
    assert_eq!(approved["state"], "approved");
    assert_eq!(approved["handoff"]["readiness"], "conditional");
    assert_eq!(
        approved["handoff"]["before_clearing"][0],
        "commit the work in progress"
    );

    // Uncommitted lifecycle evidence under .specsync/ never counts; committing the work does.
    git(&["add", "--all"]);
    git(&["commit", "-m", "implement"]);
    let approved_text = status_text(root);
    let lines = handoff_lines(&approved_text);
    assert_eq!(
        lines.len(),
        1,
        "exactly one Handoff line, got:\n{approved_text}"
    );
    assert!(
        lines[0].contains("Handoff: safe —") && !lines[0].contains("Before clearing:"),
        "a committed approved change is a clean boundary, got:\n{approved_text}"
    );
    assert!(
        !lines[0]
            .split(|character: char| !character.is_ascii_hexdigit())
            .any(|word| word.len() >= 40),
        "handoff prose never carries a digest, got:\n{}",
        lines[0]
    );
}

// Verifies REQ-change-123 through the built binary (#914). A change that was reviewed, then sent
// back, re-answered, re-approved and re-checked is reviewed again and finalizes. The review
// recorded before the send-back stays in its history, unchanged, and until the fresh review
// `change status` calls it stale and names `change review` rather than calling it unreadable.
#[test]
fn a_change_reviewed_before_its_send_back_is_reviewed_again_and_finalizes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    // Workflow-v2 baseline is byte-compared; keep committed LF JSON as LF on Windows.
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let invoke = |args: &[&str]| {
        let mut full = vec!["--root", root.to_str().unwrap()];
        full.extend_from_slice(args);
        specsync().args(&full).output().unwrap()
    };
    let run = |args: &[&str]| -> String {
        let output = invoke(args);
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let created: Value = serde_json::from_str(&run(&[
        "--json",
        "change",
        "new",
        "Archive a greeting",
        "--kind",
        "documentation",
        "--path",
        "docs/greeting.md",
        "--no-spec-change",
        "--rationale",
        "Documentation-only behavior does not alter a technical contract",
    ]))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The greeting is archived"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", &id, question, answer]);
    }
    let active_dir = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["workflow_version"], 2, "reproduced under workflow v2");
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let artifact = artifact.as_str().unwrap();
        let content = if artifact == "tasks" {
            "# Tasks\n\n- [x] Archive the greeting.\n".to_string()
        } else {
            format!("# {artifact}\n\nComplete evidence for the greeting.\n")
        };
        fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
    }
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    fs::write(root.join("docs/greeting.md"), "# Greeting\n\nHello.\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement greeting"]);
    run(&["change", "check", &id, "--commit"]);
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let first_attempts: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("review-attempts.json")).unwrap())
            .unwrap();
    assert_eq!(first_attempts["reviews"].as_array().unwrap().len(), 1);

    // The scope turns out wrong after the review: send it back, re-answer, re-approve, re-check.
    run(&[
        "change",
        "send-back",
        &id,
        "--actor",
        "Scope owner",
        "--reason",
        "the criteria must change",
    ]);
    run(&[
        "change",
        "answer",
        &id,
        "acceptance_criteria",
        "The greeting is archived and dated",
    ]);
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    run(&["change", "check", &id, "--commit"]);

    // Before the fresh review the earlier one is stale, not unreadable, and nothing ships on it.
    let status: Value = serde_json::from_str(&run(&["--json", "change", "status", &id])).unwrap();
    assert_eq!(status["summary"]["approval_valid"], true, "{status}");
    assert!(
        status["summary"]["next_action"]
            .as_str()
            .unwrap()
            .contains(&format!("specsync change review {id}")),
        "{status}"
    );
    let ship_status = run(&["change", "ship-status", &id]);
    let review_line = ship_status
        .lines()
        .find(|line| line.trim_start().starts_with("Review:"))
        .unwrap_or_else(|| panic!("no Review: line in\n{ship_status}"));
    assert!(
        review_line.contains("stale") && !review_line.contains("unreadable"),
        "the pre-send-back review is stale history, not unreadable evidence:\n{ship_status}"
    );
    assert!(
        !invoke(&["change", "ship", &id]).status.success(),
        "a review recorded before the send-back must not ship the change"
    );

    // The fresh review records, and finalize archives the change with both attempts kept.
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let finalized: Value =
        serde_json::from_str(&run(&["--json", "change", "finalize", &id])).unwrap();
    let archive_dir = root.join(".specsync/archive/changes").join(
        PathBuf::from(finalized["archived"].as_str().unwrap())
            .file_name()
            .unwrap(),
    );
    let archived_attempts: Value = serde_json::from_str(
        &fs::read_to_string(archive_dir.join("review-attempts.json")).unwrap(),
    )
    .unwrap();
    let reviews = archived_attempts["reviews"].as_array().unwrap();
    assert_eq!(reviews.len(), 2, "{archived_attempts}");
    assert_eq!(
        reviews[0], first_attempts["reviews"][0],
        "the pre-send-back review is kept unchanged"
    );
    let archived: Value = serde_json::from_str(&run(&["--json", "change", "status", &id])).unwrap();
    assert_eq!(archived["change"]["state"], "archived", "{archived}");
}

// Verifies REQ-change-093 and REQ-cmd-change-014 through the built binary (#880, audit D3). A
// committed workflow-v2 archive whose verification evidence is tampered reports terminal
// evidence `corrupt-history`; the handoff must then be `not-yet` in JSON and text, and every
// next action must name the restore that recovers it. The test runs the named command verbatim,
// then repeats the tamper as a commit and restores from the archiving commit.
#[test]
fn a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    // Workflow-v2 baseline is byte-compared; keep committed LF JSON as LF on Windows.
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let run = |args: &[&str]| -> String {
        let mut full = vec!["--root", root.to_str().unwrap()];
        full.extend_from_slice(args);
        let output = specsync()
            .args(&full)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(output).unwrap()
    };
    let created: Value = serde_json::from_str(&run(&[
        "--json",
        "change",
        "new",
        "Archive a greeting",
        "--kind",
        "documentation",
        "--path",
        "docs/greeting.md",
        "--no-spec-change",
        "--rationale",
        "Documentation-only behavior does not alter a technical contract",
    ]))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The greeting is archived"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", &id, question, answer]);
    }
    let active_dir = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(
        state["workflow_version"], 2,
        "the defect is reproduced under workflow v2"
    );
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let artifact = artifact.as_str().unwrap();
        let content = if artifact == "tasks" {
            "# Tasks\n\n- [x] Archive the greeting.\n".to_string()
        } else {
            format!("# {artifact}\n\nComplete evidence for the greeting.\n")
        };
        fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
    }
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    fs::write(root.join("docs/greeting.md"), "# Greeting\n\nHello.\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement greeting"]);
    run(&["change", "check", &id]);
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let finalized: Value =
        serde_json::from_str(&run(&["--json", "change", "finalize", &id])).unwrap();
    // Rebuilt from the directory name: the reported path may resolve a symlinked temp root.
    let package = format!(
        ".specsync/archive/changes/{}",
        PathBuf::from(finalized["archived"].as_str().unwrap())
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    let archive_dir = root.join(&package);
    assert!(archive_dir.join("state.json").is_file(), "{package}");
    // Finalize commits the archive move itself, so HEAD is the archiving commit.
    assert_eq!(
        git(&["status", "--porcelain"]),
        "",
        "finalize leaves a clean tree"
    );

    let status_json =
        || -> Value { serde_json::from_str(&run(&["--json", "change", "status", &id])).unwrap() };
    let status_text = || run(&["change", "status", &id]);
    let line = |text: &str, prefix: &str| -> String {
        text.lines()
            .map(str::trim_start)
            .find(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("no `{prefix}` line in:\n{text}"))
            .to_string()
    };
    let no_digest = |text: &str| {
        assert!(
            !text
                .split(|character: char| !character.is_ascii_hexdigit())
                .any(|word| word.len() >= 40),
            "guidance must never carry a digest: {text}"
        );
    };
    let assert_authenticated_and_safe = |when: &str| {
        let json = status_json();
        assert_eq!(
            json["summary"]["terminal_evidence"]["validity"], "authenticated-history",
            "{when}: {json}"
        );
        assert_eq!(
            json["summary"]["handoff"]["readiness"], "safe",
            "{when}: {json}"
        );
        assert_eq!(
            json["summary"]["next_action"], "merge the PR on GitHub if it is still open",
            "{when}"
        );
        let text = status_text();
        assert_eq!(
            line(&text, "Next:"),
            "Next: no further action",
            "{when}:\n{text}"
        );
        assert!(
            line(&text, "Handoff:").starts_with("Handoff: safe —"),
            "{when}:\n{text}"
        );
    };
    assert_authenticated_and_safe("freshly archived and committed");

    // The auditor's tamper: the archived verification's workspace digest replaced with zeros.
    let verification = archive_dir.join("verification.json");
    let original = fs::read(&verification).unwrap();
    let tamper = || {
        let mut evidence: Value = serde_json::from_slice(&original).unwrap();
        evidence["workspace_digest"] = Value::String("0".repeat(64));
        fs::write(
            &verification,
            serde_json::to_string_pretty(&evidence).unwrap() + "\n",
        )
        .unwrap();
        fs::read(&verification).unwrap()
    };
    let tampered = tamper();

    // Uncommitted damage: HEAD still holds the committed archive.
    let json = status_json();
    assert_eq!(
        json["summary"]["terminal_evidence"]["validity"], "corrupt-history",
        "{json}"
    );
    let handoff = &json["summary"]["handoff"];
    assert_eq!(
        handoff["readiness"], "not-yet",
        "a corrupt archive is never safe: {json}"
    );
    assert!(
        handoff["reason"]
            .as_str()
            .unwrap()
            .contains("fails its integrity check"),
        "{json}"
    );
    let step = handoff["before_clearing"][0].as_str().unwrap().to_string();
    let restore_from_head = format!("git restore --source=HEAD --staged --worktree -- '{package}'");
    assert_eq!(
        step,
        format!(
            "run `{restore_from_head}` to put back the committed archive, then `specsync change status '{id}'`"
        )
    );
    assert_eq!(
        json["summary"]["next_action"], step,
        "JSON next action and handoff agree"
    );
    assert!(!step.contains("specsync change check"));
    no_digest(&step);
    let text = status_text();
    assert_eq!(line(&text, "Next:"), format!("Next: {step}"), "{text}");
    let handoff_line = line(&text, "Handoff:");
    assert!(handoff_line.starts_with("Handoff: not yet —"), "{text}");
    assert!(
        handoff_line.ends_with(&format!("Before clearing: {step}")),
        "{text}"
    );
    no_digest(&handoff_line);
    let ship_status = run(&["change", "ship-status", &id]);
    assert!(
        ship_status.contains(&step),
        "ship-status names the restore:\n{ship_status}"
    );
    // Reading status changes nothing: the tampered bytes are still there to restore.
    assert_eq!(fs::read(&verification).unwrap(), tampered);

    // Run the named command verbatim, through a shell, exactly as an operator would paste it.
    let shell = |command: &str| -> String {
        let output = bash_command()
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "`{command}` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    let named = step
        .split('`')
        .nth(1)
        .expect("the step names its command in backticks");
    assert_eq!(named, restore_from_head);
    shell(named);
    assert_eq!(fs::read(&verification).unwrap(), original);
    assert_eq!(
        git(&["status", "--porcelain"]),
        "",
        "the restore leaves a clean tree"
    );
    assert_authenticated_and_safe("after the named HEAD restore");

    // Committed damage: HEAD now holds the corrupt bytes, so the commit that archived the
    // package is where the trusted bytes are.
    tamper();
    git(&["commit", "-am", "tamper with the archive"]);
    let json = status_json();
    assert_eq!(json["summary"]["handoff"]["readiness"], "not-yet", "{json}");
    let step = json["summary"]["handoff"]["before_clearing"][0]
        .as_str()
        .unwrap()
        .to_string();
    let find_archiving_commit =
        format!("git log -1 --diff-filter=A --format=%H -- '{package}/state.json'");
    let restore_from_commit =
        format!("git restore --source=<commit> --staged --worktree -- '{package}'");
    assert_eq!(
        step,
        format!(
            "run `{restore_from_commit}` with the commit that archived it (`{find_archiving_commit}`), commit the restore, then run `specsync change status '{id}'`"
        )
    );
    assert_eq!(json["summary"]["next_action"], step);
    no_digest(&step);
    let text = status_text();
    assert_eq!(line(&text, "Next:"), format!("Next: {step}"), "{text}");
    assert!(
        line(&text, "Handoff:").starts_with("Handoff: not yet —"),
        "{text}"
    );

    let archiving_commit = shell(&find_archiving_commit);
    assert_eq!(archiving_commit, git(&["rev-parse", "HEAD~1"]));
    shell(&restore_from_commit.replace("<commit>", &archiving_commit));
    git(&[
        "commit",
        "-m",
        "restore the archive from the commit that archived it",
    ]);
    assert_eq!(fs::read(&verification).unwrap(), original);
    assert_authenticated_and_safe("after the archiving-commit restore is committed");

    // Damage the commit that archived the package (#966, CHANGE-7.a.2): fold the tamper into
    // that commit, so the commit that added its state.json holds the same corrupt bytes as HEAD.
    // Neither restore would change anything, so the step names the status JSON field that says
    // what failed. That field must exist at exactly the path the step prints.
    assert_eq!(archiving_commit, git(&["rev-parse", "HEAD~2"]));
    tamper();
    git(&["reset", "-q", "--soft", &format!("{archiving_commit}~1")]);
    git(&[
        "commit",
        "-q",
        "-am",
        "archive with the damage already in it",
    ]);
    let json = status_json();
    assert_eq!(
        json["summary"]["terminal_evidence"]["validity"], "corrupt-history",
        "{json}"
    );
    let step = json["summary"]["handoff"]["before_clearing"][0]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        step.starts_with(&format!(
            "run `specsync change status '{id}' --json`, restore what its `"
        )),
        "the step reads the status JSON when no package restore would help: {step}"
    );
    assert_eq!(json["summary"]["next_action"], step);
    no_digest(&step);
    let field = step
        .split('`')
        .nth(3)
        .expect("the step names the JSON field in backticks");
    let named = field.split('.').fold(&json, |value, key| &value[key]);
    assert!(
        named.as_str().is_some_and(|reason| !reason.is_empty()),
        "`{field}` must name a non-empty string in `change status --json`, got {named}: {json}"
    );
}

// Verifies CHANGE-12.e and REQ-change-123 through the built binary (#960, from the Codex review of
// #944). A reviewed change is sent back and re-approved with its scope unchanged, so the new
// definition approval has the same digest as the one the review followed, and `change check`
// runs without `--commit`, so HEAD, the execution digest and the project inputs do not move
// either. The earlier review must still not count: ship-status calls it stale, status names
// `change review`, and `change ship` refuses until a fresh review is recorded.
#[test]
fn a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    // Workflow-v2 baseline is byte-compared; keep committed LF JSON as LF on Windows.
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let invoke = |args: &[&str]| {
        let mut full = vec!["--root", root.to_str().unwrap()];
        full.extend_from_slice(args);
        specsync().args(&full).output().unwrap()
    };
    let run = |args: &[&str]| -> String {
        let output = invoke(args);
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let created: Value = serde_json::from_str(&run(&[
        "--json",
        "change",
        "new",
        "Archive a greeting",
        "--kind",
        "documentation",
        "--path",
        "docs/greeting.md",
        "--no-spec-change",
        "--rationale",
        "Documentation-only behavior does not alter a technical contract",
    ]))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The greeting is archived"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", &id, question, answer]);
    }
    let active_dir = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["workflow_version"], 2, "reproduced under workflow v2");
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let artifact = artifact.as_str().unwrap();
        let content = if artifact == "tasks" {
            "# Tasks\n\n- [x] Archive the greeting.\n".to_string()
        } else {
            format!("# {artifact}\n\nComplete evidence for the greeting.\n")
        };
        fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
    }
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    fs::write(root.join("docs/greeting.md"), "# Greeting\n\nHello.\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement greeting"]);
    run(&["change", "check", &id, "--commit"]);
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let first_attempts: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("review-attempts.json")).unwrap())
            .unwrap();
    assert_eq!(first_attempts["reviews"].as_array().unwrap().len(), 1);

    // Send it back, then re-approve the same scope and re-check without committing.
    run(&[
        "change",
        "send-back",
        &id,
        "--actor",
        "Scope owner",
        "--reason",
        "the scope needs another look",
    ]);
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    run(&["change", "check", &id]);

    let status: Value = serde_json::from_str(&run(&["--json", "change", "status", &id])).unwrap();
    assert_eq!(status["summary"]["approval_valid"], true, "{status}");
    assert!(
        status["summary"]["next_action"]
            .as_str()
            .unwrap()
            .contains(&format!("specsync change review {id}")),
        "after a send-back the next action is a fresh review:\n{status}"
    );
    let ship_status = run(&["change", "ship-status", &id]);
    let review_line = ship_status
        .lines()
        .find(|line| line.trim_start().starts_with("Review:"))
        .unwrap_or_else(|| panic!("no Review: line in\n{ship_status}"));
    assert!(
        review_line.contains("stale"),
        "a review recorded before the send-back is stale even when the scope came back unchanged:\n{ship_status}"
    );
    assert!(
        !invoke(&["change", "ship", &id]).status.success(),
        "a review recorded before the send-back must not ship the change"
    );
    assert!(
        active_dir.join("state.json").exists(),
        "the refused ship leaves the change active"
    );

    // A fresh review records, and finalize archives the change with both attempts kept.
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let finalized: Value =
        serde_json::from_str(&run(&["--json", "change", "finalize", &id])).unwrap();
    let archive_dir = root.join(".specsync/archive/changes").join(
        PathBuf::from(finalized["archived"].as_str().unwrap())
            .file_name()
            .unwrap(),
    );
    let archived_attempts: Value = serde_json::from_str(
        &fs::read_to_string(archive_dir.join("review-attempts.json")).unwrap(),
    )
    .unwrap();
    let reviews = archived_attempts["reviews"].as_array().unwrap();
    assert_eq!(reviews.len(), 2, "{archived_attempts}");
    assert_eq!(
        reviews[0], first_attempts["reviews"][0],
        "the pre-send-back review is kept unchanged"
    );
}

// Verifies REQ-cmd-change-008 and SHIP-4.a through the built binary (#986, #956). A no-spec-change change scoped to a
// directory that holds production source a module owns passes approve, check and review: approve
// cannot see which owned files a directory scope will deliver. ship-status and `ship --dry-run`
// used to say "ready to finalize", and only finalize refused on canonical ownership. Both now run
// finalize's ownership check, refuse to call the change ready, and name the recovery; the test
// runs that recovery and ships.
#[test]
fn ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    // `billing` owns src/billing/charge.ts; the change scopes the whole directory.
    fs::create_dir_all(root.join("specs/billing")).unwrap();
    fs::write(
        root.join("specs/billing/billing.spec.md"),
        "---\nmodule: billing\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/charge.ts\n---\n\n# Billing\n\n## Purpose\n\nCharges.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(root.join("src/billing/charge.ts"), "const rate = 1;\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);

    let invoke = |args: &[&str]| {
        let mut full = vec!["--root", root.to_str().unwrap()];
        full.extend_from_slice(args);
        specsync().args(&full).output().unwrap()
    };
    let run = |args: &[&str]| -> String {
        let output = invoke(args);
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let created: Value = serde_json::from_str(&run(&[
        "--json",
        "change",
        "new",
        "Tune the charge rate",
        "--kind",
        "bug-fix",
        "--path",
        "src/billing/",
        "--no-spec-change",
        "--rationale",
        "The rate is internal and no contract describes it",
    ]))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The charge rate is tuned"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", &id, question, answer]);
    }
    let active_dir = root.join(".specsync/changes").join(&id);
    let write_artifacts = || {
        let state: Value =
            serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let artifact = artifact.as_str().unwrap();
            let content = if artifact == "tasks" {
                "# Tasks\n\n- [x] Tune the rate.\n".to_string()
            } else {
                format!("# {artifact}\n\nComplete evidence for the rate.\n")
            };
            fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
        }
    };
    write_artifacts();
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "tune the rate"]);
    run(&["change", "check", &id, "--commit"]);
    // A passing review refuses a change finalize would refuse on ownership and names the
    // recovery, so the Archive check never sees a passing review to call ready (#1002).
    let review = invoke(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let review_text = format!(
        "{}{}",
        String::from_utf8_lossy(&review.stdout),
        String::from_utf8_lossy(&review.stderr)
    );
    assert!(
        !review.status.success() && review_text.contains("scoped review cannot pass"),
        "the review refuses to pass:\n{review_text}"
    );
    assert!(
        review_text.contains(&format!(
            "specsync change answer '{id}' affected_specs 'billing'"
        )),
        "the review refusal names the recovery:\n{review_text}"
    );

    // Readiness tells the truth before finalize does.
    let ship_status = run(&["change", "ship-status", &id]);
    assert!(
        !ship_status.contains("ready to finalize"),
        "a change finalize refuses on ownership is not ready:\n{ship_status}"
    );
    for expected in [
        "src/billing/charge.ts",
        "billing",
        &format!("specsync change send-back '{id}'"),
        &format!("specsync change answer '{id}' affected_specs 'billing'"),
    ] {
        assert!(
            ship_status.contains(expected),
            "ship-status names `{expected}`:\n{ship_status}"
        );
    }
    let dry_run = invoke(&["change", "ship", &id, "--dry-run"]);
    let dry_text = format!(
        "{}{}",
        String::from_utf8_lossy(&dry_run.stdout),
        String::from_utf8_lossy(&dry_run.stderr)
    );
    assert!(
        !dry_run.status.success() && !dry_text.contains("ready to finalize"),
        "the dry run refuses as the real ship would:\n{dry_text}"
    );
    assert!(
        dry_text.contains(&format!(
            "specsync change answer '{id}' affected_specs 'billing'"
        )),
        "the dry run names the recovery:\n{dry_text}"
    );

    // The named recovery: send back, declare the owner, re-approve, re-check, a fresh review, ship.
    run(&[
        "change",
        "send-back",
        &id,
        "--actor",
        "Scope owner",
        "--reason",
        "declare the module that owns the charge",
    ]);
    run(&["change", "answer", &id, "affected_specs", "billing"]);
    run(&["change", "answer", &id, "constraints", "none"]);
    write_artifacts();
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    run(&["change", "check", &id, "--commit"]);
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    let ready = run(&["change", "ship-status", &id]);
    assert!(ready.contains("ready to finalize"), "{ready}");
    run(&["change", "ship", &id]);
    let archived: Value = serde_json::from_str(&run(&["--json", "change", "status", &id])).unwrap();
    assert_eq!(archived["change"]["state"], "archived", "{archived}");
}

// Verifies REQ-change-111 and REQ-cmd-change-005 through the built binary. An approved semantic
// delta edited after approval reads as an invalid approval in JSON and text status, and
// `change check` and `change audit` both refuse it. Restoring the approved bytes clears all three.
#[test]
fn status_json_and_text_report_an_edited_approved_delta() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();
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
    for dir in [".specsync", "src", "specs/auth"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
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
    git(&["add", "--all"]);
    git(&["commit", "-m", "base"]);
    let run = |args: &[&str]| specsync().args(["--root", root_arg]).args(args).assert();
    let id = "reword-the-auth-purpose";
    run(&[
        "change",
        "new",
        "Reword the auth purpose",
        "--kind",
        "bug-fix",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ])
    .success();
    for (question, answer) in [
        ("acceptance_criteria", "The auth purpose is reworded"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", id, question, answer]).success();
    }
    let workspace = root.join(".specsync/changes").join(id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    assert_eq!(
        state["workflow_version"], 2,
        "the delta binding is workflow-v2's"
    );
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "tasks" => "# Tasks\n\n- [x] Reword the purpose.\n",
            "plan" => super::helpers::SUBSTANTIVE_PLAN,
            _ => "# Complete\n\nReviewed delta wording.\n",
        };
        fs::write(workspace.join(format!("{name}.md")), content).unwrap();
    }
    let delta = workspace.join("deltas/auth.md");
    let approved =
        "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuthentication with reviewed wording.\n";
    fs::write(&delta, approved).unwrap();
    run(&["change", "approve", id, "--actor", "Reviewer"]).success();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approve"]);
    let status_json = || -> Value {
        let output = run(&["--json", "change", "status", id])
            .success()
            .get_output()
            .stdout
            .clone();
        serde_json::from_slice(&output).unwrap()
    };
    let check_next = format!("run `specsync change check {id} --commit`");
    let current = status_json();
    assert_eq!(current["summary"]["approval_valid"], true, "{current}");
    assert_eq!(current["summary"]["next_action"], check_next.as_str());

    fs::write(
        &delta,
        format!("{approved}\n## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL carry wording nobody approved.\n\nAcceptance Criteria\n- Nobody reviewed this.\n"),
    )
    .unwrap();

    let reapprove = format!("specsync change approve {id} --actor <name>");
    let edited = status_json();
    let summary = &edited["summary"];
    assert_eq!(summary["approval_valid"], false, "{edited}");
    assert_eq!(
        summary["next_action"],
        format!("run `{reapprove}`").as_str()
    );
    assert_eq!(
        summary["changed_approved_deltas"],
        serde_json::json!(["auth"])
    );
    assert_eq!(summary["handoff"]["readiness"], "not-yet");
    let text = String::from_utf8(
        run(&["change", "status", id])
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();
    let line = |prefix: &str| {
        text.lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap_or_else(|| panic!("status prints a {prefix} line:\n{text}"))
            .to_string()
    };
    assert!(line("Next:").contains(&reapprove), "{text}");
    assert!(line("Handoff:").contains("Handoff: not yet —"), "{text}");
    run(&["change", "check", id])
        .failure()
        .stderr(predicate::str::contains("changed after approval"));
    run(&["change", "audit"])
        .failure()
        .stderr(predicate::str::contains(format!(
            "{id}: semantic delta for `auth` changed after approval"
        )));

    fs::write(&delta, approved).unwrap();
    let restored = status_json();
    assert_eq!(restored["summary"]["approval_valid"], true, "{restored}");
    assert_eq!(restored["summary"]["next_action"], check_next.as_str());
    assert!(restored["summary"].get("changed_approved_deltas").is_none());
    run(&["change", "audit"]).success();
}

// Verifies REQ-change-112 and REQ-change-023 through the built binary. Under configured
// `enforcement = "strict"`, the first `change check` after approval refuses an undocumented
// export in a declared module and records that refusal, `--commit` commits nothing, and status
// points back at `change check`. Documenting the export lets the next check pass with the single
// scoped command.
#[test]
fn change_check_records_the_effective_contract_refusal_on_the_first_run() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    for dir in [".specsync", "src", "specs/auth"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".specsync/config.toml"),
        "enforcement = \"strict\"\n",
    )
    .unwrap();
    fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
    let spec_path = root.join("specs/auth/auth.spec.md");
    fs::write(
        &spec_path,
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
    git(&["add", "--all"]);
    git(&["commit", "-m", "base"]);
    let run = |args: &[&str]| specsync().args(["--root", root_arg]).args(args).assert();
    let id = "add-an-auth-probe";
    run(&[
        "change",
        "new",
        "Add an auth probe",
        "--kind",
        "bug-fix",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ])
    .success();
    for (question, answer) in [
        ("acceptance_criteria", "The auth probe is documented"),
        ("public_contract", "yes"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", id, question, answer]).success();
    }
    let workspace = root.join(".specsync/changes").join(id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["workflow_version"], 2);
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "tasks" => "# Tasks\n\n- [x] Add the probe.\n",
            "plan" => super::helpers::SUBSTANTIVE_PLAN,
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
    run(&["change", "approve", id, "--actor", "Reviewer"]).success();
    fs::write(
        root.join("src/auth.rs"),
        "// Authentication module.\npub fn undocumented_probe() {}\n",
    )
    .unwrap();
    git(&["add", "--all"]);
    git(&["commit", "-m", "implement"]);
    let head = git(&["rev-parse", "HEAD"]);
    let verification = || -> Value {
        serde_json::from_str(&fs::read_to_string(workspace.join("verification.json")).unwrap())
            .unwrap()
    };
    let gate_entry = format!("specsync change check {id} (effective contract: auth)");

    run(&["--json", "change", "check", id])
        .failure()
        .stdout(predicate::str::contains(
            "Undocumented export 'undocumented_probe'",
        ));
    let first = verification();
    assert_eq!(first["passed"], false, "{first}");
    assert_eq!(
        first["commands"][0]["command"],
        "specsync check --spec auth"
    );
    assert_eq!(first["commands"][1]["command"], gate_entry.as_str());
    assert_eq!(first["commands"][1]["success"], false);
    // The gate-refusal entry carries the refusal texts, not just an exit code
    // (#640, REQ-change-153 AC3).
    let gate_output = first["commands"][1]["output"].as_str().unwrap_or_default();
    assert!(
        gate_output.contains("effective contract `auth`")
            && gate_output.contains("undocumented_probe"),
        "the gate refusal records its texts in output: {first}"
    );

    run(&["change", "check", id, "--commit"]).failure();
    assert_eq!(
        git(&["rev-parse", "HEAD"]),
        head,
        "a refused check commits nothing"
    );
    let status: Value = serde_json::from_slice(
        &run(&["--json", "change", "status", id])
            .success()
            .get_output()
            .stdout,
    )
    .unwrap();
    assert_eq!(
        status["summary"]["next_action"],
        format!("run `specsync change check {id} --commit`").as_str(),
        "{status}"
    );

    let spec = fs::read_to_string(&spec_path).unwrap().replacen(
        "## Public API\n\nNone.\n",
        "## Public API\n\n| Function | Description |\n|----------|-------------|\n| `undocumented_probe` | Probe the auth module |\n",
        1,
    );
    fs::write(&spec_path, spec).unwrap();
    run(&["change", "check", id]).success();
    let passed = verification();
    assert_eq!(passed["passed"], true, "{passed}");
    assert_eq!(
        passed["commands"],
        serde_json::json!([{"command": "specsync check --spec auth", "success": true, "exit_code": 0}])
    );
}

// Issue #825 through the built binary. A workflow-v2 project with one change for `src/auth.rs`,
// approved and verified with `change check --commit`. `entries` is written into `sdd.json` as raw
// JSON, committed, and re-verified (step D, first half), so the fixture also runs against a binary
// that ignores the key.
struct VerificationIgnoredCli {
    _temp: TempDir,
    root: PathBuf,
    id: String,
}

impl VerificationIgnoredCli {
    fn root_arg(&self) -> &str {
        self.root.to_str().unwrap()
    }

    fn git(&self, args: &[&str]) {
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
    }

    fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        specsync()
            .args(["--root", self.root_arg()])
            .args(args)
            .assert()
    }

    fn status_json(&self) -> Value {
        let output = self
            .run(&["--json", "change", "status", &self.id])
            .success()
            .get_output()
            .stdout
            .clone();
        serde_json::from_slice(&output).unwrap()
    }

    fn next_action(&self) -> String {
        self.status_json()["summary"]["next_action"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn review_next(&self) -> String {
        format!(
            "run `specsync change review {} --reviewer <human>` after the PR's scoped review passes",
            self.id
        )
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn set_list(&self, entries: &[&str]) {
        let path = self.root.join(".specsync/sdd.json");
        let mut policy: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        policy["verification_ignored_paths"] = serde_json::json!(entries);
        fs::write(path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
    }

    fn new(entries: &[&str]) -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let fixture = Self {
            _temp: temp,
            root,
            id: "ship-the-auth-module".into(),
        };
        fixture.git(&["init", "-b", "main"]);
        fixture.git(&["config", "user.email", "test@example.com"]);
        fixture.git(&["config", "user.name", "Test"]);
        fixture.write("src/auth.rs", "// Authentication module.\n");
        fixture.write(
            "specs/auth/auth.spec.md",
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        );
        fixture.write(
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
        fixture.git(&["add", "--all"]);
        fixture.git(&["commit", "-m", "base"]);
        let id = fixture.id.clone();
        fixture
            .run(&[
                "change",
                "new",
                "Ship the auth module",
                "--kind",
                "bug-fix",
                "--spec",
                "auth",
                "--path",
                "src/auth.rs",
                "--no-spec-change",
                "--rationale",
                "Internal behavior only",
            ])
            .success();
        for (question, answer) in [
            ("acceptance_criteria", "The auth module ships"),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            fixture
                .run(&["change", "answer", &id, question, answer])
                .success();
        }
        let workspace = fixture.root.join(".specsync/changes").join(&id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        assert_eq!(state["workflow_version"], 2);
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Ship the module.\n",
                "plan" => super::helpers::SUBSTANTIVE_PLAN,
                _ => "# Complete\n\nReviewed auth delivery.\n",
            };
            fs::write(workspace.join(format!("{name}.md")), content).unwrap();
        }
        fixture
            .run(&["change", "approve", &id, "--actor", "Reviewer"])
            .success();
        fixture.write("src/auth.rs", "// Authentication module, shipped.\n");
        fixture.git(&["add", "--all"]);
        fixture.git(&["commit", "-m", "implement"]);
        fixture.run(&["change", "check", &id, "--commit"]).success();
        assert_eq!(fixture.next_action(), fixture.review_next());
        if !entries.is_empty() {
            fixture.set_list(entries);
            fixture.git(&["add", ".specsync/sdd.json"]);
            fixture.git(&["commit", "-m", "list another tool's records"]);
            assert_eq!(
                fixture.next_action(),
                format!("run `specsync change check {id} --commit`"),
                "editing the list stales verification"
            );
        }
        fixture
    }
}

const VIP_NOTE: &str = "SpecSync cannot tell whether project tests or CI read files under these prefixes; edits there do not stale verification.";

// DISCRIMINATOR. Issue #825 steps B, D and F with `verification_ignored_paths` in place of the
// workarounds the issue tried: the record never stales verification before the review, the review
// passes and finalize accepts it with an uncommitted edit under the prefix.
// Verifies REQ-change-114 and REQ-cmd-change-024.
/// Every file under `.specsync/`, with its bytes.
fn specsync_tree_bytes(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    let mut pending = vec![root.join(".specsync")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(path.clone(), fs::read(&path).unwrap());
            }
        }
    }
    files
}

// REQ-cmd-change-032, REQ-change-137: a read past its deadline exits 124, names the phase and the
// exact resume command, and writes nothing; the resume command gives the full verdict.
#[test]
fn an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    fixture.run(&["change", "check", &id, "--commit"]).success();
    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ])
        .success();
    fixture.run(&["change", "finalize", &id]).success();
    fixture.git(&["add", "--all"]);
    fixture.git(&["commit", "--allow-empty", "-m", "archive the change"]);
    let before = specsync_tree_bytes(&fixture.root);

    for timeout in [["--timeout", "0"], ["--timeout=0", ""]] {
        let mut args = vec!["change", "status", id.as_str(), "--json"];
        args.extend(timeout.iter().filter(|argument| !argument.is_empty()));
        let output = fixture.run(&args).code(124).get_output().stdout.clone();
        let report: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["error"], "timeout", "{report}");
        assert_eq!(report["wrote"], false);
        assert_eq!(report["command"], "change status");
        assert!(
            report["phase"]
                .as_str()
                .is_some_and(|phase| !phase.is_empty())
        );
        let resume = report["resume"].as_str().unwrap();
        assert!(resume.starts_with("specsync "), "{resume}");
        assert!(
            resume.contains(&format!("change status {id} --json")),
            "{resume}"
        );
        assert!(!resume.contains("--timeout"), "{resume}");
    }
    let from_env = specsync()
        .env("SPECSYNC_TIMEOUT", "0")
        .args([
            "--root",
            fixture.root_arg(),
            "change",
            "status",
            &id,
            "--json",
        ])
        .assert()
        .code(124)
        .get_output()
        .stdout
        .clone();
    let from_env: Value = serde_json::from_slice(&from_env).unwrap();
    assert_eq!(from_env["error"], "timeout");
    // #953: the resume command must not inherit the deadline it is resuming from.
    let resume = from_env["resume"].as_str().unwrap();
    assert!(
        resume.starts_with("SPECSYNC_TIMEOUT= specsync "),
        "with SPECSYNC_TIMEOUT set, the resume command must clear it: {resume}"
    );
    // Text `change status` of an archived change is cheap and reaches no long phase, so it
    // completes; `ship-status` reads the archive-introduction index and reports in text.
    let text = fixture
        .run(&["change", "ship-status", &id, "--timeout", "0"])
        .code(124)
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8(text).unwrap();
    assert!(
        text.contains("change ship-status timed out after 0s during")
            && text.contains("Nothing was written")
            && text.contains(&format!(
                "specsync --root {} change ship-status {id}",
                fixture.root_arg()
            )),
        "{text}"
    );
    assert_eq!(
        specsync_tree_bytes(&fixture.root),
        before,
        "a timed-out read writes nothing"
    );

    let full = fixture.status_json();
    assert_eq!(full["change"]["state"], "archived", "{full}");
    assert_eq!(
        full["summary"]["terminal_evidence"]["validity"], "authenticated-history",
        "{full}"
    );
    for bad in ["-1", "abc"] {
        fixture
            .run(&["change", "status", &id, "--timeout", bad])
            .failure()
            .code(2);
    }
}

#[test]
fn listed_tool_records_do_not_stale_review_through_the_cli() {
    let fixture = VerificationIgnoredCli::new(&[".chough/"]);
    let id = fixture.id.clone();
    let checked = fixture
        .run(&["change", "check", &id, "--commit"])
        .success()
        .get_output()
        .stdout
        .clone();
    let checked = String::from_utf8(checked).unwrap();
    assert_eq!(fixture.next_action(), fixture.review_next());

    // Step B: another tool writes an untracked record after verification.
    fixture.write(".chough/evidence/obs.json", "{\"observed\":true}\n");
    let status = fixture.status_json();
    assert_eq!(
        status["summary"]["next_action"],
        fixture.review_next().as_str(),
        "{status}"
    );
    assert_eq!(
        status["summary"]["verification_ignored_paths"],
        serde_json::json!([".chough/"])
    );
    let text = String::from_utf8(
        fixture
            .run(&["change", "status", &id])
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();

    // Step D, second half: remove the record, then restore it.
    fs::remove_file(fixture.root.join(".chough/evidence/obs.json")).unwrap();
    assert_eq!(fixture.next_action(), fixture.review_next());
    fixture.write(".chough/evidence/obs.json", "{\"observed\":true}\n");
    assert_eq!(fixture.next_action(), fixture.review_next());

    // Step F: commit the record before the review.
    fixture.git(&["add", ".chough/evidence/obs.json"]);
    fixture.git(&["commit", "-m", "record another tool's observation"]);
    assert_eq!(fixture.next_action(), fixture.review_next());

    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ])
        .success();
    fixture.write(".chough/evidence/obs.json", "{\"observed\":\"again\"}\n");
    assert_eq!(
        fixture.next_action(),
        format!("run `specsync change finalize {id}`")
    );
    fixture.run(&["change", "finalize", &id]).success();

    // Disclosure, asserted last so the behaviour above is what a80748ff fails on first.
    for output in [&checked, &text] {
        assert!(
            output.contains("Verification ignores: .chough/"),
            "{output}"
        );
        assert!(output.contains(VIP_NOTE), "{output}");
    }
}

// DISCRIMINATOR for "a commit after the review stales it THROUGH THE DESCENDANT WALK". On
// a80748ff the `.chough/` commit stales verification itself, so status asks for `change check`
// and ship-status gives no walk message; with the list the digest stays put and only the
// unchanged walk can make the review stale.
// Verifies REQ-change-114.
#[test]
fn listed_prefix_commit_after_review_still_stales_through_the_cli() {
    let fixture = VerificationIgnoredCli::new(&[".chough/"]);
    let id = fixture.id.clone();
    fixture.run(&["change", "check", &id, "--commit"]).success();
    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ])
        .success();
    fixture.write(".chough/obs.json", "committed after the review\n");
    fixture.git(&["add", ".chough/obs.json"]);
    fixture.git(&["commit", "-m", "record after the review"]);
    assert_ne!(
        fixture.next_action(),
        format!("run `specsync change finalize {id}`")
    );
    // The listed commit leaves verification current, so status asks for a fresh review rather
    // than a fresh check, and the reason ship-status gives is the descendant walk's own message
    // naming the path the post-review commit touched.
    assert_eq!(fixture.next_action(), fixture.review_next());
    fixture
        .run(&["change", "finalize", &id])
        .failure()
        .stderr(predicate::str::contains(
            "independent scoped review is stale",
        ));
    let shipped = fixture
        .run(&["--json", "change", "ship-status", &id])
        .get_output()
        .stdout
        .clone();
    let shipped: Value = serde_json::from_slice(&shipped).unwrap();
    let walk = "changed `.chough/obs.json`, which is outside this change's own lifecycle records";
    assert!(
        shipped["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker.as_str().unwrap().contains(walk)),
        "{shipped}"
    );
}

// DISCRIMINATOR. `change check` reads the configured `schema_dir` in-process to judge spec
// `db_tables`, so an entry covering it is refused by every verb rather than letting a schema edit
// leave verification and review current.
// Verifies REQ-change-115.
#[test]
fn entry_covering_schema_dir_is_refused_through_the_cli() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    fixture.write(
        ".specsync/config.toml",
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nschema_dir = \"db/schema\"\n",
    );
    fixture.write(
        "db/schema/001_users.sql",
        "CREATE TABLE users (id INTEGER);\n",
    );
    for entry in ["db/schema/", "db/"] {
        fixture.set_list(&[entry]);
        let refusal = format!(
            "invalid SDD policy: verification_ignored_paths entry `{entry}` overlaps schema_dir `db/schema`"
        );
        fixture
            .run(&[
                "change",
                "new",
                "Another change beside a hidden schema",
                "--kind",
                "bug-fix",
                "--path",
                "src/auth.rs",
                "--no-spec-change",
                "--rationale",
                "Internal behavior only",
            ])
            .failure()
            .stderr(predicate::str::contains(refusal.as_str()));
        for args in [
            vec!["change", "check", id.as_str(), "--commit"],
            vec![
                "change",
                "review",
                id.as_str(),
                "--reviewer",
                "Independent reviewer",
            ],
        ] {
            fixture
                .run(&args)
                .failure()
                .stderr(predicate::str::contains(refusal.as_str()));
        }
        let next = fixture.next_action();
        assert!(next.contains(refusal.as_str()), "{next}");
    }
}

// DISCRIMINATOR. An invalid list fails closed in every verb, status names it, and ship-status
// lists it as a blocker.
// Verifies REQ-change-115 and REQ-cmd-change-024.
#[test]
fn invalid_verification_ignored_paths_are_refused_through_the_cli() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    fixture.set_list(&["src/"]);
    let refusal = "invalid SDD policy: verification_ignored_paths entry `src/`";
    fixture
        .run(&[
            "change",
            "new",
            "Another change under an invalid list",
            "--kind",
            "bug-fix",
            "--path",
            "src/auth.rs",
            "--no-spec-change",
            "--rationale",
            "Internal behavior only",
        ])
        .failure()
        .stderr(predicate::str::contains(refusal));
    for args in [
        vec!["change", "approve", id.as_str(), "--actor", "Reviewer"],
        vec!["change", "check", id.as_str()],
        vec![
            "change",
            "review",
            id.as_str(),
            "--reviewer",
            "Independent reviewer",
        ],
        vec!["change", "finalize", id.as_str()],
        vec!["change", "audit"],
    ] {
        fixture
            .run(&args)
            .failure()
            .stderr(predicate::str::contains(refusal));
    }
    let next = fixture.next_action();
    assert!(next.contains(refusal), "{next}");
    let shipped = fixture
        .run(&["--json", "change", "ship-status", &id])
        .get_output()
        .stdout
        .clone();
    let shipped: Value = serde_json::from_slice(&shipped).unwrap();
    let blockers = shipped["blockers"].as_array().unwrap();
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.as_str().unwrap().contains(refusal)),
        "{shipped}"
    );
}

// CONTROL. Without the key the issue #825 behaviour is unchanged: an untracked record stales
// verification and review refuses with the stale-digest message.
// Verifies REQ-change-114.
#[test]
fn absent_list_keeps_issue_825_behaviour() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    fixture.write(".chough/evidence/obs.json", "{\"observed\":true}\n");
    assert_eq!(
        fixture.next_action(),
        format!("run `specsync change check {id} --commit`")
    );
    let status = fixture.status_json();
    assert!(
        status["summary"]
            .get("verification_ignored_paths")
            .is_none()
    );
    let text = String::from_utf8(
        fixture
            .run(&["change", "status", &id])
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();
    assert!(!text.contains("Verification ignores"), "{text}");
    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ])
        .failure()
        .stderr(predicate::str::contains(
            "verification project-input digest is stale",
        ));
    fixture.git(&["add", ".chough/evidence/obs.json"]);
    fixture.git(&["commit", "-m", "commit the record instead"]);
    assert_eq!(
        fixture.next_action(),
        format!("run `specsync change check {id} --commit`")
    );
}

// Verifies REQ-change-113 and REQ-cmd-change-023 through the built binary. `change status --json`
// and `change show --json` carry a top-level `approved_deltas` array: the requirement IDs and
// sections an intact approved delta declares, withheld once the body is edited, CRLF-agnostic,
// and `materialized` after `change check`. Text output, `summary` and `change list --json` gain
// nothing, a draft reports `[]`, and `change depend --json` on an approved change carries the
// entries `change status --json` reports.
#[test]
fn status_and_show_json_report_approved_delta_requirements() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();
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
    for dir in [".specsync", "src", "specs/auth", "specs/billing"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    for module in ["auth", "billing"] {
        fs::write(
            root.join(format!("src/{module}.rs")),
            format!("// {module} module.\n"),
        )
        .unwrap();
        fs::write(
            root.join(format!("specs/{module}/{module}.spec.md")),
            format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/{module}.rs\n---\n\n# {module}\n\n## Purpose\n\nThe {module} module.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
        )
        .unwrap();
    }
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
    git(&["add", "--all"]);
    git(&["commit", "-m", "base"]);
    let run = |args: &[&str]| specsync().args(["--root", root_arg]).args(args).assert();
    let json = |args: &[&str]| -> Value {
        let mut full = vec!["--json"];
        full.extend_from_slice(args);
        let output = run(&full).success().get_output().stdout.clone();
        serde_json::from_slice(&output).unwrap()
    };
    let draft = |id: &str, description: &str, module: &str, delta: &str| {
        run(&[
            "change",
            "new",
            description,
            "--kind",
            "bug-fix",
            "--spec",
            module,
            "--path",
            &format!("src/{module}.rs"),
        ])
        .success();
        for (question, answer) in [
            ("acceptance_criteria", "The session requirement is declared"),
            ("public_contract", "yes"),
            ("architecture_risk", "no"),
        ] {
            run(&["change", "answer", id, question, answer]).success();
        }
        let workspace = root.join(".specsync/changes").join(id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        assert_eq!(state["workflow_version"], 2);
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Declare the requirement.\n",
                "plan" => super::helpers::SUBSTANTIVE_PLAN,
                "testing" => {
                    "# Testing\n\nThe integration test covers REQ-auth-002 and REQ-billing-002.\n"
                }
                _ => "# Complete\n\nReviewed delta wording.\n",
            };
            fs::write(workspace.join(format!("{name}.md")), content).unwrap();
        }
        let path = workspace.join(format!("deltas/{module}.md"));
        fs::write(&path, delta).unwrap();
        path
    };
    let id = "declare-the-session-requirement";
    let approved = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL keep a session alive across a refresh; REQ-auth-009 is only mentioned.\n\nAcceptance Criteria\n- A session survives a page refresh.\n\n## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuthentication and sessions.\n";
    let delta = draft(id, "Declare the session requirement", "auth", approved);

    let draft_status = json(&["change", "status", id]);
    assert_eq!(
        draft_status["approved_deltas"],
        serde_json::json!([]),
        "a draft reports an empty list, and the key is present: {draft_status}"
    );
    let draft_keys: Vec<String> = draft_status.as_object().unwrap().keys().cloned().collect();

    run(&["change", "approve", id, "--actor", "Reviewer"]).success();
    git(&["add", "--all"]);
    git(&["commit", "-m", "approve"]);
    let expected = serde_json::json!([{
        "module": "auth",
        "delta_intact": true,
        "requirements": {"added": ["REQ-auth-002"], "modified": [], "removed": []},
        "sections": {"added": [], "modified": ["Purpose"], "removed": []},
        "materialized": false
    }]);
    let status = json(&["change", "status", id]);
    let show = json(&["change", "show", id]);
    assert_eq!(status["approved_deltas"], expected, "{status}");
    assert_eq!(show["approved_deltas"], expected, "{show}");
    assert_eq!(status["summary"]["approval_valid"], true);
    assert!(
        status["summary"].get("approved_deltas").is_none(),
        "summary gains nothing: {status}"
    );
    let mut keys: Vec<String> = status.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    let mut previous = vec![
        "acceptance_entries",
        "approved_deltas",
        "briefing",
        "change",
        "corrections",
        "effective_definition",
        "questions",
        "summary",
    ];
    previous.sort();
    // REQ-cmd-change-relayed-decision-001 (#633): an approved change also carries the approval it
    // records as `recorded_claims`, and a draft, which records none, omits the key.
    let mut approved_keys = previous.clone();
    approved_keys.push("recorded_claims");
    approved_keys.sort();
    assert_eq!(
        keys, approved_keys,
        "one new top-level key, plus the recorded approval claim, and nothing else"
    );
    assert_eq!(
        status["recorded_claims"],
        serde_json::json!([{"gate": "definition", "name": "Reviewer"}]),
        "{status}"
    );
    let mut draft_keys = draft_keys;
    draft_keys.sort();
    assert_eq!(draft_keys, previous);
    // The `summary` key set a80748f prints for this same fixture, in the draft and the approved
    // intact state: `summary` gains, loses and renames nothing.
    let previous_summary = vec![
        "approval_valid",
        "artifacts_complete",
        "correction_count",
        "correction_valid",
        "definition_digest",
        "handoff",
        "id",
        "next_action",
        "scoped_review_current",
        "state",
        "strict_validation_required",
        "title",
        "verification_commands",
    ];
    let summary_keys = |value: &Value| -> Vec<String> {
        let mut keys: Vec<String> = value["summary"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    };
    assert_eq!(
        summary_keys(&draft_status),
        previous_summary,
        "{draft_status}"
    );
    assert_eq!(summary_keys(&status), previous_summary, "{status}");
    assert_eq!(summary_keys(&show), previous_summary, "{show}");

    fs::write(&delta, approved.replace('\n', "\r\n")).unwrap();
    assert_eq!(
        json(&["change", "status", id])["approved_deltas"],
        expected,
        "CRLF alone is not an edit"
    );

    fs::write(
        &delta,
        approved.replace("keep a session alive", "carry wording nobody approved"),
    )
    .unwrap();
    let edited = json(&["change", "status", id]);
    assert_eq!(
        edited["approved_deltas"],
        serde_json::json!([{"module": "auth", "delta_intact": false}]),
        "{edited}"
    );
    assert_eq!(
        edited["summary"]["changed_approved_deltas"],
        serde_json::json!(["auth"])
    );
    assert_eq!(edited["summary"]["approval_valid"], false);
    fs::write(&delta, approved).unwrap();

    run(&["change", "check", id]).success();
    let checked = json(&["change", "status", id]);
    assert_eq!(
        checked["approved_deltas"][0]["materialized"], true,
        "{checked}"
    );
    assert_eq!(
        checked["approved_deltas"][0]["requirements"]["added"],
        serde_json::json!(["REQ-auth-002"])
    );

    let text = String::from_utf8(
        run(&["change", "status", id])
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();
    assert!(
        !text.contains("REQ-auth-002"),
        "text output is unchanged:\n{text}"
    );
    assert!(!text.contains("approved_deltas"), "{text}");
    let list = json(&["change", "list"]);
    assert!(
        !list.to_string().contains("approved_deltas"),
        "`change list --json` gains nothing: {list}"
    );

    // `change depend` on an approved change: the dependency is declared before approval, so
    // re-declaring it leaves the approved scope current and the in-lock entries are intact.
    let prerequisite = "add-a-billing-prerequisite";
    draft(
        prerequisite,
        "Add a billing prerequisite",
        "billing",
        "## ADDED\n\n### REQUIREMENT REQ-billing-002\n\nThe system SHALL bill once.\n\nAcceptance Criteria\n- One charge.\n",
    );
    let dependent = "add-a-billing-dependent";
    draft(
        dependent,
        "Add a billing dependent",
        "billing",
        "## MODIFIED\n\n### SPEC SECTION Purpose\n\nBilling, after its prerequisite.\n",
    );
    run(&["change", "depend", dependent, prerequisite]).success();
    run(&["change", "approve", dependent, "--actor", "Reviewer"]).success();
    let depended = json(&["change", "depend", dependent, prerequisite]);
    let dependent_status = json(&["change", "status", dependent]);
    assert_eq!(depended["summary"]["approval_valid"], true, "{depended}");
    assert_eq!(
        depended["approved_deltas"],
        serde_json::json!([{
            "module": "billing",
            "delta_intact": true,
            "requirements": {"added": [], "modified": [], "removed": []},
            "sections": {"added": [], "modified": ["Purpose"], "removed": []},
            "materialized": false
        }]),
        "{depended}"
    );
    assert_eq!(
        depended["approved_deltas"],
        dependent_status["approved_deltas"]
    );

    // `change new --json` renders the draft it just created, so its list is always empty. The
    // key, the other top-level keys and the `summary` keys match `change status --json`.
    let created = json(&[
        "change",
        "new",
        "Add a draft for the new output",
        "--kind",
        "bug-fix",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ]);
    assert_eq!(created["change"]["state"], "draft", "{created}");
    assert_eq!(
        created["approved_deltas"],
        serde_json::json!([]),
        "{created}"
    );
    let mut created_keys: Vec<String> = created.as_object().unwrap().keys().cloned().collect();
    created_keys.sort();
    assert_eq!(created_keys, previous);
    assert_eq!(summary_keys(&created), previous_summary, "{created}");
}

// REQ-change-127: one change driven through the lifecycle stays within a git process ceiling per
// command. Counted through a `git` wrapper on PATH, so every spawn the binary makes is seen,
// whichever module makes it, and the bound is hardware independent.
#[cfg(unix)]
#[test]
fn lifecycle_commands_stay_within_git_spawn_ceilings() {
    use std::os::unix::fs::PermissionsExt;

    /// Git processes each lifecycle command may spawn on this small fixture.
    ///
    /// At 67aa7dc2 they spawned: new 63, answer 71, approve 89, check 388, status 66, review 323,
    /// finalize 739. With repository facts asked once per command and committed objects read
    /// through one `cat-file --batch` (REQ-git-utils-009, REQ-git-utils-010) they spawn: new 29,
    /// answer 15, approve 32, check 110, status 53, review 176, finalize 287. Each ceiling is that
    /// count plus about 10%. Evidence capture keeps its working-tree reads and repository-identity
    /// reads live on purpose: it reads before and after each capture to catch a concurrent edit,
    /// and that race check is never answered from memory. A passing `change review` also runs the
    /// ownership preflight finalize and ship-status run (#1002), measured at 203, so its ceiling
    /// is 203 plus about 10%.
    const CEILINGS: &[(&str, usize)] = &[
        ("new", 32),
        ("answer", 17),
        ("approve", 35),
        ("check", 121),
        ("status", 58),
        ("review", 223),
        ("finalize", 316),
    ];

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    let shim_dir = temp.path().join("shim");
    fs::create_dir_all(&shim_dir).unwrap();
    let count_file = temp.path().join("git-spawns");
    let real_git = String::from_utf8(
        Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let shim = shim_dir.join("git");
    fs::write(
        &shim,
        format!(
            "#!/bin/sh\necho x >> '{}'\nexec '{}' \"$@\"\n",
            count_file.display(),
            real_git
        ),
    )
    .unwrap();
    fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    let shim_path = format!(
        "{}:{}",
        shim_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let git = |args: &[&str]| {
        let output = Command::new(&real_git)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    let spawns = || {
        fs::read_to_string(&count_file)
            .map(|text| text.lines().count())
            .unwrap_or(0)
    };
    let mut counts: Vec<(&str, usize)> = Vec::new();
    let mut run = |label: &'static str, args: &[&str]| -> Vec<u8> {
        let before = spawns();
        let output = specsync()
            .env("PATH", &shim_path)
            .arg("--root")
            .arg(&root)
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let spawned = spawns() - before;
        match counts.iter_mut().find(|(name, _)| *name == label) {
            Some((_, most)) => *most = (*most).max(spawned),
            None => counts.push((label, spawned)),
        }
        output
    };

    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(&["switch", "-c", "spawn-ceiling"]);

    let created = run(
        "new",
        &[
            "--json",
            "change",
            "new",
            "Bound git spawns per lifecycle command",
            "--kind",
            "documentation",
            "--path",
            "docs/spawn-ceiling.md",
            "--no-spec-change",
            "--rationale",
            "Documentation-only behavior does not alter a technical contract",
        ],
    );
    let created: Value = serde_json::from_slice(&created).unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The ceiling holds for every command"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run("answer", &["change", "answer", &id, question, answer]);
    }
    let active_dir = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(active_dir.join("state.json")).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let artifact = artifact.as_str().unwrap();
        let content = if artifact == "tasks" {
            "# Tasks\n\n- [x] Bound the spawns.\n".to_string()
        } else {
            format!("# {}\n\nComplete.\n", artifact.replace('-', " "))
        };
        fs::write(active_dir.join(format!("{artifact}.md")), content).unwrap();
    }
    run(
        "approve",
        &["change", "approve", &id, "--actor", "Scope owner"],
    );
    fs::write(root.join("docs/spawn-ceiling.md"), "# Spawn ceiling\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "implement"]);
    run("check", &["change", "check", &id]);
    run("status", &["--json", "change", "status", &id]);
    run(
        "review",
        &[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ],
    );
    run("finalize", &["--json", "change", "finalize", &id]);

    eprintln!("git spawns per lifecycle command: {counts:?}");
    for (label, ceiling) in CEILINGS {
        let spawned = counts
            .iter()
            .find(|(name, _)| name == label)
            .map(|(_, spawned)| *spawned)
            .unwrap_or_else(|| panic!("`change {label}` did not run"));
        assert!(
            spawned <= *ceiling,
            "`change {label}` spawned {spawned} git processes; its ceiling is {ceiling} \
             (all commands: {counts:?})"
        );
    }
}

// Issue #892 (audit SS-M05) through the built binary, under workflow v2: one task box is opened
// before the first check, and later reopened after a passing check. Every verb that would pass,
// review or finalize the change refuses, names the open task and the exact command to re-run, and
// the test runs that command after ticking the box. Bypassing the incomplete-task guard (the
// SS-M05 mutation) makes the first check pass, so this test fails.
struct OpenTaskCli {
    _temp: TempDir,
    root: PathBuf,
    id: String,
}

const OPEN_TASKS_DONE: &str = "# Tasks\n\n- [x] Write the fix.\n- [x] Prove the fix.\n";
const OPEN_TASKS_ONE_OPEN: &str = "# Tasks\n\n- [x] Write the fix.\n- [ ] Prove the fix.\n";

impl OpenTaskCli {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let fixture = Self {
            _temp: temp,
            root,
            id: String::new(),
        };
        fixture.git(&["init", "-b", "main"]);
        fixture.git(&["config", "user.email", "test@example.com"]);
        fixture.git(&["config", "user.name", "Test"]);
        fixture.git(&["config", "core.autocrlf", "false"]);
        fixture.git(&["config", "core.eol", "lf"]);
        let root = &fixture.root;
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(root.join(".github/scripts")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(".github/scripts/lifecycle-validation-limits.json"),
            root.join(".github/scripts/lifecycle-validation-limits.json"),
        )
        .unwrap();
        fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
        fs::write(
            root.join(".specsync/sdd.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "version": 2,
                "enabled": true,
                "require_change_for_meaningful_files": false,
                "meaningful_paths": [],
                "ignored_paths": [".specsync/"],
                "verification_commands": ["true"],
                "custom_artifacts": {},
                "principles_file": null
            }))
            .unwrap(),
        )
        .unwrap();
        fixture.git(&["add", "."]);
        fixture.git(&["commit", "-m", "base"]);
        let created: Value = serde_json::from_slice(
            &fixture
                .run(&[
                    "--json",
                    "change",
                    "new",
                    "Keep the guide current",
                    "--kind",
                    "bug-fix",
                    "--path",
                    "docs/guide.md",
                    "--no-spec-change",
                    "--rationale",
                    "Documentation-only behavior does not alter a technical contract",
                ])
                .success()
                .get_output()
                .stdout,
        )
        .unwrap();
        let id = created["change"]["id"].as_str().unwrap().to_string();
        let fixture = Self { id, ..fixture };
        for (question, answer) in [
            ("acceptance_criteria", "The guide is current"),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            fixture
                .run(&["change", "answer", &fixture.id, question, answer])
                .success();
        }
        let state: Value = serde_json::from_str(
            &fs::read_to_string(fixture.workspace().join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            state["workflow_version"], 2,
            "a workflow-v2 defect needs a v2 change"
        );
        let artifacts = state["selected_artifacts"].as_array().unwrap();
        assert!(
            artifacts.iter().any(|artifact| artifact == "tasks"),
            "the fixture must select tasks.md: {artifacts:?}"
        );
        for artifact in artifacts {
            let artifact = artifact.as_str().unwrap();
            let content = match artifact {
                "tasks" => OPEN_TASKS_DONE.to_string(),
                "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
                _ => format!("# {artifact}\n\nComplete evidence for the guide.\n"),
            };
            fs::write(fixture.workspace().join(format!("{artifact}.md")), content).unwrap();
        }
        fixture
            .run(&[
                "change",
                "approve",
                &fixture.id,
                "--actor",
                "Fixture Approver",
            ])
            .success();
        fs::write(fixture.root.join("docs/guide.md"), "# Guide\n\nCurrent.\n").unwrap();
        fixture.git(&["add", "--all"]);
        fixture.git(&["commit", "-m", "implement"]);
        fixture
    }

    fn workspace(&self) -> PathBuf {
        self.root.join(".specsync/changes").join(&self.id)
    }

    fn tasks_path(&self) -> String {
        format!(".specsync/changes/{}/tasks.md", self.id)
    }

    fn git(&self, args: &[&str]) {
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
    }

    fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        specsync()
            .args(["--root", self.root.to_str().unwrap()])
            .args(args)
            .assert()
    }

    fn set_tasks(&self, content: &str) {
        fs::write(self.workspace().join("tasks.md"), content).unwrap();
    }

    /// Run a refused command; return its stderr after asserting that it names the open task.
    fn refused(&self, args: &[&str]) -> String {
        let output = self.run(args).failure().get_output().clone();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("tasks.md contains incomplete tasks"),
            "`{}` must refuse the open task: {stderr}",
            args.join(" ")
        );
        assert!(
            stderr.contains(&self.tasks_path())
                && stderr.contains("line 4")
                && stderr.contains("Prove the fix."),
            "the refusal names the task file, line and text: {stderr}"
        );
        stderr
    }

    /// The exact command a refusal names after "then re-run".
    fn named_rerun(stderr: &str) -> String {
        let start = stderr
            .find("then re-run `")
            .unwrap_or_else(|| panic!("the refusal names no command to re-run: {stderr}"))
            + "then re-run `".len();
        let length = stderr[start..].find('`').unwrap();
        stderr[start..start + length].to_string()
    }

    /// Run a command exactly as a refusal printed it.
    fn run_named(&self, command: &str) -> assert_cmd::assert::Assert {
        let words = shell_words_for_test(command);
        assert_eq!(
            words.first().map(String::as_str),
            Some("specsync"),
            "{command}"
        );
        let args: Vec<&str> = words[1..].iter().map(String::as_str).collect();
        self.run(&args)
    }

    fn status_text_next(&self) -> String {
        let output = self
            .run(&["change", "status", &self.id])
            .success()
            .get_output()
            .clone();
        let stdout = String::from_utf8(output.stdout).unwrap();
        stdout
            .lines()
            .find_map(|line| line.trim().strip_prefix("Next: "))
            .unwrap_or_else(|| panic!("status prints no Next line: {stdout}"))
            .to_string()
    }

    fn status_json_next(&self) -> String {
        let output = self
            .run(&["--json", "change", "status", &self.id])
            .success()
            .get_output()
            .clone();
        let status: Value = serde_json::from_slice(&output.stdout).unwrap();
        status["summary"]["next_action"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn assert_next_names_open_task_then(&self, text_then: &str, json_then: &str) {
        for (surface, next, then) in [
            ("text", self.status_text_next(), text_then),
            ("json", self.status_json_next(), json_then),
        ] {
            let task = next.find("line 4").unwrap_or_else(|| {
                panic!("{surface} status must name the open task first: {next}")
            });
            assert!(
                next.contains(&self.tasks_path()) && next.contains("Prove the fix."),
                "{surface} status names the task file and text: {next}"
            );
            let command = next
                .find(then)
                .unwrap_or_else(|| panic!("{surface} status must then name `{then}`: {next}"));
            assert!(
                task < command,
                "{surface} status names the task before `{then}`: {next}"
            );
        }
    }

    fn archived(&self) -> bool {
        let archive = self.root.join(".specsync/archive/changes");
        fs::read_dir(&archive).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|entry| entry.file_name().to_string_lossy().ends_with(&self.id))
        })
    }
}

/// Split a printed command on spaces, honouring single quotes (enough for the commands the
/// lifecycle prints: ids, flags and a quoted reviewer).
fn shell_words_for_test(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    for character in command.chars() {
        match character {
            '\'' => quoted = !quoted,
            ' ' if !quoted => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            _ => word.push(character),
        }
    }
    assert!(!quoted, "unbalanced quote in `{command}`");
    if !word.is_empty() {
        words.push(word);
    }
    words
}

// Verifies REQ-change-131, REQ-change-132 and REQ-cmd-change-031 (issue #892, audit SS-M05).
// DISCRIMINATOR for the guard: with it bypassed the first `change check` passes and this fails.
#[test]
fn open_task_gate_refuses_and_names_the_command_that_recovers() {
    let fixture = OpenTaskCli::new();
    let id = fixture.id.clone();

    // 1. One task open before the first check.
    fixture.set_tasks(OPEN_TASKS_ONE_OPEN);
    fixture.git(&["commit", "-am", "leave one task open"]);
    let stderr = fixture.refused(&["change", "check", &id, "--commit"]);
    assert_eq!(
        OpenTaskCli::named_rerun(&stderr),
        format!("specsync change check {id} --commit"),
        "check names itself with the flags it was given"
    );
    assert!(
        !fixture.workspace().join("verification.json").exists(),
        "a refused check records no verification"
    );
    let check_next = format!("then run `specsync change check {id} --commit`");
    fixture.assert_next_names_open_task_then(&check_next, &check_next);

    // Tick the box and run exactly what the refusal named.
    fixture.set_tasks(OPEN_TASKS_DONE);
    fixture
        .run_named(&OpenTaskCli::named_rerun(&stderr))
        .success();
    assert!(fixture.workspace().join("verification.json").exists());

    // 2. A task reopened after the passing check: review refuses.
    fixture.set_tasks(OPEN_TASKS_ONE_OPEN);
    fixture.git(&["commit", "-am", "reopen one task"]);
    fixture.assert_next_names_open_task_then(
        &format!("then run `specsync change check {id} --commit` if needed"),
        &format!("then run `specsync change review {id}"),
    );
    let stderr = fixture.refused(&["change", "review", &id, "--reviewer", "Fixture Reviewer"]);
    assert_eq!(
        OpenTaskCli::named_rerun(&stderr),
        format!("specsync change review {id} --reviewer='Fixture Reviewer'")
    );
    assert!(
        !fixture.workspace().join("review.json").exists(),
        "a refused review records nothing"
    );
    // A blocking verdict on unfinished work is still recordable.
    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Fixture Reviewer",
            "--verdict",
            "block",
        ])
        .success();
    let review: Value =
        serde_json::from_str(&fs::read_to_string(fixture.workspace().join("review.json")).unwrap())
            .unwrap();
    assert_eq!(review["verdict"], "block", "{review}");
    fixture.set_tasks(OPEN_TASKS_DONE);
    fixture
        .run_named(&OpenTaskCli::named_rerun(&stderr))
        .success();
    let review: Value =
        serde_json::from_str(&fs::read_to_string(fixture.workspace().join("review.json")).unwrap())
            .unwrap();
    assert_eq!(
        review["verdict"], "pass",
        "the named review command records the pass: {review}"
    );

    // 3. Reopened again after the review: ship-status, finalize and ship refuse.
    fixture.set_tasks(OPEN_TASKS_ONE_OPEN);
    let output = fixture
        .run(&["--json", "change", "ship-status", &id])
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ready_to_finalize"], false, "{report}");
    assert!(
        report["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| {
                blocker
                    .as_str()
                    .is_some_and(|text| text.contains("tasks.md contains incomplete tasks"))
            }),
        "ship-status reports the open task as a blocker: {report}"
    );
    let stderr = fixture.refused(&["change", "finalize", &id]);
    assert_eq!(
        OpenTaskCli::named_rerun(&stderr),
        format!("specsync change finalize {id}")
    );
    assert!(!fixture.archived(), "a refused finalize archives nothing");
    let stderr = fixture.refused(&["change", "ship", &id, "--dry-run"]);
    assert_eq!(
        OpenTaskCli::named_rerun(&stderr),
        format!("specsync change ship {id} --dry-run")
    );
    let stderr = fixture.refused(&["change", "ship", &id]);
    assert_eq!(
        OpenTaskCli::named_rerun(&stderr),
        format!("specsync change ship {id}")
    );
    assert!(!fixture.archived(), "a refused ship archives nothing");

    // Tick the box and run the named ship command: the change archives.
    fixture.set_tasks(OPEN_TASKS_DONE);
    fixture
        .run_named(&OpenTaskCli::named_rerun(&stderr))
        .success();
    assert!(
        fixture.archived(),
        "the named ship command archives the change"
    );
}

// REQ-change-148: SpecSync ignores environment that rewrites the history Git walks, as it already
// does for `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE`. `GIT_REPLACE_REF_BASE` points Git at
// replacement refs outside `refs/replace/`, the only namespace the read-cache and index guards
// check. A replacement there that swaps `HEAD` for a copy with a downgraded archived record
// must not change what `change status` says, cached or fresh.
#[test]
fn a_custom_replace_ref_base_does_not_change_what_status_reads() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    fixture.run(&["change", "check", &id, "--commit"]).success();
    fixture
        .run(&[
            "change",
            "review",
            &id,
            "--reviewer",
            "Independent reviewer",
        ])
        .success();
    fixture.run(&["change", "finalize", &id]).success();
    fixture.git(&["add", "--all"]);
    fixture.git(&["commit", "--allow-empty", "-m", "archive the change"]);
    let git = |args: &[&str], envs: &[(&str, &str)]| {
        let mut command = Command::new("git");
        command.args(args).current_dir(&fixture.root);
        for (key, value) in envs {
            command.env(key, value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    let read = |envs: &[(&str, &str)], fresh: bool| {
        if fresh {
            let _ = fs::remove_file(fixture.root.join(".git/specsync/read-cache-v1.json"));
        }
        let mut command = specsync();
        command.args([
            "--root",
            fixture.root_arg(),
            "change",
            "status",
            &id,
            "--json",
        ]);
        for (key, value) in envs {
            command.env(key, value);
        }
        let output = command.output().unwrap();
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).to_string(),
        )
    };
    let plain = read(&[], true);

    // A copy of HEAD whose archived state.json has lost its workflow version.
    let archive = fs::read_dir(fixture.root.join(".specsync/archive/changes"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.join("state.json").exists())
        .unwrap();
    let state_path = archive
        .join("state.json")
        .strip_prefix(&fixture.root)
        .unwrap()
        .to_string_lossy()
        .to_string();
    let mut state: Value =
        serde_json::from_slice(&fs::read(archive.join("state.json")).unwrap()).unwrap();
    let object = state.as_object_mut().unwrap();
    object.remove("workflow_version");
    object.remove("workflow_origin_version");
    let downgraded = fixture.root.join("downgraded-state.json");
    fs::write(
        &downgraded,
        format!("{}\n", serde_json::to_string_pretty(&state).unwrap()),
    )
    .unwrap();
    let blob = git(&["hash-object", "-w", downgraded.to_str().unwrap()], &[]);
    fs::remove_file(&downgraded).unwrap();
    let index = fixture.root.join(".git/replacement-index");
    let index = index.to_str().unwrap();
    git(&["read-tree", "HEAD"], &[("GIT_INDEX_FILE", index)]);
    git(
        &[
            "update-index",
            "--cacheinfo",
            &format!("100644,{blob},{state_path}"),
        ],
        &[("GIT_INDEX_FILE", index)],
    );
    let tree = git(&["write-tree"], &[("GIT_INDEX_FILE", index)]);
    let head = git(&["rev-parse", "HEAD"], &[]);
    let parent = git(&["rev-parse", "HEAD~1"], &[]);
    let replacement = git(
        &["commit-tree", &tree, "-p", &parent, "-m", "downgraded copy"],
        &[],
    );
    git(
        &["replace", &head, &replacement],
        &[("GIT_REPLACE_REF_BASE", "refs/alt/")],
    );
    // Git itself now sees the downgraded copy under that namespace.
    assert_eq!(
        git(
            &["rev-parse", "HEAD^{tree}"],
            &[("GIT_REPLACE_REF_BASE", "refs/alt/")]
        ),
        tree
    );

    let env = [("GIT_REPLACE_REF_BASE", "refs/alt/")];
    assert_eq!(
        read(&env, true),
        plain,
        "a fresh read under GIT_REPLACE_REF_BASE"
    );
    assert_eq!(
        read(&env, false),
        plain,
        "a cached read under GIT_REPLACE_REF_BASE"
    );
    let no_replace = [("GIT_NO_REPLACE_OBJECTS", "1")];
    assert_eq!(
        read(&no_replace, true),
        plain,
        "a fresh read under GIT_NO_REPLACE_OBJECTS"
    );
}

// G-deadline (#925, REQ-change-149): a timeout too large to add to the clock is no deadline,
// never a panic, whether it comes from `--timeout` or `SPECSYNC_TIMEOUT`.
#[test]
fn a_timeout_too_large_for_the_clock_reads_normally() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let id = fixture.id.clone();
    let huge = u64::MAX.to_string();
    let flag = fixture.run(&["change", "status", &id, "--json", "--timeout", &huge]);
    let output = flag.get_output();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let env = specsync()
        .env("SPECSYNC_TIMEOUT", &huge)
        .args([
            "--root",
            fixture.root_arg(),
            "change",
            "status",
            &id,
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        env.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&env.stderr)
    );
}

// G-deadline (#924, REQ-change-149): loading active changes is a phase with a deadline
// boundary, so `change list` past its deadline stops with 124 instead of printing a verdict.
#[test]
fn change_list_past_its_timeout_stops_while_loading_active_changes() {
    let fixture = VerificationIgnoredCli::new(&[]);
    let output = fixture.run(&["change", "list", "--json", "--timeout", "0"]);
    let output = output.get_output();
    assert_eq!(
        output.status.code(),
        Some(124),
        "change list --timeout 0 printed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["error"], "timeout");
    assert_eq!(report["phase"], "active change loading", "{report}");
}

/// A failed verification records the findings that failed it, so a flake and a
/// regression are classifiable from the artifact alone (#640).
#[cfg(unix)]
#[test]
fn failed_change_check_records_findings_in_command_evidence() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    // Break the scoped spec: a listed source file does not exist.
    let spec_path = root.join("specs/lib/lib.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        &spec_path,
        spec.replace("  - src/lib.rs", "  - src/lib.rs\n  - src/gone.rs"),
    )
    .unwrap();

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .failure();

    let attempts: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join(format!(".specsync/changes/{id}/verification-attempts.json")),
        )
        .unwrap(),
    )
    .unwrap();
    let attempt = attempts["attempts"].as_array().unwrap().last().unwrap();
    let command = &attempt["commands"][0];
    assert_eq!(command["success"], false);
    let output = command["output"].as_str().unwrap_or("");
    assert!(
        output.contains("gone.rs"),
        "the failed command evidence names the missing file, got: {output}"
    );
}

/// #1027 review: a non-strict check fails on its errors, not its warnings, so the bounded
/// `output` is built from what failed. An error followed by enough warnings to fill 4 KiB is
/// still recorded, rather than pushed out of the tail by findings that did not fail anything.
#[cfg(unix)]
#[test]
fn failed_change_check_output_keeps_the_error_behind_many_warnings() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    let spec_path = root.join("specs/lib/lib.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        &spec_path,
        spec.replace("  - src/lib.rs", "  - src/lib.rs\n  - src/gone.rs"),
    )
    .unwrap();
    let mut source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    for index in 0..150 {
        source.push_str(&format!(
            "pub fn undocumented_export_number_{index}() {{}}\n"
        ));
    }
    fs::write(root.join("src/lib.rs"), source).unwrap();

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .failure();

    let attempts: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join(format!(".specsync/changes/{id}/verification-attempts.json")),
        )
        .unwrap(),
    )
    .unwrap();
    let attempt = attempts["attempts"].as_array().unwrap().last().unwrap();
    let command = &attempt["commands"][0];
    assert_eq!(command["success"], false);
    let output = command["output"].as_str().unwrap_or("");
    assert!(
        output.contains("gone.rs"),
        "the error that failed the check is recorded: {output}"
    );
    assert!(
        !output.contains("warning: "),
        "a non-strict check is not failed by its warnings, so they are not its output: {output}"
    );
}

/// #1047 review: under `--strict` a check's warnings are failures, so a strict check failed
/// only by warnings records those warnings as its output (REQ-change-153).
#[cfg(unix)]
#[test]
fn failed_strict_change_check_output_records_the_warnings_that_failed_it() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    // Warnings only: undocumented exports, and no error.
    let mut source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    for index in 0..3 {
        source.push_str(&format!(
            "pub fn undocumented_export_number_{index}() {{}}\n"
        ));
    }
    fs::write(root.join("src/lib.rs"), source).unwrap();

    specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "check",
            &id,
            "--strict",
        ])
        .assert()
        .failure();

    let attempts: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join(format!(".specsync/changes/{id}/verification-attempts.json")),
        )
        .unwrap(),
    )
    .unwrap();
    let attempt = attempts["attempts"].as_array().unwrap().last().unwrap();
    let command = &attempt["commands"][0];
    assert_eq!(command["success"], false);
    let output = command["output"].as_str().unwrap_or("");
    assert!(
        output.contains("warning: ") && output.contains("undocumented_export_number_0"),
        "a strict check failed by its warnings records them: {output}"
    );
}

/// Passing command evidence omits `output`, so passing records serialize
/// exactly as they did before the field existed (#640).
#[cfg(unix)]
#[test]
fn passing_change_check_records_no_output_key() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let cargo = executable_script(&root.join("stand-in/cargo"), "#!/bin/sh\nexit 0\n");
    let id = change_ready_to_verify(root, &[format!("{} test", cargo.display())]);

    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .success();

    let raw =
        fs::read_to_string(root.join(format!(".specsync/changes/{id}/verification.json"))).unwrap();
    assert!(
        !raw.contains("\"output\""),
        "a passing record carries no output key: {raw}"
    );
}
// #839 (REQ-change-004): `change check --commit --json` must print one JSON document on
// success — the run previously exited 0 with empty stdout, so a runner parsing the output got
// nothing to parse.
#[cfg(unix)]
#[test]
fn change_check_commit_json_prints_the_outcome() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &[]);
    let output = specsync()
        .args([
            "--root",
            root.to_str().unwrap(),
            "change",
            "check",
            &id,
            "--commit",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "a successful --commit --json run must print its outcome"
    );
    let document: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|error| panic!("stdout is one JSON document: {error}\n{stdout}"));
    assert_eq!(document["change"], id, "{document}");
    assert_eq!(document["verified"], true, "{document}");
    assert_eq!(document["pushed"], false, "{document}");
    assert!(
        document["materialize_commit"].is_string() || document["materialize_commit"].is_null(),
        "{document}"
    );
    assert!(
        document["verification_commit"].is_string(),
        "the verification commit is always recorded: {document}"
    );
    assert!(
        document["configured_commands_not_run"].is_array(),
        "the outcome document carries the same disclosure as every other JSON success: {document}"
    );
}

// #839 (REQ-change-004, the change's refusal criterion): a refused `change check --commit
// --json` run prints exactly one JSON error document on stdout and exits non-zero. A progress
// line or a second document on stdout would make the output unparseable for a runner.
#[test]
fn change_check_commit_json_refusal_prints_one_error_document() {
    let fixture = OpenTaskCli::new();
    let id = fixture.id.clone();
    fixture.set_tasks(OPEN_TASKS_ONE_OPEN);
    fixture.git(&["commit", "-am", "leave one task open"]);
    let head_before = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&fixture.root)
        .output()
        .unwrap()
        .stdout;

    let output = fixture
        .run(&["change", "check", &id, "--commit", "--json"])
        .failure()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut documents = serde_json::Deserializer::from_str(&stdout).into_iter::<Value>();
    let document = documents
        .next()
        .unwrap_or_else(|| panic!("a refused --commit --json run prints a document: {stdout:?}"))
        .unwrap_or_else(|error| panic!("stdout starts with a JSON document: {error}\n{stdout}"));
    // The streaming reader fails on any text before the document and yields Some for any
    // text or second document after it, so this pins "one document, nothing else".
    assert!(
        documents.next().is_none(),
        "stdout holds exactly one JSON document and nothing after it: {stdout:?}"
    );
    let error = document["error"]
        .as_str()
        .unwrap_or_else(|| panic!("the document carries an `error` string: {document}"));
    assert!(
        error.contains("tasks.md contains incomplete tasks"),
        "the error names the refusal: {error}"
    );
    let head_after = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&fixture.root)
        .output()
        .unwrap()
        .stdout;
    assert_eq!(head_before, head_after, "a refused run commits nothing");
}
// #676 (REQ-change-155): an ignored_paths entry whose whole coverage is always meaningful is
// disclosed at policy load, on stderr, with the file to edit — not three verbs later at audit.
#[cfg(unix)]
#[test]
fn an_inert_ignored_paths_entry_is_disclosed_at_policy_load() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &[]);
    let policy_path = root.join(".specsync/sdd.json");
    let mut policy: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    policy["ignored_paths"] = serde_json::json!([".specsync/", "specs/"]);
    fs::write(&policy_path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();

    // `change status` loads the policy (the briefing reads the principles file); `change list`
    // only reads the roster and never loads it.
    let output = specsync()
        .args(["--root", root.to_str().unwrap(), "change", "status", &id])
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ignored_paths entry `specs/` is inert"),
        "the note names the inert entry: {stderr}"
    );
    assert!(
        stderr.contains(".specsync/sdd.json"),
        "the note names the file to edit: {stderr}"
    );
    assert!(
        !stderr.contains("`.specsync/` is inert"),
        "the .specsync/ entry does real work and stays silent: {stderr}"
    );
}
// #847 (REQ-change-154): a check refused before verification records the refusal, so a reader
// can see the check was tried and why it failed.
#[cfg(unix)]
#[test]
fn a_refused_check_records_the_refusal_as_an_attempt() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &[]);
    // Reopen the task so the tasks gate refuses before verification starts.
    let tasks_path = root.join(format!(".specsync/changes/{id}/tasks.md"));
    let tasks = fs::read_to_string(&tasks_path).unwrap();
    fs::write(&tasks_path, tasks.replace("- [x]", "- [ ]")).unwrap();

    let output = specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let expected_line = format!(
        "error: tasks.md contains incomplete tasks: finish the 1 open task in \
         `.specsync/changes/{id}/tasks.md` (line 3: \"Complete verification preparation.\") \
         and mark it `[x]`, then re-run `specsync change check {id}`"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.lines().last(),
        Some(expected_line.as_str()),
        "the refusal names the gate, the task and the re-run verbatim: {stderr}"
    );

    // The refusal was recorded: one passed:false attempt carrying the reason.
    let attempts_path = root.join(format!(".specsync/changes/{id}/verification-attempts.json"));
    let attempts: Value = serde_json::from_str(&fs::read_to_string(&attempts_path).unwrap())
        .expect("the refusal left an attempts ledger");
    let entries = attempts["attempts"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "{attempts}");
    assert_eq!(entries[0]["passed"], false, "{attempts}");
    assert!(
        entries[0]["refusal"]
            .as_str()
            .is_some_and(|reason| reason.contains("incomplete tasks")),
        "the attempt carries the refusal reason: {attempts}"
    );
    // And it did not pose as completed verification.
    assert!(
        !root
            .join(format!(".specsync/changes/{id}/verification.json"))
            .exists(),
        "a refusal never writes verification.json"
    );
}

// #847 (REQ-change-154): a refusal after a completed verification preserves the recorded
// verification byte for byte and only appends the refusal to the attempts ledger.
#[cfg(unix)]
#[test]
fn a_refusal_preserves_the_prior_verification_and_attempts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let id = change_ready_to_verify(root, &[]);
    // A completed check records verification.json and a passed attempt.
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .success();
    let verification_path = root.join(format!(".specsync/changes/{id}/verification.json"));
    let attempts_path = root.join(format!(".specsync/changes/{id}/verification-attempts.json"));
    let verification_before = fs::read_to_string(&verification_path)
        .expect("the completed check wrote verification.json");
    let attempts_before: Value = serde_json::from_str(
        &fs::read_to_string(&attempts_path).expect("the completed check left an attempts ledger"),
    )
    .unwrap();
    let passed_before = attempts_before["attempts"].as_array().unwrap().len();
    assert!(passed_before >= 1, "{attempts_before}");

    // Reopen the task so the next check is refused at the tasks gate.
    let tasks_path = root.join(format!(".specsync/changes/{id}/tasks.md"));
    let tasks = fs::read_to_string(&tasks_path).unwrap();
    fs::write(&tasks_path, tasks.replace("- [x]", "- [ ]")).unwrap();
    specsync()
        .args(["--root", root.to_str().unwrap(), "change", "check", &id])
        .assert()
        .code(1);

    // The completed verification is untouched and the refusal only appended.
    assert_eq!(
        fs::read_to_string(&verification_path).unwrap(),
        verification_before,
        "a refusal never overwrites the recorded verification"
    );
    let attempts_after: Value =
        serde_json::from_str(&fs::read_to_string(&attempts_path).unwrap()).unwrap();
    let entries = attempts_after["attempts"].as_array().unwrap();
    assert_eq!(entries.len(), passed_before + 1, "{attempts_after}");
    assert_eq!(
        entries[..passed_before],
        attempts_before["attempts"].as_array().unwrap()[..],
        "the prior attempts are preserved: {attempts_after}"
    );
    let refusal = &entries[passed_before];
    assert_eq!(refusal["passed"], false, "{attempts_after}");
    assert!(
        refusal["refusal"]
            .as_str()
            .is_some_and(|reason| reason.contains("incomplete tasks")),
        "the appended attempt carries the refusal reason: {attempts_after}"
    );
}

/// A change's recorded `base_commit` is used as the coverage diff base only when it is a full commit
/// id (REQ-change-013); any other value falls back to the base used when no change records one, so
/// the coverage gate still inspects the changed paths.
#[test]
fn coverage_uses_a_changes_recorded_base_only_when_it_is_a_full_commit_id() {
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
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    super::helpers::write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": true,
  "meaningful_paths": ["src/"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": [],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        super::helpers::valid_spec("auth", &["src/auth.rs"]),
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    let root_arg = root.to_str().unwrap();
    specsync()
        .args([
            "--root",
            root_arg,
            "change",
            "new",
            "Tune auth",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--no-spec-change",
            "--rationale",
            "internal",
            "--path",
            "src/auth.rs",
        ])
        .assert()
        .success();
    let state_path = root.join(".specsync/changes/tune-auth/state.json");
    let mut state: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["base_commit"] = Value::String("--not-a-commit".into());
    fs::write(
        &state_path,
        serde_json::to_string_pretty(&state).unwrap() + "\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "record"]);
    fs::write(root.join("src/auth.rs"), "pub fn auth() { }\n").unwrap();
    git(&["commit", "-q", "-am", "edit"]);

    // No remote and no CI base, so the gate would fall back to the change's recorded base.
    let output = specsync()
        .args(["--root", root_arg, "change", "audit", "--strict"])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !text.contains("unable to inspect"),
        "a recorded base that is not a full commit id is not used as the diff base: {text}"
    );
}

/// #1026 review: the bootstrap exemption is exactly "no spec files and no registry entries". A
/// project whose only declaration is a registry entry is checked, and a missing registered spec
/// is named as registered.
#[test]
fn change_new_checks_affected_specs_when_only_the_registry_declares_modules() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();
    super::helpers::write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/billing.rs"), "pub fn bill() {}\n").unwrap();
    fs::write(
        root.join("specsync-registry.toml"),
        "name = \"proj\"\n\n[specs]\nbilling = \"specs/billing/billing.spec.md\"\n",
    )
    .unwrap();
    specsync()
        .args([
            "--root",
            root_arg,
            "change",
            "new",
            "Bill",
            "--spec",
            "billing",
            "--path",
            "src/billing.rs",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "is registered at `specs/billing/billing.spec.md`, but no spec exists there",
        ));
}

/// #1026 review: an answer that parses to no value is refused rather than silently clearing a
/// list, and `none` clears the lists that may be empty.
#[test]
fn change_answer_refuses_empty_lists_and_none_clears_optional_lists() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let root_arg = root.to_str().unwrap();
    super::helpers::write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    let run = |args: &[&str]| specsync().args(["--root", root_arg]).args(args).assert();
    run(&[
        "change",
        "new",
        "Add passkeys",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ])
    .success();
    let id = "add-passkeys";
    let state = || -> Value {
        serde_json::from_str(
            &fs::read_to_string(root.join(format!(".specsync/changes/{id}/state.json"))).unwrap(),
        )
        .unwrap()
    };
    for (question, answer) in [
        ("acceptance_criteria", "[]"),
        ("affected_paths", ","),
        ("affected_paths", "[]"),
        ("affected_specs", " , "),
        ("dependencies", ","),
    ] {
        let before = state();
        run(&["change", "answer", id, question, answer])
            .failure()
            .stderr(predicate::str::contains("names no"));
        assert_eq!(
            state(),
            before,
            "{question} {answer:?} must persist nothing"
        );
    }
    run(&["change", "answer", id, "dependencies", "none"]).success();
    assert_eq!(state()["dependencies"], serde_json::json!([]));

    run(&["change", "answer", id, "affected_specs", "none"]).success();
    assert_eq!(state()["affected_specs"], serde_json::json!([]));
    // `None` is a module name, so it is looked up, and the refusal says how to clear instead.
    run(&["change", "answer", id, "affected_specs", "None"])
        .failure()
        .stderr(predicate::str::contains(
            "to clear the list, answer exactly `none` or `'[]'`",
        ));
    // A yes/no question takes yes or no: anything else is refused naming the choices and stores
    // nothing (it used to fail as a corrupt correction ledger), and a recognised spelling is
    // stored as the choice it names (#1026 review).
    let before = state();
    run(&["change", "answer", id, "public_contract", "maybe"])
        .failure()
        .stderr(predicate::str::contains(
            "public_contract answer `maybe` is not one of its choices; answer yes or no",
        ))
        .stderr(predicate::str::contains("correction ledger").not());
    assert_eq!(state(), before, "a refused choice persists nothing");
    run(&["change", "answer", id, "architecture_risk", " N "]).success();
    assert_eq!(state()["answers"]["architecture_risk"], "no");
    // A case variant names the real module instead of a scaffold that would collide.
    run(&["change", "answer", id, "affected_specs", "AUTH"])
        .failure()
        .stderr(predicate::str::contains(
            "module names are case-sensitive, so name it as `auth`",
        ));
    // An empty constraints answer keeps its own guidance.
    run(&["change", "answer", id, "constraints", "  "])
        .failure()
        .stderr(predicate::str::contains("answer none if none apply"));
    // A module literally named `none` makes `none` ambiguous, so it is refused, not a silent clear.
    fs::create_dir_all(root.join("specs/none")).unwrap();
    fs::write(
        root.join("specs/none/none.spec.md"),
        "---\nmodule: none\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# None\n",
    )
    .unwrap();
    run(&["change", "answer", id, "affected_specs", "none"])
        .failure()
        .stderr(predicate::str::contains("is ambiguous"));
    // `[]` names nothing, so it clears even while something is named `none`.
    run(&["change", "answer", id, "affected_specs", "[]"]).success();
    assert_eq!(state()["affected_specs"], serde_json::json!([]));
    // A spec that declares the module off its canonical path is moved or registered, never
    // duplicated by a scaffold at the canonical path.
    fs::create_dir_all(root.join("specs/pay")).unwrap();
    fs::write(
        root.join("specs/pay/main.spec.md"),
        "---\nmodule: pay\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Pay\n",
    )
    .unwrap();
    let refused = run(&["change", "answer", id, "affected_specs", "pay"]).failure();
    let error = String::from_utf8_lossy(&refused.get_output().stderr).to_string();
    assert!(
        error.contains("has a spec at `specs/pay/main.spec.md`, but not at its canonical path")
            && !error.contains("specsync new"),
        "{error}"
    );
    // A registered module whose spec file is missing is named as registered, not as moved.
    fs::write(
        root.join("specsync-registry.toml"),
        "name = \"proj\"\n\n[specs]\nbilling = \"specs/billing/main.spec.md\"\n",
    )
    .unwrap();
    run(&["change", "answer", id, "affected_specs", "billing"])
        .failure()
        .stderr(predicate::str::contains(
            "affected spec `billing` is registered at `specs/billing/main.spec.md`, but no spec exists there",
        ));
}

/// #1043, #434 (REQ-change-079): every lifecycle rewrite keeps the members a newer SpecSync wrote
/// that this one does not know, at the top level and inside entries it did not change, through
/// send-back, answer, approve, check, review and ship, and the audit stays clean.
#[test]
fn lifecycle_rewrites_keep_members_a_newer_specsync_wrote() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let git = |args: &[&str]| -> String {
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
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "core.autocrlf", "false"]);
    git(&["config", "core.eol", "lf"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join(".github/scripts")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".github/scripts/lifecycle-validation-limits.json"),
        root.join(".github/scripts/lifecycle-validation-limits.json"),
    )
    .unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        r#"{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": false,
  "meaningful_paths": [],
  "ignored_paths": [".specsync/"],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
"#,
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    let run = |args: &[&str]| -> String {
        let mut full = vec!["--root", root.to_str().unwrap()];
        full.extend_from_slice(args);
        let output = specsync().args(&full).output().unwrap();
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let created: Value = serde_json::from_str(&run(&[
        "--json",
        "change",
        "new",
        "Archive a greeting",
        "--kind",
        "documentation",
        "--path",
        "docs/greeting.md",
        "--no-spec-change",
        "--rationale",
        "Documentation-only behavior does not alter a technical contract",
    ]))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "The greeting is archived"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(&["change", "answer", &id, question, answer]);
    }
    let dir = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("state.json")).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let artifact = artifact.as_str().unwrap();
        let content = if artifact == "tasks" {
            "# Tasks\n\n- [x] Archive the greeting.\n".to_string()
        } else {
            format!("# {artifact}\n\nComplete evidence for the greeting.\n")
        };
        fs::write(dir.join(format!("{artifact}.md")), content).unwrap();
    }
    // What a newer SpecSync would have written: a member appended to an object, with its exact
    // text (a decimal and an integer past u64 must not be rounded).
    let add_member = |path: &Path, object_end: &str, member: &str| {
        let text = fs::read_to_string(path).unwrap();
        let at = text.rfind(object_end).unwrap();
        let mut text = text;
        text.insert_str(at, member);
        fs::write(path, text).unwrap();
    };
    const STATE_MEMBER: &str = "\"future_state\": {\n    \"ratio\": 2.50,\n    \"big\": 123456789012345678901234567890\n  }";
    const LEDGER_MEMBER: &str = "\"future_ledger\": \"kept\"";
    const APPROVAL_MEMBER: &str = "\"future_approval\": \"kept\"";
    const ATTEMPT_MEMBER: &str = "\"future_attempt\": \"kept\"";
    add_member(
        &dir.join("state.json"),
        "\n}",
        &format!(",\n  {STATE_MEMBER}"),
    );
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    add_member(
        &dir.join("approvals.json"),
        "\n}",
        &format!(",\n  {LEDGER_MEMBER}"),
    );
    // The first approval's closing brace sits before the ledger's `send_backs` or end.
    let approvals = fs::read_to_string(dir.join("approvals.json")).unwrap();
    let first_close = approvals.find("\n    }").unwrap();
    let mut approvals = approvals;
    approvals.insert_str(first_close, &format!(",\n      {APPROVAL_MEMBER}"));
    fs::write(dir.join("approvals.json"), approvals).unwrap();
    fs::write(root.join("docs/greeting.md"), "# Greeting\n\nHello.\n").unwrap();
    git(&["add", "."]);
    git(&[
        "commit",
        "-m",
        "implement greeting (a newer SpecSync's records)",
    ]);
    run(&["change", "check", &id, "--commit"]);
    let attempts = fs::read_to_string(dir.join("verification-attempts.json")).unwrap();
    let first_close = attempts.find("\n    }").unwrap();
    let mut attempts = attempts;
    attempts.insert_str(first_close, &format!(",\n      {ATTEMPT_MEMBER}"));
    fs::write(dir.join("verification-attempts.json"), attempts).unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "a newer SpecSync's attempt record"]);

    let kept = |file: &str, member: &str, when: &str| {
        let text = fs::read_to_string(dir.join(file)).unwrap();
        assert!(
            text.contains(member),
            "{when}: {file} lost {member}:\n{text}"
        );
    };
    // Each rewrite keeps every member.
    run(&[
        "change",
        "send-back",
        &id,
        "--actor",
        "Scope owner",
        "--reason",
        "rescope",
    ]);
    kept("state.json", STATE_MEMBER, "send-back");
    kept("approvals.json", LEDGER_MEMBER, "send-back");
    kept("approvals.json", APPROVAL_MEMBER, "send-back");
    run(&[
        "change",
        "answer",
        &id,
        "acceptance_criteria",
        "The greeting is archived",
    ]);
    kept("state.json", STATE_MEMBER, "answer");
    run(&["change", "approve", &id, "--actor", "Scope owner"]);
    kept("approvals.json", LEDGER_MEMBER, "approve");
    kept("approvals.json", APPROVAL_MEMBER, "approve");
    run(&["change", "check", &id, "--commit"]);
    kept("verification-attempts.json", ATTEMPT_MEMBER, "check");
    kept("state.json", STATE_MEMBER, "check");
    run(&[
        "change",
        "review",
        &id,
        "--reviewer",
        "Independent reviewer",
    ]);
    // No change-record finding (the throwaway repository has no remote, which doctor notes apart).
    let doctor = run(&["doctor"]);
    assert!(!doctor.contains("audit-error"), "{doctor}");
    run(&["change", "ship", &id]);
    let archived = fs::read_dir(root.join(".specsync/archive/changes"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.to_string_lossy().ends_with(&id))
        .expect("archived package");
    for (file, member) in [
        ("state.json", STATE_MEMBER),
        ("approvals.json", LEDGER_MEMBER),
        ("approvals.json", APPROVAL_MEMBER),
        ("verification-attempts.json", ATTEMPT_MEMBER),
    ] {
        let text = fs::read_to_string(archived.join(file)).unwrap();
        assert!(
            text.contains(member),
            "ship: archived {file} lost {member}:\n{text}"
        );
    }
    run(&["change", "audit", "--strict"]);
}

// REQ-change-156 / REQ-change-157 / #535: after approve, the guided next
// action names `change check --commit`, and review and ship refuse a
// verification commit that does not contain the change's work.
fn git_ok(root: &Path, args: &[&str]) -> String {
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

fn ss_ok(root: &Path, args: &[&str]) -> String {
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

fn ss_status(root: &Path, args: &[&str]) -> std::process::Output {
    specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn ss535_no_spec_project() -> (TempDir, PathBuf, String) {
    ss535_no_spec_project_with_scope("README.md", |root| {
        fs::write(root.join("README.md"), "hello\n\nThe greeter says hello.\n").unwrap();
    })
}

/// A no-spec documentation change scoped to `scope`, approved after `work` ran
/// and left its result uncommitted.
fn ss535_no_spec_project_with_scope(
    scope: &str,
    work: impl FnOnce(&Path),
) -> (TempDir, PathBuf, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    git_ok(&root, &["init", "-q", "-b", "main"]);
    git_ok(&root, &["config", "user.email", "test@example.com"]);
    git_ok(&root, &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "hello\n").unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    git_ok(&root, &["add", "README.md", "docs/guide.md"]);
    git_ok(&root, &["commit", "-q", "-m", "seed"]);
    ss_ok(&root, &["init"]);
    git_ok(&root, &["add", "--all"]);
    git_ok(&root, &["commit", "-q", "-m", "init specsync"]);
    let id = "document-the-greeter";
    ss_ok(
        &root,
        &[
            "change",
            "new",
            "Document the greeter",
            "--kind",
            "documentation",
            "--path",
            scope,
            "--no-spec-change",
            "--rationale",
            "Documentation only; no canonical spec changes",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "The README describes the greeter"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        ss_ok(&root, &["change", "answer", id, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(id);
    let record: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in record["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        fs::write(
            workspace.join(format!("{name}.md")),
            "# Complete\n\nThe README documents the greeter.\n",
        )
        .unwrap();
    }
    work(&root);
    ss_ok(&root, &["change", "approve", id, "--actor", "Repro535"]);
    (temp, root, id.to_string())
}

fn next_action_json(root: &Path, id: &str) -> String {
    let output = ss_status(root, &["--json", "change", "status", id]);
    assert!(
        output.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    value["summary"]["next_action"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn after_approve_next_action_names_check_commit() {
    let (_temp, root, id) = ss535_no_spec_project();
    let expected = format!("run `specsync change check {id} --commit`");
    assert_eq!(next_action_json(&root, &id), expected);
    let show = ss_ok(&root, &["change", "show", &id]);
    assert!(
        show.contains(&format!("specsync change check {id} --commit")),
        "show must name check --commit:\n{show}"
    );
    assert!(
        !show.contains(&format!("specsync change check {id}`"))
            || show.contains(&format!("specsync change check {id} --commit")),
        "show must not name a bare check as the next command:\n{show}"
    );
    let status = ss_ok(&root, &["change", "status", &id]);
    assert!(
        status.contains(&format!("specsync change check {id} --commit")),
        "status must name check --commit:\n{status}"
    );
}

#[test]
fn review_and_ship_refuse_a_verification_commit_that_lacks_the_work() {
    let (_temp, root, id) = ss535_no_spec_project();
    let seed = git_ok(&root, &["rev-parse", "HEAD"]);
    ss_ok(&root, &["change", "check", &id]);
    let workspace = root.join(".specsync/changes").join(&id);
    let verification: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("verification.json")).unwrap())
            .unwrap();
    assert_eq!(
        verification["commit"].as_str().unwrap(),
        seed,
        "bare check binds verification to HEAD, which lacks the work"
    );

    let review = ss_status(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
    assert_ne!(review.status.code(), Some(0), "review must refuse");
    let review_err = format!(
        "{}{}",
        String::from_utf8_lossy(&review.stdout),
        String::from_utf8_lossy(&review.stderr)
    );
    assert!(
        review_err.contains(&seed)
            && review_err.contains(&format!("specsync change check {id} --commit")),
        "review refusal must name the verification commit and check --commit:\n{review_err}"
    );
    assert!(
        !workspace.join("review.json").exists(),
        "a refused review records nothing"
    );

    let ship = ss_status(&root, &["change", "ship", &id]);
    assert_ne!(ship.status.code(), Some(0), "ship must refuse");
    let ship_err = format!(
        "{}{}",
        String::from_utf8_lossy(&ship.stdout),
        String::from_utf8_lossy(&ship.stderr)
    );
    assert!(
        ship_err.contains(&seed)
            && ship_err.contains(&format!("specsync change check {id} --commit")),
        "ship refusal must name the verification commit and check --commit:\n{ship_err}"
    );
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    assert_ne!(
        state["state"].as_str().unwrap_or(""),
        "archived",
        "a refused ship must leave the change unarchived: {state}"
    );

    // After check --commit records verification against a commit that contains
    // the work, review and ship may proceed. The only uncommitted files then
    // are under `.specsync/` (invariant 42) and must not retrigger the refusal.
    ss_ok(&root, &["change", "check", &id, "--commit"]);
    let committed = git_ok(&root, &["rev-parse", "HEAD"]);
    assert_ne!(committed, seed, "check --commit must record a new commit");
    assert_eq!(
        git_ok(&root, &["show", "HEAD:README.md"]),
        "hello\n\nThe greeter says hello."
    );
    ss_ok(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
    assert!(workspace.join("review.json").exists());
    let review: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("review.json")).unwrap()).unwrap();
    assert_ne!(
        review["implementation_commit"].as_str().unwrap(),
        seed,
        "the passing review must not bind to the pre-change commit"
    );
}

fn ss_output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn review_and_ship_refuse_an_untracked_file_under_a_directory_scope() {
    // t384 on #1059: `git diff` does not see an untracked file, so a new file
    // inside a directory scope passed review and ship while uncommitted.
    let (_temp, root, id) = ss535_no_spec_project_with_scope("docs/", |root| {
        fs::write(root.join("docs/new-note.md"), "A new note.\n").unwrap();
    });
    let seed = git_ok(&root, &["rev-parse", "HEAD"]);
    ss_ok(&root, &["change", "check", &id]);
    let workspace = root.join(".specsync/changes").join(&id);

    let review = ss_status(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
    let review_text = ss_output_text(&review);
    assert_ne!(
        review.status.code(),
        Some(0),
        "review must refuse:\n{review_text}"
    );
    assert!(
        review_text.contains(&seed)
            && review_text.contains(&format!("specsync change check {id} --commit")),
        "review refusal must name the verification commit and check --commit:\n{review_text}"
    );
    assert!(
        !workspace.join("review.json").exists(),
        "a refused review records nothing"
    );

    let ship = ss_status(&root, &["change", "ship", &id]);
    let ship_text = ss_output_text(&ship);
    assert_ne!(
        ship.status.code(),
        Some(0),
        "ship must refuse:\n{ship_text}"
    );
    assert!(
        ship_text.contains(&format!("specsync change check {id} --commit")),
        "ship refusal must name check --commit:\n{ship_text}"
    );
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    assert_ne!(state["state"].as_str().unwrap_or(""), "archived", "{state}");
    assert!(
        git_ok(&root, &["status", "--porcelain", "--", "docs/"]).contains("docs/new-note.md"),
        "the new note is still uncommitted"
    );

    assert!(
        review_text.contains("docs/new-note.md") && review_text.contains("git add"),
        "the refusal names the untracked file and the `git add` it needs:\n{review_text}"
    );

    // `check --commit` never commits an untracked file, so the author adds it;
    // once a verification commit holds it, review proceeds.
    git_ok(&root, &["add", "docs/new-note.md"]);
    ss_ok(&root, &["change", "check", &id, "--commit"]);
    assert_eq!(
        git_ok(&root, &["show", "HEAD:docs/new-note.md"]),
        "A new note."
    );
    ss_ok(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
}

#[test]
fn review_refuses_a_verification_commit_that_is_not_a_full_commit_id() {
    // t384 on #1059: the recorded verification commit is used as a Git revision
    // only when it is a full commit id, after `--end-of-options` (REQ-change-013).
    let (_temp, root, id) = ss535_no_spec_project();
    ss_ok(&root, &["change", "check", &id, "--commit"]);
    let workspace = root.join(".specsync/changes").join(&id);
    let path = workspace.join("verification.json");
    let original = fs::read_to_string(&path).unwrap();
    let head = git_ok(&root, &["rev-parse", "HEAD"]);
    let short = head[..12].to_string();
    for value in ["--not-a-commit", "-x", short.as_str(), "HEAD", "main"] {
        let mut verification: Value = serde_json::from_str(&original).unwrap();
        verification["commit"] = Value::String(value.to_string());
        fs::write(&path, serde_json::to_string_pretty(&verification).unwrap()).unwrap();
        let review = ss_status(
            &root,
            &["change", "review", &id, "--reviewer", "Independent535"],
        );
        let text = ss_output_text(&review);
        assert_ne!(
            review.status.code(),
            Some(0),
            "{value}: review must refuse:\n{text}"
        );
        assert!(
            text.contains("is not a full commit id"),
            "{value}: the refusal names the bad commit identity:\n{text}"
        );
        assert!(
            !text.contains("failed to compare"),
            "{value}: the value never reaches git diff:\n{text}"
        );
        assert!(
            !workspace.join("review.json").exists(),
            "{value}: nothing recorded"
        );
    }
    fs::write(&path, original).unwrap();
    ss_ok(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
}

#[test]
fn uncommitted_specsync_evidence_alone_does_not_refuse_review() {
    let (_temp, root, id) = ss535_no_spec_project();
    ss_ok(&root, &["change", "check", &id, "--commit"]);
    // Review, then leave review.json uncommitted, then confirm a second
    // status/show still works and a finalize/ship is not refused for
    // uncommitted `.specsync/` evidence.
    ss_ok(
        &root,
        &["change", "review", &id, "--reviewer", "Independent535"],
    );
    let status = git_ok(&root, &["status", "--porcelain"]);
    assert!(
        status.lines().any(|line| line.contains(".specsync/")),
        "review evidence should still be uncommitted:\n{status}"
    );
    assert!(
        !status.lines().any(|line| line.contains("README.md")),
        "the affected path must already be committed:\n{status}"
    );
    let ship = ss_status(&root, &["change", "ship", &id]);
    assert_eq!(
        ship.status.code(),
        Some(0),
        "uncommitted .specsync/ evidence alone must not refuse ship\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ship.stdout),
        String::from_utf8_lossy(&ship.stderr)
    );
}
