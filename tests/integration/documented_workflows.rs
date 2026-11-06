//! The README's verified change workflow is what a new user copies first (DOCS-1). These tests
//! read the shell block straight out of the README and run every command in it as written, in
//! order, through `bash` with the built `specsync` first on `PATH`, in a throwaway git project.
//!
//! The block also carries prose comments for the steps a reader does by hand, such as "answer
//! the remaining interview questions". The harness does each of those steps the way a reader
//! would and nothing more. A prose step it does not know needs no action of its own; if the
//! reader really had to do something there, the next documented command fails.
//!
//! After every step the harness runs the real Archive gate (`.github/scripts/archive-readiness.sh`,
//! the script the hosted `Archive` job runs). Wherever the block tells the reader to wait for CI,
//! that gate has to pass, because CI refuses a pull request whose change is still open (DOCS-2).
//! Before that point the gate must refuse the open change. Each refusal names a command, and the
//! harness runs that command in a copy of the project to show the command really works.
//!
//! The actor and reviewer labels in the block (`"Ada"`) are simulated decisions in a fixture.
//! They are not a human approval or review of anything.

use crate::helpers::{SUBSTANTIVE_PLAN, specsync};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// One entry of a documented shell block.
#[derive(Debug)]
enum Step {
    /// A command line to run exactly as written (continuations joined).
    Command(String),
    /// A `# ...` comment: something the reader is told, or does by hand.
    Prose(String),
}

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// The first ```` ```bash ```` block after the line that equals `heading`.
fn documented_block(relative: &str, heading: &str) -> String {
    let text = fs::read_to_string(manifest_path(relative)).unwrap();
    let mut lines = text.lines().skip_while(|line| line.trim_end() != heading);
    assert!(
        lines.next().is_some(),
        "{relative} has no `{heading}` heading; this test runs the shell block under it"
    );
    let mut lines = lines.skip_while(|line| line.trim() != "```bash");
    assert!(
        lines.next().is_some(),
        "{relative} has no ```bash block under `{heading}`"
    );
    let block: Vec<&str> = lines.take_while(|line| line.trim() != "```").collect();
    block.join("\n")
}

/// Split a block into commands and prose, joining `\` continuation lines into one command.
fn documented_steps(block: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    let mut pending: Option<String> = None;
    for raw in block.lines() {
        let line = raw.trim();
        if let Some(mut command) = pending.take() {
            command.push(' ');
            command.push_str(line.trim_end_matches('\\').trim());
            if line.ends_with('\\') {
                pending = Some(command);
            } else {
                steps.push(Step::Command(command));
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if let Some(prose) = line.strip_prefix('#') {
            steps.push(Step::Prose(prose.trim().to_string()));
        } else if line.ends_with('\\') {
            pending = Some(line.trim_end_matches('\\').trim().to_string());
        } else {
            steps.push(Step::Command(line.to_string()));
        }
    }
    assert!(
        pending.is_none(),
        "documented block ends inside a continuation"
    );
    steps
}

/// True when a prose step tells the reader to wait for, or watch, CI or the checks.
fn tells_reader_to_wait_for_ci(prose: &str) -> bool {
    let words: Vec<String> = prose
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let names_ci = words.iter().any(|word| word == "ci" || word == "checks");
    let waits = words
        .iter()
        .any(|word| word.starts_with("wait") || word.starts_with("observ"));
    names_ci && waits
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            copy_dir(&entry.path(), &target);
        } else if kind.is_file() {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn describe(output: &Output) -> String {
    format!(
        "exit {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The Archive gate's verdict on one tree.
struct Gate {
    passed: bool,
    message: String,
}

impl Gate {
    /// The command a refusal names (`Run \`...\`.`).
    fn named_command(&self) -> Option<String> {
        let (_, rest) = self.message.split_once("Run `")?;
        let (command, _) = rest.split_once('`')?;
        Some(command.to_string())
    }
}

/// A throwaway git project in which documented commands run as written.
struct Project {
    _temp: TempDir,
    root: PathBuf,
    path_env: std::ffi::OsString,
}

impl Project {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("project");
        let hooks = temp.path().join("no-hooks");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&hooks).unwrap();
        let binary = assert_cmd::cargo::cargo_bin("specsync");
        let mut paths = vec![binary.parent().unwrap().to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let project = Self {
            _temp: temp,
            root,
            path_env: std::env::join_paths(paths).unwrap(),
        };
        project.git(&["init", "-q", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Documented Workflow Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
            ("core.autocrlf", "false"),
            ("core.eol", "lf"),
            ("core.hooksPath", hooks.to_str().unwrap()),
        ] {
            project.git(&["config", key, value]);
        }
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
            "git {args:?} failed: {}",
            describe(&output)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    fn commit_all(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// Run one documented line exactly as written, as a reader's shell would.
    fn run_documented(&self, root: &Path, line: &str) -> Output {
        let mut command = Command::new("bash");
        command
            .args(["-e", "-o", "pipefail", "-c", line])
            .current_dir(root)
            .env("PATH", &self.path_env)
            .env_remove("GITHUB_EVENT_NAME")
            .env_remove("GITHUB_BASE_REF");
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("SPECSYNC_") {
                command.env_remove(key);
            }
        }
        command.output().unwrap()
    }

    /// A harness step through the built binary (not a documented command).
    fn specsync_json(&self, args: &[&str]) -> Value {
        let output = specsync()
            .arg("--root")
            .arg(&self.root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "specsync {args:?} failed: {}",
            describe(&output)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn archive_gate(&self, root: &Path) -> Gate {
        let output = Command::new("bash")
            .arg(manifest_path(".github/scripts/archive-readiness.sh"))
            .arg(root)
            .output()
            .unwrap();
        Gate {
            passed: output.status.success(),
            message: format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
            .trim()
            .to_string(),
        }
    }

    /// Run a refusal's named command in a copy of the project and check that it works there.
    fn prove_refusal_recovery(&self, gate: &Gate, step: &str) {
        let command = gate.named_command().unwrap_or_else(|| {
            panic!(
                "the Archive gate refused after `{step}` without naming a command:\n{}",
                gate.message
            )
        });
        let copy = TempDir::new().unwrap();
        let root = copy.path().join("project");
        copy_dir(&self.root, &root);
        let output = self.run_documented(&root, &command);
        assert!(
            output.status.success(),
            "the Archive gate refused after `{step}` and named `{command}`, which fails:\n{}",
            describe(&output)
        );
        if command.starts_with("specsync change status ") {
            assert!(
                String::from_utf8_lossy(&output.stdout).contains("Next:"),
                "`{command}` must print the next action:\n{}",
                describe(&output)
            );
        } else {
            let after = self.archive_gate(&root);
            assert!(
                after.passed,
                "running `{command}`, which the Archive gate named, did not satisfy the gate:\n{}",
                after.message
            );
        }
    }

    /// The README's example module: `auth`, one Rust export, one requirement, SDD adopted.
    fn readme_auth_module() -> Self {
        let project = Self::new();
        assert!(
            project
                .run_documented(&project.root, "specsync init")
                .status
                .success()
        );
        project.write(
            "src/auth.rs",
            "pub fn authenticate(token: &str) -> bool {\n    !token.is_empty()\n}\n",
        );
        project.write(
            "specs/auth/auth.spec.md",
            "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auth.rs\n---\n\n\
             # Auth\n\n## Purpose\n\nAuthenticate a user.\n\n## Public API\n\n\
             | Name | Kind | Description |\n|---|---|---|\n\
             | `authenticate` | function | Accept a credential the user presents. |\n\n\
             ## Invariants\n\nAn empty credential never authenticates.\n\n\
             ## Behavioral Examples\n\n`authenticate(\"token\")` is true.\n\n\
             ## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n\
             | Version | Date | Change |\n|---|---|---|\n| 1 | 2026-11-05 | Baseline |\n",
        );
        project.write(
            "specs/auth/requirements.md",
            "### REQ-auth-001\n\nThe module SHALL accept a non-empty credential.\n\n\
             Acceptance Criteria\n- A non-empty credential authenticates.\n",
        );
        let adopted = project.run_documented(&project.root, "specsync change adopt");
        assert!(adopted.status.success(), "{}", describe(&adopted));
        project.commit_all("Baseline auth module with the change workflow adopted");
        project
    }

    /// What a reader does for "answer the remaining interview questions and complete the
    /// selected artifacts/deltas": answer what is still asked (the recommended choice, which is
    /// what the interview offers), fill every selected artifact and write the module delta.
    fn complete_definition(&self, id: &str, mut questions: Vec<Value>, requirement: &str) {
        while let Some(question) = questions.first() {
            let question_id = question["id"].as_str().unwrap().to_string();
            let answer = match question["recommended"].as_str() {
                Some(recommended) => recommended.to_string(),
                None => panic!(
                    "the interview asks `{question_id}` with no recommended answer; teach \
                     documented_workflows.rs what a reader answers there"
                ),
            };
            let answered =
                self.specsync_json(&["change", "answer", id, &question_id, &answer, "--json"]);
            questions = answered["questions"]
                .as_array()
                .cloned()
                .unwrap_or_default();
        }
        let status = self.specsync_json(&["change", "status", id, "--json"]);
        let change = &status["change"];
        for artifact in change["selected_artifacts"].as_array().unwrap() {
            let artifact = artifact.as_str().unwrap();
            let body = match artifact {
                "plan" => SUBSTANTIVE_PLAN.to_string(),
                "tasks" => "# Tasks\n\n- [x] Carry out the documented change.\n".to_string(),
                "testing" => format!(
                    "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n\
                     |---|---|\n| {requirement} | SpecSync structural check of the module |\n"
                ),
                other => format!("# {other}\n\nThe documented example change, as approved.\n"),
            };
            self.write(
                &format!(".specsync/changes/{id}/{artifact}.md"),
                &format!("---\nchange: {id}\nartifact: {artifact}\n---\n\n{body}"),
            );
        }
        for module in change["affected_specs"].as_array().unwrap() {
            let module = module.as_str().unwrap();
            self.write(
                &format!(".specsync/changes/{id}/deltas/{module}.md"),
                &format!(
                    "## MODIFIED\n\n### REQUIREMENT {requirement}\n\n\
                     The module SHALL do what the documented example change describes.\n\n\
                     Acceptance Criteria\n- The documented example change is observable.\n"
                ),
            );
        }
    }
}

#[test]
fn readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci() {
    let steps = documented_steps(&documented_block(
        "README.md",
        "## Verified change workflow",
    ));
    let project = Project::readme_auth_module();
    let mut change_id: Option<String> = None;
    let mut questions: Vec<Value> = Vec::new();
    let mut refusals_while_open = 0;
    let mut proven_recoveries = std::collections::BTreeSet::new();
    let mut waits_for_ci = 0;

    for step in &steps {
        let label = match step {
            Step::Command(command) => {
                let output = project.run_documented(&project.root, command);
                assert!(
                    output.status.success(),
                    "README command `{command}` failed when run as written:\n{}",
                    describe(&output)
                );
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(id) = json["change"]["id"].as_str() {
                        change_id = Some(id.to_string());
                    }
                    if let Some(next) = json["questions"].as_array() {
                        questions = next.clone();
                    }
                }
                command.clone()
            }
            Step::Prose(prose) => {
                let lower = prose.to_ascii_lowercase();
                if tells_reader_to_wait_for_ci(prose) {
                    waits_for_ci += 1;
                    let gate = project.archive_gate(&project.root);
                    assert!(
                        gate.passed,
                        "the README tells the reader to wait for CI at `# {prose}`, but CI's \
                         Archive gate refuses this tip:\n{}\nThe closeout is archive-first: \
                         verify, review and finalize locally, then push and wait for CI.",
                        gate.message
                    );
                } else if lower.contains("answer") && lower.contains("artifact") {
                    let id = change_id.as_deref().expect("`change new` runs first");
                    project.complete_definition(id, std::mem::take(&mut questions), "REQ-auth-001");
                } else if lower.starts_with("implement") {
                    // The approved delta reworded REQ-auth-001; the export stays the same. The
                    // fixture has no product test suite to run.
                    project.write(
                        "src/auth.rs",
                        "pub fn authenticate(token: &str) -> bool {\n    \
                         !token.trim().is_empty()\n}\n",
                    );
                    project.commit_all("Implement the approved auth change");
                }
                format!("# {prose}")
            }
        };
        if let Some(id) = change_id.as_deref() {
            // An open change keeps its workspace under `.specsync/changes/`; finalize moves it
            // into the dated archive.
            if project.root.join(".specsync/changes").join(id).is_dir() {
                let gate = project.archive_gate(&project.root);
                assert!(
                    !gate.passed,
                    "the Archive gate passed after `{label}` while `{id}` is still open"
                );
                refusals_while_open += 1;
                // Each distinct named command is run once; the gate names the same
                // `change status` command at most steps.
                if proven_recoveries.insert(gate.named_command()) {
                    project.prove_refusal_recovery(&gate, &label);
                }
            }
        }
    }

    let id = change_id.expect("the README block creates a change");
    let status = project.specsync_json(&["change", "status", &id, "--json"]);
    assert_eq!(
        status["change"]["state"], "archived",
        "following the README block must end with `{id}` archived"
    );
    assert!(
        refusals_while_open > 0,
        "the Archive gate never refused the open change"
    );
    assert!(
        proven_recoveries.contains(&Some(format!("specsync change ship {id}"))),
        "after the scoped review the Archive gate names `specsync change ship {id}`, and the \
         harness must have run it; it ran {proven_recoveries:?}"
    );
    assert!(
        waits_for_ci > 0,
        "the README block must tell the reader when to wait for CI"
    );
    assert!(project.archive_gate(&project.root).passed);
}
