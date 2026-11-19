// Issue #1084, MERGE-1: a change's record in a module's history is its own file,
// `specs/<module>/history/<change-id>.md`, not a Change Log row and a `version:` bump. Two changes
// to one module then touch different lines, so they merge on GitHub's merge button, which runs no
// merge driver (REQ-change-159, REQ-change-160). `specsync check` validates the fragments
// (REQ-validator-048) and `specsync history` renders one table (REQ-cmd-change-037).
//
// These drive the built binary, so they read the bytes SpecSync writes.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

const CHANGE_LOG: &str =
    "## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n";

fn base_spec() -> String {
    format!(
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n{CHANGE_LOG}"
    )
}

const BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n";

struct Project {
    _temp: TempDir,
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let project = Self { _temp: temp, root };
        project.git(&["init", "-b", "main"]);
        project.git(&["config", "user.email", "test@example.com"]);
        project.git(&["config", "user.name", "Test"]);
        project.git(&["config", "core.autocrlf", "false"]);
        project.git(&["config", "core.eol", "lf"]);
        project.write("src/auth.rs", "// Authentication module.\n");
        project.write("specs/auth/auth.spec.md", &base_spec());
        project.write("specs/auth/requirements.md", BASE_REQUIREMENTS);
        project.write(".gitignore", ".specsync/hashes.json\n");
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
        project.git(&["commit", "-m", "base"]);
        project
    }

    fn git_output(&self, args: &[&str]) -> Output {
        Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .unwrap()
    }

    fn git(&self, args: &[&str]) -> String {
        let output = self.git_output(args);
        assert!(
            output.status.success(),
            "git {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.root.join(relative))
            .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
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

    /// Open, define, approve and `check --commit` one change to `auth` on `branch`, cut from
    /// `main`, the way an agent ships it.
    fn check_change_on_branch(&self, branch: &str, title: &str, delta: &str) -> String {
        self.git(&["checkout", "-q", "-B", branch, "main"]);
        let created: Value = serde_json::from_str(&self.run(&[
            "--json",
            "change",
            "new",
            title,
            "--kind",
            "feature",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ]))
        .unwrap();
        let id = created["change"]["id"].as_str().unwrap().to_string();
        for (question, answer) in [
            ("acceptance_criteria", "The auth contract says it"),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            self.run(&["change", "answer", &id, question, answer]);
        }
        let workspace = self.root.join(".specsync/changes").join(&id);
        let state: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Say it in the contract.\n".to_string(),
                "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
                "testing" if delta.contains("REQ-auth-002") => "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-002 | SpecSync structural check of the module |\n".to_string(),
                _ => "# Complete\n\nReviewed contract wording.\n".to_string(),
            };
            fs::write(workspace.join(format!("{name}.md")), content).unwrap();
        }
        fs::write(workspace.join("deltas/auth.md"), delta).unwrap();
        self.run(&["change", "approve", &id, "--actor", "Scope owner"]);
        self.git(&["add", "--all"]);
        self.git(&["commit", "-q", "-m", &format!("approve {id}")]);
        self.run(&["change", "check", &id, "--commit"]);
        id
    }
}

fn change_log_section(spec: &str) -> &str {
    &spec[spec.find("## Change Log").expect("spec has a Change Log")..]
}

fn frontmatter_value<'a>(fragment: &'a str, key: &str) -> Option<&'a str> {
    let body = fragment.strip_prefix("---\n")?;
    let end = body.find("\n---\n")?;
    body[..end]
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(str::trim)
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.char_indices().all(|(index, character)| {
            if index == 4 || index == 7 {
                character == '-'
            } else {
                character.is_ascii_digit()
            }
        })
}

const ADD_REQUIREMENT: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL let a user sign in with a passkey.\n\nAcceptance Criteria\n- A passkey signs the user in.\n";

const REWORD_INVARIANTS: &str =
    "## MODIFIED\n\n### SPEC SECTION Invariants\n\nA user holds at most one session.\n";

/// MERGE-1 on GitHub's merge button, which never runs a merge driver: two changes to one module,
/// each checked and committed on its own branch, merge with plain `git merge`, and each keeps its
/// own history record. Before #1084 both bumped `version: 1` to `2` and appended a Change Log row
/// after the same line, so this merge conflicted on both.
#[test]
fn two_changes_to_one_module_merge_with_plain_git_and_keep_their_history() {
    let project = Project::new();
    assert!(
        !project.root.join(".gitattributes").exists(),
        "no merge driver is configured"
    );

    let passkeys = project.check_change_on_branch(
        "passkeys",
        "Let a user sign in with a passkey",
        ADD_REQUIREMENT,
    );
    let spec = project.read("specs/auth/auth.spec.md");
    assert!(
        spec.contains("\nversion: 1\n"),
        "materialization must not bump version:\n{spec}"
    );
    assert_eq!(
        change_log_section(&spec),
        CHANGE_LOG,
        "materialization must leave the Change Log byte for byte"
    );
    let fragment_path = format!("specs/auth/history/{passkeys}.md");
    let fragment = project.read(&fragment_path);
    assert_eq!(frontmatter_value(&fragment, "change"), Some(passkeys.as_str()));
    let date = frontmatter_value(&fragment, "date").unwrap_or_default();
    assert!(is_iso_date(date), "fragment date is YYYY-MM-DD:\n{fragment}");
    assert!(
        fragment.trim_end().ends_with("Let a user sign in with a passkey"),
        "fragment summary is the change title:\n{fragment}"
    );
    assert!(
        project
            .git(&["ls-files", "--error-unmatch", &fragment_path])
            .contains(&fragment_path),
        "`change check --commit` commits the fragment"
    );
    assert!(
        project.git(&["status", "--porcelain"]).trim().is_empty(),
        "`change check --commit` leaves nothing behind"
    );

    let session = project.check_change_on_branch(
        "one-session",
        "Say a user holds one session",
        REWORD_INVARIANTS,
    );
    assert!(project.read("specs/auth/auth.spec.md").contains("\nversion: 1\n"));

    project.git(&["checkout", "-q", "passkeys"]);
    let merge = project.git_output(&["merge", "--no-edit", "one-session"]);
    assert!(
        merge.status.success(),
        "plain git merge conflicted:\n{}{}\n{}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr),
        project.git(&["status", "--porcelain"])
    );

    let merged = project.read("specs/auth/auth.spec.md");
    assert!(!merged.contains("<<<<<<<"), "{merged}");
    assert!(merged.contains("\nversion: 1\n"), "{merged}");
    assert!(merged.contains("A user holds at most one session."), "{merged}");
    assert_eq!(change_log_section(&merged), CHANGE_LOG);
    assert!(project
        .read("specs/auth/requirements.md")
        .contains("### REQ-auth-002"));
    for id in [&passkeys, &session] {
        assert!(
            project
                .root
                .join(format!("specs/auth/history/{id}.md"))
                .is_file(),
            "the merge keeps {id}'s history fragment"
        );
    }

    project.run(&["check"]);
    let history = project.run(&["history", "auth"]);
    assert!(history.contains("| 2026-01-01 | Initial |"), "{history}");
    assert!(
        history.contains(&format!("{passkeys}: Let a user sign in with a passkey")),
        "{history}"
    );
    assert!(
        history.contains(&format!("{session}: Say a user holds one session")),
        "{history}"
    );
}

/// A change an earlier SpecSync materialized is recorded by its Change Log row: checking it again
/// writes no fragment and touches nothing (REQ-change-159).
#[test]
fn a_change_its_change_log_already_names_gets_no_fragment() {
    let project = Project::new();
    let id = project.check_change_on_branch(
        "passkeys",
        "Let a user sign in with a passkey",
        ADD_REQUIREMENT,
    );
    // Rewrite the record the way 6.3.0 wrote it: a row and a bump, no fragment.
    let spec = project.read("specs/auth/auth.spec.md").replace(
        "\nversion: 1\n",
        "\nversion: 2\n",
    ) + &format!("| 2026-11-18 | {id}: Let a user sign in with a passkey |\n");
    project.write("specs/auth/auth.spec.md", &spec);
    fs::remove_file(project.root.join(format!("specs/auth/history/{id}.md"))).unwrap();
    project.git(&["add", "--all"]);
    project.git(&["commit", "-q", "-m", "legacy record"]);

    project.run(&["change", "check", &id, "--commit"]);

    assert_eq!(project.read("specs/auth/auth.spec.md"), spec);
    assert!(!project.root.join("specs/auth/history").exists());
}

fn history_project(spec_version: &str, table: &str) -> Project {
    let project = Project::new();
    let spec = base_spec()
        .replace("\nversion: 1\n", &format!("\nversion: {spec_version}\n"))
        .replace(CHANGE_LOG, table);
    project.write("specs/auth/auth.spec.md", &spec);
    project
}

fn fragment(id: &str, date: &str, summary: &str) -> String {
    format!("---\nchange: {id}\ndate: {date}\n---\n\n{summary}\n")
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// REQ-validator-048: each malformed fragment fails `specsync check`, and the error names the file.
#[test]
fn check_refuses_a_malformed_history_fragment_and_names_it() {
    let cases: [(&str, String); 7] = [
        (
            "renamed.md",
            fragment("another-change", "2026-11-18", "Summary"),
        ),
        ("bad-date.md", fragment("bad-date", "18 November", "Summary")),
        ("empty.md", fragment("empty", "2026-11-18", "")),
        (
            "two-lines.md",
            fragment("two-lines", "2026-11-18", "First line\nSecond line"),
        ),
        ("bare.md", "Just a summary, no frontmatter.\n".to_string()),
        ("notes.txt", fragment("notes", "2026-11-18", "Summary")),
        (
            "recorded-twice.md",
            fragment("recorded-twice", "2026-11-18", "Summary"),
        ),
    ];
    for (name, content) in cases {
        let project = Project::new();
        if name == "recorded-twice.md" {
            let spec = base_spec() + "| 2026-11-18 | recorded-twice: Summary |\n";
            project.write("specs/auth/auth.spec.md", &spec);
        }
        project.run(&["check"]);
        project.write(&format!("specs/auth/history/{name}"), &content);

        let output = project.invoke(&["check"]);

        let text = combined(&output);
        assert!(
            !output.status.success(),
            "check accepted the malformed fragment {name}:\n{text}"
        );
        assert!(
            text.contains(&format!("history/{name}")),
            "check did not name {name}:\n{text}"
        );
    }
}

/// REQ-validator-048: well-formed fragments pass a strict check.
#[test]
fn well_formed_history_fragments_pass_a_strict_check() {
    let project = Project::new();
    project.write(
        "specs/auth/history/add-passkeys.md",
        &fragment("add-passkeys", "2026-11-18", "Let a user sign in with a passkey"),
    );
    project.write(
        "specs/auth/history/one-session.md",
        "---\nchange: one-session\ndate: 2026-11-19\nnote: kept and ignored\n---\n\nSay a user holds one session\n",
    );
    project.run(&["check", "--strict"]);
}

/// REQ-hash-cache-006: a cached clean verdict does not hide a fragment written after it.
#[test]
fn a_cached_clean_check_does_not_hide_a_new_malformed_fragment() {
    let project = Project::new();
    project.run(&["check"]);
    project.run(&["check"]);
    project.write(
        "specs/auth/history/broken.md",
        &fragment("broken", "not-a-date", "Summary"),
    );
    let output = project.invoke(&["check"]);
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("history/broken.md"));
}

/// REQ-cmd-change-037: the frozen table, rows unchanged, then one row per fragment in its
/// columns, the Version column numbered on from the frozen `version:`, in date then id order.
#[test]
fn history_prints_the_frozen_table_then_one_row_per_fragment() {
    let table = "## Change Log\n\n| Version | Date | Change |\n|---------|------|--------|\n| 3 | 2026-01-01 | Initial \\| first cut |\n| 4 | 2026-02-01 | Second |\n";
    let project = history_project("4", table);
    project.write(
        "specs/auth/history/zeta.md",
        &fragment("zeta", "2026-11-01", "Zeta lands first by date"),
    );
    project.write(
        "specs/auth/history/beta.md",
        &fragment("beta", "2026-11-02", "Beta with a | pipe"),
    );
    project.write(
        "specs/auth/history/alpha.md",
        &fragment("alpha", "2026-11-02", "Alpha shares Beta's date"),
    );

    let text = project.run(&["history", "auth"]);

    let expected = "| Version | Date | Change |\n|---------|------|--------|\n| 3 | 2026-01-01 | Initial \\| first cut |\n| 4 | 2026-02-01 | Second |\n| 5 | 2026-11-01 | zeta: Zeta lands first by date |\n| 6 | 2026-11-02 | alpha: Alpha shares Beta's date |\n| 7 | 2026-11-02 | beta: Beta with a \\| pipe |\n";
    assert!(text.contains(expected), "{text}");

    let json: Value = serde_json::from_str(&project.run(&["--json", "history", "auth"])).unwrap();
    assert_eq!(json["module"], "auth");
    assert_eq!(json["version"], "4");
    assert_eq!(
        json["rows"],
        serde_json::json!([
            "| 3 | 2026-01-01 | Initial \\| first cut |",
            "| 4 | 2026-02-01 | Second |"
        ])
    );
    let fragments = json["fragments"].as_array().unwrap();
    assert_eq!(fragments.len(), 3);
    assert_eq!(fragments[0]["change"], "zeta");
    assert_eq!(fragments[1]["change"], "alpha");
    assert_eq!(fragments[2]["date"], "2026-11-02");
    assert_eq!(fragments[2]["summary"], "Beta with a | pipe");
}

/// REQ-cmd-change-037: a spec with no Change Log table gets a `| Date | Change |` table.
#[test]
fn history_of_a_spec_without_a_change_log_uses_a_date_and_change_table() {
    let project = history_project("1", "");
    project.write(
        "specs/auth/history/alpha.md",
        &fragment("alpha", "2026-11-02", "Alpha"),
    );
    let text = project.run(&["history", "auth"]);
    assert!(
        text.contains("| Date | Change |\n|------|--------|\n| 2026-11-02 | alpha: Alpha |\n"),
        "{text}"
    );
}

/// REQ-cmd-change-037: an unknown module and a malformed fragment exit 1 and name what is wrong.
#[test]
fn history_refuses_an_unknown_module_and_a_malformed_fragment() {
    let project = Project::new();
    let unknown = project.invoke(&["history", "billing"]);
    assert_eq!(unknown.status.code(), Some(1), "{}", combined(&unknown));
    assert!(combined(&unknown).contains("billing"), "{}", combined(&unknown));

    project.write(
        "specs/auth/history/broken.md",
        &fragment("other", "2026-11-02", "Broken"),
    );
    let malformed = project.invoke(&["history", "auth"]);
    assert_eq!(malformed.status.code(), Some(1), "{}", combined(&malformed));
    assert!(
        combined(&malformed).contains("history/broken.md"),
        "{}",
        combined(&malformed)
    );
}
