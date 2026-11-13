//! #1070 (REQ-change-159, REQ-cmd-change-037): `change close` moves changes that will not ship
//! into `.specsync/archive/closed/<date>-<id>/`, byte for byte, with a `close.json` record. Nothing
//! is reviewed, accepted or written into a spec, and every verb then names the close.

use crate::helpers::specsync_std;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Stdout and stderr together, and whether the command succeeded.
fn run(root: &Path, args: &[&str]) -> (bool, String) {
    let output = specsync_std()
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (passed, text) = run(root, args);
    assert!(passed, "{args:?}: {text}");
    text
}

/// An adopted project with one module, `lib`, owning `src/lib.rs`.
fn project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
    fs::write(root.join("README.md"), "# probe\n").unwrap();
    ok(root, &["init"]);
    fs::create_dir_all(root.join("specs/lib")).unwrap();
    fs::write(
        root.join("specs/lib/lib.spec.md"),
        "---\nmodule: lib\nversion: 1\nstatus: active\nfiles:\n  - src/lib.rs\n---\n\n# Lib\n\n\
         ## Purpose\n\nThe lib.\n\n## Public API\n\n| Export | Description |\n|--------|-------------|\n\
         | `ready` | Whether the lib is ready |\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\n\
         Works.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n\
         | Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
    )
    .unwrap();
    ok(root, &["change", "adopt"]);
    git(root, &["add", "--all"]);
    git(root, &["commit", "-q", "-m", "adopt"]);
    temp
}

/// A change over `README.md` with no spec change, its interview answered and artifacts filled.
fn draft(root: &Path, description: &str) -> String {
    let created = ok(
        root,
        &[
            "--json",
            "change",
            "new",
            description,
            "--kind",
            "bug-fix",
            "--no-spec-change",
            "--rationale",
            "probe",
            "--path",
            "README.md",
        ],
    );
    let id = serde_json::from_str::<Value>(&created).unwrap()["change"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    answer_and_fill(root, &id);
    id
}

fn answer_and_fill(root: &Path, id: &str) {
    for (question, answer) in [
        ("acceptance_criteria", "the probe reads"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
        ("constraints", "none"),
    ] {
        ok(root, &["change", "answer", id, question, answer]);
    }
    let dir = root.join(".specsync/changes").join(id);
    for artifact in ["context", "tasks", "testing"] {
        let path = dir.join(format!("{artifact}.md"));
        if path.exists() {
            fs::write(
                &path,
                format!("---\nchange: {id}\nartifact: {artifact}\n---\n\n# Probe\n\nProbe text.\n"),
            )
            .unwrap();
        }
    }
}

fn closed_dir(root: &Path, id: &str) -> PathBuf {
    let parent = root.join(".specsync/archive/closed");
    fs::read_dir(&parent)
        .unwrap_or_else(|error| panic!("{}: {error}", parent.display()))
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(&format!("-{id}")))
        })
        .unwrap_or_else(|| panic!("no closed package for {id}"))
}

#[test]
fn closing_several_changes_moves_each_package_and_every_verb_names_the_close() {
    let temp = project();
    let root = temp.path();
    let first = draft(root, "first abandoned probe");
    let second = draft(root, "second abandoned probe");
    ok(root, &["change", "approve", &second, "--actor", "Ada"]);
    let state_before = fs::read(
        root.join(".specsync/changes")
            .join(&first)
            .join("state.json"),
    )
    .unwrap();

    let closed = ok(
        root,
        &[
            "--json",
            "change",
            "close",
            &first,
            &second,
            "--actor",
            "Ada",
            "--reason",
            "superseded by the new design",
        ],
    );
    let closed: Value = serde_json::from_str(&closed).unwrap();
    assert_eq!(closed["closed"].as_array().unwrap().len(), 2, "{closed}");

    for (id, from) in [(&first, "draft"), (&second, "approved")] {
        assert!(!root.join(".specsync/changes").join(id).exists());
        let package = closed_dir(root, id);
        let record: Value =
            serde_json::from_slice(&fs::read(package.join("close.json")).unwrap()).unwrap();
        assert_eq!(record["actor"], "Ada", "{record}");
        assert_eq!(record["reason"], "superseded by the new design");
        assert_eq!(record["from_state"], from);
        assert_eq!(record["reviewed"], false);

        let status = ok(root, &["change", "status", id]);
        assert!(
            status.contains("closed; never reviewed")
                && status.contains("superseded by the new design"),
            "{status}"
        );
        let json: Value =
            serde_json::from_str(&ok(root, &["--json", "change", "status", id])).unwrap();
        assert_eq!(json["summary"]["state"], "closed", "{json}");
        assert!(ok(root, &["change", "show", id]).contains("closed; never reviewed"));
        assert!(ok(root, &["change", "ship-status", id]).contains("closed; never reviewed"));
        let (reopened, text) = run(root, &["change", "check", id]);
        assert!(!reopened && text.contains("was closed"), "{text}");
    }
    assert_eq!(
        fs::read(closed_dir(root, &first).join("state.json")).unwrap(),
        state_before,
        "the package keeps its bytes"
    );
    assert!(!ok(root, &["change", "list"]).contains(&first));
    let (minted, text) = run(
        root,
        &[
            "change",
            "new",
            "first abandoned probe",
            "--kind",
            "bug-fix",
            "--no-spec-change",
            "--rationale",
            "probe",
            "--path",
            "README.md",
        ],
    );
    assert!(!minted && text.contains("stays reserved"), "{text}");
}

#[test]
fn a_close_that_cannot_close_every_change_closes_none() {
    let temp = project();
    let root = temp.path();
    let first = draft(root, "a probe that stays");
    let (closed, text) = run(
        root,
        &[
            "change",
            "close",
            &first,
            "no-such-change",
            "--actor",
            "Ada",
            "--reason",
            "cleanup",
        ],
    );
    assert!(!closed && text.contains("nothing was closed"), "{text}");
    assert!(root.join(".specsync/changes").join(&first).is_dir());
    assert!(!root.join(".specsync/archive/closed").exists());

    let (closed, text) = run(
        root,
        &[
            "change", "close", &first, "--actor", "Ada", "--reason", "  ",
        ],
    );
    assert!(!closed && text.contains("reason"), "{text}");
}

#[test]
fn a_close_records_its_replacement_and_a_relayed_decision() {
    let temp = project();
    let root = temp.path();
    let old = draft(root, "the old approach");
    let new = draft(root, "the new approach");
    let (refused, text) = run(
        root,
        &[
            "change",
            "close",
            &old,
            "--actor",
            "Ada",
            "--reason",
            "replaced",
            "--replaced-by",
            "missing-change",
        ],
    );
    assert!(!refused && text.contains("missing-change"), "{text}");
    let (refused, text) = run(
        root,
        &[
            "change",
            "close",
            &old,
            "--actor",
            "Ada",
            "--reason",
            "replaced",
            "--words",
            "Close it.",
        ],
    );
    assert!(
        !refused && text.contains("--where") && text.contains("--recorded-by"),
        "a partial relay is refused naming the missing flags: {text}"
    );
    assert!(root.join(".specsync/changes").join(&old).is_dir());
    ok(
        root,
        &[
            "change",
            "close",
            &old,
            "--actor",
            "Ada",
            "--reason",
            "replaced",
            "--replaced-by",
            &new,
            "--words",
            "Close the old one.",
            "--where",
            "standup, 2026-11-13",
            "--recorded-by",
            "an agent",
        ],
    );
    let record: Value =
        serde_json::from_slice(&fs::read(closed_dir(root, &old).join("close.json")).unwrap())
            .unwrap();
    assert_eq!(record["replaced_by"], new.as_str(), "{record}");
    assert_eq!(record["relayed_by"]["decided_by"], "Ada", "{record}");
    assert_eq!(record["relayed_by"]["words"], "Close the old one.");
    assert!(ok(root, &["change", "status", &old]).contains(&format!("`{new}` replaces it")));
}

#[test]
fn a_change_whose_materialized_spec_text_has_not_merged_is_refused_until_it_is_restored() {
    let temp = project();
    let root = temp.path();
    let origin = TempDir::new().unwrap();
    git(origin.path(), &["init", "-q", "--bare", "-b", "main"]);
    git(
        root,
        &["remote", "add", "origin", origin.path().to_str().unwrap()],
    );
    git(root, &["push", "-q", "origin", "main"]);
    git(root, &["fetch", "-q", "origin"]);
    git(root, &["remote", "set-head", "origin", "main"]);

    let created = ok(
        root,
        &[
            "--json",
            "change",
            "new",
            "edit the lib purpose",
            "--kind",
            "bug-fix",
            "--spec",
            "lib",
            "--path",
            "src/lib.rs",
        ],
    );
    let id = serde_json::from_str::<Value>(&created).unwrap()["change"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    answer_and_fill(root, &id);
    fs::write(
        root.join(".specsync/changes")
            .join(&id)
            .join("deltas/lib.md"),
        "## MODIFIED\n\n### SPEC SECTION Purpose\n\nThe lib reports readiness.\n",
    )
    .unwrap();
    ok(root, &["change", "approve", &id, "--actor", "Ada"]);
    ok(root, &["change", "check", &id]);
    let spec = fs::read_to_string(root.join("specs/lib/lib.spec.md")).unwrap();
    assert!(
        spec.contains("The lib reports readiness."),
        "materialized: {spec}"
    );

    let (closed, text) = run(
        root,
        &[
            "change", "close", &id, "--actor", "Ada", "--reason", "dropped",
        ],
    );
    assert!(
        !closed && text.contains("has not merged") && text.contains("git checkout origin/main --"),
        "{text}"
    );
    assert!(root.join(".specsync/changes").join(&id).is_dir());

    assert!(
        text.contains("remove specs/lib/requirements.md"),
        "a file the change created is removed, not checked out: {text}"
    );
    git(
        root,
        &["checkout", "origin/main", "--", "specs/lib/lib.spec.md"],
    );
    fs::remove_file(root.join("specs/lib/requirements.md")).unwrap();
    ok(
        root,
        &[
            "change", "close", &id, "--actor", "Ada", "--reason", "dropped",
        ],
    );
    assert!(
        !fs::read_to_string(root.join("specs/lib/lib.spec.md"))
            .unwrap()
            .contains("The lib reports readiness.")
    );
}
