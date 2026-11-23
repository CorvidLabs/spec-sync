//! #922, #933 and #938 through the binary, as a person runs `change ship-status`, `change ship`
//! and `change reopen` on a workflow-v2 project.
//!
//! * #922 (REQ-cmd-change-033): an archived change is told to reopen only when its own accepted
//!   evidence is stale, never to run `change check`, and `change reopen` agrees each time; an
//!   unreadable `verification.json` is told to restore it.
//! * #933 (REQ-cmd-change-031): after a finalization that stopped between acceptance and the
//!   archive move, ship-status calls the accepted change ready and `change ship` archives it.
//! * #938: with and without a sequence-ledger freeze, ship-status's blockers, `ship_next` and
//!   lifecycle next action read the one freeze value.

use crate::helpers::{SUBSTANTIVE_PLAN, specsync};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const ID: &str = "add-an-auth-probe";

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

/// What the command printed, stdout then stderr, and whether it succeeded.
fn run(root: &Path, args: &[&str]) -> (bool, String) {
    let output = specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), printed)
}

fn run_ok(root: &Path, args: &[&str]) -> String {
    let (ok, printed) = run(root, args);
    assert!(ok, "specsync {} failed:\n{printed}", args.join(" "));
    printed
}

fn state(root: &Path) -> String {
    let (_, printed) = run(root, &["--json", "change", "status", ID]);
    let status: Value = serde_json::from_str(&printed).unwrap_or_default();
    status["summary"]["state"]
        .as_str()
        .or_else(|| status["state"].as_str())
        .unwrap_or_default()
        .to_string()
}

/// `change ship-status --json`: its one document.
fn ship_status(root: &Path) -> Value {
    let (_, printed) = run(root, &["--json", "change", "ship-status", ID]);
    serde_json::from_str(&printed)
        .unwrap_or_else(|error| panic!("ship-status prints one document ({error}): {printed}"))
}

/// A workflow-v2 project whose change over `src/auth.rs` is approved, checked with `--commit`
/// and reviewed: ready to finalize.
fn reviewed_project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("work");
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    for dir in [".specsync", "src", "specs/auth"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".specsync/config.toml"),
        "enforcement = \"strict\"\n",
    )
    .unwrap();
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
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "base"]);
    run_ok(
        &root,
        &[
            "change",
            "new",
            "Add an auth probe",
            "--kind",
            "bug-fix",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "The auth probe is documented"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
        ("constraints", "none"),
    ] {
        run_ok(&root, &["change", "answer", ID, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(ID);
    let record: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in record["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "tasks" => "# Tasks\n\n- [x] Add the probe.\n",
            "plan" => SUBSTANTIVE_PLAN,
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
    run_ok(&root, &["change", "approve", ID, "--actor", "Reviewer"]);
    fs::write(
        root.join("src/auth.rs"),
        "// Authentication module, probed.\n",
    )
    .unwrap();
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "implement"]);
    run_ok(&root, &["change", "check", ID, "--commit"]);
    run_ok(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    (temp, root)
}

/// The reviewed project, finalized; `change finalize` commits the archive itself.
fn archived_project() -> (TempDir, PathBuf) {
    let (temp, root) = reviewed_project();
    run_ok(&root, &["change", "finalize", ID]);
    if !git(&root, &["status", "--porcelain"]).is_empty() {
        git(&root, &["add", "--all"]);
        git(&root, &["commit", "-q", "-m", "archive"]);
    }
    assert_eq!(state(&root), "archived");
    (temp, root)
}

fn commit_file(root: &Path, path: &str, text: &str, message: &str) {
    fs::write(root.join(path), text).unwrap();
    git(root, &["add", path]);
    git(root, &["commit", "-q", "-m", message]);
}

fn reopen(root: &Path) -> (bool, String) {
    run(
        root,
        &[
            "change",
            "reopen",
            ID,
            "--actor",
            "Owner",
            "--reason",
            "re-verify",
        ],
    )
}

/// The `Next:` line of text `change ship-status`.
fn next_line(root: &Path) -> String {
    let (_, printed) = run(root, &["change", "ship-status", ID]);
    printed
        .lines()
        .find(|line| line.trim_start().starts_with("Next:"))
        .unwrap_or_else(|| panic!("ship-status prints a Next: line:\n{printed}"))
        .to_string()
}

/// #922: unrelated drift leaves an archived change's product stage done and reopen refuses;
/// an edit to its own input names reopen in the stage and in `Next:`, never `change check`, and
/// reopen accepts.
#[test]
fn ship_status_names_reopen_for_an_archived_change_only_when_reopen_runs() {
    let (_temp, root) = archived_project();
    commit_file(&root, "NOTES.md", "unrelated\n", "an unrelated file");
    let report = ship_status(&root);
    let product = &report["stages"][0];
    assert_eq!(product["id"], "product_tip", "{report}");
    assert_eq!(product["status"], "done", "{report}");
    assert!(
        !report.to_string().contains("specsync change reopen"),
        "{report}"
    );
    let (reopened, refused) = reopen(&root);
    assert!(
        !reopened,
        "reopen agrees: nothing of the change's own is stale"
    );
    assert!(refused.contains("delivery inputs are current"), "{refused}");

    commit_file(
        &root,
        "src/auth.rs",
        "// Authentication module, edited after the archive.\n",
        "edit the change's own input",
    );
    let reopen_command = format!("specsync change reopen {ID}");
    let report = ship_status(&root);
    let product = &report["stages"][0];
    assert_eq!(product["status"], "current", "{report}");
    assert!(
        product["action"]
            .as_str()
            .unwrap_or_default()
            .contains(&reopen_command),
        "{report}"
    );
    let ship_next = report["ship_next"].as_str().unwrap_or_default();
    assert!(
        ship_next.contains(&reopen_command) && !ship_next.contains("specsync change check"),
        "{report}"
    );
    let next = next_line(&root);
    assert!(
        next.contains(&reopen_command) && !next.contains("specsync change check"),
        "{next}"
    );
    let (reopened, printed) = reopen(&root);
    assert!(
        reopened,
        "reopen agrees: the change's own evidence is stale\n{printed}"
    );
}

/// #922: an archived change whose `verification.json` cannot be read is told to restore it, not
/// to reopen, and reopen cannot run.
#[test]
fn ship_status_names_the_restore_for_an_unreadable_archived_verification() {
    let (_temp, root) = archived_project();
    let archive = fs::read_dir(root.join(".specsync/archive/changes"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(ID))
        .expect("the archive package");
    fs::write(archive.join("verification.json"), "not json\n").unwrap();
    git(&root, &["add", "--all"]);
    git(
        &root,
        &["commit", "-q", "-m", "damage the archived verification"],
    );
    let report = ship_status(&root);
    for text in [
        report["stages"][0]["action"].as_str().unwrap_or_default(),
        report["ship_next"].as_str().unwrap_or_default(),
    ] {
        assert!(
            text.contains("verification.json") && !text.contains("specsync change reopen"),
            "{report}"
        );
    }
    assert!(!reopen(&root).0, "reopen cannot load the evidence");
}

/// A normal clone of the default branch after a squash need not have the archived change's
/// original implementation commit. The surviving archive authenticates terminal history, but
/// moving it out of the archive for reopen cannot recover the missing implementation history.
#[test]
fn ship_status_explains_reopen_is_unavailable_after_a_squash_clone() {
    let (temp, root) = archived_project();
    let archive = fs::read_dir(root.join(".specsync/archive/changes"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().ends_with(ID))
        .expect("the archive package");
    let finalization: Value =
        serde_json::from_slice(&fs::read(archive.join("finalization.json")).unwrap()).unwrap();
    let implementation = finalization["implementation_commit"].as_str().unwrap();
    let base = git(&root, &["rev-list", "--max-parents=0", "HEAD"]);
    git(&root, &["reset", "--soft", &base]);
    git(
        &root,
        &["commit", "-q", "-m", "squash the published archive"],
    );
    commit_file(&root, "NOTES.md", "unrelated\n", "an unrelated follow-up");
    commit_file(
        &root,
        "src/auth.rs",
        "// Authentication module, changed in later work.\n",
        "later work changes an input the archive delivered",
    );
    let clone = temp.path().join("fresh-clone");
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            "--no-local",
            root.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let in_clone = Command::new("git")
        .args(["cat-file", "-e", &format!("{implementation}^{{commit}}")])
        .current_dir(&clone)
        .output()
        .unwrap();
    assert!(
        !in_clone.status.success(),
        "the fresh clone must omit the implementation"
    );
    assert_eq!(state(&clone), "archived");

    let (reopened, refused) = reopen(&clone);
    assert!(!reopened, "{refused}");
    assert!(
        refused.contains("finalization implementation commit is unavailable"),
        "{refused}"
    );
    assert_eq!(
        state(&clone),
        "archived",
        "the refused reopen restores the archive"
    );

    let report = ship_status(&clone);
    for action in [
        report["stages"][0]["action"].as_str().unwrap_or_default(),
        report["ship_next"].as_str().unwrap_or_default(),
    ] {
        assert!(!action.contains("specsync change reopen"), "{report}");
        assert!(
            action.contains("finalization implementation commit is unavailable"),
            "{report}"
        );
    }
    assert!(!next_line(&clone).contains("specsync change reopen"));
}

/// The UTC date of now, `YYYY-MM-DD`, as SpecSync names an archive directory.
fn utc_today() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = (seconds / 86_400) as i64;
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// #933: a finalization that accepted the change and then stopped before the archive move,
/// because its archive destination already existed. ship-status calls the accepted change ready
/// and `change ship` archives it.
#[test]
fn change_ship_finishes_a_finalization_that_stopped_before_the_archive_move() {
    let (_temp, root) = reviewed_project();
    let blocking = root
        .join(".specsync/archive/changes")
        .join(format!("{}-{ID}", utc_today()));
    fs::create_dir_all(&blocking).unwrap();
    let (finalized, printed) = run(&root, &["change", "finalize", ID]);
    assert!(!finalized, "the archive move is blocked:\n{printed}");
    fs::remove_dir(&blocking).unwrap();
    assert_eq!(
        state(&root),
        "accepted",
        "the finalization stopped after acceptance"
    );
    assert!(
        root.join(".specsync/changes")
            .join(ID)
            .join("finalization.json")
            .is_file()
    );

    let report = ship_status(&root);
    assert_eq!(report["ready_to_finalize"], true, "{report}");
    assert!(
        report["ship_next"]
            .as_str()
            .unwrap_or_default()
            .contains(&format!("specsync change ship {ID}")),
        "{report}"
    );
    let blocked = root
        .join(".specsync/archive/changes")
        .join(format!("{}-broken-retirement", utc_today()));
    fs::create_dir(&blocked).unwrap();
    let mut sibling: Value = serde_json::from_slice(
        &fs::read(root.join(".specsync/changes").join(ID).join("state.json")).unwrap(),
    )
    .unwrap();
    sibling["id"] = Value::from("broken-retirement");
    sibling["slug"] = Value::from("broken-retirement");
    sibling["state"] = Value::from("retired");
    fs::write(
        blocked.join("state.json"),
        serde_json::to_vec(&sibling).unwrap(),
    )
    .unwrap();

    let (finalized, refusal) = run(&root, &["change", "finalize", ID]);
    assert!(
        !finalized && refusal.contains("retirement.json"),
        "{refusal}"
    );
    let report = ship_status(&root);
    assert_eq!(report["ready_to_finalize"], false, "{report}");
    assert!(
        report["blockers"].to_string().contains("retirement.json"),
        "{report}"
    );
    assert!(
        report["stages"][0]["action"]
            .as_str()
            .unwrap()
            .contains("retirement.json"),
        "{report}"
    );
    assert!(
        !report["ship_next"]
            .as_str()
            .unwrap()
            .contains(&format!("change ship {ID}")),
        "{report}"
    );
    let (shipped, refusal) = run(&root, &["change", "ship", ID]);
    assert!(!shipped && refusal.contains("retirement.json"), "{refusal}");

    fs::remove_dir_all(blocked).unwrap();
    assert_eq!(ship_status(&root)["ready_to_finalize"], true);
    run_ok(&root, &["change", "ship", ID]);
    assert_eq!(state(&root), "archived");
}

/// #938: with and without a freeze, ship-status's blockers, `ship_next` and lifecycle next action
/// agree on the one freeze value.
#[test]
fn ship_status_reads_one_sequence_ledger_freeze_for_its_whole_report() {
    let (_temp, root) = reviewed_project();
    let report = ship_status(&root);
    assert!(
        !report["blockers"].to_string().contains("sequence"),
        "{report}"
    );
    assert!(
        !report["ship_next"].to_string().contains("sequence"),
        "{report}"
    );

    let path = root.join(".specsync/change-sequence.json");
    let mut ledger: Value = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| serde_json::json!({"schema_version": 1, "sequence": 0, "id": ID}));
    ledger["acknowledged_collisions"] =
        serde_json::json!([{"sequence": 1, "ids": ["CHG-0001-first", "CHG-0001-second"]}]);
    fs::write(&path, serde_json::to_vec_pretty(&ledger).unwrap()).unwrap();
    let report = ship_status(&root);
    let freeze = report["blockers"][0]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        freeze.contains("sequence"),
        "a freeze is the first blocker: {report}"
    );
    assert_eq!(report["ship_next"], freeze.as_str(), "{report}");
    assert_eq!(report["lifecycle_next"], freeze.as_str(), "{report}");
}
