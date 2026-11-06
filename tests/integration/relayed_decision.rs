// Issue #633, audit D7: an approval is a recorded claim bound to content, not authentication,
// and an agent that records a decision a person made somewhere else can say who decided, their
// exact words, where they said it and which agent wrote it down (CHANGE-6.d, CHANGE-13 to
// CHANGE-13.d, SHIP-2.e).
//
// These drive the built binary, so they read the bytes SpecSync writes and the text a person
// reads, not the in-memory types. The shape is shared with Chough (chough#123): `relayed_by`
// with exactly `decided_by`, `words`, `where` and `recorded_by`.

use super::helpers::specsync;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

const ID: &str = "ship-the-auth-module";
const ACTOR: &str = "Ada";
const WORDS: &str = "Approve the scope as written";
const WHERE: &str = "Herdr orc tab, 2026-11-05 21:40 MST, relayed by orc t18";
const AGENT: &str = "claude (ss-d7)";
const REVIEW_WORDS: &str = "Record the review as a pass";
const BOUNDARY: &str = "a relayed claim, not authenticated: it records who is said to have decided, not proof that they did.";

struct Project {
    _temp: TempDir,
    root: PathBuf,
}

impl Project {
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

    fn stdout(&self, args: &[&str]) -> String {
        String::from_utf8(self.run(args).success().get_output().stdout.clone()).unwrap()
    }

    fn status_json(&self) -> Value {
        serde_json::from_str(&self.stdout(&["--json", "change", "status", ID])).unwrap()
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn workspace(&self) -> PathBuf {
        self.root.join(".specsync/changes").join(ID)
    }

    fn approvals_path(&self) -> PathBuf {
        self.workspace().join("approvals.json")
    }

    fn approvals(&self) -> Value {
        serde_json::from_slice(&fs::read(self.approvals_path()).unwrap()).unwrap()
    }

    fn last_approval(&self) -> Value {
        self.approvals()["approvals"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone()
    }

    fn state(&self) -> String {
        let state: Value =
            serde_json::from_slice(&fs::read(self.workspace().join("state.json")).unwrap())
                .unwrap();
        state["state"].as_str().unwrap().to_string()
    }

    /// A draft change with a complete definition, ready for `change approve`.
    fn ready_to_approve() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let project = Self { _temp: temp, root };
        project.git(&["init", "-b", "main"]);
        project.git(&["config", "user.email", "test@example.com"]);
        project.git(&["config", "user.name", "Recording agent"]);
        project.write("src/auth.rs", "// Authentication module.\n");
        project.write(
            "specs/auth/auth.spec.md",
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
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
        project.git(&["commit", "-m", "base"]);
        project
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
            project
                .run(&["change", "answer", ID, question, answer])
                .success();
        }
        let state: Value =
            serde_json::from_slice(&fs::read(project.workspace().join("state.json")).unwrap())
                .unwrap();
        for artifact in state["selected_artifacts"].as_array().unwrap() {
            let name = artifact.as_str().unwrap();
            let content = match name {
                "tasks" => "# Tasks\n\n- [x] Ship the module.\n",
                "plan" => super::helpers::SUBSTANTIVE_PLAN,
                _ => "# Complete\n\nReviewed auth delivery.\n",
            };
            fs::write(project.workspace().join(format!("{name}.md")), content).unwrap();
        }
        project
    }

    fn approve_relayed(&self) {
        self.run(&[
            "change",
            "approve",
            ID,
            "--actor",
            ACTOR,
            "--words",
            WORDS,
            "--where",
            WHERE,
            "--recorded-by",
            AGENT,
        ])
        .success();
    }

    /// Implement, then verify and commit, leaving the change ready for `change review`.
    fn implemented_and_checked(&self) {
        self.write("src/auth.rs", "// Authentication module, shipped.\n");
        self.git(&["add", "--all"]);
        self.git(&["commit", "-m", "implement"]);
        self.run(&["change", "check", ID, "--commit"]).success();
    }

    fn review_relayed(&self) {
        self.run(&[
            "change",
            "review",
            ID,
            "--reviewer",
            ACTOR,
            "--words",
            REVIEW_WORDS,
            "--where",
            WHERE,
            "--recorded-by",
            AGENT,
        ])
        .success();
    }

    fn archive_dir(&self) -> PathBuf {
        let archive = self.root.join(".specsync/archive/changes");
        fs::read_dir(&archive)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .find(|path| path.file_name().unwrap().to_str().unwrap().ends_with(ID))
            .unwrap_or_else(|| panic!("no archive package for {ID} in {}", archive.display()))
    }
}

fn expected_relay(words: &str) -> Value {
    serde_json::json!({
        "decided_by": ACTOR,
        "words": words,
        "where": WHERE,
        "recorded_by": AGENT,
    })
}

/// The bytes `serde_json::to_vec` produces for a record SpecSync wrote pretty-printed: the same
/// keys in the same order with the whitespace outside strings removed. That is what a binary
/// hashes after reading the file, without needing the binary's own types.
fn compact(pretty: &str) -> String {
    let mut out = String::with_capacity(pretty.len());
    let (mut in_string, mut escaped) = (false, false);
    for character in pretty.chars() {
        if in_string {
            out.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            out.push(character);
        } else if !character.is_whitespace() {
            out.push(character);
        }
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// DISCRIMINATOR for CHANGE-13: the approval keeps who decided, their exact words, where they
/// said it and which agent recorded it, and `decided_by` is the approval's own actor.
#[test]
fn a_relayed_approval_records_who_decided_their_words_where_and_the_recording_agent() {
    let project = Project::ready_to_approve();
    project.approve_relayed();

    let approval = project.last_approval();
    assert_eq!(approval["actor"], ACTOR);
    assert_eq!(
        approval["relayed_by"],
        expected_relay(WORDS),
        "{approval:#}"
    );
    assert_eq!(
        approval["relayed_by"].as_object().unwrap().len(),
        4,
        "the shared shape has exactly four keys: {approval:#}"
    );
}

/// GUARD for CHANGE-13.a: approving and reviewing at your own terminal asks for no relay and
/// writes no `relayed_by` key, so these records keep the bytes an older binary writes.
#[test]
fn an_approval_and_a_review_without_a_relay_omit_the_record() {
    let project = Project::ready_to_approve();
    project
        .run(&["change", "approve", ID, "--actor", ACTOR])
        .success();
    assert!(
        project.last_approval().get("relayed_by").is_none(),
        "{:#}",
        project.last_approval()
    );
    project.implemented_and_checked();
    project
        .run(&["change", "review", ID, "--reviewer", ACTOR])
        .success();
    let review = fs::read_to_string(project.workspace().join("review.json")).unwrap();
    assert!(!review.contains("relayed_by"), "{review}");
}

/// DISCRIMINATOR for CHANGE-6.d, CHANGE-13.b, CHANGE-13.c and SHIP-2.e: status shows each name
/// as a recorded claim, and each relay beside the record it belongs to with the boundary.
#[test]
fn status_shows_the_approval_and_review_claims_with_the_relay_and_its_boundary() {
    let project = Project::ready_to_approve();
    project.approve_relayed();

    let text = project.stdout(&["change", "status", ID]);
    let approval_line = format!("  Approval: {ACTOR} (a recorded claim, not authenticated)");
    let approval_relay =
        format!("    Relayed: {ACTOR} said \"{WORDS}\" ({WHERE}); recorded by {AGENT}; {BOUNDARY}");
    assert!(text.contains(&approval_line), "{text}");
    assert!(text.contains(&approval_relay), "{text}");
    assert!(!text.contains("  Review: "), "no review yet: {text}");

    project.implemented_and_checked();
    project.review_relayed();
    let text = project.stdout(&["change", "status", ID]);
    let review_line = format!("  Review: {ACTOR} (pass; a recorded claim, not authenticated)");
    let review_relay = format!(
        "    Relayed: {ACTOR} said \"{REVIEW_WORDS}\" ({WHERE}); recorded by {AGENT}; {BOUNDARY}"
    );
    let approval_at = text
        .find(&approval_line)
        .unwrap_or_else(|| panic!("{text}"));
    let review_at = text.find(&review_line).unwrap_or_else(|| panic!("{text}"));
    assert_eq!(
        text[approval_at..].lines().nth(1),
        Some(approval_relay.as_str()),
        "the approval's relay sits on the line after the approval: {text}"
    );
    assert_eq!(
        text[review_at..].lines().nth(1),
        Some(review_relay.as_str()),
        "the review's relay sits on the line after the review: {text}"
    );

    let claims = project.status_json()["recorded_claims"].clone();
    assert_eq!(
        claims,
        serde_json::json!([
            {"gate": "definition", "name": ACTOR, "relayed_by": expected_relay(WORDS)},
            {"gate": "review", "name": ACTOR, "verdict": "pass", "relayed_by": expected_relay(REVIEW_WORDS)},
        ]),
        "{claims:#}"
    );
}

/// GUARD for CHANGE-13.b: with nothing approved or reviewed there is no claim to show.
#[test]
fn a_change_with_no_approval_shows_no_claim() {
    let project = Project::ready_to_approve();
    let text = project.stdout(&["change", "status", ID]);
    assert!(!text.contains("  Approval: "), "{text}");
    assert!(project.status_json().get("recorded_claims").is_none());
}

/// DISCRIMINATOR for CHANGE-13.d: a relay missing a detail is refused before anything is
/// written, the refusal names every missing flag and the command to re-run, and that re-run
/// records the decision.
#[test]
fn an_incomplete_relay_is_refused_before_anything_is_written_and_the_named_rerun_records_it() {
    let project = Project::ready_to_approve();
    let before = fs::read(project.approvals_path()).unwrap();

    let refused = project
        .run(&["change", "approve", ID, "--actor", ACTOR, "--words", WORDS])
        .failure();
    let error = String::from_utf8(refused.get_output().stderr.clone()).unwrap();
    assert!(error.contains("missing: --where, --recorded-by"), "{error}");
    let rerun = format!(
        "specsync change approve {ID} --actor <who decided> --words \"<their exact words>\" --where \"<where and when, and who relayed it>\" --recorded-by \"<this agent>\""
    );
    assert!(error.contains(&rerun), "{error}");
    assert_eq!(fs::read(project.approvals_path()).unwrap(), before);
    assert_eq!(project.state(), "draft");

    // The recovery the refusal names: the same command with all three flags.
    project.approve_relayed();
    assert_eq!(project.last_approval()["relayed_by"], expected_relay(WORDS));

    project.implemented_and_checked();
    let review_path = project.workspace().join("review.json");
    let refused = project
        .run(&[
            "change",
            "review",
            ID,
            "--reviewer",
            ACTOR,
            "--recorded-by",
            AGENT,
        ])
        .failure();
    let error = String::from_utf8(refused.get_output().stderr.clone()).unwrap();
    assert!(error.contains("missing: --words, --where"), "{error}");
    assert!(
        error.contains(&format!(
            "specsync change review {ID} --reviewer <who decided>"
        )),
        "{error}"
    );
    assert!(!review_path.exists(), "a refused review writes nothing");
    project.review_relayed();
    assert!(review_path.exists());
}

/// DISCRIMINATOR for CHANGE-13.d: a blank or over-long detail is refused and named.
#[test]
fn a_blank_or_oversized_relay_detail_is_refused_and_named() {
    let project = Project::ready_to_approve();
    let before = fs::read(project.approvals_path()).unwrap();
    let blank = project
        .run(&[
            "change",
            "approve",
            ID,
            "--actor",
            ACTOR,
            "--words",
            "   ",
            "--where",
            WHERE,
            "--recorded-by",
            AGENT,
        ])
        .failure();
    let error = String::from_utf8(blank.get_output().stderr.clone()).unwrap();
    assert!(error.contains("--words must not be blank"), "{error}");

    let long_where = "w".repeat(4097);
    let long = project
        .run(&[
            "change",
            "approve",
            ID,
            "--actor",
            ACTOR,
            "--words",
            WORDS,
            "--where",
            &long_where,
            "--recorded-by",
            AGENT,
        ])
        .failure();
    let error = String::from_utf8(long.get_output().stderr.clone()).unwrap();
    assert!(
        error.contains("--where is 4097 bytes; a relayed decision field holds at most 4096"),
        "{error}"
    );
    assert_eq!(fs::read(project.approvals_path()).unwrap(), before);
    assert_eq!(project.state(), "draft");
}

/// DISCRIMINATOR for CHANGE-13: without `--actor` the actor would come from Git or `$USER`,
/// which names the recording agent, not the person. A relay therefore needs an explicit actor.
#[test]
fn a_relayed_approval_needs_an_explicit_actor() {
    let project = Project::ready_to_approve();
    let before = fs::read(project.approvals_path()).unwrap();
    let refused = project
        .run(&[
            "change",
            "approve",
            ID,
            "--words",
            WORDS,
            "--where",
            WHERE,
            "--recorded-by",
            AGENT,
        ])
        .failure();
    let error = String::from_utf8(refused.get_output().stderr.clone()).unwrap();
    assert!(
        error.contains("a relayed decision names who decided with --actor <who decided>"),
        "{error}"
    );
    assert_eq!(fs::read(project.approvals_path()).unwrap(), before);
    project.approve_relayed();
}

/// DISCRIMINATOR for SHIP-2.e and Leif's "Keep it out": a relayed review finalizes, and the
/// finalization's review digest is the digest of the review with `relayed_by` left out, which is
/// exactly what a 6.x binary that drops the unknown key computes from the same file.
#[test]
fn a_relayed_review_finalizes_and_its_review_digest_leaves_the_relay_out() {
    let project = Project::ready_to_approve();
    project.approve_relayed();
    project.implemented_and_checked();
    project.review_relayed();
    project.run(&["change", "finalize", ID]).success();

    let archive = project.archive_dir();
    let review = fs::read_to_string(archive.join("review.json")).unwrap();
    let finalization: Value =
        serde_json::from_slice(&fs::read(archive.join("finalization.json")).unwrap()).unwrap();
    let with_relay = compact(&review);
    let relay_at = with_relay
        .find(",\"relayed_by\":{")
        .unwrap_or_else(|| panic!("the archived review keeps its relay: {review}"));
    let relay_end = relay_at + with_relay[relay_at..].find('}').unwrap() + 1;
    let without_relay = format!("{}{}", &with_relay[..relay_at], &with_relay[relay_end..]);

    assert_eq!(
        finalization["review_digest"],
        sha256_hex(without_relay.as_bytes()),
        "the review digest must not cover the relay: {review}"
    );
    assert_ne!(
        finalization["review_digest"],
        sha256_hex(with_relay.as_bytes())
    );

    let text = project.stdout(&["change", "status", ID]);
    assert!(
        text.contains(&format!(
            "  Review: {ACTOR} (pass; a recorded claim, not authenticated)"
        )),
        "{text}"
    );
    assert!(text.contains(REVIEW_WORDS), "{text}");
}

/// DISCRIMINATOR for the read side of the shared shape: a hand-edited relay (an unknown key, or
/// `null`) is refused, the refusal names restoring the file from Git, and that command recovers.
#[test]
fn a_malformed_relay_record_is_refused_naming_git_checkout_and_that_command_recovers() {
    let project = Project::ready_to_approve();
    project.approve_relayed();
    project.git(&["add", "--all"]);
    project.git(&["commit", "-m", "approve"]);
    let relative = format!(".specsync/changes/{ID}/approvals.json");
    let original = fs::read_to_string(project.approvals_path()).unwrap();

    for replacement in [
        "\"relayed_by\": {\n        \"signed\": true,",
        "\"relayed_by\": null, \"unused\": {",
    ] {
        let edited = original.replacen("\"relayed_by\": {", replacement, 1);
        assert_ne!(edited, original);
        fs::write(project.approvals_path(), edited).unwrap();
        let refused = project.run(&["change", "status", ID]).failure();
        let error = String::from_utf8(refused.get_output().stderr.clone()).unwrap();
        assert!(error.contains("relayed_by"), "{error}");
        let recovery = format!("git checkout HEAD -- {relative}");
        assert!(error.contains(&recovery), "{error}");

        project.git(&["checkout", "HEAD", "--", &relative]);
        project.run(&["change", "status", ID]).success();
    }
}

/// DISCRIMINATOR for REQ-cmd-change-relayed-decision-001: recorded words are printed, not
/// obeyed, so a control character cannot recolour or move the terminal.
#[test]
fn relayed_words_are_escaped_in_text_status() {
    let project = Project::ready_to_approve();
    project
        .run(&[
            "change",
            "approve",
            ID,
            "--actor",
            ACTOR,
            "--words",
            "Approve\u{1b}[31m it\nnow",
            "--where",
            WHERE,
            "--recorded-by",
            AGENT,
        ])
        .success();
    assert_eq!(
        project.last_approval()["relayed_by"]["words"],
        "Approve\u{1b}[31m it\nnow",
        "the record keeps the words verbatim"
    );
    let text = project.stdout(&["change", "status", ID]);
    assert!(!text.contains('\u{1b}'), "{text:?}");
    assert!(
        text.contains("said \"Approve\\u{1b}[31m it\\nnow\""),
        "{text}"
    );
}
