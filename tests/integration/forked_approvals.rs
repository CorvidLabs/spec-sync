//! #532 through the binary: two real clones of one origin approve the same change with different
//! actors and each commits; clone B pushes a branch and clone A merges it, keeping its own side
//! where the two conflict (`git merge -X ours`). Definition-approval validation refuses the fork,
//! naming both approvers, until one fresh `change approve` on the merged branch.

use crate::helpers::specsync;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const ID: &str = "document-the-greeter";

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

fn commit_all(root: &Path, message: &str) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", message]);
}

/// A defined, unapproved change in an origin, and two clones of it, `a` and `b`.
fn two_clones(temp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let seed = temp.path().join("seed");
    fs::create_dir_all(seed.join(".specsync")).unwrap();
    git(&seed, &["init", "-q", "-b", "main"]);
    git(&seed, &["config", "user.email", "seed@example.com"]);
    git(&seed, &["config", "user.name", "Seed"]);
    fs::write(seed.join("README.md"), "hello\n").unwrap();
    fs::write(
        seed.join(".specsync/sdd.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 2, "enabled": true, "require_change_for_meaningful_files": false,
            "meaningful_paths": [], "ignored_paths": [".specsync/"],
            "verification_commands": ["true"], "custom_artifacts": {}, "principles_file": null
        }))
        .unwrap(),
    )
    .unwrap();
    commit_all(&seed, "base");
    run_ok(
        &seed,
        &[
            "change",
            "new",
            "Document the greeter",
            "--kind",
            "documentation",
            "--path",
            "README.md",
            "--no-spec-change",
            "--rationale",
            "a reproduction of #532",
        ],
    );
    for (question, answer) in [
        ("acceptance_criteria", "the greeter is documented"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
        ("constraints", "none"),
    ] {
        run_ok(&seed, &["change", "answer", ID, question, answer]);
    }
    let workspace = seed.join(".specsync/changes").join(ID);
    for entry in fs::read_dir(&workspace).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".md") || name == "change.md" {
            continue;
        }
        let body = if name == "tasks.md" {
            "# Tasks\n\n- [x] Document it.\n".to_string()
        } else {
            format!("# {}\n\nComplete.\n", name.trim_end_matches(".md"))
        };
        fs::write(&path, body).unwrap();
    }
    commit_all(&seed, "define the change");
    let origin = temp.path().join("origin.git");
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            "--bare",
            seed.to_str().unwrap(),
            origin.to_str().unwrap(),
        ],
    );
    let clone = |name: &str| {
        let path = temp.path().join(name);
        git(
            temp.path(),
            &[
                "clone",
                "-q",
                origin.to_str().unwrap(),
                path.to_str().unwrap(),
            ],
        );
        git(
            &path,
            &["config", "user.email", &format!("{name}@example.com")],
        );
        git(&path, &["config", "user.name", &format!("clone-{name}")]);
        path
    };
    (clone("a"), clone("b"))
}

#[test]
fn approvals_forked_across_two_clones_are_refused_until_one_fresh_approval() {
    let temp = TempDir::new().unwrap();
    let (a, b) = two_clones(&temp);
    run_ok(&a, &["change", "approve", ID, "--actor", "clone-a"]);
    run_ok(&b, &["change", "approve", ID, "--actor", "clone-b"]);
    commit_all(&a, "clone A approves");
    git(&a, &["push", "-q", "origin", "main"]);
    commit_all(&b, "clone B approves");
    git(&b, &["push", "-q", "origin", "HEAD:clone-b"]);
    git(&a, &["fetch", "-q", "origin"]);
    git(
        &a,
        &[
            "merge",
            "-q",
            "-X",
            "ours",
            "origin/clone-b",
            "-m",
            "merge clone B",
        ],
    );

    let fresh = format!("specsync change approve {ID}");
    for args in [
        vec!["change", "check", ID],
        vec!["change", "audit", "--strict"],
    ] {
        let (passed, printed) = run(&a, &args);
        assert!(!passed, "{args:?} refuses the fork:\n{printed}");
        assert!(
            printed.contains("clone-a") && printed.contains("clone-b") && printed.contains(&fresh),
            "{args:?} names both approvers and the fresh approval:\n{printed}"
        );
    }
    let (_, status) = run(&a, &["--json", "change", "status", ID]);
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["summary"]["approval_valid"], false, "{status}");

    run_ok(&a, &["change", "approve", ID, "--actor", "Scope owner"]);
    commit_all(&a, "one fresh approval");
    run_ok(&a, &["change", "check", ID, "--commit"]);
    run_ok(&a, &["change", "audit", "--strict"]);
}
