use super::helpers::specsync;
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
    if let Some((drive, rest)) = forward.split_once(':') {
        if drive.len() == 1
            && drive
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
        {
            return format!("/{}{}", drive.to_ascii_lowercase(), rest);
        }
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
            "run `specsync change review harden-verification-freshness --reviewer <human>` after the PR's scoped review passes",
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
            "run `specsync change check harden-verification-freshness`",
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
            "run `specsync change review harden-verification-freshness --reviewer <human>` after the PR's scoped review passes",
            None,
            environment,
        );
    }
}

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
        assert_eq!(finalized["next_action"], "merge the PR on GitHub");
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

        git(&["add", ".specsync"]);
        let archive_diff = Command::new("git")
            .args([
                "diff",
                "--cached",
                "--name-status",
                "-z",
                "--find-renames",
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

        git(&["add", "."]);
        git(&["commit", "-m", &format!("finalize Rune change {index}")]);
        assert_eq!(git(&["rev-parse", "HEAD^"]), implementation_commit);
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
            "exact-only delivery input `README.md` changed after acceptance and requires an audited reopen; run `specsync change reopen update-review-instructions` to re-verify the accepted change",
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
            "verification_commands": verification_commands,
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
            "Reap an interrupted verification child",
            "--kind",
            "bug-fix",
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
