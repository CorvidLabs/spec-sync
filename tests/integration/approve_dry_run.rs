// Issue #1091, CHANGE-15: `specsync change approve <id> --dry-run` says whether the approval would
// be accepted, and writes nothing. It runs the approval's own checks, reports the first problem
// from each check that can run, and names the checks it skipped (REQ-change-163). The command
// prints plain lines or one JSON document and exits 0 or 1 (REQ-cmd-change-040).
//
// The faulty drafts are the ones the Chough tester found, where Chough's `change ready` said
// "Nothing is missing." and `change approve` then refused (CorvidLabs/chough#311). Each row here
// asks the dry run first, checks that not one file changed, and then runs the real approval, which
// must refuse with the same message.
//
// These drive the built binary, so they read the bytes SpecSync prints and writes.

use super::helpers::{specsync, state_folder};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const SPEC: &str = "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n";

const REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n";

/// One valid item, and the valid delta that holds it.
const ITEM: &str = "### REQUIREMENT REQ-auth-002\n\nThe system SHALL offer passkeys.\n\nAcceptance Criteria\n- A user can register a passkey.\n";

fn clean_delta() -> String {
    format!("## ADDED\n\n{ITEM}")
}

/// The checks of a definition approval, in the order `change approve` makes them.
const CHECKS: [&str; 13] = [
    "relay",
    "lifecycle_lock",
    "change",
    "sequence",
    "state",
    "roster",
    "baseline",
    "definition",
    "deltas",
    "ownership",
    "verification_ignored_paths",
    "approval_record",
    "actor",
];

const SPEC_ARGS: [&str; 6] = [
    "--kind",
    "feature",
    "--spec",
    "auth",
    "--path",
    "src/auth.rs",
];

struct Project {
    _temp: TempDir,
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let project = Self { _temp: temp, root };
        project.git(&["init", "-q", "-b", "main"]);
        project.git(&["config", "user.email", "test@example.com"]);
        project.git(&["config", "user.name", "Test"]);
        project.git(&["config", "core.autocrlf", "false"]);
        project.git(&["config", "core.eol", "lf"]);
        project.write("src/auth.rs", "// Authentication module.\n");
        project.write("src/other.rs", "// No module owns this file.\n");
        project.write("specs/auth/auth.spec.md", SPEC);
        project.write("specs/auth/requirements.md", REQUIREMENTS);
        project.write(
            ".gitignore",
            ".specsync/hashes.json\n.specsync/change.lock\n",
        );
        project.write(
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
        project.git(&["add", "--all"]);
        project.git(&["commit", "-q", "-m", "base"]);
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
            "git {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    /// Plant a transaction journal where SpecSync keeps its own, in the user's cache folder, as
    /// an interrupted write leaves it (REQ-change-162).
    fn write_journal(&self, content: &str) {
        let path = state_folder(&self.root).join("change-transaction.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn invoke(&self, args: &[&str]) -> Output {
        specsync()
            .args(["--root", self.root.to_str().unwrap()])
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "specsync {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    /// Open a change and finish its definition: the interview when `answered`, every selected
    /// artifact, and `delta` as the `auth` delta when there is one.
    fn draft(&self, title: &str, new_args: &[&str], delta: Option<&str>, answered: bool) -> String {
        let mut args = vec!["--json", "change", "new", title];
        args.extend_from_slice(new_args);
        let created: Value = serde_json::from_str(&self.run(&args)).unwrap();
        let id = created["change"]["id"].as_str().unwrap().to_string();
        if answered {
            for (question, answer) in [
                ("acceptance_criteria", "The auth contract says it"),
                ("public_contract", "no"),
                ("architecture_risk", "no"),
            ] {
                self.run(&["change", "answer", &id, question, answer]);
            }
        }
        let workspace = self.root.join(".specsync/changes").join(&id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let body = match name {
                "tasks" => "# Tasks\n\n- [x] Say it in the contract.\n".to_string(),
                "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
                other => format!("# {other}\n\nFilled by the fixture.\n"),
            };
            fs::write(
                workspace.join(format!("{name}.md")),
                format!("---\nchange: {id}\nartifact: {name}\n---\n\n{body}"),
            )
            .unwrap();
        }
        if let Some(delta) = delta {
            self.write_delta(&id, delta);
        }
        id
    }

    /// A complete draft over `auth` with a valid delta: the approval accepts it.
    fn clean_draft(&self) -> String {
        self.draft("add passkeys", &SPEC_ARGS, Some(&clean_delta()), true)
    }

    fn write_delta(&self, id: &str, delta: &str) {
        self.write(&format!(".specsync/changes/{id}/deltas/auth.md"), delta);
    }

    /// Every file of the project and its bytes: tracked, untracked and ignored files, and
    /// everything under `.git`, so the index and SpecSync's read cache are compared too.
    fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        fn walk(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
            let mut entries: Vec<_> = fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            entries.sort();
            for path in entries {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if path.is_dir() {
                    files.insert(format!("{relative}/"), Vec::new());
                    walk(root, &path, files);
                } else {
                    files.insert(relative, fs::read(&path).unwrap());
                }
            }
        }
        let mut files = BTreeMap::new();
        walk(&self.root, &self.root, &mut files);
        files
    }

    /// Run the dry run with `--json`. Returns its exit code and the one document it printed.
    fn dry_run(&self, id: &str, flags: &[&str]) -> (i32, Value) {
        let mut args = vec!["--json", "change", "approve", id, "--dry-run"];
        args.extend_from_slice(flags);
        let output = self.invoke(&args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let document: Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
            panic!(
                "the dry run prints one JSON document ({error}):\n{stdout}{}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().unwrap_or(-1), document)
    }

    /// Run the dry run and assert that not one file of the project changed.
    fn dry_run_leaving_the_tree(&self, id: &str, flags: &[&str]) -> (i32, Value) {
        let before = self.snapshot();
        let result = self.dry_run(id, flags);
        let after = self.snapshot();
        assert_same_tree(&before, &after);
        result
    }

    /// Run the real approval with `--json`: `Err` with its refusal, or `Ok` when it approved.
    fn approve(&self, id: &str, flags: &[&str]) -> Result<(), String> {
        let mut args = vec!["--json", "change", "approve", id];
        args.extend_from_slice(flags);
        let output = self.invoke(&args);
        if output.status.success() {
            return Ok(());
        }
        let document: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "a refused approve prints its error document ({error}):\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        Err(document["error"]
            .as_str()
            .unwrap_or_else(|| panic!("no error in {document}"))
            .to_string())
    }

    fn state(&self, id: &str) -> String {
        let status: Value =
            serde_json::from_str(&self.run(&["--json", "change", "status", id])).unwrap();
        status["summary"]["state"].as_str().unwrap().to_string()
    }
}

const ACTOR: [&str; 2] = ["--actor", "Scope owner"];

fn assert_same_tree(before: &BTreeMap<String, Vec<u8>>, after: &BTreeMap<String, Vec<u8>>) {
    let created: Vec<&String> = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .collect();
    let removed: Vec<&String> = before
        .keys()
        .filter(|path| !after.contains_key(*path))
        .collect();
    let changed: Vec<&String> = before
        .iter()
        .filter(|(path, bytes)| after.get(*path).is_some_and(|now| now != *bytes))
        .map(|(path, _)| path)
        .collect();
    assert!(
        created.is_empty() && removed.is_empty() && changed.is_empty(),
        "the dry run wrote to the project: created {created:?}, removed {removed:?}, changed {changed:?}"
    );
}

/// The entry of one check in a dry-run document.
fn check<'a>(document: &'a Value, name: &str) -> &'a Value {
    document["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("no checks in {document}"))
        .iter()
        .find(|entry| entry["check"] == name)
        .unwrap_or_else(|| panic!("no `{name}` check in {document}"))
}

/// The refused checks of a dry-run document, as (check, message), in approval order.
fn refusals(document: &Value) -> Vec<(String, String)> {
    document["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("no checks in {document}"))
        .iter()
        .filter(|entry| entry["result"] == "refused")
        .map(|entry| {
            (
                entry["check"].as_str().unwrap().to_string(),
                entry["message"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// A draft refused for exactly one reason: the dry run names `expected_check` with a message
/// holding `fragment`, changes no file, and the real approval then refuses with that message.
fn assert_one_refusal(
    project: &Project,
    id: &str,
    case: &str,
    expected_check: &str,
    fragment: &str,
) {
    let (code, document) = project.dry_run_leaving_the_tree(id, &ACTOR);
    assert_eq!(code, 1, "{case}: a refused dry run exits 1: {document}");
    assert_eq!(document["would_approve"], false, "{case}: {document}");
    let refused = refusals(&document);
    assert_eq!(
        refused.len(),
        1,
        "{case}: one fault is one refusal: {document}"
    );
    let (refused_check, message) = &refused[0];
    assert_eq!(refused_check, expected_check, "{case}: {document}");
    assert!(
        message.contains(fragment),
        "{case}: the refusal says `{fragment}`: {message}"
    );
    let real = project
        .approve(id, &ACTOR)
        .expect_err(&format!("{case}: the real approve refuses too"));
    assert_eq!(
        &real, message,
        "{case}: the dry run and the real approve give one message"
    );
    assert_eq!(project.state(id), "draft", "{case}: nothing was approved");
}

// REQ-change-163, criterion 3: the delta faults from the Chough tester's list. One complete draft,
// one fault at a time in its delta.
#[test]
fn the_dry_run_reports_what_approve_then_refuses_for_each_faulty_delta() {
    let project = Project::new();
    let id = project.clean_draft();
    let item_with_heading = |heading: &str| {
        format!(
            "## ADDED\n\n{heading}\n\nThe system SHALL offer passkeys.\n\nAcceptance Criteria\n- One.\n"
        )
    };
    let mut cases: Vec<(String, String, &str)> = vec![
        (
            "an unknown operation heading".into(),
            format!("## CHANGED\n\n{ITEM}"),
            "invalid delta operation heading `## CHANGED`",
        ),
        (
            "a heading with no item under it".into(),
            item_with_heading("###"),
            "no items under a recognized operation heading",
        ),
        (
            "prose with no heading".into(),
            "The system should offer passkeys some day.\n".into(),
            "no recognized operation headings",
        ),
        (
            "an empty item".into(),
            "## ADDED\n\n### REQUIREMENT REQ-auth-002\n".into(),
            "requirement `REQ-auth-002` must contain a normative SHALL statement",
        ),
        (
            "an empty delta file".into(),
            String::new(),
            "semantic delta for `auth` is empty",
        ),
        (
            "an ADDED requirement whose id exists".into(),
            "## ADDED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep two sessions per user.\n\nAcceptance Criteria\n- Two sessions.\n".into(),
            "cannot add existing block `REQ-auth-001` with different content; use ## MODIFIED",
        ),
        (
            "one item declared twice".into(),
            "## MODIFIED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep two sessions per user.\n\nAcceptance Criteria\n- Two.\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL keep three sessions per user.\n\nAcceptance Criteria\n- Three.\n".into(),
            "declares Modified Requirement `REQ-auth-001` more than once",
        ),
        (
            "a requirement with no SHALL".into(),
            "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system offers passkeys.\n\nAcceptance Criteria\n- A user can register a passkey.\n".into(),
            "requirement `REQ-auth-002` must contain a normative SHALL statement",
        ),
        (
            "an ADDED requirement with no Acceptance Criteria".into(),
            "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL offer passkeys.\n".into(),
            "requirement `REQ-auth-002` must include Acceptance Criteria",
        ),
        (
            "another module's requirement id".into(),
            "## ADDED\n\n### REQUIREMENT REQ-other-001\n\nThe system SHALL offer passkeys.\n\nAcceptance Criteria\n- One.\n".into(),
            "requirement ID `REQ-other-001` must match affected module `auth`",
        ),
        (
            "a malformed requirement id".into(),
            item_with_heading("### REQUIREMENT REQ-auth-001 — Größe"),
            "must use REQ-<module>-<number>",
        ),
        // #1088: the `## Change Log` table is frozen, so `change approve` refuses a delta aimed at
        // it, and the dry run says so in the `deltas` check.
        (
            "a delta aimed at the Change Log".into(),
            "## MODIFIED\n\n### SPEC SECTION Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-11-19 | Rewritten |\n".into(),
            "targets its `## Change Log` section, which SpecSync no longer writes",
        ),
    ];
    // Item headings that are not `### REQUIREMENT <id>` or `### SPEC SECTION <name>`: short,
    // bare, non-ASCII and near-miss forms, none of which may panic.
    for heading in [
        "### REQ-auth-001",
        "### Notes",
        "### N",
        "### ",
        "### Größe",
        "### aÉÉÉÉÉÉÉ",
        "### REQUIREMENT",
        "### REQUIREMENTÉ",
        "### SPEC SECTIONé",
        "### 🙂…",
    ] {
        cases.push((
            format!("the item heading `{heading}`"),
            item_with_heading(heading),
            "invalid delta item heading",
        ));
    }
    for (case, delta, fragment) in &cases {
        project.write_delta(&id, delta);
        assert_one_refusal(&project, &id, case, "deltas", fragment);
    }

    // The same draft with its fault removed is approved: the faults were the only reason.
    project.write_delta(&id, &clean_delta());
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 0, "{document}");
    project
        .approve(&id, &ACTOR)
        .expect("the clean draft is approved");
}

// REQ-change-163, criterion 3: the ownership faults from the tester's list, and an unfinished
// interview.
#[test]
fn the_dry_run_reports_what_approve_then_refuses_for_unowned_paths_and_an_open_interview() {
    // A justified no-spec change over a path a module owns, with no module declared.
    let project = Project::new();
    let id = project.draft(
        "tidy auth",
        &[
            "--kind",
            "bug-fix",
            "--no-spec-change",
            "--rationale",
            "internal",
            "--path",
            "src/auth.rs",
        ],
        None,
        true,
    );
    assert_one_refusal(
        &project,
        &id,
        "a no-spec change that declares no module",
        "ownership",
        "no declared module canonically owns this affected path: src/auth.rs",
    );

    // A declared module that does not own the path.
    let project = Project::new();
    let id = project.draft(
        "add passkeys",
        &[
            "--kind",
            "feature",
            "--spec",
            "auth",
            "--path",
            "src/other.rs",
        ],
        Some(&clean_delta()),
        true,
    );
    assert_one_refusal(
        &project,
        &id,
        "a path no declared module owns",
        "ownership",
        "no declared module canonically owns this affected path: src/other.rs",
    );

    // A declared spec whose front matter no longer parses owns nothing.
    let project = Project::new();
    let id = project.clean_draft();
    project.write(
        "specs/auth/auth.spec.md",
        &SPEC.replace("files:\n  - src/auth.rs\n", "files: [src/auth.rs\n"),
    );
    assert_one_refusal(
        &project,
        &id,
        "a declared spec that does not parse",
        "ownership",
        "no declared module canonically owns this affected path: src/auth.rs",
    );

    // Nothing answered yet: the definition check names the first answer to give.
    let project = Project::new();
    let id = project.draft("add passkeys", &SPEC_ARGS, Some(&clean_delta()), false);
    assert_one_refusal(
        &project,
        &id,
        "an unfinished interview",
        "definition",
        "at least one acceptance criterion is required; run `specsync change answer",
    );
}

// REQ-change-163 and REQ-cmd-change-040, criteria 1 and 2: a clean draft passes every check, exit
// 0, and not one file changes, whether or not a lifecycle lock and a hash cache are already
// there. The real approval then succeeds.
#[test]
fn a_clean_draft_passes_the_dry_run_and_is_then_approved() {
    let project = Project::new();
    let id = project.clean_draft();

    // With no lifecycle lock and no hash cache, neither appears. Opening the draft took the
    // lock, so it is removed first: a fresh clone has none.
    fs::remove_file(project.root.join(".specsync/change.lock")).unwrap();
    assert!(!project.root.join(".specsync/hashes.json").exists());
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert!(!project.root.join(".specsync/change.lock").exists());
    assert_eq!(code, 0, "a clean dry run exits 0: {document}");
    assert_eq!(document["id"], id.as_str(), "{document}");
    assert_eq!(document["dry_run"], true, "{document}");
    assert_eq!(document["would_approve"], true, "{document}");
    let checks: Vec<&str> = document["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .map(|entry| entry["check"].as_str().unwrap())
        .collect();
    assert_eq!(checks, CHECKS, "one entry per check, in approval order");
    for entry in document["checks"].as_array().unwrap() {
        assert_eq!(entry["result"], "passed", "{document}");
        assert!(entry.get("message").is_none(), "{document}");
        assert!(entry.get("skipped_because").is_none(), "{document}");
    }
    assert!(document.get("error").is_none(), "{document}");

    // With a lock and a hash cache in place, both are left as they are.
    project.write(".specsync/change.lock", "");
    project.write(".specsync/hashes.json", "{\"kept\": \"as it is\"}\n");
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 0, "{document}");

    // In text: a closing line that says so, and that nothing was written.
    let before = project.snapshot();
    let output = project.invoke(&[
        "change",
        "approve",
        &id,
        "--dry-run",
        "--actor",
        "Scope owner",
    ]);
    assert_same_tree(&before, &project.snapshot());
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{text}");
    let closing = text.lines().last().unwrap_or_default();
    assert!(
        closing.contains("would be approved") && closing.contains("nothing was written"),
        "the closing line: {text}"
    );

    assert_eq!(project.state(&id), "draft", "the dry run approved nothing");
    project
        .approve(&id, &ACTOR)
        .expect("the real approve succeeds");
    assert_eq!(project.state(&id), "approved");
}

// REQ-change-163, criterion 6: two independent problems are both reported in one run, each under
// its own check, and the approval record is skipped because it reads the refused delta. The first
// one is what the real approve says.
#[test]
fn the_dry_run_reports_two_problems_in_one_run() {
    let project = Project::new();
    let id = project.draft(
        "add passkeys",
        &SPEC_ARGS,
        Some(&format!("## CHANGED\n\n{ITEM}")),
        false,
    );
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    let names: Vec<&str> = refused.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["definition", "deltas"], "{document}");
    assert!(
        refused[0]
            .1
            .contains("at least one acceptance criterion is required"),
        "{document}"
    );
    assert!(
        refused[1].1.contains("invalid delta operation heading"),
        "{document}"
    );
    let record = check(&document, "approval_record");
    assert_eq!(record["result"], "skipped", "{document}");
    assert_eq!(record["skipped_because"], "deltas", "{document}");
    for name in ["ownership", "verification_ignored_paths", "actor"] {
        assert_eq!(check(&document, name)["result"], "passed", "{document}");
    }
    let real = project.approve(&id, &ACTOR).expect_err("refused");
    assert_eq!(
        real, refused[0].1,
        "the first refusal is the real approve's"
    );

    // In text: one line per refused check, one per skipped check, then the closing line.
    let output = project.invoke(&[
        "change",
        "approve",
        &id,
        "--dry-run",
        "--actor",
        "Scope owner",
    ]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{text}");
    for (name, message) in &refused {
        assert!(
            text.contains(&format!("{name}: {message}")),
            "a line for `{name}`: {text}"
        );
    }
    assert!(
        text.lines()
            .any(|line| line.contains("approval_record") && line.contains("deltas")),
        "a line for the skipped check and what it waits on: {text}"
    );
    let closing = text.lines().last().unwrap_or_default();
    assert!(
        closing.contains("would be refused") && closing.contains("nothing was written"),
        "the closing line: {text}"
    );
}

// REQ-change-163: a change that does not load is one refusal, and every later check is skipped
// because of it.
#[test]
fn a_change_that_does_not_exist_is_one_refusal_and_the_rest_is_skipped() {
    let project = Project::new();
    project.clean_draft();
    let (code, document) = project.dry_run_leaving_the_tree("no-such-change", &ACTOR);
    assert_eq!(code, 1, "{document}");
    assert_eq!(document["would_approve"], false, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "change", "{document}");
    let real = project
        .approve("no-such-change", &ACTOR)
        .expect_err("refused");
    assert_eq!(real, refused[0].1);
    let position = CHECKS.iter().position(|name| *name == "change").unwrap();
    for name in &CHECKS[position + 1..] {
        let entry = check(&document, name);
        assert_eq!(entry["result"], "skipped", "`{name}`: {document}");
        assert_eq!(entry["skipped_because"], "change", "`{name}`: {document}");
    }
    for name in &CHECKS[..position] {
        assert_eq!(check(&document, name)["result"], "passed", "{document}");
    }
}

// REQ-change-163, criterion 5: a change that was sent back is a draft again, and the dry run says
// what its re-approval would.
#[test]
fn a_sent_back_change_gets_the_dry_run_of_its_reapproval() {
    let project = Project::new();
    let id = project.clean_draft();
    project.approve(&id, &ACTOR).expect("approved");
    project.run(&[
        "change",
        "send-back",
        &id,
        "--actor",
        "Scope owner",
        "--reason",
        "The scope needs another look",
    ]);
    assert_eq!(project.state(&id), "draft");

    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(
        code, 0,
        "an unchanged scope can be approved again: {document}"
    );
    assert_eq!(document["would_approve"], true, "{document}");

    project.write_delta(
        &id,
        "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL offer passkeys.\n",
    );
    assert_one_refusal(
        &project,
        &id,
        "a sent-back change with a faulty delta",
        "deltas",
        "requirement `REQ-auth-002` must include Acceptance Criteria",
    );

    project.write_delta(&id, &clean_delta());
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 0, "{document}");
    project.approve(&id, &ACTOR).expect("approved again");
    assert_eq!(project.state(&id), "approved");
}

// REQ-cmd-change-040: the dry run takes the approval's flags and checks them as the approval does.
// A relayed decision with a detail missing is refused by the `relay` check, in the approval's own
// words, and the rest still runs.
#[test]
fn the_dry_run_checks_the_relay_flags_as_the_approval_does() {
    let project = Project::new();
    let id = project.clean_draft();
    let flags = ["--actor", "Ada", "--words", "Approve it"];
    let (code, document) = project.dry_run_leaving_the_tree(&id, &flags);
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "relay", "{document}");
    let real = project.approve(&id, &flags).expect_err("refused");
    assert_eq!(real, refused[0].1);
    assert_eq!(check(&document, "definition")["result"], "passed");
    assert_eq!(project.state(&id), "draft");

    // The whole relay passes, and it is not recorded.
    let flags = [
        "--actor",
        "Ada",
        "--words",
        "Approve it",
        "--where",
        "the team chat",
        "--recorded-by",
        "an agent",
    ];
    let (code, document) = project.dry_run_leaving_the_tree(&id, &flags);
    assert_eq!(code, 0, "{document}");
    assert_eq!(project.state(&id), "draft");
}

// REQ-change-163: an interrupted lifecycle write is recovered by the next real approval, which is
// a write. The dry run says it is waiting, runs no other check, and leaves the journal alone.
#[test]
fn the_dry_run_refuses_while_an_interrupted_write_is_waiting() {
    let project = Project::new();
    let id = project.clean_draft();
    // A durable journal with no entries, as the change workflow writes one.
    project.write_journal(
        "{\"schema_version\":1,\"entry_count\":0,\"entries_digest\":\"4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945\",\"entries\":[]}",
    );
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "lifecycle_lock", "{document}");
    assert!(
        refused[0]
            .1
            .contains("the transaction journal in SpecSync's local state folder"),
        "the refusal names the waiting write: {document}"
    );
    // The whole sentence, once, as `--json` carries it and as the text output prints it: a
    // message built from a wrapped source string printed runs of spaces and no test noticed.
    let waiting = "an interrupted lifecycle write is waiting to be recovered (the transaction \
                   journal in SpecSync's local state folder); the next lifecycle command that \
                   writes recovers it before it checks anything, and a dry run writes nothing, so \
                   it cannot say what `change approve` would find. Run the dry run again once that \
                   write is recovered";
    assert_eq!(refused[0].1, waiting, "{document}");
    let output = project.invoke(&[
        "change",
        "approve",
        &id,
        "--dry-run",
        "--actor",
        "Scope owner",
    ]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains(&format!("  lifecycle_lock: {waiting}\n")),
        "the text output prints the same sentence: {text}"
    );
    let position = CHECKS
        .iter()
        .position(|name| *name == "lifecycle_lock")
        .unwrap();
    for name in &CHECKS[position + 1..] {
        let entry = check(&document, name);
        assert_eq!(entry["result"], "skipped", "`{name}`: {document}");
        assert_eq!(
            entry["skipped_because"], "lifecycle_lock",
            "`{name}`: {document}"
        );
    }
}

// REQ-change-163: a journal that cannot be read, or that names a path outside the project, is a
// refusal `change approve` itself gives when it takes the lock. So the dry run gives that same
// refusal, word for word, runs nothing after it, and leaves the journal alone. Only a journal
// that reads is "waiting to be recovered".
#[test]
fn the_dry_run_gives_the_approvals_own_refusal_for_a_journal_it_cannot_replay() {
    for (case, journal) in [
        ("a journal that does not parse", "not a journal\n"),
        (
            "a journal entry outside the project",
            "[{\"path\": \"../outside.txt\", \"original\": \"x\"}]\n",
        ),
        (
            "a journal whose integrity envelope does not hold",
            "{\"schema_version\": 1, \"entry_count\": 2, \"entries_digest\": \"0\", \"entries\": []}\n",
        ),
    ] {
        let project = Project::new();
        let id = project.clean_draft();
        project.write_journal(journal);
        let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
        assert_eq!(code, 1, "{case}: {document}");
        let refused = refusals(&document);
        assert_eq!(refused.len(), 1, "{case}: {document}");
        assert_eq!(refused[0].0, "lifecycle_lock", "{case}: {document}");
        assert!(
            refused[0].1.contains("invalid transaction journal"),
            "{case}: the dry run says what approve says: {document}"
        );
        assert_eq!(
            check(&document, "change")["skipped_because"],
            "lifecycle_lock",
            "{case}: {document}"
        );
        let real = project.approve(&id, &ACTOR).expect_err("refused");
        assert_eq!(
            real, refused[0].1,
            "{case}: the dry run and the real approve give one message"
        );
    }
}

/// Run a `fix` exactly as printed, with this build first on `PATH` as `specsync`, from the
/// project root, the way a person or a tool would paste it.
fn run_as_printed(project: &Project, fix: &str) -> Output {
    let binary = assert_cmd::cargo::cargo_bin("specsync");
    let path = format!(
        "{}:{}",
        binary.parent().unwrap().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = Command::new("sh");
    command
        .args(["-c", fix])
        .current_dir(&project.root)
        .env("PATH", path)
        .env_remove("GITHUB_EVENT_NAME")
        .env_remove("GITHUB_BASE_REF");
    for (key, _) in std::env::vars() {
        if key.starts_with("SPECSYNC_") {
            command.env_remove(key);
        }
    }
    command.output().unwrap()
}

// REQ-cmd-change-040, the `fix` field: a refused `definition` entry for a draft with an
// unanswered question carries the `change answer` command its message names, as plain text. It
// may hold a placeholder, so it is not always runnable as printed.
#[test]
fn a_definition_refusal_for_an_open_question_carries_its_command_as_fix() {
    let project = Project::new();
    let id = project.draft("add passkeys", &SPEC_ARGS, Some(&clean_delta()), false);
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 1, "{document}");
    let definition = check(&document, "definition");
    assert_eq!(definition["result"], "refused", "{document}");
    let expected =
        format!("specsync change answer {id} acceptance_criteria \"<observable outcome>\"");
    assert_eq!(definition["fix"], expected.as_str(), "{document}");
    assert!(
        definition["message"]
            .as_str()
            .unwrap()
            .contains(&format!("`{expected}`")),
        "the message names the same command: {document}"
    );

    // Only `public_contract` still open: its two choices are the placeholder.
    project.run(&[
        "change",
        "answer",
        &id,
        "acceptance_criteria",
        "The auth contract says it",
    ]);
    project.run(&["change", "answer", &id, "architecture_risk", "no"]);
    let (_, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    let definition = check(&document, "definition");
    let expected = format!("specsync change answer {id} public_contract <yes|no>");
    assert_eq!(definition["fix"], expected.as_str(), "{document}");
    assert!(
        definition["message"]
            .as_str()
            .unwrap()
            .contains(&format!("`{expected}`")),
        "{document}"
    );
}

// REQ-cmd-change-040, the `fix` field: a refused `ownership` entry whose message names the
// `change answer … affected_specs` command that declares the owning module carries that command,
// and run as printed it clears the ownership refusal.
#[test]
fn an_ownership_refusal_carries_the_declaring_command_as_fix_and_it_runs_as_printed() {
    let project = Project::new();
    let id = project.draft(
        "tidy auth",
        &[
            "--kind",
            "bug-fix",
            "--no-spec-change",
            "--rationale",
            "internal",
            "--path",
            "src/auth.rs",
        ],
        None,
        true,
    );
    let (_, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    let ownership = check(&document, "ownership");
    assert_eq!(ownership["result"], "refused", "{document}");
    let fix = ownership["fix"]
        .as_str()
        .unwrap_or_else(|| panic!("the ownership refusal carries a fix: {document}"));
    assert!(
        fix.starts_with("specsync change answer ") && fix.contains("affected_specs"),
        "{fix}"
    );
    assert!(
        ownership["message"].as_str().unwrap().contains(fix),
        "the message names the same command: {document}"
    );
    let output = run_as_printed(&project, fix);
    assert!(
        output.status.success(),
        "`{fix}` runs as printed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let (_, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(
        check(&document, "ownership")["result"],
        "passed",
        "{document}"
    );
}

// REQ-cmd-change-040, the `fix` field: no other refused entry carries one, and the text output
// shows none.
#[test]
fn only_definition_and_ownership_refusals_carry_a_fix() {
    let project = Project::new();
    let id = project.draft(
        "add passkeys",
        &SPEC_ARGS,
        Some(&format!("## CHANGED\n\n{ITEM}")),
        true,
    );
    let (_, document) =
        project.dry_run_leaving_the_tree(&id, &["--actor", "Scope owner", "--words", "Approve it"]);
    let refused: Vec<&Value> = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["result"] == "refused")
        .collect();
    assert!(
        refused.iter().any(|entry| entry["check"] == "deltas")
            && refused.iter().any(|entry| entry["check"] == "relay"),
        "{document}"
    );
    for entry in &refused {
        assert!(entry.get("fix").is_none(), "no fix on {entry}");
    }
    let output = project.invoke(&[
        "change",
        "approve",
        &id,
        "--dry-run",
        "--actor",
        "Scope owner",
    ]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        !text.contains("fix"),
        "the text output shows no fix: {text}"
    );
}

// REQ-change-163, the lifecycle-lock check (Codex on #1100): a lock path the approval cannot open
// is refused in the approval's own words, not reported as one it could take.
#[test]
fn a_lock_path_the_approval_cannot_open_is_refused_in_its_words() {
    let project = Project::new();
    let id = project.clean_draft();
    // Opening the draft took the lock, which leaves the lock file behind.
    let lock = project.root.join(".specsync/change.lock");
    if lock.exists() {
        fs::remove_file(&lock).unwrap();
    }
    fs::create_dir_all(&lock).unwrap();
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "lifecycle_lock", "{document}");
    let real = project
        .approve(&id, &ACTOR)
        .expect_err("approve cannot open a directory");
    assert_eq!(
        real, refused[0].1,
        "the dry run and the real approve give one message"
    );
}

/// Puts a directory's mode back when the test ends, also when it fails.
#[cfg(unix)]
struct RestoreMode(PathBuf, u32);

#[cfg(unix)]
impl Drop for RestoreMode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(self.1));
    }
}

// REQ-change-163, the lifecycle-lock check (Codex on #1100, round 2): the lock file is
// gitignored, so a fresh clone has none, and the approval creates it. Where `.specsync` cannot
// take a new file, the dry run refuses in the approval's own words instead of predicting an
// approval that fails, and it creates nothing.
#[cfg(unix)]
#[test]
fn a_lock_the_approval_cannot_create_is_refused_in_its_words() {
    use std::os::unix::fs::PermissionsExt;
    let uid = Command::new("id").arg("-u").output().unwrap();
    if String::from_utf8_lossy(&uid.stdout).trim() == "0" {
        eprintln!("skipped: root writes into a read-only directory");
        return;
    }
    let project = Project::new();
    let id = project.clean_draft();
    let metadata = project.root.join(".specsync");
    let lock = metadata.join("change.lock");
    if lock.exists() {
        fs::remove_file(&lock).unwrap();
    }
    let mode = fs::metadata(&metadata).unwrap().permissions().mode() & 0o7777;
    let _restore = RestoreMode(metadata.clone(), mode);
    fs::set_permissions(&metadata, fs::Permissions::from_mode(0o555)).unwrap();
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert!(!lock.exists(), "the dry run creates no lock file");
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "lifecycle_lock", "{document}");
    let real = project
        .approve(&id, &ACTOR)
        .expect_err("approve cannot create the lock");
    assert_eq!(
        real, refused[0].1,
        "the dry run and the real approve give one message"
    );
}

// REQ-cmd-change-040, text output (Codex on #1100): one line for each refused check, even when its
// message holds line breaks, as the ownership refusal's does.
#[test]
fn each_refused_check_is_one_line_of_text() {
    let project = Project::new();
    let id = project.draft(
        "add passkeys",
        &[
            "--kind",
            "feature",
            "--spec",
            "auth",
            "--path",
            "src/other.rs",
        ],
        Some(&clean_delta()),
        true,
    );
    let (_, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    let ownership = check(&document, "ownership")["message"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        ownership.contains('\n'),
        "the fixture's message holds a line break: {ownership}"
    );
    let refused = refusals(&document).len();
    let skipped = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["result"] == "skipped")
        .count();
    let output = project.invoke(&[
        "change",
        "approve",
        &id,
        "--dry-run",
        "--actor",
        "Scope owner",
    ]);
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(
        lines.len(),
        refused + skipped + 1,
        "one line per refused check, one per skipped check and the closing line: {text}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("  ownership: "))
            .count(),
        1,
        "{text}"
    );
}

// REQ-change-162 and REQ-change-163: a journal in the work tree, where a repository can put one,
// is never replayed, and `change approve` refuses while it is there. The dry run gives that same
// refusal and leaves the file alone.
#[test]
fn the_dry_run_gives_the_approvals_own_refusal_for_a_journal_in_the_work_tree() {
    let project = Project::new();
    let id = project.clean_draft();
    project.write(".specsync/change-transaction.json", "[]\n");
    let (code, document) = project.dry_run_leaving_the_tree(&id, &ACTOR);
    assert_eq!(code, 1, "{document}");
    let refused = refusals(&document);
    assert_eq!(refused.len(), 1, "{document}");
    assert_eq!(refused[0].0, "lifecycle_lock", "{document}");
    assert!(
        refused[0]
            .1
            .contains(".specsync/change-transaction.json is in the work tree"),
        "{document}"
    );
}
