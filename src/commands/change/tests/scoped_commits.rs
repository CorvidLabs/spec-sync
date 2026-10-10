use super::super::lifecycle_commit::{
    LEFT_OUT_LISTING_LIMIT, disclose_left_out, git_commit_lifecycle, left_out_warning,
    path_is_within, run_checked_commit, stage_lifecycle_paths,
};
use super::super::shipping::run_ship;
use super::fixtures::{
    STRAYS, approved_change_fixture, assert_strays_left_alone, git_in, git_project_fixture,
    paths_in_history, worktree_status, write_strays,
};
use crate::change;
use crate::types::OutputFormat;
use std::fs;
use tempfile::TempDir;

/// The floor must be WIRED, not merely present (#533).
///
/// `floor_sequence_ledger_to_committed` has its own unit tests, but those
/// exercise the function directly. Nothing asserted that the lifecycle
/// staging path (`git_commit_lifecycle`, formerly `git_commit_all`)
/// actually calls it, so deleting the call left the entire suite green
/// while every lifecycle commit went back to staging a stale ledger over a
/// higher committed mark — the exact regression #533 is about.
///
/// This test drives the real staging path and inspects what landed in the
/// commit, so it fails if the call is removed.
#[test]
fn lifecycle_commit_raises_a_stale_ledger_before_staging_it() {
    use std::process::Command;
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .expect("git")
                .success(),
            "git {args:?}"
        );
    };
    let write_ledger = |sequence: u64| {
        std::fs::create_dir_all(root.join(".specsync")).unwrap();
        std::fs::write(
                root.join(".specsync/change-sequence.json"),
                format!(
                    "{{\n  \"schema_version\": 1,\n  \"sequence\": {sequence},\n  \"id\": \"CHG-{sequence:04}-fixture\",\n  \"acknowledged_collisions\": []\n}}\n"
                ),
            )
            .unwrap();
    };
    let committed_sequence = || -> u64 {
        let out = Command::new("git")
            .args(["show", "HEAD:.specsync/change-sequence.json"])
            .current_dir(root)
            .output()
            .expect("git show");
        let text = String::from_utf8_lossy(&out.stdout);
        let value: serde_json::Value = serde_json::from_str(&text).expect("ledger json");
        value["sequence"].as_u64().expect("sequence")
    };

    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    write_ledger(3);
    std::fs::write(root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "committed high-water mark"]);
    assert_eq!(committed_sequence(), 3);

    // The precondition: a ledger written before the branch caught up.
    write_ledger(1);
    std::fs::write(root.join("README.md"), "work\n").unwrap();

    // Nothing is owned here: the ledger is tracked, so the tracked-edit rule stages it.
    let scope = change::LifecycleCommitScope {
        owned: Vec::new(),
        runtime: Vec::new(),
    };
    git_commit_lifecycle(root, &scope, "lifecycle commit", true).expect("commit");

    assert_eq!(
        committed_sequence(),
        3,
        "the staging path must raise the stale ledger before staging it; \
committing 1 over a committed 3 is the #533 regression, and it is what happens \
if the floor call is removed from this function"
    );
}
/// Honest label: DISCRIMINATOR for the `git add -A` sweep. On the unfixed binary every
/// stray lands in the materialize commit, and this fails on the first one.
///
/// The second half is the CONTROL that keeps the fix honest: the cheap way to pass the
/// first half is to stage nothing untracked at all, which would leave the change's own
/// workspace, deliberately left uncommitted here, out of the commit that records it.
#[test]
fn check_commit_never_commits_an_unrelated_untracked_file() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_project_fixture(root);
    let id = approved_change_fixture(root);
    // The delivery is a tracked edit.
    fs::write(root.join("README.md"), "# fixture\n\nthe delivery\n").unwrap();
    write_strays(root);

    run_checked_commit(root, Some(&id), false, false, OutputFormat::Json).expect("check --commit");

    assert_strays_left_alone(root, &root.join(".git"));

    let history = paths_in_history(&root.join(".git"));
    for owned in [
        "state.json",
        "change.md",
        "verification.json",
        "approvals.json",
    ] {
        let path = format!(".specsync/changes/{id}/{owned}");
        assert!(
            history.contains(&path),
            "the change's own untracked workspace must still be committed: {path} missing from {history:?}"
        );
    }
    // `change new` wrote this ledger in the freshly adopted fixture. Left out, the change
    // would reach history without the origin anchor every later read of it checks.
    assert!(
        history.contains(&".specsync/workflow-v2-baseline.json".to_string()),
        "the lifecycle ledger `change new` wrote must be committed: {history:?}"
    );
    let committed_readme = std::process::Command::new("git")
        .args(["show", "HEAD:README.md"])
        .current_dir(root)
        .output()
        .expect("git show");
    assert_eq!(
        String::from_utf8_lossy(&committed_readme.stdout),
        "# fixture\n\nthe delivery\n",
        "the tracked delivery edit must be committed"
    );
    // The fixture has no `.specsync/.gitignore`, so the project lock is untracked too. It is
    // a runtime file: never committed, never reported, and allowed to stay behind.
    let runtime = change::lifecycle_commit_scope(root, &id)
        .expect("scope")
        .runtime;
    let leftovers: Vec<String> = worktree_status(root)
        .into_iter()
        .filter(|line| !STRAYS.iter().any(|stray| *line == format!("?? {stray}")))
        .filter(|line| !runtime.iter().any(|path| *line == format!("?? {path}")))
        .collect();
    assert!(
        leftovers.is_empty(),
        "only the strays and runtime files may remain outside history: {leftovers:?}"
    );
}

/// Honest label: DISCRIMINATOR for the archive commit `ship --push` makes, which went
/// through the same `git add -A`. It asserts on what reached the remote, because a pushed
/// commit is the one that cannot be taken back.
///
/// CONTROL in the same test: the archive package still reaches the remote, and the active
/// workspace it moved out of is gone from the pushed tree.
#[test]
fn ship_push_never_commits_an_unrelated_untracked_file() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    let remote_parent = TempDir::new().expect("temp remote");
    let remote = remote_parent.path().join("origin.git");
    git_project_fixture(root);
    git_in(
        remote_parent.path(),
        &["init", "--bare", "-b", "main", "origin.git"],
    );
    git_in(
        root,
        &["remote", "add", "origin", remote.to_str().expect("utf-8")],
    );
    git_in(root, &["push", "-u", "origin", "main"]);

    let id = approved_change_fixture(root);
    // Present from before verification, as in the reports: a stray written after review
    // would stale the review and block ship for an unrelated reason.
    write_strays(root);
    run_checked_commit(root, Some(&id), false, false, OutputFormat::Json).expect("check --commit");
    change::record_scoped_review_with_verdict(
        root,
        &id,
        "Independent".into(),
        change::ScopedReviewVerdict::Pass,
    )
    .expect("review");

    run_ship(
        root,
        Some(&id),
        false,
        true,
        false,
        1,
        OutputFormat::Json,
        true,
    )
    .expect("ship --push");

    assert_strays_left_alone(root, &remote);

    let pushed = std::process::Command::new("git")
        .arg(format!("--git-dir={}", remote.display()))
        .args(["ls-tree", "-r", "--name-only", "main"])
        .output()
        .expect("git ls-tree");
    let pushed = String::from_utf8_lossy(&pushed.stdout).into_owned();
    assert!(
        pushed
            .lines()
            .any(|path| path.starts_with(".specsync/archive/changes/")
                && path.ends_with(&format!("-{id}/finalization.json"))),
        "the archive package must reach the remote: {pushed}"
    );
    assert!(
        !pushed
            .lines()
            .any(|path| path.starts_with(&format!(".specsync/changes/{id}/"))),
        "the archived workspace must be gone from the pushed tree: {pushed}"
    );
    let subject = std::process::Command::new("git")
        .arg(format!("--git-dir={}", remote.display()))
        .args(["log", "-1", "--pretty=%s", "main"])
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&subject.stdout).trim(),
        format!("chore(lifecycle): archive {id}")
    );
}

/// Porcelain `-z` writes a rename as two fields, destination then source. Reading the source
/// as an entry of its own would take the first two bytes of a filename as status codes. A
/// staged rename, a tracked deletion, a tracked edit and a stray are each classified once:
/// the edit and the deletion are staged, the stray is reported and left untracked.
#[test]
fn staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits() {
    let temp = TempDir::new().expect("temp project");
    let root = temp.path();
    git_in(root, &["init", "-b", "main"]);
    git_in(root, &["config", "user.email", "t@example.com"]);
    git_in(root, &["config", "user.name", "T"]);
    for name in ["moved.txt", "gone.txt", "edited.txt"] {
        fs::write(root.join(name), format!("{name}\n")).unwrap();
    }
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "base"]);
    git_in(root, &["mv", "moved.txt", "renamed.txt"]);
    fs::remove_file(root.join("gone.txt")).unwrap();
    fs::write(root.join("edited.txt"), "edited\n").unwrap();
    fs::write(root.join("exp2.sh"), "echo stray\n").unwrap();

    let scope = change::LifecycleCommitScope {
        owned: Vec::new(),
        runtime: Vec::new(),
    };
    let left_out = stage_lifecycle_paths(root, &scope, true).expect("stage");

    assert_eq!(left_out, vec!["exp2.sh".to_string()]);
    let staged = worktree_status(root);
    for expected in [
        "R  moved.txt -> renamed.txt",
        "D  gone.txt",
        "M  edited.txt",
        "?? exp2.sh",
    ] {
        assert!(
            staged.contains(&expected.to_string()),
            "missing `{expected}`: {staged:?}"
        );
    }
}

/// A scope owns itself and what lies beneath it, never a sibling sharing its prefix: the
/// workspace of `add-auth` must not own the workspace of `add-auth-v2`.
#[test]
fn a_lifecycle_scope_owns_its_subtree_and_not_a_prefix_sibling() {
    let workspace = ".specsync/changes/add-auth";
    assert!(path_is_within(workspace, workspace));
    assert!(path_is_within(
        ".specsync/changes/add-auth/state.json",
        workspace
    ));
    assert!(path_is_within(
        ".specsync/changes/add-auth/deltas/auth.md",
        &format!("{workspace}/")
    ));
    assert!(!path_is_within(
        ".specsync/changes/add-auth-v2/state.json",
        workspace
    ));
    assert!(!path_is_within(".specsync/changes", workspace));
    assert!(!path_is_within("exp2.sh", workspace));
}

/// The warning names each left-out file, bounds a long list, and tells the author what
/// removing one does to verification only when there is verification to stale.
#[test]
fn the_left_out_warning_names_the_files_and_what_to_do() {
    let few: Vec<String> = STRAYS.iter().map(|stray| stray.to_string()).collect();
    let few_refs: Vec<&String> = few.iter().collect();
    let checked = left_out_warning(&few_refs, Some("add-auth"));
    for stray in STRAYS {
        assert!(checked.contains(&format!("\n  {stray}")), "{checked}");
    }
    assert!(checked.contains("git add"), "{checked}");
    assert!(
        checked.contains("specsync change check add-auth --commit"),
        "{checked}"
    );
    let archived = left_out_warning(&few_refs, None);
    assert!(!archived.contains("change check"), "{archived}");

    let many: Vec<String> = (0..LEFT_OUT_LISTING_LIMIT + 5)
        .map(|index| format!("scratch/{index}.log"))
        .collect();
    let many_refs: Vec<&String> = many.iter().collect();
    let bounded = left_out_warning(&many_refs, None);
    assert!(bounded.contains("… and 5 more"), "{bounded}");
    assert!(!bounded.contains(&format!("scratch/{}.log", LEFT_OUT_LISTING_LIMIT)));

    // A second commit in the same run does not repeat what the first already named.
    let mut disclosed = few.clone();
    let before = disclosed.len();
    disclose_left_out(&few, &mut disclosed, None);
    assert_eq!(disclosed.len(), before);
}
