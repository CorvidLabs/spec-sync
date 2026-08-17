//! Regression tests for the W1 exit-code / CI-gating workstream.
//!
//! Issues: #418 (changelog), #419 (deps), #422 (resolve --remote),
//! #425 (coverage), #430 (report), #431 (diff), #441 (score), #444 (resolve local).

use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

fn git_init(root: &std::path::Path) {
    for args in [
        vec!["init"],
        vec!["config", "user.email", "test@test.com"],
        vec!["config", "user.name", "Test"],
    ] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
    }
}

fn git_commit_all(root: &std::path::Path, msg: &str) {
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", msg])
        .current_dir(root)
        .output()
        .unwrap();
}

// ─── #425: coverage gates on missing referenced files ────────────────────

#[test]
fn coverage_missing_referenced_file_exits_nonzero_by_default() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    // Spec references a file that does not exist on disk.
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts", "src/auth/ghost.ts"]),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--root"])
        .arg(root)
        .assert()
        .failure();
}

#[test]
fn coverage_missing_referenced_file_warn_enforcement_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts", "src/auth/ghost.ts"]),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--enforcement", "warn", "--root"])
        .arg(root)
        .assert()
        .success();
}

#[test]
fn coverage_json_includes_missing_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts", "src/auth/ghost.ts"]),
    )
    .unwrap();

    let output = specsync()
        .args(["coverage", "--json", "--root"])
        .arg(root)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let missing = json["missing_files"]
        .as_array()
        .expect("missing_files must be an array");
    assert!(
        missing
            .iter()
            .any(|f| f.as_str().unwrap_or("").contains("ghost.ts")),
        "missing_files should list the referenced-but-absent file: {json}"
    );
}

// ─── #430: report honors --require-coverage ──────────────────────────────

/// A 100%-covered project in a git repo with one commit.
///
/// The git repo is load-bearing (#607). `report` exits 1 whenever staleness is
/// unmeasurable, and a bare `TempDir` is not a git repo, so an un-gitted fixture
/// exits 1 for *every* `--require-coverage` value — including `0`. Under that
/// fixture `report_require_coverage_above_actual_exits_1` passed while proving
/// nothing: it would have passed with `--require-coverage` deleted outright.
///
/// Committing the fixture stops the staleness exit from shadowing the coverage
/// gate, so these two tests once again measure the flag. The shadowing itself is
/// a real product defect and is tracked separately as #605 — this fixture only
/// stops the tests being blind to it, it does not fix or hide it.
fn report_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = setup_minimal_project(tmp);
    git_init(&root);
    git_commit_all(&root, "initial");
    root
}

#[test]
fn report_require_coverage_above_actual_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = report_project(&tmp);

    specsync()
        .args(["report", "--require-coverage", "101", "--root"])
        .arg(&root)
        .assert()
        .failure();
}

#[test]
fn report_require_coverage_at_actual_exits_0() {
    let tmp = TempDir::new().unwrap();
    let root = report_project(&tmp);

    specsync()
        .args(["report", "--require-coverage", "100", "--root"])
        .arg(&root)
        .assert()
        .success();
}

// ─── #419: deps scalar depends_on + dedupe + --require-coverage ─────────

#[test]
fn deps_scalar_depends_on_is_not_silently_dropped() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    // Scalar depends_on (invalid YAML shape) pointing at a module with no spec.
    // Before #419 it was silently dropped; now it must surface as a missing dep.
    let spec = valid_spec("auth", &["src/auth/service.ts"])
        .replace("depends_on: []", "depends_on: nonexistent-module");
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["deps", "--root"])
        .arg(root)
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("nonexistent-module"),
        "scalar depends_on must be honored, got: {combined}"
    );
    assert!(!output.status.success(), "missing dep must fail deps");
}

/// Run `deps` against a project whose `auth` spec repeats `nonexistent-module`
/// `repeats` times, returning combined stdout+stderr.
fn deps_output_with_repeated_dep(repeats: usize) -> String {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let entries: String = "  - nonexistent-module\n".repeat(repeats);
    let spec = valid_spec("auth", &["src/auth/service.ts"])
        .replace("depends_on: []", &format!("depends_on:\n{entries}"));
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["deps", "--root"])
        .arg(root)
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn deps_duplicate_depends_on_entries_deduped() {
    // The real dedupe signal is the edge count in the printed graph summary: N
    // repeated `depends_on` entries must collapse to a single edge.
    //
    // The original assertion here counted occurrences of the module *name* in
    // the output and required exactly 1. That never encoded a satisfiable claim
    // — a correct, fully-deduping implementation still prints the name twice
    // (see the #606 pin below), so it would have failed even with a single
    // `depends_on` entry. It measured a different quantity than its name said.
    for repeats in 1..=3 {
        let combined = deps_output_with_repeated_dep(repeats);
        assert!(
            combined.contains("Edges: 1"),
            "{repeats} repeated depends_on entries must collapse to 1 edge, got: {combined}"
        );
    }

    // Repetition must make no difference at all, not merely keep the edge count
    // right: 1, 2 and 3 entries must produce byte-identical output.
    let one = deps_output_with_repeated_dep(1);
    let three = deps_output_with_repeated_dep(3);
    assert_eq!(
        one, three,
        "repeating a depends_on entry must not change deps output at all"
    );

    // ─── PIN for #606 — asserts today's WRONG behaviour on purpose ───────
    //
    // One missing dependency is currently reported TWICE, by two independent
    // code paths that each emit their own finding:
    //   src/validator.rs → "Dependency spec not found: {dep}"
    //   src/deps.rs      → "{}: depends on '{}' but no spec exists for that module"
    // A single defect is therefore double-counted in deps error totals.
    //
    // This is NOT the desired behaviour. The pin exists so that fixing #606
    // fails this assertion loudly instead of silently un-pinning. When #606
    // lands, change the expected count from 2 to 1 and delete this comment.
    let occurrences = one.matches("nonexistent-module").count();
    assert_eq!(
        occurrences, 2,
        "PIN(#606): one missing dependency is currently reported twice by two \
         separate code paths. If this now reads 1, #606 is fixed — update this \
         assertion to 1 and remove the pin. Got: {one}"
    );
}

#[test]
fn deps_require_coverage_gate_enforced() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    // Add an unspecced source file so coverage drops below 100%.
    fs::write(root.join("src/extra.ts"), "export function extra() {}\n").unwrap();

    specsync()
        .args(["deps", "--require-coverage", "100", "--root"])
        .arg(&root)
        .assert()
        .failure();
}

// ─── #444: resolve local deps gate the exit code ─────────────────────────

fn resolve_project(dep: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = valid_spec("auth", &["src/auth/service.ts"])
        .replace("depends_on: []", &format!("depends_on:\n  - {dep}"));
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();
    (tmp, root)
}

#[test]
fn resolve_local_missing_path_exits_1_without_strict() {
    let (_tmp, root) = resolve_project("src/auth/nonexistent.ts");
    specsync()
        .args(["resolve", "--root"])
        .arg(&root)
        .assert()
        .failure();
}

#[test]
fn resolve_local_outside_root_traversal_exits_1() {
    let (_tmp, root) = resolve_project("../outside.ts");
    let output = specsync()
        .args(["resolve", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("outside") || combined.contains("escapes"),
        "traversal dep must be reported, got: {combined}"
    );
}

#[test]
fn resolve_malformed_cross_project_ref_exits_1() {
    let (_tmp, root) = resolve_project("owner/repo@");
    let output = specsync()
        .args(["resolve", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("malformed") || combined.contains("owner/repo@"),
        "malformed ref must be reported, got: {combined}"
    );
}

#[test]
fn resolve_all_ok_exits_0() {
    let (_tmp, root) = resolve_project("src/auth/service.ts");
    specsync()
        .args(["resolve", "--root"])
        .arg(&root)
        .assert()
        .success();
}

// ─── #431: diff base-ref fallback ────────────────────────────────────────

#[test]
fn diff_head_fallback_prints_loud_stderr_warning() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    git_commit_all(root, "initial");

    let output = specsync()
        .args(["diff", "--root"])
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "diff fallback should still succeed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("HEAD") || stderr.to_lowercase().contains("fallback"),
        "HEAD fallback must print a loud stderr warning, got: {stderr}"
    );
}

#[test]
fn diff_strict_refuses_head_fallback() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    git_commit_all(root, "initial");

    specsync()
        .args(["diff", "--strict", "--root"])
        .arg(root)
        .assert()
        .failure();
}

// ─── #418: changelog validates refs ──────────────────────────────────────

#[test]
fn changelog_bogus_ref_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    git_commit_all(root, "initial");

    specsync()
        .args(["changelog", "HEAD..bogus-ref-that-does-not-exist", "--root"])
        .arg(root)
        .assert()
        .failure();
}

#[test]
fn changelog_valid_range_exits_0() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    git_commit_all(root, "initial");
    fs::write(
        root.join("src/a.ts"),
        "export function a() {}\nexport function b() {}\n",
    )
    .unwrap();
    git_commit_all(root, "second");

    specsync()
        .args(["changelog", "HEAD~1..HEAD", "--root"])
        .arg(root)
        .assert()
        .success();
}

// ─── #441: score zero-spec JSON ──────────────────────────────────────────

#[test]
fn score_json_with_zero_specs_outputs_valid_json() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    let output = specsync()
        .args(["score", "--json", "--root"])
        .arg(root)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("score --json with zero specs must emit valid JSON");
    assert!(json["specs"].is_array(), "JSON must include specs array");
    assert_eq!(json["specs"].as_array().unwrap().len(), 0);
}

#[test]
fn score_strict_with_zero_specs_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    specsync()
        .args(["score", "--strict", "--json", "--root"])
        .arg(root)
        .assert()
        .failure();
}

// ─── Default enforcement: errors gate even without flags ────────────────

#[test]
fn check_validation_errors_exit_nonzero_by_default() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    // Spec references a file that does not exist → validation error.
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/ghost.ts"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--root"])
        .arg(root)
        .assert()
        .failure();
}

#[test]
fn check_validation_errors_warn_enforcement_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/ghost.ts"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--enforcement", "warn", "--root"])
        .arg(root)
        .assert()
        .success();
}
