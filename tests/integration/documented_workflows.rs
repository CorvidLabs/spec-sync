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

    /// The Archive gate on what a push sends: the tree must be clean, and the gate runs on a
    /// fresh clone of `HEAD`, as CI's checkout does. A finalize that moved the workspace
    /// without committing the move would pass on the live tree and fail here.
    fn archive_gate_on_pushed_head(&self, at: &str) -> Gate {
        let dirty = self.git(&["status", "--porcelain", "--untracked-files=all"]);
        assert!(
            dirty.is_empty(),
            "at `{at}` the reader pushes HEAD, but the tree has uncommitted changes:\n{dirty}"
        );
        let clone = TempDir::new().unwrap();
        let checkout = clone.path().join("pushed");
        let output = Command::new("git")
            .args(["clone", "--quiet", "--no-hardlinks"])
            .arg(&self.root)
            .arg(&checkout)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git clone failed: {}",
            describe(&output)
        );
        self.archive_gate(&checkout)
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

    /// The landing page's Quick Start prerequisites for its change workflow: `specsync init`
    /// and `specsync add-spec auth` from the same block, run as written, with `src/auth.ts`
    /// written first as the block's comment says ("after you write a source file"). The block's
    /// other lines (install, coverage, score, hooks, MCP) are a command list, not steps of the
    /// change workflow, and are not run here.
    fn landing_page_quick_start(steps: &[Step]) -> Self {
        let project = Self::new();
        let command = |prefix: &str| {
            steps
                .iter()
                .find_map(|step| match step {
                    Step::Command(command) if command.starts_with(prefix) => Some(command.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("the landing-page Quick Start has no `{prefix}` line"))
        };
        let init = command("specsync init");
        let add_spec = command("specsync add-spec auth");
        assert!(
            add_spec.contains("after you write a source file"),
            "the harness writes src/auth.ts because `{add_spec}` says to write a source file first"
        );
        let output = project.run_documented(&project.root, &init);
        assert!(output.status.success(), "`{init}`: {}", describe(&output));
        project.write(
            "src/auth.ts",
            "export function login(): boolean {\n  return true;\n}\n",
        );
        let output = project.run_documented(&project.root, &add_spec);
        assert!(
            output.status.success(),
            "`{add_spec}`: {}",
            describe(&output)
        );
        project.commit_all("Existing project with the auth module and its scaffolded spec");
        project
    }

    /// What a reader does for a prose step about completing the selected artifacts: answer
    /// what the interview still asks (the recommended choice it offers), fill every selected
    /// artifact and write the module delta.
    fn complete_definition(&self, id: &str, mut questions: Vec<Value>, delta: &Delta) {
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
        let requirement = delta.requirement;
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
                    "## {}\n\n### REQUIREMENT {requirement}\n\n\
                     The module SHALL do what the documented example change describes.\n\n\
                     Acceptance Criteria\n- The documented example change is observable.\n",
                    delta.operation
                ),
            );
        }
    }

    /// Follow a documented block from its first step, as a reader would.
    ///
    /// Commands run as written and must succeed. A prose step naming the selected artifacts
    /// completes the definition; one starting "implement" makes a non-API edit and commits it;
    /// one telling the reader to wait for CI requires the Archive gate to pass. After every
    /// step while the change is open the gate must refuse, and each distinct command it names
    /// is proven in a copy of the project.
    fn follow(&self, source: &str, steps: &[Step], delta: &Delta) -> Followed {
        let mut followed = Followed::default();
        let mut questions: Vec<Value> = Vec::new();
        for step in steps {
            let label = match step {
                Step::Command(command) => {
                    let output = self.run_documented(&self.root, command);
                    assert!(
                        output.status.success(),
                        "{source} command `{command}` failed when run as written:\n{}",
                        describe(&output)
                    );
                    if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                        if let Some(id) = json["change"]["id"].as_str() {
                            followed.change_id = Some(id.to_string());
                        }
                        if let Some(next) = json["questions"].as_array() {
                            questions = next.clone();
                        }
                        if let Some(criteria) = json["change"]["acceptance_criteria"].as_array() {
                            followed.acceptance_criteria = criteria.clone();
                        }
                    }
                    command.clone()
                }
                Step::Prose(prose) => {
                    let lower = prose.to_ascii_lowercase();
                    if tells_reader_to_wait_for_ci(prose) {
                        followed.waits_for_ci += 1;
                        let gate = self.archive_gate_on_pushed_head(&format!("# {prose}"));
                        assert!(
                            gate.passed,
                            "{source} tells the reader to wait for CI at `# {prose}`, but CI's \
                             Archive gate refuses this tip:\n{}\nThe closeout is archive-first: \
                             verify, review and finalize locally, then push and wait for CI.",
                            gate.message
                        );
                    } else if lower.contains("artifact") {
                        let id = followed
                            .change_id
                            .as_deref()
                            .expect("`change new` runs first");
                        self.complete_definition(id, std::mem::take(&mut questions), delta);
                    } else if lower.starts_with("implement") {
                        // The approved delta changes a requirement; the export stays the same.
                        // The fixture has no product test suite to run.
                        self.write(
                            "src/auth.rs",
                            "pub fn authenticate(token: &str) -> bool {\n    \
                             !token.trim().is_empty()\n}\n",
                        );
                        self.commit_all("Implement the approved auth change");
                    }
                    format!("# {prose}")
                }
            };
            if let Some(id) = followed.change_id.as_deref() {
                // An open change keeps its workspace under `.specsync/changes/`; finalize moves
                // it into the dated archive.
                if self.root.join(".specsync/changes").join(id).is_dir() {
                    let gate = self.archive_gate(&self.root);
                    assert!(
                        !gate.passed,
                        "the Archive gate passed after `{label}` while `{id}` is still open"
                    );
                    followed.refusals_while_open += 1;
                    // Each distinct named command is run once; the gate names the same
                    // `change status` command at most steps.
                    if followed.proven_recoveries.insert(gate.named_command()) {
                        self.prove_refusal_recovery(&gate, &label);
                    }
                }
            }
        }
        followed
    }

    /// DOCS-3.a: approving a definition that `change new` just created must refuse, and each
    /// refusal about the interview must name the `change answer` command that finishes it.
    /// The probe runs the named command (filling its placeholder: a sentence for the outcome,
    /// the recommended `no` for a yes-or-no question, `none` for free text) and approves again, until the refusal is
    /// about the selected artifacts, which a reader writes by hand. Returns the named commands.
    fn approval_refusals_name_answer_commands(&self, root: &Path, approve: &str) -> Vec<String> {
        let mut named = Vec::new();
        loop {
            let output = self.run_documented(root, approve);
            assert!(
                !output.status.success(),
                "`{approve}` approved an unfinished definition:\n{}",
                describe(&output)
            );
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            let Some((_, rest)) = error.split_once("run `specsync change answer ") else {
                assert!(
                    error.contains("artifact is incomplete"),
                    "`{approve}` refused without naming the `specsync change answer` command \
                     that finishes the definition:\n{error}"
                );
                break;
            };
            let (arguments, _) = rest.split_once('`').unwrap();
            let command = format!("specsync change answer {arguments}");
            let runnable = command
                .replace("\"<observable outcome>\"", "\"login returns true\"")
                .replace("<yes|no>", "no")
                .replace("\"<answer>\"", "\"none\"");
            assert!(
                !runnable.contains('<'),
                "the refusal names `{command}`, whose placeholder the harness cannot fill"
            );
            let answered = self.run_documented(root, &runnable);
            assert!(
                answered.status.success(),
                "the refusal named `{command}`, which fails:\n{}",
                describe(&answered)
            );
            named.push(command);
            assert!(
                named.len() <= 8,
                "approval kept naming answer commands without progress: {named:?}"
            );
        }
        named
    }
}

/// The delta a reader writes for the example module's requirement.
struct Delta {
    operation: &'static str,
    requirement: &'static str,
}

/// What following a documented block observed.
#[derive(Default)]
struct Followed {
    change_id: Option<String>,
    acceptance_criteria: Vec<Value>,
    refusals_while_open: usize,
    proven_recoveries: std::collections::BTreeSet<Option<String>>,
    waits_for_ci: usize,
}

impl Followed {
    /// The closeout ends archived, after the gate refused the open change and its named
    /// `change ship` recovery was proven.
    fn assert_archived_after_refusals(&self, project: &Project, source: &str) -> String {
        let id = self
            .change_id
            .clone()
            .unwrap_or_else(|| panic!("the {source} block creates a change"));
        let status = project.specsync_json(&["change", "status", &id, "--json"]);
        assert_eq!(
            status["change"]["state"], "archived",
            "following the {source} block must end with `{id}` archived"
        );
        assert!(
            self.refusals_while_open > 0,
            "the Archive gate never refused the open change"
        );
        assert!(
            self.proven_recoveries
                .contains(&Some(format!("specsync change ship {id}"))),
            "after the scoped review the Archive gate names `specsync change ship {id}`, and the \
             harness must have run it; it ran {:?}",
            self.proven_recoveries
        );
        let pushed = project.archive_gate_on_pushed_head(&format!("the end of the {source} block"));
        assert!(
            pushed.passed,
            "a fresh clone of HEAD still has an open change:\n{}",
            pushed.message
        );
        id
    }
}

/// The landing page's verified change workflow: the Quick Start steps after its "Optional:
/// verified change workflow" line.
fn landing_page_steps() -> (Vec<Step>, Vec<Step>) {
    let mut steps = documented_steps(&documented_block(
        "site/src/content/docs/index.md",
        "## Quick Start",
    ));
    let start = steps
        .iter()
        .position(|step| {
            matches!(step, Step::Prose(prose) if prose.starts_with("Optional: verified change workflow"))
        })
        .expect("the landing-page Quick Start has an `# Optional: verified change workflow` line");
    let workflow = steps.split_off(start);
    (steps, workflow)
}

#[test]
fn readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci() {
    let steps = documented_steps(&documented_block(
        "README.md",
        "## Verified change workflow",
    ));
    let project = Project::readme_auth_module();
    let followed = project.follow(
        "README",
        &steps,
        &Delta {
            operation: "MODIFIED",
            requirement: "REQ-auth-001",
        },
    );
    followed.assert_archived_after_refusals(&project, "README");
    assert!(
        followed.waits_for_ci > 0,
        "the README block must tell the reader when to wait for CI"
    );
}

/// DOCS-3 and DOCS-1: the landing page's change workflow runs as written from its own Quick
/// Start, its approval binds the definition it documents, and it ends archived.
#[test]
fn landing_page_change_workflow_runs_from_its_quick_start() {
    let (quick_start, workflow) = landing_page_steps();
    let project = Project::landing_page_quick_start(&quick_start);
    let followed = project.follow(
        "landing page",
        &workflow,
        &Delta {
            operation: "ADDED",
            requirement: "REQ-auth-001",
        },
    );
    let id = followed.assert_archived_after_refusals(&project, "landing page");
    assert!(
        !followed.acceptance_criteria.is_empty(),
        "the landing page must show the acceptance-criteria answer before approval"
    );
    let archive = fs::read_dir(project.root.join(".specsync/archive/changes"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(&format!("-{id}")))
        .expect("the archived package");
    let approvals: Value =
        serde_json::from_str(&fs::read_to_string(archive.join("approvals.json")).unwrap()).unwrap();
    let scope = &approvals["approvals"][0]["approved_scope"];
    assert_eq!(
        scope["acceptance_criteria"],
        Value::Array(followed.acceptance_criteria.clone())
    );
    assert_eq!(scope["affected_specs"], serde_json::json!(["auth"]));
    assert_eq!(scope["affected_paths"], serde_json::json!(["src/auth.ts"]));
}

/// DOCS-3.a: right after the landing page's `change new`, its own approve line refuses, and
/// every refusal about the interview names the `change answer` command that finishes it.
#[test]
fn approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them() {
    let (quick_start, workflow) = landing_page_steps();
    let project = Project::landing_page_quick_start(&quick_start);
    let documented = |prefix: &str| {
        workflow
            .iter()
            .find_map(|step| match step {
                Step::Command(command) if command.starts_with(prefix) => Some(command.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("the landing-page workflow has no `{prefix}` line"))
    };
    for prefix in ["specsync change adopt", "specsync change new "] {
        let command = documented(prefix);
        let output = project.run_documented(&project.root, &command);
        assert!(
            output.status.success(),
            "`{command}`: {}",
            describe(&output)
        );
    }
    let approve = documented("specsync change approve ");
    let named = project.approval_refusals_name_answer_commands(&project.root, &approve);
    assert!(
        named
            .first()
            .is_some_and(|command| command.contains(" acceptance_criteria ")),
        "the first refusal must name the acceptance-criteria answer; named {named:?}"
    );
    assert!(
        named.len() >= 2,
        "the unfinished interview must name its own answer command too; named {named:?}"
    );
}
