//! `specsync doctor` (REQ-cmd-change-035, REQ-cli-args-025) and its late close (REQ-change-143).
//!
//! Doctor lists every problem in the repository's change records with its fix. `--fix` applies
//! only the safe fixes. A late close is a decision: it applies only to the changes named with
//! `--close-late`, and only with a human actor and a reason. A closed-late package moves, byte for
//! byte, into `.specsync/archive/closed-late/`, which SpecSync 6.0 to 6.2 never read. The fixtures
//! rebuild the shapes found on CorvidLabs/site and Corvidinho (#833):
//! - a stale workflow-v1 `accepted` change that blocks every later archive;
//! - a hand-archived `accepted` package;
//! - a finalize that stopped halfway on a refused preflight (site #616);
//! - a never-approved draft.

use crate::helpers::{SUBSTANTIVE_PLAN, specsync};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const BLOCKER: &str = "update-the-readme";
const ACTOR: &str = "Leif";
const REASON: &str = "Merged in #232 and never archived; its evidence can no longer be checked";
const CLOSED_LATE: &str = ".specsync/archive/closed-late";

fn git(root: &Path, args: &[&str]) -> String {
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

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_ok(root: &Path, args: &[&str]) -> String {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "specsync {} failed\n{}",
        args.join(" "),
        text(&output)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn run_err(root: &Path, args: &[&str]) -> String {
    let output = run(root, args);
    assert!(
        !output.status.success(),
        "specsync {} succeeded but must refuse\n{}",
        args.join(" "),
        text(&output)
    );
    text(&output)
}

/// `doctor --fix --close-late <id>... --actor ACTOR --reason REASON`.
fn close_late(root: &Path, ids: &[&str]) -> std::process::Output {
    let mut args = vec!["doctor", "--fix"];
    for id in ids {
        args.extend_from_slice(&["--close-late", id]);
    }
    args.extend_from_slice(&["--actor", ACTOR, "--reason", REASON]);
    run(root, &args)
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

/// The findings of `--json doctor`.
fn findings(root: &Path) -> Vec<Value> {
    let output = run(root, &["--json", "doctor"]);
    let report: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("doctor JSON: {error}\n{}", text(&output)));
    report["findings"].as_array().cloned().unwrap_or_default()
}

fn finding<'a>(findings: &'a [Value], kind: &str, change: &str) -> &'a Value {
    findings
        .iter()
        .find(|finding| finding["kind"] == kind && finding["change"] == change)
        .unwrap_or_else(|| panic!("no `{kind}` finding for `{change}` in {findings:#?}"))
}

/// Every file under `root` except `.git`, by relative path.
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".git")
    {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            files.insert(relative, fs::read(entry.path()).unwrap());
        }
    }
    files
}

fn active(root: &Path, id: &str) -> PathBuf {
    root.join(".specsync/changes").join(id)
}

/// The single dated package for `id` under `base`, if there is one.
fn dated_package(base: &Path, id: &str) -> Option<PathBuf> {
    let mut found: Vec<_> = fs::read_dir(base)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(&format!("-{id}")))
        })
        .collect();
    assert!(found.len() <= 1, "several packages for {id}: {found:?}");
    found.pop()
}

fn archived(root: &Path, id: &str) -> Option<PathBuf> {
    dated_package(&root.join(".specsync/archive/changes"), id)
}

fn closed_late_package(root: &Path, id: &str) -> Option<PathBuf> {
    dated_package(&root.join(CLOSED_LATE), id)
}

/// A Git project with a bare `origin` whose default branch is `main`, and SpecSync workflow v1.
fn project(temp: &TempDir) -> PathBuf {
    let root = temp.path().join("work");
    fs::create_dir_all(root.join(".specsync")).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    write_sdd(&root, 1);
    fs::write(root.join("README.md"), "Initial.\n").unwrap();
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "base"]);
    let origin = temp.path().join("origin.git");
    git(
        temp.path(),
        &[
            "init",
            "-q",
            "--bare",
            "-b",
            "main",
            origin.to_str().unwrap(),
        ],
    );
    git(
        &root,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    publish(&root);
    root
}

fn write_sdd(root: &Path, version: u32) {
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": version,
            "enabled": true,
            "require_change_for_meaningful_files": false,
            "meaningful_paths": [],
            "ignored_paths": [],
            "verification_commands": ["true"],
            "custom_artifacts": {},
            "principles_file": null
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Push `HEAD` as origin's `main` and point `origin/HEAD` at it: what is here has merged.
fn publish(root: &Path) {
    git(
        root,
        &["push", "-q", "--force", "origin", "HEAD:refs/heads/main"],
    );
    git(root, &["fetch", "-q", "origin"]);
    git(root, &["remote", "set-head", "origin", "main"]);
}

fn commit_all(root: &Path, message: &str) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", message]);
}

fn draft(root: &Path, description: &str, id: &str, path: &str) {
    run_ok(
        root,
        &[
            "change",
            "new",
            description,
            "--kind",
            "documentation",
            "--path",
            path,
            "--no-spec-change",
            "--rationale",
            "Documentation only",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "Readers can follow the document"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run_ok(root, &["change", "answer", id, question, answer]);
    }
    if json(&active(root, id).join("state.json"))["workflow_version"] == 2 {
        run_ok(root, &["change", "answer", id, "constraints", "none"]);
    }
    let record = json(&active(root, id).join("state.json"));
    for artifact in record["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "plan" => SUBSTANTIVE_PLAN,
            "tasks" => "# Tasks\n\n- [x] Write it.\n",
            _ => "# Done\n\nDocument evidence.\n",
        };
        fs::write(active(root, id).join(format!("{name}.md")), content).unwrap();
    }
}

/// A workflow-v1 change taken through accept on the current branch.
fn v1_accepted(root: &Path, description: &str, id: &str, path: &str) {
    fs::write(root.join(path), format!("{description}.\n")).unwrap();
    draft(root, description, id, path);
    run_ok(root, &["change", "approve", id, "--actor", "Definer"]);
    run_ok(root, &["change", "start", id]);
    run_ok(root, &["change", "verify", id]);
    run_ok(root, &["change", "accept", id, "--actor", "Closer"]);
    assert_eq!(
        json(&active(root, id).join("state.json"))["state"],
        "accepted"
    );
}

/// The site's blocker: a v1 change accepted and merged, then left behind by a later edit of its
/// delivery input, so its accepted evidence is stale and every later archive refuses on it.
fn stale_blocker(root: &Path) {
    v1_accepted(root, "Update the readme", BLOCKER, "README.md");
    commit_all(root, "Accept the readme change");
    fs::write(root.join("README.md"), "Rewritten later.\n").unwrap();
    commit_all(root, "Rewrite the readme");
    publish(root);
}

fn switch_to_v2(root: &Path) {
    write_sdd(root, 2);
    commit_all(root, "Adopt workflow v2");
    publish(root);
}

/// A workflow-v2 change checked and reviewed on the current branch.
fn v2_reviewed(root: &Path, description: &str, id: &str, path: &str) {
    fs::write(root.join(path), format!("{description}.\n")).unwrap();
    draft(root, description, id, path);
    run_ok(root, &["change", "approve", id, "--actor", "Definer"]);
    commit_all(root, &format!("Approve {id}"));
    run_ok(root, &["change", "check", id, "--commit"]);
    run_ok(root, &["change", "review", id, "--reviewer", "Reviewer"]);
}

/// Site #616: a reviewed v2 change whose finalize the stale blocker refused, leaving it
/// `accepted` with a written `finalization.json` in its workspace.
fn stopped_finalize(root: &Path, id: &str) {
    v2_reviewed(root, "Add a guide", id, "GUIDE.md");
    let refused = run_err(root, &["change", "finalize", id]);
    assert!(refused.contains(BLOCKER), "finalize:\n{refused}");
    assert_eq!(
        json(&active(root, id).join("state.json"))["state"],
        "accepted"
    );
    assert!(active(root, id).join("finalization.json").is_file());
}

fn assert_late_record(root: &Path, package: &Path, id: &str, from_state: &str, from_path: &str) {
    let record = json(&package.join("late-close.json"));
    assert_eq!(record["schema_version"], 1, "{record}");
    assert_eq!(record["change_id"], id, "{record}");
    assert_eq!(record["actor"], ACTOR, "{record}");
    assert_eq!(record["reason"], REASON, "{record}");
    assert_eq!(record["from_state"], from_state, "{record}");
    assert_eq!(record["from_path"], from_path, "{record}");
    assert_eq!(record["verified"], false, "{record}");
    assert!(
        record["timestamp"].as_u64().is_some_and(|value| value > 0),
        "{record}"
    );
    assert_eq!(record["default_ref"], "origin/main", "{record}");
    assert_eq!(
        record["default_commit"],
        git(root, &["rev-parse", "origin/main"]),
        "{record}"
    );
    let added = git(
        root,
        &[
            "log",
            "-1",
            "--format=%H %cI",
            "--diff-filter=A",
            "origin/main",
            "--",
            &format!("{from_path}/state.json"),
        ],
    );
    let (merged_commit, merged_at) = added.split_once(' ').unwrap();
    assert_eq!(record["merged_commit"], merged_commit, "{record}");
    assert_eq!(record["merged_at"], merged_at, "{record}");
    assert!(
        record.get("workflow_version").is_some_and(Value::is_u64),
        "{record}"
    );
}

/// Every file of `before` keeps its bytes in `after`, and `late-close.json` is the only addition.
fn assert_bytes_kept(before: &BTreeMap<String, Vec<u8>>, after: &BTreeMap<String, Vec<u8>>) {
    for (path, bytes) in before {
        assert_eq!(after.get(path), Some(bytes), "{path} changed or vanished");
    }
    let added: Vec<_> = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .collect();
    assert_eq!(added, vec!["late-close.json"], "unexpected new files");
}

// ─── The scan (RECOVER-3, RECOVER-3.a, RECOVER-3.c) ────────────────────────

/// One change of each kind doctor names:
/// - the merged stale blocker;
/// - a merged draft;
/// - a hand-archived `accepted` package (`write-notes`);
/// - an unmerged stopped finalize;
/// - an unmerged change whose verification went stale.
fn every_kind(root: &Path) {
    stale_blocker(root);
    fs::write(root.join("FAQ.md"), "FAQ.\n").unwrap();
    draft(root, "Write a FAQ", "write-a-faq", "FAQ.md");
    // Its ID sorts after the blocker's, so the archive preflight still stops at the blocker.
    v1_accepted(root, "Write notes", "write-notes", "NOTES.md");
    let package = root
        .join(".specsync/archive/changes")
        .join("2026-01-01-write-notes");
    fs::create_dir_all(package.parent().unwrap()).unwrap();
    fs::rename(active(root, "write-notes"), &package).unwrap();
    commit_all(root, "Draft the FAQ and archive the notes change by hand");
    switch_to_v2(root);
    git(root, &["checkout", "-q", "-b", "feature"]);
    stopped_finalize(root, "add-a-guide");
    commit_all(root, "Keep the stopped finalize");
    fs::write(root.join("LOG.md"), "Log.\n").unwrap();
    draft(root, "Add a log", "add-a-log", "LOG.md");
    run_ok(
        root,
        &["change", "approve", "add-a-log", "--actor", "Definer"],
    );
    commit_all(root, "Approve the log change");
    run_ok(root, &["change", "check", "add-a-log", "--commit"]);
    fs::write(root.join("LOG.md"), "Log, edited after the check.\n").unwrap();
    commit_all(root, "Edit the log after its check");
}

#[test]
fn doctor_lists_each_problem_with_its_class_and_fix() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    every_kind(&root);
    let found = findings(&root);

    let merged = finding(&found, "merged-never-archived", BLOCKER);
    assert_eq!(merged["fix"]["class"], "decision", "{merged}");
    assert!(
        merged["fix"]["command"]
            .as_str()
            .unwrap()
            .contains(&format!("specsync doctor --fix --close-late {BLOCKER}")),
        "{merged}"
    );
    let drifted = finding(&found, "accepted-inputs-drifted", BLOCKER);
    assert_eq!(drifted["fix"]["class"], "decision", "{drifted}");
    assert!(
        drifted["reason"].as_str().unwrap().contains("README.md"),
        "{drifted}"
    );
    let draft_finding = finding(&found, "merged-never-archived", "write-a-faq");
    assert_eq!(draft_finding["fix"]["class"], "decision", "{draft_finding}");
    let hand_archived = finding(&found, "accepted-in-archive", "write-notes");
    assert_eq!(hand_archived["fix"]["class"], "decision", "{hand_archived}");
    let stopped = finding(&found, "stopped-finalize", "add-a-guide");
    assert_eq!(stopped["fix"]["class"], "safe", "{stopped}");
    assert!(
        stopped["fix"]["command"]
            .as_str()
            .unwrap()
            .contains("specsync change finalize add-a-guide"),
        "{stopped}"
    );
    let stale = finding(&found, "stale-verification", "add-a-log");
    assert_eq!(stale["fix"]["class"], "report-only", "{stale}");
    assert!(
        stale["fix"]["command"]
            .as_str()
            .unwrap()
            .contains("specsync change check add-a-log --commit"),
        "{stale}"
    );
    // Unmerged changes are not offered a late close.
    assert!(
        !found
            .iter()
            .any(|f| f["kind"] == "merged-never-archived" && f["change"] == "add-a-guide"),
        "{found:#?}"
    );

    let listing = run_ok(&root, &["doctor"]);
    for needle in [
        BLOCKER,
        "write-a-faq",
        "write-notes",
        "add-a-guide",
        "add-a-log",
    ] {
        assert!(
            listing.contains(needle),
            "text listing lacks {needle}:\n{listing}"
        );
    }
}

// RECOVER-3.d: a record doctor cannot read is a finding, and the rest is still reported.
#[test]
fn doctor_reports_what_it_cannot_check_and_everything_else() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    let broken = active(&root, "a-broken-record");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("state.json"), "{ not json").unwrap();
    let found = findings(&root);
    let unreadable = finding(&found, "could-not-check", "a-broken-record");
    assert!(
        unreadable["reason"].as_str().is_some_and(|r| !r.is_empty()),
        "{unreadable}"
    );
    finding(&found, "accepted-inputs-drifted", BLOCKER);
    finding(&found, "merged-never-archived", BLOCKER);
}

// ─── Read-only by default (RECOVER-3.b) ─────────────────────────────────────

#[test]
fn doctor_and_its_dry_run_write_nothing() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    every_kind(&root);
    let before = tree(&root);
    run_ok(&root, &["doctor"]);
    run_ok(&root, &["--json", "doctor"]);
    let dry = run_ok(
        &root,
        &[
            "doctor",
            "--fix",
            "--dry-run",
            "--close-late",
            "all",
            "--actor",
            ACTOR,
            "--reason",
            REASON,
        ],
    );
    assert!(
        dry.contains(BLOCKER) && dry.contains("write-a-faq") && dry.contains("write-notes"),
        "dry run:\n{dry}"
    );
    assert!(
        dry.contains("add-a-guide"),
        "dry run omits the safe fix:\n{dry}"
    );
    assert_eq!(tree(&root), before, "doctor or its dry run wrote");
}

// ─── Safe fixes (RECOVER-3.c, D2) ───────────────────────────────────────────

#[test]
fn doctor_fix_applies_only_the_safe_fixes() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    switch_to_v2(&root);
    git(&root, &["checkout", "-q", "-b", "feature"]);
    stopped_finalize(&root, "add-a-guide");
    // Bring the blocker's input back to its accepted bytes, so nothing blocks the stopped
    // finalize any more.
    fs::write(root.join("README.md"), "Update the readme.\n").unwrap();
    commit_all(&root, "Restore the readme");
    fs::write(root.join("LOG.md"), "Log.\n").unwrap();
    draft(&root, "Add a log", "add-a-log", "LOG.md");
    run_ok(
        &root,
        &["change", "approve", "add-a-log", "--actor", "Definer"],
    );
    commit_all(&root, "Approve the log change");
    run_ok(&root, &["change", "check", "add-a-log", "--commit"]);
    fs::write(root.join("LOG.md"), "Log, edited after the check.\n").unwrap();
    commit_all(&root, "Edit the log after its check");

    run_ok(&root, &["doctor", "--fix"]);

    let finished = archived(&root, "add-a-guide").expect("the stopped finalize did not resume");
    assert_eq!(json(&finished.join("state.json"))["state"], "archived");
    // The blocker's evidence is current again, so archiving the accepted v1 change is a safe
    // fix; nothing is closed late, because no decision was given.
    let blocker = archived(&root, BLOCKER).expect("the current v1 change was not archived");
    assert_eq!(json(&blocker.join("state.json"))["state"], "archived");
    assert!(closed_late_package(&root, BLOCKER).is_none());
    // Stale verification is report-only: its record is untouched and still reported.
    assert_eq!(
        json(&active(&root, "add-a-log").join("state.json"))["state"],
        "verifying"
    );
    finding(&findings(&root), "stale-verification", "add-a-log");
}

#[test]
fn doctor_fix_backfills_a_5_0_ledger_once() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let workspace = root.join(".specsync/changes/adopt-trust");
    fs::create_dir_all(&workspace).unwrap();
    let ledger = serde_json::json!({
        "approvals": [],
        "reopenings": [{
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
        }]
    });
    let approvals = workspace.join("approvals.json");
    fs::write(
        &approvals,
        format!("{}\n", serde_json::to_string_pretty(&ledger).unwrap()),
    )
    .unwrap();
    fs::write(
        workspace.join("verification.json"),
        "{\n  \"timestamp\": 200,\n  \"commit\": null,\n  \"contract_digest\": \"ccc\",\n  \"workspace_digest\": \"www\",\n  \"acceptance_input_digest\": \"current-digest-bbb\",\n  \"passed\": true,\n  \"commands\": [],\n  \"requirement_ids\": []\n}\n",
    )
    .unwrap();

    let found = findings(root);
    let other = finding(&found, "other-version-record", "adopt-trust");
    assert_eq!(other["fix"]["class"], "safe", "{other}");
    assert!(
        other["fix"]["command"]
            .as_str()
            .unwrap()
            .contains("specsync migrate 5.0"),
        "{other}"
    );
    run_ok(root, &["doctor", "--fix"]);
    let repaired = json(&approvals);
    assert_eq!(
        repaired["reopenings"][0]["stale_acceptance_input_digest"],
        "stale-digest-aaa"
    );
    let once = fs::read(&approvals).unwrap();
    run_ok(root, &["doctor", "--fix"]);
    assert_eq!(
        fs::read(&approvals).unwrap(),
        once,
        "a second fix rewrote the ledger"
    );
}

// ─── Late close (SHIP-12, REQ-change-143) ───────────────────────────────────

// SHIP-12, SHIP-12.a, SHIP-12.c: a merged workflow-v2 change closes late in one step.
#[test]
fn a_merged_change_closes_late_byte_for_byte_as_never_verified() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    switch_to_v2(&root);
    let id = "add-a-guide";
    fs::write(root.join("GUIDE.md"), "Guide.\n").unwrap();
    draft(&root, "Add a guide", id, "GUIDE.md");
    run_ok(&root, &["change", "approve", id, "--actor", "Definer"]);
    commit_all(&root, "Approve the guide");
    run_ok(&root, &["change", "check", id, "--commit"]);
    publish(&root);
    let before = tree(&active(&root, id));

    let output = close_late(&root, &[id]);
    assert!(output.status.success(), "{}", text(&output));

    assert!(!active(&root, id).exists(), "the workspace is still open");
    assert!(
        archived(&root, id).is_none(),
        "it was archived as if checked"
    );
    let package = closed_late_package(&root, id).expect("no closed-late package");
    assert_late_record(
        &root,
        &package,
        id,
        "verifying",
        &format!(".specsync/changes/{id}"),
    );
    assert_eq!(json(&package.join("state.json"))["state"], "verifying");
    assert_bytes_kept(&before, &tree(&package));

    let status = text(&run(&root, &["change", "status", id]));
    for needle in [
        "closed late",
        "never re-checked",
        "not verified",
        ACTOR,
        REASON,
    ] {
        assert!(
            status.contains(needle),
            "status lacks `{needle}`:\n{status}"
        );
    }
    let status: Value =
        serde_json::from_str(&run_ok(&root, &["--json", "change", "status", id])).unwrap();
    assert_eq!(status["closed_late"]["verified"], false, "{status}");
    assert_eq!(status["closed_late"]["actor"], ACTOR, "{status}");
    let ship = text(&run(&root, &["change", "ship-status", id]));
    assert!(ship.contains("closed late"), "ship-status:\n{ship}");
}

// SHIP-12.c: closing late takes none of the delivery-evidence reads that fail on such a change.
#[test]
fn a_late_close_reads_no_delivery_input() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    // Any read of the delivery input now refuses, as the bounded read did for site CHG-0026.
    git(&root, &["update-index", "--assume-unchanged", "README.md"]);
    let reopen = run_err(
        &root,
        &[
            "change", "reopen", BLOCKER, "--actor", ACTOR, "--reason", REASON,
        ],
    );
    assert!(reopen.contains("assume-unchanged"), "reopen:\n{reopen}");

    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));
    let package = closed_late_package(&root, BLOCKER).expect("no closed-late package");
    assert_late_record(
        &root,
        &package,
        BLOCKER,
        "accepted",
        &format!(".specsync/changes/{BLOCKER}"),
    );
}

// #833's Corvidinho shape: an `accepted` package moved into the archive by hand.
#[test]
fn an_accepted_package_in_the_archive_moves_out_of_it() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    let from = format!(".specsync/archive/changes/2026-01-01-{BLOCKER}");
    fs::create_dir_all(root.join(".specsync/archive/changes")).unwrap();
    fs::rename(active(&root, BLOCKER), root.join(&from)).unwrap();
    commit_all(&root, "Archive the readme change by hand");
    publish(&root);
    let before = tree(&root.join(&from));

    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));

    assert!(
        !root.join(&from).exists(),
        "the accepted package is still in the archive"
    );
    let package = closed_late_package(&root, BLOCKER).expect("no closed-late package");
    assert_late_record(&root, &package, BLOCKER, "accepted", &from);
    assert_bytes_kept(&before, &tree(&package));
}

// SHIP-12, SHIP-12.b: the stale change no longer stands in a later change's way.
#[test]
fn closing_the_blocker_late_lets_a_later_archive_pass() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    v1_accepted(&root, "Add notes", "add-notes", "NOTES.md");
    let refused = run_err(&root, &["change", "archive", "add-notes"]);
    assert!(
        refused.contains("archive post-move preflight would invalidate")
            && refused.contains(BLOCKER),
        "archive:\n{refused}"
    );

    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));

    // With the blocker closed late, the archive it refused goes through: `--fix` archives the
    // current v1 change as a safe fix in the same call.
    let notes = archived(&root, "add-notes").expect("the archive the blocker refused did not pass");
    assert_eq!(json(&notes.join("state.json"))["state"], "archived");
}

// SHIP-12.e and site #616: an unmerged stopped finalize is refused and left as it was; once its
// blocker is closed late, its own finalize resumes from that state.
#[test]
fn an_unmerged_stopped_finalize_is_refused_and_resumes_once_its_blocker_closes_late() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    switch_to_v2(&root);
    git(&root, &["checkout", "-q", "-b", "feature"]);
    let id = "add-a-guide";
    stopped_finalize(&root, id);

    let before = tree(&root);
    let output = close_late(&root, &[id]);
    let refusal = text(&output);
    assert!(
        !output.status.success(),
        "closed an unmerged change:\n{refusal}"
    );
    assert!(
        refusal.contains(id) && refusal.contains("origin/main") && refusal.contains("not merged"),
        "doctor:\n{refusal}"
    );
    assert_eq!(tree(&root), before, "a refused late close wrote");

    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));
    git(
        &root,
        &[
            "add",
            "--all",
            "--",
            CLOSED_LATE,
            &format!(".specsync/changes/{BLOCKER}"),
        ],
    );
    git(
        &root,
        &["commit", "-q", "-m", "Close the readme change late"],
    );

    run_ok(&root, &["change", "finalize", id]);
    let finished = archived(&root, id).expect("finalize did not archive the stopped change");
    assert_eq!(json(&finished.join("state.json"))["state"], "archived");
}

// SHIP-12.d, SHIP-12.e: every refusal writes nothing.
#[test]
fn late_close_refusals_write_nothing() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);

    let refuse = |args: &[&str], needle: &str| {
        let before = tree(&root);
        let refusal = run_err(&root, args);
        assert!(refusal.contains(needle), "expected `{needle}`:\n{refusal}");
        assert_eq!(tree(&root), before, "refused `{}` wrote", args.join(" "));
    };
    fn late(id: &str, actor: &str, reason: &str) -> Vec<String> {
        [
            "doctor",
            "--fix",
            "--close-late",
            id,
            "--actor",
            actor,
            "--reason",
            reason,
        ]
        .iter()
        .map(|argument| (*argument).to_string())
        .collect()
    }
    fn as_refs(args: &[String]) -> Vec<&str> {
        args.iter().map(String::as_str).collect()
    }
    refuse(&as_refs(&late(BLOCKER, " ", REASON)), "--actor");
    refuse(&as_refs(&late(BLOCKER, ACTOR, " ")), "--reason");

    // Not on the remote default branch yet.
    v1_accepted(&root, "Add notes", "add-notes", "NOTES.md");
    refuse(&as_refs(&late("add-notes", ACTOR, REASON)), "not merged");

    // Closed late once already.
    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));
    commit_all(&root, "Close the readme change late");
    publish(&root);
    refuse(&as_refs(&late(BLOCKER, ACTOR, REASON)), "closed late");

    // Archived through the normal path: the late close above unblocked it, and `--fix` archived
    // the current v1 change as a safe fix in the same call.
    assert!(
        archived(&root, "add-notes").is_some(),
        "add-notes was not archived"
    );
    // That archive is already in the late-close commit above, so it has merged.
    refuse(&as_refs(&late("add-notes", ACTOR, REASON)), "archived");

    // No remote default to prove the merge.
    git(&root, &["remote", "remove", "origin"]);
    v1_accepted(&root, "Add a log", "add-a-log", "LOG.md");
    commit_all(&root, "Accept the log change");
    refuse(
        &as_refs(&late("add-a-log", ACTOR, REASON)),
        "remote default",
    );
}

// SHIP-12.b: SpecSync never counts a closed-late change as checked.
#[test]
fn a_closed_late_change_never_counts_as_verified() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    let output = close_late(&root, &[BLOCKER]);
    assert!(output.status.success(), "{}", text(&output));

    for args in [
        vec![
            "change", "reopen", BLOCKER, "--actor", ACTOR, "--reason", REASON,
        ],
        vec!["change", "review", BLOCKER, "--reviewer", ACTOR],
        vec!["change", "check", BLOCKER],
        vec!["change", "accept", BLOCKER, "--actor", ACTOR],
        vec!["change", "archive", BLOCKER],
        vec!["change", "finalize", BLOCKER],
    ] {
        let before = tree(&root);
        let refusal = run_err(&root, &args);
        assert!(
            refusal.contains("closed late"),
            "{}:\n{refusal}",
            args.join(" ")
        );
        assert_eq!(tree(&root), before, "{} wrote", args.join(" "));
    }
    let refusal = run_err(
        &root,
        &[
            "change",
            "new",
            "Update the readme",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "again",
        ],
    );
    assert!(refusal.contains("closed late"), "change new:\n{refusal}");
    let audit = run_ok(&root, &["change", "audit"]);
    assert!(audit.contains("0 active"), "audit:\n{audit}");
}

// SHIP-12.f and D2: several at once, all or nothing; a draft says it was never approved.
#[test]
fn late_close_takes_several_changes_all_or_nothing() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    stale_blocker(&root);
    let never_approved = "write-a-faq";
    fs::write(root.join("FAQ.md"), "FAQ.\n").unwrap();
    draft(&root, "Write a FAQ", never_approved, "FAQ.md");
    commit_all(&root, "Draft the FAQ change");
    publish(&root);
    let unmerged = "add-notes";
    v1_accepted(&root, "Add notes", unmerged, "NOTES.md");

    let before = tree(&root);
    let output = close_late(&root, &[BLOCKER, never_approved, unmerged]);
    let refusal = text(&output);
    assert!(
        !output.status.success(),
        "closed an unmerged change:\n{refusal}"
    );
    assert!(
        refusal.contains(unmerged),
        "the refusal does not name it:\n{refusal}"
    );
    assert_eq!(tree(&root), before, "a refused batch wrote");

    let output = close_late(&root, &[BLOCKER, never_approved]);
    assert!(output.status.success(), "{}", text(&output));
    let blocker = closed_late_package(&root, BLOCKER).expect("blocker not closed late");
    assert_late_record(
        &root,
        &blocker,
        BLOCKER,
        "accepted",
        &format!(".specsync/changes/{BLOCKER}"),
    );
    let faq = closed_late_package(&root, never_approved).expect("draft not closed late");
    assert_late_record(
        &root,
        &faq,
        never_approved,
        "draft",
        &format!(".specsync/changes/{never_approved}"),
    );
    let status = text(&run(&root, &["change", "status", never_approved]));
    assert!(status.contains("never approved"), "status:\n{status}");
    assert!(
        closed_late_package(&root, unmerged).is_none(),
        "the unmerged change was closed late"
    );
}

#[test]
fn close_late_all_closes_every_eligible_change() {
    let temp = TempDir::new().unwrap();
    let root = project(&temp);
    every_kind(&root);
    let output = run(
        &root,
        &[
            "doctor",
            "--fix",
            "--close-late",
            "all",
            "--actor",
            ACTOR,
            "--reason",
            REASON,
        ],
    );
    assert!(output.status.success(), "{}", text(&output));
    for id in [BLOCKER, "write-a-faq", "write-notes"] {
        assert!(
            closed_late_package(&root, id).is_some(),
            "{id} not closed late"
        );
    }
    // Unmerged changes are never closed late; the stopped finalize resumes as a safe fix.
    assert!(closed_late_package(&root, "add-a-guide").is_none());
    assert!(closed_late_package(&root, "add-a-log").is_none());
    assert!(
        archived(&root, "add-a-guide").is_some(),
        "the stopped finalize did not resume"
    );
}
