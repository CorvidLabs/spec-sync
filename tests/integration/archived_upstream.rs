//! Audit D1 (#694, #690). The auditor's squash-before-finalize steps, rebuilt
//! without the audit fixture. A change is approved and checked on a feature
//! branch, then `main` is reset to the base and the feature is squash-merged.
//!
//! * When the remote default already holds an archive of that change, review,
//!   status, ship-status and finalize must say so, name the reference and the
//!   archive commit, and refuse before acceptance (REQ-change-129,
//!   REQ-cmd-change-029).
//! * Without an upstream archive the same squash still reviews and finalizes
//!   (#690 guard).

use crate::helpers::{SUBSTANTIVE_PLAN, specsync};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    specsync()
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn run_ok(root: &Path, args: &[&str]) -> String {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "specsync {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn state(root: &Path) -> String {
    let path = root.join(".specsync/changes").join(ID).join("state.json");
    let record: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    record["state"].as_str().unwrap().to_string()
}

/// A workflow-v2 project with one change approved and checked on `feature`.
/// Returns the work tree, the base commit and the verified feature commit.
fn verified_feature(temp: &TempDir) -> (PathBuf, String, String) {
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
    let base = git(&root, &["rev-parse", "HEAD"]);

    git(&root, &["checkout", "-q", "-b", "feature"]);
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
        ("public_contract", "yes"),
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
    let verified = git(&root, &["rev-parse", "HEAD"]);
    assert_eq!(state(&root), "verifying");
    (root, base, verified)
}

/// `main` reset to `base` with the verified feature squash-merged on top.
fn squash_onto_base(root: &Path, base: &str, verified: &str) {
    git(root, &["checkout", "-q", "-B", "main", base]);
    git(root, &["merge", "-q", "--squash", verified]);
    git(
        root,
        &[
            "commit",
            "-q",
            "-m",
            "Squash implementation before native finalize",
        ],
    );
}

// D1 as the auditor found it: origin/main finalized and archived the change,
// and this clone squash-merged the unarchived feature onto the base.
#[test]
fn a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance() {
    let temp = TempDir::new().unwrap();
    let (root, base, verified) = verified_feature(&temp);

    // Finalize on the feature branch and publish it as origin's default branch.
    run_ok(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    run_ok(&root, &["change", "finalize", ID]);
    let archived = git(&root, &["rev-parse", "HEAD"]);
    assert!(
        git(&root, &["log", "-1", "--format=%s"]).contains("archive"),
        "finalize did not commit an archive"
    );
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
    git(
        &root,
        &[
            "push",
            "-q",
            "origin",
            &format!("{archived}:refs/heads/main"),
        ],
    );
    git(&root, &["fetch", "-q", "origin"]);
    git(&root, &["remote", "set-head", "origin", "main"]);

    squash_onto_base(&root, &base, &verified);
    assert_eq!(state(&root), "verifying");
    let named = |text: &str| {
        text.contains("already archived")
            && text.contains("origin/main")
            && text.contains(&archived[..12])
            && text.contains("rebase this branch onto `origin/main`")
    };

    let review = run(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    let review_text = format!(
        "{}{}",
        String::from_utf8_lossy(&review.stdout),
        String::from_utf8_lossy(&review.stderr)
    );
    assert!(!review.status.success(), "review accepted:\n{review_text}");
    assert!(
        named(&review_text),
        "review did not name the archive:\n{review_text}"
    );
    assert_eq!(
        state(&root),
        "verifying",
        "review moved the change before refusing"
    );

    let status: Value =
        serde_json::from_str(&run_ok(&root, &["--json", "change", "status", ID])).unwrap();
    assert_ne!(
        status["summary"]["handoff"]["readiness"], "safe",
        "{status}"
    );
    let next = status["summary"]["next_action"]
        .as_str()
        .unwrap_or_default();
    assert!(named(next), "status next action: {next}");
    assert!(
        !next.contains("finalize") && !next.contains("change ship"),
        "{next}"
    );

    let ship = run(&root, &["change", "ship-status", ID]);
    let ship_text = String::from_utf8_lossy(&ship.stdout).into_owned();
    assert!(named(&ship_text), "ship-status:\n{ship_text}");

    let finalize = run(&root, &["change", "finalize", ID]);
    let finalize_text = format!(
        "{}{}",
        String::from_utf8_lossy(&finalize.stdout),
        String::from_utf8_lossy(&finalize.stderr)
    );
    assert!(
        !finalize.status.success(),
        "finalize archived twice:\n{finalize_text}"
    );
    assert!(named(&finalize_text), "finalize:\n{finalize_text}");
    assert!(
        !root.join(".specsync/archive").exists(),
        "finalize wrote a second archive"
    );

    // The named next step must actually recover (review thread on #889): drop the commit that
    // added the unarchived copy the message names, and the change reads as archived.
    let copy = format!(".specsync/changes/{ID}");
    assert!(
        review_text.contains(&format!("`{copy}/`")),
        "the refusal does not name the unarchived copy to delete:\n{review_text}"
    );
    // The branch holds nothing but the squash, so dropping that commit is a reset onto the
    // remote default. Merging, or deleting the copy in a new commit, fails history integrity.
    git(&root, &["reset", "-q", "--hard", "origin/main"]);
    let status: Value =
        serde_json::from_str(&run_ok(&root, &["--json", "change", "status", ID])).unwrap();
    assert_eq!(status["summary"]["state"], "archived", "{status}");
}

// #690 guard: with no upstream archive the same squash still closes normally.
#[test]
fn a_squash_before_finalize_without_an_upstream_archive_still_reviews_and_finalizes() {
    let temp = TempDir::new().unwrap();
    let (root, base, verified) = verified_feature(&temp);
    squash_onto_base(&root, &base, &verified);
    run_ok(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    run_ok(&root, &["change", "finalize", ID]);
    let archive = root.join(".specsync/archive/changes");
    let packages: Vec<_> = fs::read_dir(&archive)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        packages.iter().any(|name| name.ends_with(ID)),
        "no archive for {ID}: {packages:?}"
    );
}

// #946, SHIP-3.c. When the remote default's archived state records cannot be read, the tool
// cannot tell whether this change is already archived there. Review and finalize stop, say how
// to find out, and write no second archive.
#[test]
fn review_and_finalize_stop_when_the_upstream_archive_cannot_be_read() {
    let temp = TempDir::new().unwrap();
    let (root, base, verified) = verified_feature(&temp);
    run_ok(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    run_ok(&root, &["change", "finalize", ID]);
    let archived = git(&root, &["rev-parse", "HEAD"]);
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
    git(
        &root,
        &[
            "push",
            "-q",
            "origin",
            &format!("{archived}:refs/heads/main"),
        ],
    );
    git(&root, &["fetch", "-q", "origin"]);
    git(&root, &["remote", "set-head", "origin", "main"]);
    let package = git(
        &root,
        &[
            "ls-tree",
            "--name-only",
            "origin/main",
            "--",
            ".specsync/archive/changes/",
        ],
    );
    let blob = git(
        &root,
        &["rev-parse", &format!("origin/main:{package}/state.json")],
    );
    squash_onto_base(&root, &base, &verified);

    // This clone lost the archived record's blob, so the batched read of the remote default's
    // archive fails.
    fs::remove_file(root.join(".git/objects").join(&blob[..2]).join(&blob[2..])).unwrap();
    let cannot_tell = |text: &str| text.contains("cannot tell") && text.contains("origin/main");

    let review = run(&root, &["change", "review", ID, "--reviewer", "Reviewer"]);
    let review_text = format!(
        "{}{}",
        String::from_utf8_lossy(&review.stdout),
        String::from_utf8_lossy(&review.stderr)
    );
    assert!(!review.status.success(), "review accepted:\n{review_text}");
    assert!(cannot_tell(&review_text), "review:\n{review_text}");
    assert_eq!(state(&root), "verifying", "review moved the change");

    let finalize = run(&root, &["change", "finalize", ID]);
    let finalize_text = format!(
        "{}{}",
        String::from_utf8_lossy(&finalize.stdout),
        String::from_utf8_lossy(&finalize.stderr)
    );
    assert!(
        !finalize.status.success(),
        "finalize archived:\n{finalize_text}"
    );
    assert!(cannot_tell(&finalize_text), "finalize:\n{finalize_text}");
    assert!(
        !root.join(".specsync/archive").exists(),
        "finalize wrote a second archive"
    );
}
