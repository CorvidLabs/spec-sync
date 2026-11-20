// The Git-environments batch through the binary: a repository owned by another user that the
// user's Git trusts (#909, REQ-change-171), a partial clone whose promisor remote answers only
// with the user's configuration (#1077, REQ-change-172), and a shallow clone (#1072,
// REQ-change-173, REQ-cmd-change-044). Every shape is made by the test itself, so nothing
// depends on what a given Git does by default.

use super::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const BASE_REQUIREMENTS: &str = "---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n";

const ADDED_DELTA: &str = "## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL log every sign-in.\n\nAcceptance Criteria\n- Every sign-in is logged.\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// `specsync --root <root> <args>` with `env` added to the harness environment.
fn invoke(root: &Path, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = specsync();
    for (key, value) in env {
        command.env(key, value);
    }
    command
        .args(["--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}

fn run(root: &Path, env: &[(&str, &str)], args: &[&str]) -> String {
    let output = invoke(root, env, args);
    assert!(
        output.status.success(),
        "specsync {} failed:\n{}",
        args.join(" "),
        combined(&output)
    );
    combined(&output)
}

/// An adopted project in `root`: a first commit before SpecSync, then an approved change that
/// adds REQ-auth-002, checked and committed, then two more commits. Returns the change id.
fn adopted_project(root: &Path) -> String {
    fs::create_dir_all(root).unwrap();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "core.autocrlf", "false"]);
    write(root, "README.md", "# Project\n");
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", "before SpecSync"]);
    write(root, "src/auth.rs", "// Authentication module.\n");
    write(
        root,
        "specs/auth/auth.spec.md",
        "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    );
    write(root, "specs/auth/requirements.md", BASE_REQUIREMENTS);
    write(root, ".gitignore", ".specsync/hashes.json\n");
    write(
        root,
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
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", "base"]);
    let created: Value = serde_json::from_str(&run(
        root,
        &[],
        &[
            "--json",
            "change",
            "new",
            "Log every sign-in",
            "--spec",
            "auth",
            "--path",
            "src/auth.rs",
        ],
    ))
    .unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", "Every sign-in is logged"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        run(root, &[], &["change", "answer", &id, question, answer]);
    }
    let workspace = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    for artifact in state["selected_artifacts"].as_array().unwrap() {
        let name = artifact.as_str().unwrap();
        let content = match name {
            "tasks" => "# Tasks\n\n- [x] Log sign-ins.\n".to_string(),
            "plan" => super::helpers::SUBSTANTIVE_PLAN.to_string(),
            "testing" => "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-002 | SpecSync structural check of the module |\n".to_string(),
            _ => "# Complete\n\nReviewed.\n".to_string(),
        };
        fs::write(workspace.join(format!("{name}.md")), content).unwrap();
    }
    fs::write(workspace.join("deltas/auth.md"), ADDED_DELTA).unwrap();
    run(
        root,
        &[],
        &["change", "approve", &id, "--actor", "Scope owner"],
    );
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", "approve"]);
    run(root, &[], &["change", "check", &id, "--commit"]);
    for note in ["one", "two"] {
        write(root, "NOTES.md", &format!("# Notes\n\n{note}\n"));
        git(root, &["add", "--all"]);
        git(root, &["commit", "-q", "-m", note]);
    }
    id
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

// ---- #1072 (REQ-change-173, REQ-cmd-change-044) ----

/// REQ-change-173: in depth-1 and depth-2 clones of an adopted project, `change new` refuses
/// before it writes, naming the shallow clone and `git fetch --unshallow origin`, and `change
/// status`, `check` and `audit` name the same fetch; after it they behave as in a full clone.
#[test]
fn a_shallow_clone_names_the_unshallow_fetch_and_change_new_writes_nothing() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let id = adopted_project(&source);
    for depth in ["1", "2"] {
        let clone = temp.path().join(format!("depth-{depth}"));
        git(
            temp.path(),
            &[
                "clone",
                "-q",
                "--depth",
                depth,
                file_url(&source).as_str(),
                clone.to_str().unwrap(),
            ],
        );
        let before = fs::read_dir(clone.join(".specsync/changes"))
            .unwrap()
            .count();

        let created = invoke(
            &clone,
            &[],
            &[
                "change",
                "new",
                "Log every sign-out",
                "--spec",
                "auth",
                "--path",
                "src/auth.rs",
            ],
        );
        let text = combined(&created);
        assert!(!created.status.success(), "depth {depth}: {text}");
        assert!(
            text.contains("this clone is shallow")
                && text.contains("`git fetch --unshallow origin`")
                && text.contains("no change was created"),
            "depth {depth}: {text}"
        );
        assert_eq!(
            fs::read_dir(clone.join(".specsync/changes"))
                .unwrap()
                .count(),
            before,
            "depth {depth}"
        );
        for args in [
            vec!["change", "status", id.as_str()],
            vec!["change", "check", id.as_str()],
            vec!["change", "audit"],
        ] {
            let refused = invoke(&clone, &[], &args);
            let text = combined(&refused);
            assert!(!refused.status.success(), "depth {depth} {args:?}: {text}");
            assert!(
                text.contains("this clone is shallow")
                    && text.contains("`git fetch --unshallow origin`"),
                "depth {depth} {args:?}: {text}"
            );
        }

        git(&clone, &["fetch", "-q", "--unshallow", "origin"]);
        run(&clone, &[], &["change", "status", &id]);
        run(&clone, &[], &["change", "audit"]);
    }
}

/// REQ-change-173: in a full clone, a baseline whose cutoff is not an ancestor keeps today's
/// message, with no word of a shallow clone.
#[test]
fn a_full_clone_keeps_the_baseline_message() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let id = adopted_project(&root);
    git(&root, &["checkout", "-q", "--orphan", "elsewhere"]);
    git(&root, &["commit", "-q", "--allow-empty", "-m", "unrelated"]);
    let unrelated = git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    git(&root, &["checkout", "-q", "-f", "main"]);
    let baseline = root.join(".specsync/workflow-v2-baseline.json");
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&baseline).unwrap()).unwrap();
    value["cutoff_commit"] = Value::String(unrelated);
    fs::write(&baseline, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    git(&root, &["commit", "-q", "-am", "a cutoff from nowhere"]);

    let refused = invoke(&root, &[], &["change", "status", &id]);
    let text = combined(&refused);

    assert!(!refused.status.success(), "{text}");
    assert!(
        text.contains("workflow-v2 baseline") && !text.contains("shallow"),
        "{text}"
    );
}

/// REQ-cmd-change-044: text `change status` in a shallow clone that holds the baseline's history
/// gives the shallow refusal instead of the correction-ledger text; a full clone keeps the
/// ledger's own text.
#[test]
fn text_status_in_a_shallow_clone_names_the_fetch_instead_of_the_ledger_text() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let id = adopted_project(&source);
    write(
        &source,
        &format!(".specsync/changes/{id}/corrections.json"),
        "{ malformed correction ledger\n",
    );
    git(&source, &["add", "--all"]);
    git(&source, &["commit", "-q", "-m", "a damaged ledger"]);
    let full = invoke(&source, &[], &["change", "status", &id]);
    assert!(
        combined(&full).contains("correction ledger integrity is invalid"),
        "{}",
        combined(&full)
    );

    // Deep enough for the baseline (its introduction and the parent before it), not for the first
    // commit.
    let commits = git(&source, &["rev-list", "--count", "HEAD"])
        .trim()
        .parse::<usize>()
        .unwrap();
    let clone = temp.path().join("clone");
    let depth = (commits - 1).to_string();
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            "--depth",
            depth.as_str(),
            file_url(&source).as_str(),
            clone.to_str().unwrap(),
        ],
    );
    assert_eq!(
        git(&clone, &["rev-parse", "--is-shallow-repository"]).trim(),
        "true"
    );

    let refused = invoke(&clone, &[], &["change", "status", &id]);
    let text = combined(&refused);

    assert!(!refused.status.success(), "{text}");
    assert!(
        text.contains("this clone is shallow")
            && text.contains("`git fetch --unshallow origin`")
            && !text.contains("correction ledger integrity is invalid"),
        "{text}"
    );
}

// ---- #909 (REQ-change-171) ----

/// The user's global configuration, holding `safe.directory` entries.
fn user_config(dir: &Path, safe: &[&str]) -> PathBuf {
    let path = dir.join("user.gitconfig");
    let mut text = String::new();
    for entry in safe {
        text.push_str(&format!("[safe]\n\tdirectory = {entry}\n"));
    }
    fs::write(&path, text).unwrap();
    path
}

/// REQ-change-171, with Git's own `GIT_TEST_ASSUME_DIFFERENT_OWNER` standing in for another
/// owner: a repository the user's global file trusts passes `change status`, `change audit` and
/// `change check`, also from a linked worktree and with `safe.directory = *`; without the entry
/// it is refused, naming dubious ownership, the path and the command. The private file is gone
/// when the command ends, and nothing is made for a repository Git does not refuse.
#[test]
fn a_repository_the_user_trusts_passes_and_one_it_does_not_names_the_command() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("project");
    let id = adopted_project(&root);
    let top = git(&root, &["rev-parse", "--show-toplevel"])
        .trim()
        .to_string();
    let tmpdir = temp.path().join("tmp");
    fs::create_dir_all(&tmpdir).unwrap();
    let tmp = tmpdir.to_str().unwrap();

    // Owned by the user: no file is made.
    run(&root, &[("TMPDIR", tmp)], &["change", "status", &id]);
    assert_eq!(fs::read_dir(&tmpdir).unwrap().count(), 0);

    for safe in [top.as_str(), "*"] {
        let config = user_config(temp.path(), &[safe]);
        let env = [
            ("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1"),
            ("GIT_CONFIG_GLOBAL", config.to_str().unwrap()),
            ("TMPDIR", tmp),
        ];
        run(&root, &env, &["change", "status", &id]);
        run(&root, &env, &["change", "audit"]);
        run(&root, &env, &["change", "check", &id]);
        assert_eq!(
            fs::read_dir(&tmpdir).unwrap().count(),
            0,
            "safe.directory = {safe}"
        );
    }

    let worktree = temp.path().join("worktree");
    git(
        &root,
        &["worktree", "add", "-q", worktree.to_str().unwrap()],
    );
    let worktree_top = git(&worktree, &["rev-parse", "--show-toplevel"])
        .trim()
        .to_string();
    let config = user_config(temp.path(), &[worktree_top.as_str()]);
    run(
        &worktree,
        &[
            ("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1"),
            ("GIT_CONFIG_GLOBAL", config.to_str().unwrap()),
            ("TMPDIR", tmp),
        ],
        &["change", "status", &id],
    );

    let untrusted = user_config(temp.path(), &[]);
    let refused = invoke(
        &root,
        &[
            ("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1"),
            ("GIT_CONFIG_GLOBAL", untrusted.to_str().unwrap()),
            ("TMPDIR", tmp),
        ],
        &["change", "status", &id],
    );
    let text = combined(&refused);
    assert!(!refused.status.success(), "{text}");
    assert!(
        text.contains("detected dubious ownership")
            && text.contains("owned by another user")
            && text.contains("`git config --global --add safe.directory ")
            && !text.contains("unable to inspect changed paths")
            && !text.contains("Git repository detection failed"),
        "{text}"
    );
    assert_eq!(fs::read_dir(&tmpdir).unwrap().count(), 0);
}

// ---- #1077 (REQ-change-172) ----

/// A blobless clone of `source` at `clone`, whose remote is `https://example.invalid/project`:
/// only the user's global `url.<x>.insteadOf` in `config` resolves it, as only the user's
/// credential helper reaches a private HTTPS remote.
fn blobless_clone(temp: &Path, source: &Path, clone: &Path) -> PathBuf {
    git(source, &["config", "uploadpack.allowFilter", "true"]);
    git(source, &["config", "uploadpack.allowAnySHA1InWant", "true"]);
    let config = temp.join("user-remote.gitconfig");
    fs::write(
        &config,
        format!(
            "[url \"{}\"]\n\tinsteadOf = https://example.invalid/project\n",
            file_url(source)
        ),
    )
    .unwrap();
    let output = Command::new("git")
        .env("GIT_CONFIG_GLOBAL", &config)
        .args([
            "clone",
            "-q",
            "--filter=blob:none",
            "https://example.invalid/project",
            clone.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(clone, &["config", "user.email", "test@example.com"]);
    git(clone, &["config", "user.name", "Test"]);
    config
}

/// The history objects under `.specsync` and `specs` the clone does not hold, listed without
/// fetching.
fn missing_objects(clone: &Path) -> usize {
    let output = Command::new("git")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "rev-list",
            "--objects",
            "--missing=print",
            "HEAD",
            "--",
            ".specsync",
            "specs",
        ])
        .current_dir(clone)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with('?'))
        .count()
}

/// REQ-change-172 (option 1): in a blobless clone whose promisor remote answers only with the
/// user's configuration, `change status`, `change audit` and `change ship-status` succeed after
/// one fetch, announced by one line, and the history objects are present afterwards; a full
/// clone makes no fetch.
#[test]
fn a_partial_clone_downloads_what_its_history_reads_need_in_one_fetch() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let id = adopted_project(&source);
    let clone = temp.path().join("clone");
    let config = blobless_clone(temp.path(), &source, &clone);
    assert!(missing_objects(&clone) > 0);
    let env = [("GIT_CONFIG_GLOBAL", config.to_str().unwrap())];

    let first = run(&clone, &env, &["change", "status", &id]);

    assert_eq!(first.matches("downloading ").count(), 1, "{first}");
    assert!(
        first.contains("objects this partial clone lacks from `origin`"),
        "{first}"
    );
    assert_eq!(missing_objects(&clone), 0);
    for args in [
        vec!["change", "audit"],
        vec!["change", "ship-status", id.as_str()],
    ] {
        let output = invoke(&clone, &env, &args);
        assert!(
            !combined(&output).contains("could not fetch"),
            "{args:?}: {}",
            combined(&output)
        );
        assert!(
            !combined(&output).contains("downloading "),
            "{args:?}: {}",
            combined(&output)
        );
    }
    let full = run(&source, &[], &["change", "status", &id]);
    assert!(!full.contains("downloading "), "{full}");
}

/// REQ-change-172: when the download fails, the refusal says the clone is partial and names a
/// `git fetch … <oid> …` that, run as printed with the user's Git, lets the same command pass.
#[test]
fn a_failed_download_names_one_fetch_that_works_as_printed() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let id = adopted_project(&source);
    let clone = temp.path().join("clone");
    let config = blobless_clone(temp.path(), &source, &clone);
    let unreachable = temp.path().join("nothing.gitconfig");
    fs::write(&unreachable, "").unwrap();

    let refused = invoke(
        &clone,
        &[("GIT_CONFIG_GLOBAL", unreachable.to_str().unwrap())],
        &["change", "status", &id],
    );
    let text = combined(&refused);

    assert!(!refused.status.success(), "{text}");
    assert!(text.contains("partial clone"), "{text}");
    let command = text
        .split('`')
        .find(|part| part.starts_with("git -C ") && part.contains(" fetch "))
        .unwrap_or_else(|| panic!("no fetch command: {text}"));
    let output = Command::new("sh")
        .args(["-c", command])
        .env("GIT_CONFIG_GLOBAL", &config)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{command}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    run(
        &clone,
        &[("GIT_CONFIG_GLOBAL", unreachable.to_str().unwrap())],
        &["change", "status", &id],
    );
}

/// ss-lead's note for #1077 over #1081: a blobless clone worked on directly on `main`, with an
/// active change, compares from the change's base commit (REQ-change-169), and the audit reads
/// that commit's requirements after the one fetch instead of saying it cannot tell.
#[test]
fn a_partial_clone_on_main_audits_the_base_commit_requirements_after_one_fetch() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let _id = adopted_project(&source);
    let clone = temp.path().join("clone");
    let config = blobless_clone(temp.path(), &source, &clone);
    assert_eq!(
        git(&clone, &["symbolic-ref", "--short", "HEAD"]).trim(),
        "main"
    );

    let audit = run(
        &clone,
        &[("GIT_CONFIG_GLOBAL", config.to_str().unwrap())],
        &["change", "audit"],
    );

    assert!(!audit.contains("cannot tell"), "{audit}");
    assert_eq!(audit.matches("downloading ").count(), 1, "{audit}");
}
