// #1104, through the binary: one change whose records fail validation no longer decides another
// change's terminal evidence. tester2's S1 shape: a change is archived, then a commit holds it
// both open and archived (as a merge left one on starling) and a later commit removes the open
// copy. Another change archived before it reads `authenticated-history` in status, list and
// ship-status; the broken change still reports its own failure. Plain `change audit` exits 1 with
// that failure while the change is open and archived at once, and passes once it is only an
// archive, where `change audit --archives` exits 1 naming it. Check, review and finalize of a
// third change run as before.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

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

fn run(root: &Path, args: &[&str]) -> Output {
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
        "specsync {} failed:\n{}{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn printed(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn project() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 2,
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
    fs::write(root.join("README.md"), "Initial.\n").unwrap();
    git(&root, &["add", "--all"]);
    git(&root, &["commit", "-q", "-m", "base"]);
    (temp, root)
}

/// A documentation change over `path`, approved, checked, reviewed and finalized, then committed.
fn shipped(root: &Path, description: &str, id: &str, path: &str) {
    fs::write(root.join(path), format!("{description}.\n")).unwrap();
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
        ("constraints", "none"),
    ] {
        run_ok(root, &["change", "answer", id, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "plan" => super::helpers::SUBSTANTIVE_PLAN,
            "tasks" => "# Tasks\n\n- [x] Write it.\n",
            _ => "# Done\n\nDocument evidence.\n",
        };
        fs::write(workspace.join(format!("{name}.md")), content).unwrap();
    }
    run_ok(root, &["change", "approve", id, "--actor", "Definer"]);
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", &format!("Approve {id}")]);
    run_ok(root, &["change", "check", id, "--commit"]);
    run_ok(root, &["change", "review", id, "--reviewer", "Reviewer"]);
    run_ok(root, &["change", "finalize", id]);
    git(root, &["add", "--all"]);
    let clean = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()
        .unwrap()
        .success();
    if !clean {
        git(root, &["commit", "-q", "-m", &format!("Records of {id}")]);
    }
}

#[test]
fn another_change_s_broken_history_does_not_decide_status_list_or_ship_status() {
    let (_temp, root) = project();
    shipped(&root, "Add a guide", "add-a-guide", "GUIDE.md");
    shipped(&root, "Add notes", "add-notes", "NOTES.md");
    // A commit holds `add-notes` both open and archived, and the next drops the open copy.
    let active = ".specsync/changes/add-notes";
    let before_archive = git(
        &root,
        &[
            "log",
            "--format=%H",
            "-n",
            "1",
            "--diff-filter=D",
            "--",
            &format!("{active}/state.json"),
        ],
    );
    git(
        &root,
        &["checkout", &format!("{before_archive}~1"), "--", active],
    );
    git(&root, &["add", "--all"]);
    git(
        &root,
        &[
            "commit",
            "-q",
            "-m",
            "a merge keeps the open copy beside the archive",
        ],
    );
    // While that commit holds the change open and archived at once, plain `change audit` exits 1
    // with the change's error.
    let audit = run(&root, &["change", "audit"]);
    assert!(!audit.status.success(), "{}", printed(&audit));
    assert!(
        printed(&audit).contains("duplicates state at commit"),
        "{}",
        printed(&audit)
    );
    git(&root, &["rm", "-r", "-q", active]);
    git(&root, &["commit", "-q", "-m", "drop the open copy"]);

    let status: Value = serde_json::from_str(&run_ok(
        &root,
        &["--json", "change", "status", "add-a-guide"],
    ))
    .unwrap();
    assert_eq!(
        status["summary"]["terminal_evidence"]["validity"], "authenticated-history",
        "{status}"
    );
    let list = run(&root, &["--json", "change", "list"]);
    assert!(list.status.success(), "{}", printed(&list));
    // `change list` lists active changes, and none is active here: it succeeds, and carries no
    // other change's failure.
    let listed = printed(&list);
    assert!(!listed.contains("duplicates state at commit"), "{listed}");
    let ship_status = run(&root, &["--json", "change", "ship-status", "add-a-guide"]);
    let report: Value = serde_json::from_slice(&ship_status.stdout).unwrap();
    assert!(
        !report.to_string().contains("duplicates state at commit"),
        "ship-status of an intact archive does not carry the broken change's failure: {report}"
    );

    // The broken change itself, and the audit of archives, still report it.
    let own = run(&root, &["change", "status", "add-notes"]);
    assert!(!own.status.success(), "{}", printed(&own));
    assert!(
        printed(&own).contains("duplicates state at commit"),
        "{}",
        printed(&own)
    );
    let audit = run(&root, &["change", "audit", "--archives"]);
    assert!(!audit.status.success(), "{}", printed(&audit));
    assert!(
        printed(&audit)
            .contains("archived change `add-notes` failed workflow-version history validation"),
        "{}",
        printed(&audit)
    );
    // Plain `change audit` inspects active changes, and the broken change is only an archive now,
    // so it passes.
    let audit = run(&root, &["change", "audit"]);
    assert!(audit.status.success(), "{}", printed(&audit));

    // check, review and finalize of a third change run as they do on main.
    shipped(&root, "Add a log", "add-a-log", "LOG.md");
    let status: Value =
        serde_json::from_str(&run_ok(&root, &["--json", "change", "status", "add-a-log"])).unwrap();
    assert_eq!(
        status["summary"]["terminal_evidence"]["validity"], "authenticated-history",
        "{status}"
    );
}
