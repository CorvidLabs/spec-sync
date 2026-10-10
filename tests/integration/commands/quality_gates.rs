//! Quality scoring and coverage/enforcement gates, including empty projects.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── Score Command Tests ─────────────────────────────────────────────────

#[test]
fn score_command_outputs_quality_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    specsync()
        .args(["score", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("/100"));
}

#[test]
fn score_json_output_has_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    let output = specsync()
        .args(["score", "--root", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["average_score"].is_number());
    assert!(json["grade"].is_string());
    assert!(json["specs"].is_array());
    let specs = json["specs"].as_array().unwrap();
    assert_eq!(specs.len(), 1);
    assert!(specs[0]["total"].as_u64().unwrap() > 0);
}

#[test]
fn score_withholds_api_credit_for_unreadable_file() {
    // Regression: a `files:` entry that can't be read (here missing) produced zero
    // exports, which the API dimension scored as a PERFECT "no exports to document"
    // (20/20) — inflating the gating total. It must withhold the credit instead.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/foo")).unwrap();
    fs::write(
        root.join("specs/foo/foo.spec.md"),
        "---\nmodule: foo\nversion: 1\nstatus: active\nfiles:\n  - src/does_not_exist.rs\n---\n# foo\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .args(["score", "--explain", "--root", root.to_str().unwrap()])
        .assert()
        .stdout(
            predicate::str::contains("could not analyze exports")
                .and(predicate::str::contains("no exports to document").not()),
        );
}

// ─── specsync score: gate flags are honored, not silent no-ops ───────────

#[test]
fn score_honors_require_coverage_gate() {
    // Regression (H5): the global --require-coverage / --enforcement flags were
    // silently ignored by `score` (it always exited 0). They must now gate.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    // 1 specced + 1 unspecced file → below 100% coverage.
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(root.join("src/uncovered.ts"), "export function b() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/a")).unwrap();
    fs::write(
        root.join("specs/a/a.spec.md"),
        valid_spec("a", &["src/a.ts"]),
    )
    .unwrap();

    // Gate flags now fail; default score stays advisory (exit 0).
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100"])
        .assert()
        .failure();
    specsync()
        .current_dir(root)
        .args(["score", "--enforcement", "enforce-new"])
        .assert()
        .failure();
    // JSON output must still gate AND remain valid JSON.
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::starts_with("{"));
    // CSV is a machine format too: it must gate WITHOUT the human failure message
    // leaking into the CSV body (regression guard for the review's CSV nit).
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100", "--format", "csv"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("--require-coverage").not());
    specsync().current_dir(root).arg("score").assert().success();
}

#[test]
fn score_no_specs_still_evaluates_requested_gate() {
    // Regression (H5/H2 class): a spec-less project must still FAIL a requested
    // gate rather than taking the no-spec early-exit, while a plain `score`
    // keeps its friendly early-exit.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100"])
        .assert()
        .failure();
    specsync().current_dir(root).arg("score").assert().success();
}

#[test]
fn coverage_no_specs_evaluates_gate() {
    // Regression (M1): `coverage` used to take the no-spec early-exit (exit 0) and
    // its JSON path always exited 0, so the gate was never evaluated. A project
    // with source but no specs is 0% covered and must FAIL a requested gate.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    specsync()
        .current_dir(root)
        .args(["coverage", "--require-coverage", "100"])
        .assert()
        .failure();
    // JSON path must gate too, and stay valid JSON.
    specsync()
        .current_dir(root)
        .args(["coverage", "--require-coverage", "100", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::starts_with("{"));
    specsync()
        .current_dir(root)
        .args(["coverage", "--enforcement", "enforce-new"])
        .assert()
        .failure();
    // A CONFIG-only enforce-new gate (no CLI flag) must also fire.
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"enforce-new\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .arg("coverage")
        .assert()
        .failure();
    // Back to a warn config → coverage report still exits 0 (no gate requested).
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"warn\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .arg("coverage")
        .assert()
        .success();
}

#[test]
fn score_config_only_enforcement_gates_no_specs() {
    // Regression (M1 sibling): a CONFIG-level enforcement gate (no CLI flag) must
    // also stop the no-spec early-exit — `score` on a spec-less project whose
    // config sets enforce-new must FAIL, matching `check`.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"enforce-new\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync().current_dir(root).arg("score").assert().failure();

    // A warn config keeps the friendly advisory early-exit (exit 0).
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"warn\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync().current_dir(root).arg("score").assert().success();
}
