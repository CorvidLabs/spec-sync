//! #688, REQ-change-168, through the binary (tester on #657 with #688, row
//! P-688-refusals-name-each-other). On a project with a version-1 SDD policy, change A is accepted
//! with `src/a.rs` among its delivery inputs; a later change B declares only its own module and
//! path, edits `src/a.rs` in its delivery without covering it, and is accepted too. A is then
//! stale, and each archive refusal names the other change and says it is blocked too, both naming
//! the reopen of A, which frees B.

use crate::helpers::specsync;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const BLOCKED: [&str; 3] = [
    "cannot be archived either",
    "blocked too",
    "wait on each other",
];

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
    String::from_utf8(output.stdout).unwrap()
}

fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "--allow-empty", "-m", message]);
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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

fn spec(module: &str) -> String {
    format!(
        "---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/{module}.rs\n---\n\n# \
         {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\n| Function | Purpose |\n|---|---|\n| \
         `ready` | Says whether it is ready |\n\n## Invariants\n\nStable.\n\n## Behavioral \
         Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change \
         Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"
    )
}

/// Two modules, `a` and `b`, under a version-1 SDD policy: accept and archive are separate steps.
fn legacy_project(root: &Path) {
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    let policy = serde_json::json!({
        "version": 2, "enabled": true, "require_change_for_meaningful_files": false,
        "meaningful_paths": ["src/"], "ignored_paths": [".specsync/"],
        "verification_commands": ["true"], "custom_artifacts": {}, "principles_file": null,
    });
    write(
        &root.join(".specsync/sdd.json"),
        &serde_json::to_string_pretty(&policy).unwrap(),
    );
    write(
        &root.join(".gitignore"),
        ".specsync/hashes.json\n.specsync/change.lock\n",
    );
    for module in ["a", "b"] {
        write(
            &root.join(format!("src/{module}.rs")),
            "pub fn ready() -> bool { true }\n",
        );
        write(
            &root.join(format!("specs/{module}/{module}.spec.md")),
            &spec(module),
        );
        write(
            &root.join(format!("specs/{module}/requirements.md")),
            &format!(
                "---\nspec: {module}.spec.md\n---\n\n# Requirements\n\n### REQ-{module}-001\n\nThe \
                 system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per \
                 user.\n"
            ),
        );
    }
    commit(root, "base");
    let mut policy = policy;
    policy["version"] = serde_json::json!(1);
    write(
        &root.join(".specsync/sdd.json"),
        &serde_json::to_string_pretty(&policy).unwrap(),
    );
    commit(root, "a version-1 SDD policy");
}

/// A change over `module` and `src/<module>.rs`, defined, approved, implemented with `files`,
/// checked and accepted.
fn accepted_change(root: &Path, module: &str, title: &str, files: &[(&str, &str)]) -> String {
    let printed = run_ok(
        root,
        &[
            "--json",
            "change",
            "new",
            title,
            "--kind",
            "bug-fix",
            "--spec",
            module,
            "--path",
            &format!("src/{module}.rs"),
        ],
    );
    let document: Value =
        serde_json::Deserializer::from_str(&printed[printed.find('{').unwrap()..])
            .into_iter()
            .next()
            .unwrap()
            .unwrap();
    let id = document["change"]["id"].as_str().unwrap().to_string();
    for (question, answer) in [
        ("acceptance_criteria", format!("{module} is tuned")),
        ("public_contract", "no".to_string()),
        ("architecture_risk", "no".to_string()),
    ] {
        run(root, &["change", "answer", &id, question, &answer]);
    }
    let workspace = root.join(".specsync/changes").join(&id);
    let state: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join("state.json")).unwrap()).unwrap();
    let mut artifacts: Vec<String> = state["selected_artifacts"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(|name| name.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    artifacts.extend(["context", "testing", "tasks"].map(str::to_string));
    artifacts.sort();
    artifacts.dedup();
    for name in artifacts {
        let body = match name.as_str() {
            "tasks" => "# Tasks\n\n- [x] Complete.\n".to_string(),
            "plan" => {
                "# Plan\n\n## Approach\n\nDo the work.\n\n## Out of scope\n\nNothing else.\n\n\
                       ## Steps\n\nLand the change.\n\n## Risks\n\nnone\n\n## Constraints \
                       consulted\n\nnone\n"
                    .to_string()
            }
            "testing" => format!(
                "# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| \
                 REQ-{module}-001 | SpecSync structural check |\n"
            ),
            other => {
                let mut title = other.to_string();
                title[..1].make_ascii_uppercase();
                format!("# {title}\n\nComplete.\n")
            }
        };
        write(&workspace.join(format!("{name}.md")), &body);
    }
    write(
        &workspace.join(format!("deltas/{module}.md")),
        &format!(
            "## MODIFIED\n\n### REQUIREMENT REQ-{module}-001\n\nThe system SHALL keep exactly one \
             session per user.\n\nAcceptance Criteria\n- One session per user.\n"
        ),
    );
    commit(root, &format!("define {id}"));
    run_ok(root, &["change", "approve", &id, "--actor", "Scope owner"]);
    commit(root, &format!("approve {id}"));
    for (path, text) in files {
        write(&root.join(path), text);
    }
    commit(root, &format!("code {id}"));
    run_ok(root, &["change", "check", &id]);
    run_ok(root, &["change", "accept", &id, "--actor", "Closer"]);
    commit(root, &format!("{id} accepted"));
    id
}

fn says_blocked(printed: &str) -> bool {
    BLOCKED.iter().any(|phrase| printed.contains(phrase))
}

#[test]
fn each_archive_refusal_over_one_stale_input_names_the_other_and_the_reopen_frees_both() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    legacy_project(root);
    let a = accepted_change(
        root,
        "a",
        "Tune module a",
        &[("src/a.rs", "pub fn ready() -> bool { false }\n")],
    );
    let b = accepted_change(
        root,
        "b",
        "Later work on b",
        &[
            ("src/b.rs", "pub fn ready() -> bool { false }\n"),
            (
                "src/a.rs",
                "pub fn ready() -> bool { true } // edited by B\n",
            ),
        ],
    );
    let reopen = format!("specsync change reopen {a}");

    let (archived, first) = run(root, &["change", "archive", &a]);
    assert!(!archived, "the earlier change is stale:\n{first}");
    assert!(
        first.contains(&b) && says_blocked(&first) && first.contains(&reopen),
        "the earlier change's refusal names the later one, says it is blocked too, and keeps the \
         reopen:\n{first}"
    );

    let (archived, second) = run(root, &["change", "archive", &b]);
    assert!(
        !archived,
        "the later change waits on the earlier one:\n{second}"
    );
    assert!(
        second.contains(&a) && says_blocked(&second) && second.contains(&reopen),
        "the later change's refusal names the earlier one, says it is blocked too, and keeps the \
         reopen:\n{second}"
    );

    run_ok(
        root,
        &[
            "change",
            "reopen",
            &a,
            "--actor",
            "Scope owner",
            "--reason",
            "src/a.rs changed in a later change",
        ],
    );
    commit(root, &format!("{a} reopened"));
    run_ok(root, &["change", "archive", &b]);
}

/// The guard: with no later change over the input, the earlier change's refusal reads as before.
#[test]
fn with_no_later_change_over_the_input_the_refusal_adds_nothing() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    legacy_project(root);
    let a = accepted_change(
        root,
        "a",
        "Tune module a",
        &[("src/a.rs", "pub fn ready() -> bool { false }\n")],
    );
    write(
        &root.join("src/a.rs"),
        "pub fn ready() -> bool { true } // a hand edit, no later change\n",
    );
    commit(root, "hand edit");
    let (archived, refused) = run(root, &["change", "archive", &a]);
    assert!(!archived, "{refused}");
    assert!(
        refused.contains(&format!("specsync change reopen {a}")) && !says_blocked(&refused),
        "{refused}"
    );
}
