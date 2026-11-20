// 6.4.1: the low items left after GHSA-7q5v-8jcc-rx54, from sec-review2's re-check of changes 7
// and 8 (W1, S1, S2, S3, L2, X2), corvid-agent's review of #1133 (H1) and #1135 (C2 to C4). Each
// test here failed on 6.4.0.

use crate::helpers::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Output};
use std::time::Duration;
use tempfile::TempDir;

/// Long enough for any command here on a loaded machine, short enough to fail fast on a hang.
const TIMEOUT: Duration = Duration::from_secs(20);

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Run `specsync <args> --root <root>` under the timeout. Panics if the command had to be killed:
/// that is a read that waited on a file instead of refusing it.
fn run_in(root: &Path, args: &[&str]) -> Output {
    run_with_env(root, args, &[])
}

fn run_with_env(root: &Path, args: &[&str], env: &[(&str, &Path)]) -> Output {
    let mut command = specsync();
    command.args(args).arg("--root").arg(root).timeout(TIMEOUT);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.code().is_some(),
        "`specsync {}` was still running after {TIMEOUT:?}: a read waited on a file",
        args.join(" ")
    );
    output
}

fn git(root: &Path, args: &[&str]) {
    let status = StdCommand::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.name=hardening test",
            "-c",
            "user.email=hardening@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

fn mkfifo(path: &Path) {
    let _ = fs::remove_file(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let status = StdCommand::new("mkfifo").arg(path).status().unwrap();
    assert!(status.success(), "mkfifo {}", path.display());
}

/// Removes a path in the shared test cache folder when the test ends, pass or fail.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Fx {
    tmp: TempDir,
    root: PathBuf,
    outside: PathBuf,
}

impl Fx {
    fn run(&self, args: &[&str]) -> Output {
        run_in(&self.root, args)
    }
}

/// A project with one spec, set up by `init`, beside an `outside/` directory.
fn project() -> Fx {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    make_project(&root);
    Fx { tmp, root, outside }
}

fn make_project(root: &Path) {
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    let init = run_in(root, &["init"]);
    assert!(init.status.success(), "{}", said(&init));
}

/// [`project`] with the change workflow switched on, in a Git repository with one commit.
fn lifecycle_project() -> Fx {
    let fx = project();
    let adopt = fx.run(&["change", "adopt"]);
    assert!(adopt.status.success(), "{}", said(&adopt));
    git(&fx.root, &["init", "-q", "-b", "main"]);
    git(&fx.root, &["add", "-A"]);
    git(&fx.root, &["commit", "-q", "-m", "seed"]);
    fx
}

/// The journal exactly as SpecSync 6.4.0 writes one, with its integrity envelope.
fn durable_journal(entries: &[(&str, Option<&str>)]) -> String {
    use sha2::{Digest, Sha256};
    let rendered: Vec<String> = entries
        .iter()
        .map(|(path, original)| {
            format!(
                r#"{{"path":{},"original":{}}}"#,
                serde_json::to_string(path).unwrap(),
                serde_json::to_string(original).unwrap()
            )
        })
        .collect();
    let array = format!("[{}]", rendered.join(","));
    let digest = format!("{:x}", Sha256::digest(array.as_bytes()));
    format!(
        r#"{{"schema_version":1,"entry_count":{},"entries_digest":"{digest}","entries":{array}}}"#,
        entries.len()
    )
}

/// Where SpecSync keeps what it knows about the Git repository whose common directory is
/// `common_dir`: `specsync/repositories/<key>` in the test cache folder (REQ-util-009).
fn repository_state_folder(common_dir: &Path) -> PathBuf {
    use sha2::{Digest, Sha256};
    let canonical = common_dir.canonicalize().unwrap();
    let mut hasher = Sha256::new();
    hasher.update(b"repository\0");
    hasher.update(canonical.as_os_str().as_encoded_bytes());
    let key = format!("{:x}", hasher.finalize())[..16].to_string();
    test_cache_home()
        .join("specsync")
        .join("repositories")
        .join(key)
}

// ─── W1: listings of SpecSync's own record folders ──────────────────────

/// W1. A committed `.specsync/archive/changes` that links out of the project is refused, naming
/// the folder; the names in the folder it leads to are never listed or printed.
#[test]
fn archived_changes_linked_out_of_the_project_are_never_listed() {
    let fx = lifecycle_project();
    let outside_change = fx.outside.join("changes/2026-01-01-outside-secret-name");
    fs::create_dir_all(&outside_change).unwrap();
    fs::write(outside_change.join("state.json"), "{}").unwrap();
    fs::create_dir_all(fx.root.join(".specsync/archive")).unwrap();
    let _ = fs::remove_dir_all(fx.root.join(".specsync/archive/changes"));
    symlink(
        "../../../outside/changes",
        fx.root.join(".specsync/archive/changes"),
    )
    .unwrap();

    let output = fx.run(&["change", "audit", "--archives"]);

    assert!(
        !said(&output).contains("outside-secret-name"),
        "a folder outside the project was listed: {}",
        said(&output)
    );
    assert!(
        said(&output).contains(".specsync/archive/changes"),
        "the refusal names the folder: {}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
}

// ─── S1, X2, H1: the hooks folder ───────────────────────────────────────

/// S1. A `core.hooksPath` the repository's own config names, outside the work tree and the Git
/// directory, gets no hook, even when that folder already exists.
#[test]
fn hooks_install_writes_no_hook_into_an_existing_folder_the_repository_config_names() {
    let fx = project();
    let bin = fx.outside.join("bin");
    fs::create_dir_all(&bin).unwrap();
    git(&fx.root, &["init", "-q", "-b", "main"]);
    git(
        &fx.root,
        &["config", "core.hooksPath", bin.to_str().unwrap()],
    );

    let output = fx.run(&["hooks", "install", "--precommit"]);

    assert!(
        !bin.join("pre-commit").exists(),
        "a hook was written outside the tree: {}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
}

/// X2. An absolute `core.hooksPath` spelled through a link to the work tree (macOS's `/var` for
/// `/private/var`) lies inside the work tree, so its missing hooks folder is made there.
#[cfg(target_os = "macos")]
#[test]
fn a_hooks_path_spelled_through_a_link_to_the_work_tree_is_inside_it() {
    let fx = project();
    assert!(
        !fx.root.starts_with("/private"),
        "the test needs a temporary folder spelled other than canonically"
    );
    git(&fx.root, &["init", "-q", "-b", "main"]);
    let hooks = fx.root.join(".githooks");
    git(
        &fx.root,
        &["config", "core.hooksPath", hooks.to_str().unwrap()],
    );

    let output = fx.run(&["hooks", "install", "--precommit"]);

    assert!(output.status.success(), "{}", said(&output));
    assert!(hooks.join("pre-commit").is_file(), "{}", said(&output));
}

/// H1. In a linked worktree, Git names the hooks folder in the main repository's Git directory;
/// with no `hooks` folder there, `hooks install --precommit` makes it, as 6.3.0 did.
#[test]
fn hooks_install_in_a_linked_worktree_makes_the_shared_hooks_folder() {
    let tmp = TempDir::new().unwrap();
    let main = tmp.path().join("main");
    let worktree = tmp.path().join("worktree");
    make_project(&main);
    git(&main, &["init", "-q", "-b", "main"]);
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "seed"]);
    git(
        &main,
        &["worktree", "add", "-q", worktree.to_str().unwrap()],
    );
    let _ = fs::remove_dir_all(main.join(".git/hooks"));

    let output = run_in(&worktree, &["hooks", "install", "--precommit"]);

    assert!(output.status.success(), "{}", said(&output));
    assert!(
        main.join(".git/hooks/pre-commit").is_file(),
        "{}",
        said(&output)
    );
}

// ─── S2: state that outlives its project ────────────────────────────────

/// S2. A transaction journal an interrupted write left for an earlier project is never replayed
/// into another project later placed at the same path.
#[test]
fn a_journal_left_for_an_earlier_project_at_the_same_path_is_not_replayed() {
    let fx = lifecycle_project();
    // Project A's interrupted write left its journal in the local state folder.
    let journal = state_folder(&fx.root).join("change-transaction.json");
    let _cleanup = Cleanup(journal.clone());
    // Project B replaces A at the same path.
    fs::rename(&fx.root, fx.tmp.path().join("project-a")).unwrap();
    make_project(&fx.root);
    let adopt = fx.run(&["change", "adopt"]);
    assert!(adopt.status.success(), "{}", said(&adopt));
    fs::create_dir_all(journal.parent().unwrap()).unwrap();
    fs::write(
        &journal,
        durable_journal(&[(
            ".specsync/changes/from-a/state.json",
            Some("{\"from\":\"a\"}"),
        )]),
    )
    .unwrap();

    let output = fx.run(&["change", "new", "A change in project B"]);

    assert!(
        !fx.root.join(".specsync/changes/from-a/state.json").exists(),
        "project A's journal was replayed into project B: {}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("change-transaction.json"),
        "{}",
        said(&output)
    );
}

// ─── S3: paths the person types ─────────────────────────────────────────

/// S3. `change correct-owner --manifest` given a FIFO returns an error instead of waiting on it.
#[test]
fn correct_owner_does_not_wait_on_a_fifo_manifest() {
    let fx = lifecycle_project();
    let manifest = fx.outside.join("manifest.json");
    mkfifo(&manifest);

    let output = fx.run(&[
        "change",
        "correct-owner",
        "CHG-0001-anything",
        "--manifest",
        manifest.to_str().unwrap(),
        "--actor",
        "Ada Reviewer",
        "--reason",
        "a manifest that is a FIFO",
    ]);

    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("not a regular file"),
        "{}",
        said(&output)
    );
}

/// S3. `scaffold --template` given a folder whose `spec.md` is a FIFO returns an error instead of
/// waiting on it.
#[test]
fn scaffold_does_not_wait_on_a_fifo_template() {
    let fx = project();
    let template = fx.outside.join("template");
    mkfifo(&template.join("spec.md"));

    let output = fx.run(&[
        "scaffold",
        "billing",
        "--template",
        template.to_str().unwrap(),
    ]);

    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("not a regular file"),
        "{}",
        said(&output)
    );
}

// ─── L2: a state folder others can write ────────────────────────────────

/// L2. A local state folder other users can write is refused: `check` says why and validates
/// every spec instead of trusting a cache someone else could have planted.
#[test]
fn a_state_folder_others_can_write_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let fx = project();
    let first = fx.run(&["check"]);
    assert!(first.status.success(), "{}", said(&first));
    let folder = state_folder(&fx.root);
    assert!(folder.is_dir(), "{}", said(&first));
    let _cleanup = Cleanup(folder.clone());
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o777)).unwrap();

    let second = fx.run(&["check"]);

    assert!(
        said(&second).contains("hash cache not used"),
        "{}",
        said(&second)
    );
    assert!(said(&second).contains("writable"), "{}", said(&second));
}

// ─── C2 to C4 (#1135) ───────────────────────────────────────────────────

/// C2. `check --only-status` keeps a spec the bounded read refuses, with its refusal, instead of
/// dropping it as if its status did not match.
#[test]
fn only_status_keeps_a_refused_spec_with_its_refusal() {
    let fx = project();
    fs::create_dir_all(fx.root.join("specs/big")).unwrap();
    fs::File::create(fx.root.join("specs/big/big.spec.md"))
        .unwrap()
        .set_len(9 * 1024 * 1024)
        .unwrap();

    let output = fx.run(&["check", "--only-status", "active"]);

    assert!(said(&output).contains("big.spec.md"), "{}", said(&output));
    assert!(said(&output).contains("larger than"), "{}", said(&output));
    assert!(!output.status.success(), "{}", said(&output));
}

/// C3. `resolve` reports a local spec the bounded read refuses, naming it, instead of skipping it
/// and its `depends_on` silently.
#[test]
fn resolve_reports_a_refused_spec() {
    let fx = project();
    fs::create_dir_all(fx.root.join("specs/big")).unwrap();
    fs::File::create(fx.root.join("specs/big/big.spec.md"))
        .unwrap()
        .set_len(9 * 1024 * 1024)
        .unwrap();

    let output = fx.run(&["resolve"]);

    assert!(said(&output).contains("big.spec.md"), "{}", said(&output));
    assert!(said(&output).contains("larger than"), "{}", said(&output));
}

/// C4. `stale` counts a spec the bounded read refuses as unmeasurable, so its summary never says
/// every spec is up to date.
#[test]
fn stale_counts_a_refused_spec_as_unmeasurable() {
    let fx = project();
    fs::create_dir_all(fx.root.join("specs/big")).unwrap();
    fs::File::create(fx.root.join("specs/big/big.spec.md"))
        .unwrap()
        .set_len(9 * 1024 * 1024)
        .unwrap();
    git(&fx.root, &["init", "-q", "-b", "main"]);
    git(&fx.root, &["add", "-A"]);
    git(&fx.root, &["commit", "-q", "-m", "seed"]);

    let output = fx.run(&["stale"]);

    assert!(said(&output).contains("big.spec.md"), "{}", said(&output));
    assert!(said(&output).contains("larger than"), "{}", said(&output));
}
