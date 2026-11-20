//! The change workflow, `check --fix --backup` and `init-registry` never write outside the
//! repository through a link it carries (SETUP-6, REQ-util-002). A monorepo subproject still
//! follows a link that stays inside its repository, and nothing follows a link into the Git
//! directory.
//!
//! Each fixture is one temporary directory holding `project/` and `outside/` side by side, so a
//! link that leaves the project still reaches nothing but the fixture.

use crate::helpers::*;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Output};
use tempfile::TempDir;

const OWNER: &str = "the owner's bytes\n";
const PLANTED: &str = "PLANTED-BY-THE-REPOSITORY";

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    outside: PathBuf,
}

impl Fixture {
    fn victim(&self, name: &str) -> PathBuf {
        let path = self.outside.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, OWNER).unwrap();
        path
    }

    /// Make `target` (a path inside the project) a relative link to `outside/<name>`.
    fn link_out(&self, target: &str, name: &str) {
        let link = self.root.join(target);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        let _ = fs::remove_file(&link);
        let depth = Path::new(target).components().count();
        symlink(format!("{}outside/{name}", "../".repeat(depth)), &link).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        specsync()
            .arg("--root")
            .arg(&self.root)
            .args(args)
            .output()
            .unwrap()
    }

    fn git(&self, args: &[&str]) {
        git_in(&self.root, args);
    }
}

fn git_in(root: &Path, args: &[&str]) {
    let output = StdCommand::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

const AUTH_SPEC: &str = "---\nmodule: auth\nversion: 1\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n";

/// A Git repository with an `auth` spec, beside an `outside/` directory.
fn change_project() -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("specs/auth/auth.spec.md"), AUTH_SPEC).unwrap();
    fs::write(root.join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
    let fixture = Fixture {
        _tmp: tmp,
        root,
        outside,
    };
    fixture.git(&["init", "-q", "-b", "main"]);
    fixture.git(&["config", "user.email", "writes@example.invalid"]);
    fixture.git(&["config", "user.name", "Writes Test"]);
    fixture.git(&["config", "commit.gpgsign", "false"]);
    fixture.git(&["add", "-A"]);
    fixture.git(&["commit", "-q", "-m", "seed"]);
    fixture
}

/// `change new` in the fixture, returning the change's ID and its selected artifacts.
fn new_change(fixture: &Fixture) -> (String, Vec<String>) {
    let output = fixture.run(&[
        "--json",
        "change",
        "new",
        "Add passkeys",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ]);
    assert!(output.status.success(), "change new: {}", said(&output));
    let created: Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = created["change"]["id"].as_str().unwrap().to_string();
    let artifacts = created["change"]["selected_artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|artifact| artifact.as_str().unwrap().to_string())
        .collect();
    (id, artifacts)
}

fn workspace(fixture: &Fixture, id: &str) -> PathBuf {
    fixture.root.join(format!(".specsync/changes/{id}"))
}

// ─── the change workflow (F2) ──────────────────────────────────────────

/// Verifies REQ-util-002 and REQ-change-163. `change answer` rewrites `change.md`; a committed
/// `change.md` linked outside the repository is refused before anything is written, so the file
/// it names keeps its bytes and the change keeps its state.
#[test]
fn change_answer_never_writes_change_md_through_a_link_out() {
    let fixture = change_project();
    let (id, _) = new_change(&fixture);
    let victim = fixture.victim("profile");
    fixture.link_out(&format!(".specsync/changes/{id}/change.md"), "profile");
    let state = fs::read(workspace(&fixture, &id).join("state.json")).unwrap();

    let output = fixture.run(&["change", "answer", &id, "acceptance_criteria", PLANTED]);

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(said(&output).contains("change.md"), "{}", said(&output));
    assert_eq!(
        fs::read(workspace(&fixture, &id).join("state.json")).unwrap(),
        state,
        "the change was saved before the refusal"
    );
}

/// Verifies REQ-util-002 and REQ-change-163. `change approve` on a change the repository
/// committed, with `change.md` linked outside it: the outside file keeps its bytes and no
/// approval is recorded.
#[test]
fn change_approve_never_writes_change_md_through_a_link_out() {
    let fixture = change_project();
    let (id, artifacts) = new_change(&fixture);
    for (question, answer) in [
        ("acceptance_criteria", "Passkeys enroll"),
        ("public_contract", "no"),
        ("architecture_risk", "no"),
    ] {
        let output = fixture.run(&["change", "answer", &id, question, answer]);
        assert!(output.status.success(), "{}", said(&output));
    }
    let dir = workspace(&fixture, &id);
    for name in &artifacts {
        let content = match name.as_str() {
            "tasks" => "# Tasks\n\n- [x] Complete verification preparation.\n",
            "plan" => SUBSTANTIVE_PLAN,
            _ => "# Complete\n\nReviewed lifecycle evidence.\n",
        };
        fs::write(dir.join(format!("{name}.md")), content).unwrap();
    }
    fs::create_dir_all(dir.join("deltas")).unwrap();
    fs::write(
        dir.join("deltas/auth.md"),
        "## ADDED\n\n### REQUIREMENT REQ-auth-900\n\nThe system SHALL hold.\n\nAcceptance Criteria\n- holds.\n",
    )
    .unwrap();
    let victim = fixture.victim("profile");
    fixture.link_out(&format!(".specsync/changes/{id}/change.md"), "profile");
    let approvals = fs::read(dir.join("approvals.json")).unwrap();

    let output = fixture.run(&["change", "approve", &id, "--actor", "Maintainer"]);

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(said(&output).contains("change.md"), "{}", said(&output));
    assert_eq!(
        fs::read(dir.join("approvals.json")).unwrap(),
        approvals,
        "an approval was recorded before the refusal"
    );
}

/// Verifies REQ-util-002 and REQ-change-163. A committed artifact that is a dangling link out of
/// the repository is never created through by `change answer`.
#[test]
fn change_answer_never_creates_the_target_of_a_dangling_artifact_link() {
    let fixture = change_project();
    let (id, artifacts) = new_change(&fixture);
    let artifact = format!(".specsync/changes/{id}/{}.md", artifacts[0]);
    fixture.link_out(&artifact, "planted.md");

    let output = fixture.run(&["change", "answer", &id, "acceptance_criteria", PLANTED]);

    assert!(
        !fixture.outside.join("planted.md").exists(),
        "the dangling artifact link was created through: {}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(said(&output).contains(&artifacts[0]), "{}", said(&output));
}

/// Verifies REQ-util-002 and REQ-change-163. A committed `.specsync/changes` linked to a
/// directory outside the repository receives no change workspace from `change new`.
#[test]
fn change_new_creates_nothing_in_a_changes_directory_linked_out() {
    let fixture = change_project();
    fs::create_dir_all(fixture.outside.join("changes")).unwrap();
    fs::create_dir_all(fixture.root.join(".specsync")).unwrap();
    fixture.link_out(".specsync/changes", "changes");

    let output = fixture.run(&[
        "change",
        "new",
        "Add passkeys",
        "--spec",
        "auth",
        "--path",
        "src/auth.rs",
    ]);

    assert_eq!(
        names(&fixture.outside.join("changes")),
        Vec::<String>::new(),
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains(".specsync/changes"),
        "{}",
        said(&output)
    );
}

// ─── check --fix --backup (F3) ─────────────────────────────────────────

/// A project whose `auth` spec misses the `logout` export, so `check --fix` has work to do.
fn fix_project() -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    write_config(&root, "specs", &["src"]);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    Fixture {
        _tmp: tmp,
        root,
        outside,
    }
}

/// Verifies REQ-util-002 and REQ-cmd-check-020. A committed `.specsync/backup-fix` linked to a
/// directory outside the repository gets no backup, and `--fix` stops before it changes a spec.
#[test]
fn check_fix_backup_copies_nothing_through_a_linked_backup_directory() {
    let fixture = fix_project();
    fs::create_dir_all(fixture.outside.join("backup")).unwrap();
    fixture.link_out(".specsync/backup-fix", "backup");
    let spec = fs::read(fixture.root.join("specs/auth/auth.spec.md")).unwrap();

    let output = fixture.run(&["check", "--fix", "--backup", "--force"]);

    assert_eq!(
        names(&fixture.outside.join("backup")),
        Vec::<String>::new(),
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(said(&output).contains("backup-fix"), "{}", said(&output));
    assert_eq!(
        fs::read(fixture.root.join("specs/auth/auth.spec.md")).unwrap(),
        spec,
        "--fix went on without its backup"
    );
}

/// Verifies REQ-util-002 and REQ-cmd-check-020. A committed backup file linked to a file outside
/// the repository keeps the outside file's bytes.
#[test]
fn check_fix_backup_never_copies_over_a_linked_backup_file() {
    let fixture = fix_project();
    let victim = fixture.victim("profile");
    fixture.link_out(".specsync/backup-fix/specs/auth/auth.spec.md", "profile");

    let output = fixture.run(&["check", "--fix", "--backup", "--force"]);

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
}

/// Verifies REQ-cmd-check-020. CONTROL: with no link, the backup is written where it always was
/// and the fix goes on.
#[test]
fn check_fix_backup_still_backs_up_a_project_with_no_link() {
    let fixture = fix_project();
    let spec = fs::read(fixture.root.join("specs/auth/auth.spec.md")).unwrap();

    let output = fixture.run(&["check", "--fix", "--backup", "--force"]);

    assert_eq!(
        fs::read(
            fixture
                .root
                .join(".specsync/backup-fix/specs/auth/auth.spec.md")
        )
        .unwrap(),
        spec,
        "{}",
        said(&output)
    );
    assert!(
        fs::read_to_string(fixture.root.join("specs/auth/auth.spec.md"))
            .unwrap()
            .contains("logout"),
        "{}",
        said(&output)
    );
}

// ─── init-registry (F8) ────────────────────────────────────────────────

/// Verifies REQ-util-002 and REQ-cmd-init-registry-004. A committed `.specsync` linked to a
/// directory outside the repository gets no `registry.toml`.
#[test]
fn init_registry_creates_nothing_in_a_specsync_directory_linked_out() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(outside.join("dotspecsync")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    symlink("../outside/dotspecsync", root.join(".specsync")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&root)
        .arg("init-registry")
        .output()
        .unwrap();

    assert_eq!(
        names(&outside.join("dotspecsync")),
        Vec::<String>::new(),
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
}

// ─── a monorepo subproject (R1) and absolute links (R2) ────────────────

/// A Git repository with shared `AGENTS.md` and `specs/app/`, and a subproject at
/// `packages/app` that links to both, as monorepos do.
fn monorepo() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    let app = repo.join("packages/app");
    fs::create_dir_all(repo.join("specs/app/auth")).unwrap();
    fs::create_dir_all(app.join("src/auth")).unwrap();
    fs::create_dir_all(app.join(".specsync")).unwrap();
    fs::write(repo.join("AGENTS.md"), "# Agents\n\nShared instructions.\n").unwrap();
    fs::write(
        repo.join("specs/app/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    fs::write(
        app.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::write(
        app.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    symlink("../../AGENTS.md", app.join("CLAUDE.md")).unwrap();
    symlink("../../specs/app", app.join("specs")).unwrap();
    git_in(&repo, &["init", "-q", "-b", "main"]);
    // A monorepo has its subprojects in Git; the confinement widens to the work tree only for a
    // project Git tracks there (REQ-util-007).
    git_in(&repo, &["add", "-A"]);
    (tmp, repo, app)
}

/// Verifies REQ-util-004. Inside a Git work tree, a subproject's link that stays inside the
/// repository is written through, as it was before 6.4.0.
#[test]
fn a_subproject_writes_through_a_link_that_stays_inside_its_repository() {
    let (_tmp, repo, app) = monorepo();

    let output = specsync()
        .arg("--root")
        .arg(&app)
        .args(["hooks", "install", "--claude"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", said(&output));
    assert!(
        fs::read_to_string(repo.join("AGENTS.md"))
            .unwrap()
            .contains("specsync:hook:begin"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-util-004. A subproject reads its specs through a `specs` link that stays inside
/// the repository, as it did before 6.4.0.
#[test]
fn a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository() {
    let (_tmp, _repo, app) = monorepo();

    let output = specsync()
        .arg("--root")
        .arg(&app)
        .arg("deps")
        .output()
        .unwrap();

    assert!(said(&output).contains("Modules: 1"), "{}", said(&output));
}

/// Verifies REQ-util-004. CONTROL: the repository is the limit. A subproject link that leaves
/// the repository is still refused.
#[test]
fn a_subproject_link_that_leaves_its_repository_is_still_refused() {
    let (tmp, _repo, app) = monorepo();
    let victim = tmp.path().join("profile");
    fs::write(&victim, OWNER).unwrap();
    fs::remove_file(app.join("CLAUDE.md")).unwrap();
    symlink("../../../profile", app.join("CLAUDE.md")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&app)
        .args(["hooks", "install", "--claude"])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(
        said(&output).contains("leaves the project"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-util-004. An absolute link is never followed, and the refusal says it is an
/// absolute link rather than that it leaves the project.
#[test]
fn an_absolute_link_is_refused_as_an_absolute_link() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("AGENTS.md"), "# Agents\n").unwrap();
    symlink(root.join("AGENTS.md"), root.join("CLAUDE.md")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&root)
        .args(["hooks", "install", "--claude"])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "# Agents\n"
    );
    assert!(said(&output).contains("absolute link"), "{}", said(&output));
}

// ─── the Git directory ─────────────────────────────────────────────────

/// Verifies REQ-util-005. A link that stays inside the project but leads into the Git directory
/// is never written through: Git runs hooks and reads its config from there.
#[test]
fn hooks_install_never_writes_through_a_link_into_the_git_directory() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    git_in(&root, &["init", "-q", "-b", "main"]);
    let hook = root.join(".git/hooks/post-checkout");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    let config = fs::read(root.join(".git/config")).unwrap();
    symlink(".git/hooks/post-checkout", root.join("CLAUDE.md")).unwrap();
    symlink(".GIT/config", root.join("AGENTS.md")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&root)
        .args(["hooks", "install", "--claude", "--agents"])
        .output()
        .unwrap();

    assert_eq!(fs::read_to_string(&hook).unwrap(), "#!/bin/sh\nexit 0\n");
    assert_eq!(fs::read(root.join(".git/config")).unwrap(), config);
    assert!(said(&output).contains("Git directory"), "{}", said(&output));
}

/// Verifies REQ-util-005 and REQ-change-163. A committed `change.md` linked to an existing hook
/// keeps the hook's bytes: the text a repository chose for its change never becomes a hook.
#[test]
fn change_answer_never_writes_change_md_into_a_hook() {
    let fixture = change_project();
    let (id, _) = new_change(&fixture);
    let hook = fixture.root.join(".git/hooks/pre-commit");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    let link = fixture
        .root
        .join(format!(".specsync/changes/{id}/change.md"));
    fs::remove_file(&link).unwrap();
    symlink("../../../.git/hooks/pre-commit", &link).unwrap();

    let output = fixture.run(&["change", "answer", &id, "acceptance_criteria", PLANTED]);

    assert_eq!(
        fs::read_to_string(&hook).unwrap(),
        "#!/bin/sh\nexit 0\n",
        "{}",
        said(&output)
    );
    assert!(!output.status.success(), "{}", said(&output));
    assert!(said(&output).contains("Git directory"), "{}", said(&output));
}

// ─── the hooks folder Git names (change 6: F6) ─────────────────────────

/// A Git repository whose hooks folder is set inside the work tree with `core.hooksPath`, as
/// husky does and as many READMEs tell people to, beside an `outside/` directory.
fn hooks_path_project(hooks_path: &str) -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    git_in(&root, &["init", "-q", "-b", "main"]);
    git_in(&root, &["config", "core.hooksPath", hooks_path]);
    (tmp, root, outside)
}

fn install_precommit(root: &Path) -> Output {
    specsync()
        .arg("--root")
        .arg(root)
        .args(["hooks", "install", "--precommit"])
        .output()
        .unwrap()
}

/// Verifies REQ-hooks-009. With `core.hooksPath` inside the work tree, the hooks folder is
/// repository content: a committed link on the way to it must not take the hook, or its chmod to
/// 0755, outside.
#[test]
fn hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path() {
    let (_tmp, root, outside) = hooks_path_project(".config/hooks");
    symlink("../outside", root.join(".config")).unwrap();

    let output = install_precommit(&root);

    assert_eq!(names(&outside), Vec::<String>::new(), "{}", said(&output));
    assert!(!output.status.success(), "{}", said(&output));
}

/// Verifies REQ-hooks-009. A dangling `pre-commit` link in a committed hooks folder must not
/// create its target.
#[test]
fn hooks_install_never_creates_the_target_of_a_dangling_pre_commit_link() {
    let (_tmp, root, outside) = hooks_path_project(".githooks");
    fs::create_dir_all(root.join(".githooks")).unwrap();
    symlink("../../outside/created", root.join(".githooks/pre-commit")).unwrap();

    let output = install_precommit(&root);

    assert_eq!(names(&outside), Vec::<String>::new(), "{}", said(&output));
    assert!(!output.status.success(), "{}", said(&output));
}

/// Verifies REQ-hooks-009. CONTROL: a real hooks folder in the work tree gets an executable
/// `pre-commit`.
#[test]
fn hooks_install_installs_into_a_hooks_folder_in_the_work_tree() {
    use std::os::unix::fs::PermissionsExt;
    let (_tmp, root, _outside) = hooks_path_project(".githooks");
    fs::create_dir_all(root.join(".githooks")).unwrap();

    let output = install_precommit(&root);

    assert!(output.status.success(), "{}", said(&output));
    let hook = root.join(".githooks/pre-commit");
    assert!(
        fs::read_to_string(&hook).unwrap().contains("specsync"),
        "{}",
        said(&output)
    );
    assert_ne!(
        fs::metadata(&hook).unwrap().permissions().mode() & 0o111,
        0,
        "the hook is not executable"
    );
}

// ─── a project inside a work tree that is not its own (change 6: F7) ───

/// Verifies REQ-util-007. A project unpacked inside another work tree, such as a home folder
/// kept under Git, is not part of that repository: a link of its own to a file of the outer
/// work tree leaves the project and is refused, and the file keeps its bytes.
#[test]
fn an_untracked_project_inside_another_work_tree_is_confined_to_itself() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = home.join("Downloads/project");
    fs::create_dir_all(project.join(".specsync")).unwrap();
    fs::write(home.join("notes.md"), OWNER).unwrap();
    git_in(&home, &["init", "-q", "-b", "main"]);
    git_in(&home, &["add", "notes.md"]);
    symlink("../../notes.md", project.join("CLAUDE.md")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&project)
        .args(["hooks", "install", "--claude"])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(home.join("notes.md")).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(
        said(&output).contains("leaves the project"),
        "{}",
        said(&output)
    );
}

// ─── literal pathspecs and the hooks path's own config (change 7) ───────

/// Verifies REQ-git-utils-012 and REQ-util-007. Git reads a pathspec as a glob unless told
/// otherwise, so a ZIP whose folder is named `*`, unpacked where the enclosing work tree tracks
/// other files, must not count as tracked: the project is still confined to itself.
#[test]
fn a_project_folder_named_like_a_glob_is_still_confined_to_itself() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = home.join("Downloads/*");
    fs::create_dir_all(project.join(".specsync")).unwrap();
    fs::write(home.join("notes.md"), OWNER).unwrap();
    fs::write(home.join("Downloads/readme.txt"), "tracked\n").unwrap();
    git_in(&home, &["init", "-q", "-b", "main"]);
    git_in(&home, &["add", "notes.md", "Downloads/readme.txt"]);
    symlink("../../notes.md", project.join("CLAUDE.md")).unwrap();

    let output = specsync()
        .arg("--root")
        .arg(&project)
        .args(["hooks", "install", "--claude"])
        .output()
        .unwrap();

    assert_eq!(
        fs::read_to_string(home.join("notes.md")).unwrap(),
        OWNER,
        "{}",
        said(&output)
    );
    assert!(
        said(&output).contains("leaves the project"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-hooks-009. A `core.hooksPath` in the repository's own config (which a tarball's
/// `.git/config` can set) that points outside the work tree must not make SpecSync create
/// folders there.
#[test]
fn hooks_install_makes_no_folder_for_a_local_hooks_path_outside_the_work_tree() {
    let (_tmp, root, outside) = hooks_path_project("../outside/made/hooks");

    let output = install_precommit(&root);

    assert_eq!(names(&outside), Vec::<String>::new(), "{}", said(&output));
    assert!(!output.status.success(), "{}", said(&output));
}

/// Verifies REQ-hooks-009. CONTROL: a hooks folder the person set in their own global config
/// is theirs, so a missing one is still made.
#[test]
fn hooks_install_still_makes_a_hooks_folder_from_the_global_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let global_hooks = tmp.path().join("global-hooks/hooks");
    fs::create_dir_all(&root).unwrap();
    git_in(&root, &["init", "-q", "-b", "main"]);
    let global = tmp.path().join("gitconfig");
    fs::write(
        &global,
        format!("[core]\n\thooksPath = {}\n", global_hooks.display()),
    )
    .unwrap();

    let output = specsync()
        .env("GIT_CONFIG_GLOBAL", &global)
        .arg("--root")
        .arg(&root)
        .args(["hooks", "install", "--precommit"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", said(&output));
    assert!(
        global_hooks.join("pre-commit").is_file(),
        "{}",
        said(&output)
    );
}

// ─── the write scan (G1, change 6: F4) ────────────────────────────────

/// Files whose every write is exempt, each with the reason. Everything else marks each direct
/// write on its own line or the line above (REQ-util-002).
const WRITE_ALLOWED_FILES: &[(&str, &str)] = &[
    (
        "src/util.rs",
        "the confined project write helpers live here",
    ),
    (
        "src/util/local_state.rs",
        "SpecSync's own state folder outside the work tree, written through no-follow handles (REQ-util-006)",
    ),
    (
        "src/mcp/publication.rs",
        "publishes MCP output through a staged, confined directory handle",
    ),
    (
        "src/mcp/snapshot.rs",
        "writes only the MCP server's snapshot copy, in its own temporary directory",
    ),
];

/// A direct write elsewhere carries one of these markers, with the reason: the file is not a
/// project file, or its path was refused as a link just before.
const WRITE_MARKERS: &[&str] = &["not a project file write:", "checked project write:"];

/// Calls that write, create, rename, link or remove a file or a directory (REQ-util-008). A
/// pattern that starts with a name matches only where no longer name ends in it.
const WRITE_CALLS: &[&str] = &[
    "fs::write(",
    "fs::copy(",
    "File::create(",
    ".write(true)",
    ".create(true)",
    ".create_new(true)",
    ".append(true)",
    ".truncate(true)",
    "create_dir_all(",
    "create_dir(",
    "fs::rename(",
    ".persist(",
    "persist_noclobber(",
    "remove_file(",
    "remove_dir(",
    "remove_dir_all(",
    "set_permissions(",
    "set_len(",
    "hard_link(",
    "symlink(",
    "symlink_file(",
    "symlink_dir(",
    "tempfile_in(",
    "NamedTempFile::new_in(",
    "tempdir_in(",
    "TempDir::new_in(",
    ".rename(",
    ".copy(",
    ".create(",
];

/// Free functions of `std::fs` that write, which an import would let a call name bare.
const WRITE_FUNCTIONS: &[&str] = &[
    "write",
    "copy",
    "rename",
    "remove_file",
    "remove_dir",
    "remove_dir_all",
    "create_dir",
    "create_dir_all",
    "hard_link",
    "soft_link",
    "set_permissions",
];

/// Verifies REQ-util-002, REQ-util-008 and the module requirements that rely on them. Every
/// write to a project file goes through the confined helpers; a new direct write fails here
/// until it does, or until it says why its path is not a project file or was checked just
/// before.
#[test]
fn no_project_file_is_written_outside_the_confined_helpers() {
    use crate::project_reads::{
        code_only, compiled_sources, has_call, has_dir_file_call, imports_fs_function,
        imports_fs_under_another_name, marked_in_statement, non_test_lines,
    };
    let mut direct = Vec::new();
    for (relative, path) in compiled_sources() {
        if WRITE_ALLOWED_FILES
            .iter()
            .any(|(file, _)| *file == relative)
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let code = code_only(&text);
        let lines = non_test_lines(&text, &code);
        for (position, line) in lines.iter().enumerate() {
            if !has_call(line.code, WRITE_CALLS)
                && !has_dir_file_call(line.code, "write")
                && !imports_fs_function(line.code, WRITE_FUNCTIONS)
                && !imports_fs_under_another_name(line.code)
            {
                continue;
            }
            if marked_in_statement(&lines, position, WRITE_MARKERS) {
                continue;
            }
            direct.push(format!("{relative}:{}: {}", line.number, line.raw.trim()));
        }
    }
    assert!(
        direct.is_empty(),
        "{} direct file write(s) bypass the confined project helpers (REQ-util-002):\n{}",
        direct.len(),
        direct.join("\n")
    );
}

/// Git arguments that make Git write files of its own choosing: a work tree, a checkout, a reset
/// or a stash of the project's files, an output file, a bundle or patches.
const GIT_WRITING_ARGUMENTS: &[&str] = &[
    "worktree",
    "checkout",
    "restore",
    "stash",
    "reset",
    "clean",
    "bundle",
    "format-patch",
    "-o",
];

/// A spawned Git command that writes files carries this marker in its statement, with the reason
/// no project file is written.
const GIT_MARKER: &str = "git writes no project file:";

/// The string literals on a line as written, without their quotes. Escapes are kept as they are.
fn string_literals(raw: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let bytes = raw.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'"' {
            let start = at + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'"' {
                end += if bytes[end] == b'\\' { 2 } else { 1 };
            }
            found.push(&raw[start..end.min(raw.len())]);
            at = end + 1;
        } else {
            at += 1;
        }
    }
    found
}

/// Verifies REQ-util-008. A Git command SpecSync spawns can write files the scans above never
/// see (`git worktree add`, `git checkout`, `--output=`): each one says, in its statement, why
/// no project file is written. A statement counts as a Git command when it names `git`.
#[test]
fn no_spawned_git_command_writes_files_unmarked() {
    use crate::project_reads::{code_only, compiled_sources, marked_in_statement, non_test_lines};
    let names_git = |raw: &str, code: &str| {
        raw.contains("\"git\"")
            || code
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|word| word == "git" || word.starts_with("git_") || word.ends_with("_git"))
    };
    let mut unmarked = Vec::new();
    for (relative, path) in compiled_sources() {
        let text = fs::read_to_string(&path).unwrap();
        let code = code_only(&text);
        let lines = non_test_lines(&text, &code);
        for (position, line) in lines.iter().enumerate() {
            if line.code.trim().is_empty() {
                continue;
            }
            let writes = string_literals(line.raw).iter().any(|literal| {
                GIT_WRITING_ARGUMENTS.contains(literal) || literal.starts_with("--output")
            });
            if !writes {
                continue;
            }
            // The statement this argument belongs to, back to the end of the one before.
            let mut statement_names_git = names_git(line.raw, line.code);
            for earlier in lines[..position].iter().rev() {
                let ended = earlier.code.trim_end();
                if ended.ends_with(';') || ended.ends_with('{') || ended.ends_with('}') {
                    break;
                }
                statement_names_git |= names_git(earlier.raw, earlier.code);
            }
            if !statement_names_git || marked_in_statement(&lines, position, &[GIT_MARKER]) {
                continue;
            }
            unmarked.push(format!("{relative}:{}: {}", line.number, line.raw.trim()));
        }
    }
    assert!(
        unmarked.is_empty(),
        "{} spawned Git argument(s) that write files say nothing about where (REQ-util-008):\n{}",
        unmarked.len(),
        unmarked.join("\n")
    );
}
