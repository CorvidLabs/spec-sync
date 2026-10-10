use crate::change;
use crate::change::ChangeKind;
use crate::change::ChangeRecord;
use crate::change::CreateChangeRequest;
use std::fs;
use std::path::Path;

pub(super) fn draft_fixture(root: &Path) -> ChangeRecord {
    change::write_default_policy(root, Vec::new()).expect("write default policy");
    change::create_change(
        root,
        CreateChangeRequest {
            description: "Ship status next action".into(),
            kind: ChangeKind::Documentation,
            affected_specs: Vec::new(),
            affected_paths: vec!["README.md".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Command test fixture".into()),
        },
    )
    .expect("create draft")
}

pub(super) fn git_in(root: &Path, args: &[&str]) {
    assert!(
        std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success(),
        "git {args:?}"
    );
}

/// A git project with a committed base tree, ready for a change to be driven through it.
pub(super) fn git_project_fixture(root: &Path) {
    git_in(root, &["init", "-b", "main"]);
    git_in(root, &["config", "user.email", "t@example.com"]);
    git_in(root, &["config", "user.name", "T"]);
    fs::write(root.join("README.md"), "# fixture\n").unwrap();
    change::write_default_policy(root, vec!["true".to_string()]).expect("policy");
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "base"]);
}

/// Drives one change to `Verifying` with committed verification and a current scoped
/// review — the exact state `ship-status` claims is ready and `finalize` is asked to
/// honour. The caller owns the repository and the branch, because the three #743/#689
/// cases differ only in what happens to history afterwards.
pub(super) fn reviewed_change_fixture(root: &Path) -> String {
    let id = approved_change_fixture(root);
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "implement"]);
    // `check_change`, not `verify_change`: the CLI's `change check` materializes the
    // approved deltas as well as verifying, and `finalize` refuses outright on a change
    // whose canonical deltas were never applied. A fixture that only verifies would make
    // `finalize` fail for a reason that has nothing to do with the review — exactly the
    // kind of accidental agreement these tests must not be built on.
    change::check_change(root, Some(&id)).expect("check");
    git_in(root, &["add", "."]);
    git_in(root, &["commit", "-m", "record verification"]);
    change::record_scoped_review_with_verdict(
        root,
        &id,
        "Independent".into(),
        change::ScopedReviewVerdict::Pass,
    )
    .expect("review");
    id
}

/// Drives one change through approval into implementation and stops there, with its
/// workspace still uncommitted: the state `change check --commit` is run from.
pub(super) fn approved_change_fixture(root: &Path) -> String {
    let record = draft_fixture(root);
    let id = record.id.clone();
    for (question, answer) in [
        ("acceptance_criteria", "the fixture verifies"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        change::answer_question(root, &id, question, answer).expect("answer");
    }
    // Fill whatever artifacts the adaptive interview selected; approval refuses on any
    // scaffold left incomplete.
    let workspace = root.join(".specsync/changes").join(&id);
    for entry in fs::read_dir(&workspace).expect("workspace") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) == Some("md")
            && path.file_name().and_then(|n| n.to_str()) != Some("change.md")
        {
            let artifact = path.file_stem().unwrap().to_string_lossy().to_string();
            fs::write(
                &path,
                format!("---\nchange: {id}\nartifact: {artifact}\n---\n\n# {artifact}\n\nFilled by the fixture.\n"),
            )
            .expect("fill artifact");
        }
    }
    change::approve_definition(root, &id, Some("Reviewer".into()), None).expect("approve");
    change::start_implementation(root, &id).expect("implement");
    id
}

/// Untracked files nobody asked the lifecycle to commit, shaped like the ones `git add -A`
/// swept into pushed commits: a private debug archive, an agent's scratch directory, and an
/// experiment script at the project root.
pub(super) const STRAYS: [&str; 3] = ["debug-dump.zip", ".agents/scratch.md", "exp2.sh"];

pub(super) fn write_strays(root: &Path) {
    for stray in STRAYS {
        let path = root.join(stray);
        fs::create_dir_all(path.parent().expect("parent")).expect("stray dir");
        fs::write(&path, format!("not for history: {stray}\n")).expect("stray");
    }
}

/// Every path any commit reachable from any ref in `git_dir` ever touched.
pub(super) fn paths_in_history(git_dir: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .args(["log", "--all", "--name-only", "--pretty=format:"])
        .output()
        .expect("git log");
    assert!(output.status.success(), "git log failed");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn worktree_status(root: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(root)
        .output()
        .expect("git status");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The strays are in no commit of `git_dir` and still sit untracked in `root`, untouched.
pub(super) fn assert_strays_left_alone(root: &Path, git_dir: &Path) {
    let history = paths_in_history(git_dir);
    let status = worktree_status(root);
    for stray in STRAYS {
        assert!(
            !history.iter().any(|path| path == stray),
            "{stray} was committed; a lifecycle commit must never stage an untracked file the change does not own: {history:?}"
        );
        assert!(
            status.contains(&format!("?? {stray}")),
            "{stray} must still be untracked in the working tree: {status:?}"
        );
        assert_eq!(
            fs::read_to_string(root.join(stray)).expect("stray still on disk"),
            format!("not for history: {stray}\n")
        );
    }
}
