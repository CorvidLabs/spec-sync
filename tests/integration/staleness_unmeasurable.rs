//! #572 — a tree with no git history must not be reported as a tree with no drift.
//!
//! `stale` has refused to answer without git history since #558. Four other
//! implementations of the same spec-behind-source distance never got the fix:
//! `report`, `check --stale`, `score`'s freshness dimension, and the lifecycle
//! `no_stale` guard. On a tree whose specs are 6 commits behind their source,
//! with `.git` removed and nothing else changed, they all reported everything
//! current — and `score` reported a HIGHER grade than the identical tree with
//! its history intact, because the git-freshness penalty needs git to fire.
//!
//! The defect class this campaign keeps shipping: a category is empty for want
//! of INPUT, and the code reads that as want of PROBLEMS.

use crate::helpers::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Run a git command in `root`, asserting it succeeds.
fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo(root: &Path) {
    git(root, &["init"]);
    git(root, &["config", "user.email", "fixture@test.invalid"]);
    git(root, &["config", "user.name", "Fixture"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    // Silence git's background housekeeping. `git commit` may run `gc --auto` or
    // `maintenance run --auto` detached, and either writes into `.git` while
    // `drifted_without_git` is removing it.
    //
    // Honest about the evidence: this is a plausible cause, not a proven one. Seven
    // commits sit far below the 6700 loose-object gc threshold, and a local probe
    // found no leftover artifacts — though it could not have detected a transient
    // lock. These settings remove the known background writers; the retry below is
    // what actually makes the removal reliable.
    git(root, &["config", "gc.auto", "0"]);
    git(root, &["config", "maintenance.auto", "false"]);
}

/// Remove `.git`, tolerating a concurrent writer.
///
/// Three tests in this file failed on `main` with `DirectoryNotEmpty` at this exact
/// removal, all in the same parallel run, while every one of them passes locally.
/// `remove_dir_all` reads a directory and then unlinks; anything that creates a file
/// in between makes it fail, and git's detached housekeeping can do exactly that.
///
/// The test's intent is "there is no git history here", not "`.git` can be removed on
/// the first attempt", so a bounded retry serves the assertion rather than weakening
/// it. It fails loudly if the directory genuinely cannot be removed.
fn remove_git_dir(root: &Path) {
    let git_dir = root.join(".git");
    for attempt in 0..10 {
        match fs::remove_dir_all(&git_dir) {
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) if attempt == 9 => {
                panic!("could not remove {}: {error}", git_dir.display())
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }
}

/// A one-module project whose spec lists one source file that exists.
fn seed_tree(root: &Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/greeter")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn greet(name: &str) -> String {\n    format!(\"hello, {name}!\")\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/greeter/greeter.spec.md"),
        valid_spec("greeter", &["src/lib.rs"]),
    )
    .unwrap();
}

/// A project whose spec is `DRIFT` commits behind its source, with committed
/// history intact. This is the control: every command must see the drift.
fn drifted_repo(root: &Path) {
    seed_tree(root);
    init_repo(root);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-m", "seed"]);
    for index in 1..=DRIFT {
        let mut source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
        source.push_str(&format!("// drift commit {index}\n"));
        fs::write(root.join("src/lib.rs"), source).unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-m", &format!("drift {index}")]);
    }
}

const DRIFT: usize = 6;

/// The same working tree, byte for byte, with `.git` REMOVED.
///
/// The spec's bytes are then written back over themselves. That leaves the tree
/// byte-identical while making the spec no OLDER than its source, which is what
/// a fresh clone, a tarball extraction, or a CI checkout actually looks like —
/// every file stamped at checkout time. It matters because `score` falls back
/// to modification times when it has no commit to measure from, and that
/// fallback is silent in exactly this state. Leaving the git-era mtimes in
/// place would let the fallback mask the missing guard and make these tests
/// pass against a broken build.
fn drifted_without_git(root: &Path) {
    drifted_repo(root);
    remove_git_dir(root);
    let spec = root.join("specs/greeter/greeter.spec.md");
    let bytes = fs::read(&spec).unwrap();
    fs::write(&spec, bytes).unwrap();
}

/// The same working tree with an UNBORN HEAD — `git init` and nothing since,
/// which is exactly the state the quick start leaves behind.
fn drifted_with_unborn_head(root: &Path) {
    drifted_without_git(root);
    init_repo(root);
}

/// A healthy repo whose spec is committed LAST, so it is genuinely 0 behind.
fn healthy_repo(root: &Path) {
    seed_tree(root);
    init_repo(root);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-m", "seed"]);
    let mut source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    source.push_str("// evolution\n");
    fs::write(root.join("src/lib.rs"), source).unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-m", "evolve"]);
    let mut spec = fs::read_to_string(root.join("specs/greeter/greeter.spec.md")).unwrap();
    spec.push_str("- 1.1 — caught up\n");
    fs::write(root.join("specs/greeter/greeter.spec.md"), spec).unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-m", "spec catches up"]);
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let output = specsync()
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("failed to run specsync");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
    )
}

/// Every tree in which the spec-behind-source distance is UNKNOWN.
fn historyless_trees() -> Vec<(&'static str, TempDir)> {
    let no_git = TempDir::new().unwrap();
    drifted_without_git(no_git.path());
    let unborn = TempDir::new().unwrap();
    drifted_with_unborn_head(unborn.path());
    vec![("no .git", no_git), ("unborn HEAD", unborn)]
}

// ─── report ──────────────────────────────────────────────────────────────

#[test]
fn report_json_never_states_a_staleness_it_could_not_measure() {
    for (label, tmp) in historyless_trees() {
        let (stdout, _, code) = run(tmp.path(), &["report", "--format", "json"]);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let module = &json["modules"][0];

        assert!(
            module["stale"].is_null(),
            "[{label}] `stale` must be null, not a verdict nobody reached: {json}"
        );
        assert!(
            module["commits_behind"].is_null(),
            "[{label}] `commits_behind` must be null, not a distance nobody measured: {json}"
        );
        // This asserted `0` when it was written, with the reasoning "an
        // unmeasured module is not a stale one". True of the module, wrong of
        // the COUNT: `0` says zero modules are stale, which is a claim about a
        // measurement that never ran — the same defect the surrounding test
        // exists to prevent, one level up in the aggregate. Sandbox drill 046
        // caught it after the fix had already merged. `null` here, and a number
        // only when at least one module was actually measured.
        assert!(
            json["stale_modules"].is_null(),
            "[{label}] the stale COUNT must be null when nothing was measured; \
             0 is an answer and there is no answer here: {json}"
        );
        assert_eq!(
            json["unmeasured_stale_modules"], 1,
            "[{label}] the unmeasured module must be counted somewhere: {json}"
        );
        assert_eq!(
            json["staleness_inconclusive"], true,
            "[{label}] the payload must say the staleness half is inconclusive: {json}"
        );
        assert_eq!(
            code, 1,
            "[{label}] `report` must not exit 0 certifying 0 stale over a tree it never \
             measured; stdout={stdout}"
        );
    }
}

#[test]
fn report_text_and_csv_render_the_absence_rather_than_a_zero() {
    for (label, tmp) in historyless_trees() {
        let (stdout, _, _) = run(tmp.path(), &["report"]);
        assert!(
            stdout.contains("1 staleness unmeasured"),
            "[{label}] the text summary must name the unmeasured module:\n{stdout}"
        );
        assert!(
            stdout.contains("Staleness inconclusive"),
            "[{label}] the text report must say why:\n{stdout}"
        );

        let (stdout, _, _) = run(tmp.path(), &["report", "--format", "csv"]);
        let row = stdout
            .lines()
            .find(|line| line.starts_with("greeter,"))
            .unwrap_or_else(|| panic!("[{label}] no module row:\n{stdout}"));
        assert!(
            row.ends_with(",,,false"),
            "[{label}] the stale and commits_behind CSV fields must be EMPTY, not `false,0`: \
             {row}"
        );

        let (stdout, _, _) = run(tmp.path(), &["report", "--format", "markdown"]);
        assert!(
            stdout.contains("| n/a | n/a |"),
            "[{label}] markdown must render `n/a`, not `no | 0`:\n{stdout}"
        );
    }
}

// ─── check --stale ───────────────────────────────────────────────────────

#[test]
fn check_stale_says_it_could_not_check_rather_than_saying_nothing() {
    for (label, tmp) in historyless_trees() {
        let (stdout, _, _) = run(tmp.path(), &["check", "--stale"]);
        assert!(
            stdout.contains("staleness not checked"),
            "[{label}] `check --stale` must disclose that it never checked:\n{stdout}"
        );

        let (stdout, _, _) = run(tmp.path(), &["check", "--stale", "--format", "json"]);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(
            json["stale"][0]["reason"], "history_unavailable",
            "[{label}] the machine payload must carry the reason: {json}"
        );

        // A gate over an unanswerable question fails closed, exactly as it does
        // over a real drift finding.
        let (_, _, code) = run(tmp.path(), &["check", "--stale", "--strict"]);
        assert_eq!(
            code, 1,
            "[{label}] `check --stale --strict` must fail closed when staleness is unknowable"
        );
    }
}

// ─── score (the site the first fix attempt missed) ───────────────────────

/// The measurement in the issue: deleting `.git` RAISED the grade a full
/// letter, because the git-freshness penalty needs git to fire.
#[test]
fn deleting_git_history_can_never_raise_a_spec_score() {
    let with_git = TempDir::new().unwrap();
    drifted_repo(with_git.path());
    let (stdout, _, _) = run(with_git.path(), &["score", "--format", "json"]);
    let measured: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let measured_total = measured["specs"][0]["total"].as_u64().unwrap();
    let measured_freshness = measured["specs"][0]["freshness"].as_u64().unwrap();

    for (label, tmp) in historyless_trees() {
        let (stdout, _, _) = run(tmp.path(), &["score", "--format", "json"]);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let total = json["specs"][0]["total"].as_u64().unwrap();
        let freshness = json["specs"][0]["freshness"].as_u64().unwrap();

        assert!(
            total <= measured_total,
            "[{label}] removing git history raised the score from {measured_total} to {total} — \
             a grade improved because measurement stopped: {json}"
        );
        assert!(
            freshness <= measured_freshness,
            "[{label}] freshness rose from {measured_freshness} to {freshness} without git: {json}"
        );
        assert!(
            json["specs"][0]["suggestions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s.as_str().unwrap_or_default().contains("unverifiable")),
            "[{label}] the score must say the drift was unverifiable: {json}"
        );
    }
}

/// The `--min-score` gate — the same number `lifecycle`'s `min_score` guard
/// consumes — must not pass on points awarded for a question git could not be
/// asked. The bar is derived from the measured score rather than hardcoded, so
/// the assertion is exactly "history cannot buy you over the bar", at whatever
/// bar the measured tree sits just under.
#[test]
fn a_min_score_gate_cannot_pass_because_git_history_is_absent() {
    let with_git = TempDir::new().unwrap();
    drifted_repo(with_git.path());
    let (stdout, _, _) = run(with_git.path(), &["score", "--format", "json"]);
    let measured: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let bar = measured["specs"][0]["total"].as_u64().unwrap() + 1;
    let bar = bar.to_string();

    let (_, _, measured_code) = run(with_git.path(), &["score", "--min-score", &bar]);
    assert_eq!(
        measured_code, 1,
        "control: the measured tree must sit just under the {bar}-point bar"
    );

    for (label, tmp) in historyless_trees() {
        let (stdout, _, code) = run(tmp.path(), &["score", "--min-score", &bar]);
        assert_eq!(
            code, 1,
            "[{label}] the same tree without git history must not CLEAR the {bar}-point bar the \
             tree with history fails; stdout={stdout}"
        );
    }
}

/// The lifecycle `min_score` guard is the caller the issue names: it consumes
/// `score.total` directly, so a score inflated by unmeasurable freshness lets a
/// spec change status on points nobody earned.
#[test]
fn the_min_score_guard_cannot_be_cleared_by_deleting_git_history() {
    let with_git = TempDir::new().unwrap();
    drifted_repo(with_git.path());
    let (stdout, _, _) = run(with_git.path(), &["score", "--format", "json"]);
    let measured: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let bar = measured["specs"][0]["total"].as_u64().unwrap() + 1;

    let guard = format!(
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n\
         [lifecycle]\ntrack_history = false\n\n\
         [lifecycle.guards.\"active→review\"]\nmin_score = {bar}\n"
    );
    let install = |root: &Path| {
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(root.join(".specsync/config.toml"), &guard).unwrap();
    };

    install(with_git.path());
    let (stdout, stderr, code) = run(
        with_git.path(),
        &["lifecycle", "demote", "specs/greeter/greeter.spec.md"],
    );
    assert_eq!(
        code, 1,
        "control: the measured spec must sit under the {bar}-point guard;\n{stdout}{stderr}"
    );

    for (label, tmp) in historyless_trees() {
        install(tmp.path());
        let (stdout, stderr, code) = run(
            tmp.path(),
            &["lifecycle", "demote", "specs/greeter/greeter.spec.md"],
        );
        assert_eq!(
            code, 1,
            "[{label}] removing git history must not lift a spec over a min_score guard;\
             \n{stdout}{stderr}"
        );
    }
}

/// `--explain` budgets must still sum exactly to the reported dimension score
/// (#441), including when the git criterion is withheld.
#[test]
fn explain_criteria_still_sum_to_the_freshness_score_when_git_is_withheld() {
    for (label, tmp) in historyless_trees() {
        let (stdout, _, _) = run(tmp.path(), &["score", "--explain", "--format", "json"]);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let freshness = json["specs"][0]["explain"]
            .as_array()
            .unwrap()
            .iter()
            .find(|detail| detail["dimension"] == "Freshness")
            .unwrap_or_else(|| panic!("[{label}] no Freshness dimension: {json}"))
            .clone();
        let summed: u64 = freshness["criteria"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["points"].as_u64().unwrap())
            .sum();
        assert_eq!(
            summed,
            freshness["score"].as_u64().unwrap(),
            "[{label}] criteria must sum to the dimension score: {freshness}"
        );
        let git = freshness["criteria"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "git_freshness")
            .unwrap()
            .clone();
        assert_eq!(
            git["points"], 0,
            "[{label}] the git criterion must award nothing when it measured nothing: {git}"
        );
        assert_eq!(git["passed"], false, "[{label}] {git}");
    }
}

// ─── lifecycle `no_stale` guard ──────────────────────────────────────────

/// Install a `draft→review` guard that refuses stale specs. `.specsync/config.toml`
/// wins over the `specsync.json` the tree already has.
fn add_no_stale_guard(root: &Path) {
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n\
         [lifecycle]\ntrack_history = false\n\n\
         [lifecycle.guards.\"active→review\"]\nno_stale = true\nstale_threshold = 5\n",
    )
    .unwrap();
}

/// A guard that git cannot evaluate must block the transition, not wave it
/// through. This is the fifth implementation of the same computation, and the
/// one that decides whether a spec may change status.
#[test]
fn the_no_stale_guard_blocks_when_it_cannot_be_verified() {
    // Control first: with history, the guard sees the drift and blocks.
    let with_git = TempDir::new().unwrap();
    drifted_repo(with_git.path());
    add_no_stale_guard(with_git.path());
    let (stdout, stderr, code) = run(
        with_git.path(),
        &["lifecycle", "demote", "specs/greeter/greeter.spec.md"],
    );
    assert_eq!(
        code, 1,
        "control: a spec {DRIFT} commits behind must be blocked by a no_stale \
         guard;\nstdout={stdout}\nstderr={stderr}"
    );

    for (label, tmp) in historyless_trees() {
        add_no_stale_guard(tmp.path());
        let (stdout, stderr, code) = run(
            tmp.path(),
            &["lifecycle", "demote", "specs/greeter/greeter.spec.md"],
        );
        assert_eq!(
            code, 1,
            "[{label}] a no_stale guard git could not evaluate must NOT be reported as \
             satisfied;\nstdout={stdout}\nstderr={stderr}"
        );
        assert!(
            format!("{stdout}{stderr}").contains("could not be verified"),
            "[{label}] the block must say the guard was unverifiable, not that the spec is \
             stale;\nstdout={stdout}\nstderr={stderr}"
        );
    }
}

// ─── stale (already correct; locked so it stays that way) ────────────────

#[test]
fn stale_still_refuses_without_history() {
    for (label, tmp) in historyless_trees() {
        let (_, stderr, code) = run(tmp.path(), &["stale"]);
        assert_eq!(code, 1, "[{label}] `stale` must exit 1");
        assert!(
            stderr.contains("staleness detection requires git history"),
            "[{label}] {stderr}"
        );
    }
}

// ─── Healthy controls: the fix must not invent findings ──────────────────

#[test]
fn a_healthy_repo_is_unchanged_in_every_command() {
    let tmp = TempDir::new().unwrap();
    healthy_repo(tmp.path());
    let root = tmp.path();

    let (stdout, _, code) = run(root, &["report", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["modules"][0]["stale"], false, "{json}");
    assert_eq!(json["modules"][0]["commits_behind"], 0, "{json}");
    assert_eq!(json["unmeasured_stale_modules"], 0, "{json}");
    assert_eq!(json["staleness_inconclusive"], false, "{json}");
    assert_eq!(code, 0, "a healthy repo must still exit 0:\n{stdout}");

    let (stdout, _, code) = run(root, &["report"]);
    assert!(
        !stdout.contains("staleness unmeasured"),
        "a measured repo must not mention unmeasured modules:\n{stdout}"
    );
    assert_eq!(code, 0);

    let (stdout, _, code) = run(root, &["check", "--stale"]);
    assert!(
        !stdout.contains("staleness not checked"),
        "a measured repo must not claim it skipped the check:\n{stdout}"
    );
    assert_eq!(code, 0);

    let (_, _, code) = run(root, &["stale"]);
    assert_eq!(code, 0, "a spec that is 0 behind is not stale");

    let (stdout, _, _) = run(root, &["score", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        !json["specs"][0]["suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap_or_default().contains("unverifiable")),
        "a measured repo must not be told its drift is unverifiable: {json}"
    );
}

/// The drifted control, from the other direction: the real finding survives.
#[test]
fn a_drifted_repo_still_reports_its_real_distance() {
    let tmp = TempDir::new().unwrap();
    drifted_repo(tmp.path());
    let root = tmp.path();

    let (stdout, _, code) = run(root, &["report", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["modules"][0]["stale"], true, "{json}");
    assert_eq!(json["modules"][0]["commits_behind"], DRIFT, "{json}");
    assert_eq!(json["unmeasured_stale_modules"], 0, "{json}");
    assert_eq!(code, 1, "a stale module still fails the gate:\n{stdout}");

    let (_, _, code) = run(root, &["stale"]);
    assert_eq!(code, 1, "`stale` still finds the drift");

    let (stdout, _, _) = run(root, &["check", "--stale"]);
    assert!(
        stdout.contains("commits behind source files"),
        "`check --stale` still reports the drift:\n{stdout}"
    );
}

/// A project whose specs list NO source files never asks git anything, so it
/// must keep its full report in a directory with no history. This is the tree
/// a blanket pre-loop guard would have broken.
#[test]
fn a_project_that_never_asks_git_is_untouched_without_history() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/greeter")).unwrap();
    fs::write(
        root.join("specs/greeter/greeter.spec.md"),
        valid_spec("greeter", &[]),
    )
    .unwrap();

    let (stdout, _, code) = run(root, &["report", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["unmeasured_stale_modules"], 0,
        "staleness was never relevant here: {json}"
    );
    assert_eq!(json["staleness_inconclusive"], false, "{json}");
    assert_eq!(
        code, 0,
        "no git history must not fail a project that never asks about drift:\n{stdout}"
    );
}
