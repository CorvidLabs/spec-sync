//! A command run in a repository somebody else wrote never writes outside it through a link it
//! found there (SETUP-6, REQ-util-002). Git stores symbolic links, so every path a command writes
//! is a path the repository's author could have made a link.
//!
//! Each fixture is one temporary directory holding `project/` and `outside/` side by side, so a
//! link that leaves the project still reaches nothing but the fixture.

use crate::helpers::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

const OWNER: &str = "the owner's bytes\n";

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    outside: PathBuf,
}

impl Fixture {
    /// A file outside the project that must keep these bytes.
    fn victim(&self, name: &str, content: &str) -> PathBuf {
        let path = self.outside.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    /// Make `target` (a path inside the project) a relative link to `outside/<name>`, the way a
    /// committed link would be written.
    fn link_out(&self, target: &str, name: &str) -> PathBuf {
        let link = self.root.join(target);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        let depth = Path::new(target).components().count();
        symlink(format!("{}outside/{name}", "../".repeat(depth)), &link).unwrap();
        link
    }

    fn run(&self, args: &[&str]) -> Output {
        specsync()
            .args(args)
            .arg("--root")
            .arg(&self.root)
            .output()
            .unwrap()
    }
}

/// A valid project with one spec, and source for a `billing` module that has no spec yet.
fn project() -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    write_config(&root, "specs", &["src"]);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/billing/service.ts"),
        "export function charge() {}\n",
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

/// A 3.x project, as `migrate` expects to find it.
fn v3_project() -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    setup_v3_project(&root);
    Fixture {
        _tmp: tmp,
        root,
        outside,
    }
}

/// The names in `directory`, sorted.
fn names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Everything the command printed. `hooks` and `agents` report a failed target on stdout, in the
/// list of targets, as they always have.
fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

// ─── hooks ───────────────────────────────────────────────────────────────

const HOOK_TARGETS: [(&str, &str); 5] = [
    ("--claude", "CLAUDE.md"),
    ("--cursor", ".cursorrules"),
    ("--agents", "AGENTS.md"),
    ("--copilot", ".github/copilot-instructions.md"),
    ("--claude-code-hook", ".claude/settings.json"),
];

/// Verifies REQ-hooks-006. Each file `hooks install` writes can be committed as a link to a file
/// of whoever clones the repository.
#[test]
fn hooks_install_refuses_a_target_linked_outside_the_project() {
    for (flag, target) in HOOK_TARGETS {
        let fx = project();
        // Valid JSON, so the settings writer gets as far as its write.
        let before = if target.ends_with(".json") {
            "{}\n"
        } else {
            OWNER
        };
        let victim = fx.victim("victim.txt", before);
        let link = fx.link_out(target, "victim.txt");

        let output = fx.run(&["hooks", "install", flag]);

        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            before,
            "hooks install {flag} wrote through {target} to a file outside the project"
        );
        assert!(!output.status.success(), "hooks install {flag}");
        assert!(
            said(&output).contains(&format!(
                "{target} is a symbolic link that leaves the project"
            )),
            "hooks install {flag} must name the refused path: {}",
            said(&output)
        );
        assert!(is_link(&link), "{target} is left as it was");
        assert_eq!(names(&fx.outside), ["victim.txt"]);
    }
}

/// Verifies REQ-hooks-006. A link whose target does not exist must not create it, and a link to a
/// link that leaves is no better than a direct one.
#[test]
fn hooks_install_refuses_a_dangling_link_and_a_chain_that_leaves() {
    let fx = project();
    fx.link_out("CLAUDE.md", "not-there-yet.md");
    let output = fx.run(&["hooks", "install", "--claude"]);
    assert!(
        names(&fx.outside).is_empty(),
        "hooks install created the target of a dangling CLAUDE.md link: {:?}",
        names(&fx.outside)
    );
    assert!(!output.status.success());
    assert!(
        said(&output).contains("CLAUDE.md is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );

    // A dangling link that stays inside the project is refused too, and says so.
    let fx = project();
    symlink("not-there-yet.md", fx.root.join("CLAUDE.md")).unwrap();
    let output = fx.run(&["hooks", "install", "--claude"]);
    assert!(!fx.root.join("not-there-yet.md").exists());
    assert!(!output.status.success());
    assert!(
        said(&output).contains("CLAUDE.md is a dangling symbolic link"),
        "{}",
        said(&output)
    );

    let fx = project();
    let victim = fx.victim("victim.txt", OWNER);
    fx.link_out("hop.md", "victim.txt");
    symlink("hop.md", fx.root.join("CLAUDE.md")).unwrap();
    let output = fx.run(&["hooks", "install", "--claude"]);
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "hooks install followed a chain of links out of the project"
    );
    assert!(!output.status.success());
    assert!(
        said(&output).contains("CLAUDE.md is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-hooks-006. The directory a target lives in can be the link.
#[test]
fn hooks_install_creates_nothing_in_a_directory_linked_outside_the_project() {
    for (flag, directory) in [("--copilot", ".github"), ("--claude-code-hook", ".claude")] {
        let fx = project();
        fs::create_dir_all(fx.outside.join("elsewhere")).unwrap();
        symlink("../outside/elsewhere", fx.root.join(directory)).unwrap();

        let output = fx.run(&["hooks", "install", flag]);

        assert!(
            names(&fx.outside.join("elsewhere")).is_empty(),
            "hooks install {flag} wrote into a directory outside the project: {:?}",
            names(&fx.outside.join("elsewhere"))
        );
        assert!(!output.status.success(), "hooks install {flag}");
        assert!(
            said(&output).contains(&format!(
                "{directory} is a symbolic link that leaves the project"
            )),
            "{}",
            said(&output)
        );
    }
}

/// Verifies REQ-hooks-006. `CLAUDE.md` as a link to `AGENTS.md` is a common setup, and a link
/// that stays inside the project is written through.
#[test]
fn hooks_install_writes_through_a_link_that_stays_inside_the_project() {
    let fx = project();
    fs::write(fx.root.join("AGENTS.md"), "# Agents\n").unwrap();
    symlink("AGENTS.md", fx.root.join("CLAUDE.md")).unwrap();

    let output = fx.run(&["hooks", "install", "--claude"]);

    assert!(output.status.success(), "{}", said(&output));
    let agents = fs::read_to_string(fx.root.join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# Agents\n"), "{agents}");
    assert!(agents.contains("Spec-Sync Integration"), "{agents}");
    assert!(is_link(&fx.root.join("CLAUDE.md")));
}

/// Verifies REQ-hooks-006. Uninstall rewrites the same files, so it is held to the same rule.
#[test]
fn hooks_uninstall_refuses_a_target_linked_outside_the_project() {
    let fx = project();
    fs::write(fx.root.join("CLAUDE.md"), "# Mine\n").unwrap();
    assert!(fx.run(&["hooks", "install", "--claude"]).status.success());
    let installed = fs::read_to_string(fx.root.join("CLAUDE.md")).unwrap();
    assert!(installed.contains("Spec-Sync Integration"));
    let victim = fx.victim("victim.md", &installed);
    fs::remove_file(fx.root.join("CLAUDE.md")).unwrap();
    fx.link_out("CLAUDE.md", "victim.md");

    let output = fx.run(&["hooks", "uninstall", "--claude"]);

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        installed,
        "hooks uninstall rewrote a file outside the project through CLAUDE.md"
    );
    assert!(!output.status.success());
    assert!(
        said(&output).contains("CLAUDE.md is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );
}

// ─── agents ──────────────────────────────────────────────────────────────

/// Verifies REQ-agents-011. A dangling link passed the "is it there" read, so the artifact was
/// created at the link's target.
#[test]
fn agents_install_refuses_a_dangling_artifact_link_before_writing_anything() {
    let fx = project();
    fx.link_out(".claude/skills/spec-sync/SKILL.md", "planted.md");

    let output = fx.run(&["agents", "install", "--claude"]);

    assert!(
        names(&fx.outside).is_empty(),
        "agents install created the target of a dangling artifact link: {:?}",
        names(&fx.outside)
    );
    assert!(!output.status.success());
    assert!(
        said(&output).contains(
            ".claude/skills/spec-sync/SKILL.md is a symbolic link that leaves the project"
        ),
        "{}",
        said(&output)
    );
    assert!(
        !fx.root.join(".claude/commands").exists(),
        "no other artifact is written once one is refused"
    );
}

/// Verifies REQ-agents-011. The manifest is written last and had no read before it.
#[test]
fn agents_install_refuses_a_manifest_linked_outside_the_project() {
    let fx = project();
    fx.link_out(".specsync/agent-artifacts.json", "not-there-yet.json");
    let output = fx.run(&["agents", "install", "--claude"]);
    assert!(
        names(&fx.outside).is_empty(),
        "agents install created the target of a dangling manifest link: {:?}",
        names(&fx.outside)
    );
    assert!(!output.status.success());
    assert!(
        said(&output)
            .contains(".specsync/agent-artifacts.json is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );
    assert!(
        !fx.root.join(".claude").exists(),
        "a refused manifest is found before any artifact is written"
    );

    let fx = project();
    let victim = fx.victim("victim.json", OWNER);
    fx.link_out(".specsync/agent-artifacts.json", "victim.json");
    let output = fx.run(&["agents", "install", "--claude"]);
    assert_eq!(fs::read_to_string(&victim).unwrap(), OWNER);
    assert!(!output.status.success());
    assert!(
        said(&output)
            .contains(".specsync/agent-artifacts.json is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );
}

/// Verifies REQ-agents-011.
#[test]
fn agents_install_creates_nothing_in_a_directory_linked_outside_the_project() {
    let fx = project();
    fs::create_dir_all(fx.outside.join("elsewhere")).unwrap();
    symlink("../outside/elsewhere", fx.root.join(".claude")).unwrap();

    let output = fx.run(&["agents", "install", "--claude"]);

    assert!(
        names(&fx.outside.join("elsewhere")).is_empty(),
        "agents install wrote into a directory outside the project: {:?}",
        names(&fx.outside.join("elsewhere"))
    );
    assert!(!output.status.success());
    assert!(
        said(&output).contains(".claude is a symbolic link that leaves the project"),
        "{}",
        said(&output)
    );
}

// ─── mcp lock ────────────────────────────────────────────────────────────

/// Verifies REQ-mcp-011.
#[test]
fn mcp_lock_write_refuses_a_lock_linked_outside_the_project() {
    let fx = project();
    let victim = fx.victim("victim.json", OWNER);
    fx.link_out(".specsync/mcp-tools.lock.json", "victim.json");
    let output = fx.run(&["mcp", "lock", "--write"]);
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        OWNER,
        "mcp lock --write wrote the lock through a link, over a file outside the project"
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output)
            .contains(".specsync/mcp-tools.lock.json is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );

    let fx = project();
    fx.link_out(".specsync/mcp-tools.lock.json", "not-there-yet.json");
    let output = fx.run(&["mcp", "lock", "--write"]);
    assert!(
        names(&fx.outside).is_empty(),
        "mcp lock --write created the target of a dangling link: {:?}",
        names(&fx.outside)
    );
    assert_eq!(output.status.code(), Some(2));
}

/// Verifies REQ-mcp-011. A lock linked to a regular file inside the project is written through,
/// and a plain lock is written as before.
#[test]
fn mcp_lock_write_writes_a_plain_lock_and_through_a_link_that_stays_inside() {
    let fx = project();
    fs::create_dir_all(fx.root.join(".specsync")).unwrap();
    fs::write(fx.root.join(".specsync/kept.json"), "{}\n").unwrap();
    symlink("kept.json", fx.root.join(".specsync/mcp-tools.lock.json")).unwrap();
    let output = fx.run(&["mcp", "lock", "--write"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        fs::read_to_string(fx.root.join(".specsync/kept.json"))
            .unwrap()
            .contains("\"tools\"")
    );
    assert!(is_link(&fx.root.join(".specsync/mcp-tools.lock.json")));

    let fx = project();
    assert!(fx.run(&["mcp", "lock", "--write"]).status.success());
    assert!(
        fx.run(&["mcp", "diff"]).status.success(),
        "the written lock matches the live catalog"
    );
}

// ─── spec creation: new, scaffold, add-spec ──────────────────────────────

const SPEC_CREATORS: [&str; 3] = ["new", "scaffold", "add-spec"];

/// Verifies REQ-cmd-new-006 and REQ-cmd-scaffold-007. The spec path was guarded by `exists()`,
/// which a dangling link passes.
#[test]
fn spec_creation_refuses_a_dangling_spec_link() {
    for command in SPEC_CREATORS {
        let fx = project();
        fx.link_out("specs/billing/billing.spec.md", "planted.md");

        let output = fx.run(&[command, "billing"]);

        assert!(
            names(&fx.outside).is_empty(),
            "{command} created the target of a dangling spec link: {:?}",
            names(&fx.outside)
        );
        assert_eq!(output.status.code(), Some(1), "{command}");
        assert!(
            stderr(&output).contains(
                "specs/billing/billing.spec.md is a symbolic link that leaves the project"
            ),
            "{command}: {}",
            stderr(&output)
        );
    }
}

/// Verifies REQ-cmd-new-006 and REQ-cmd-scaffold-007. The spec directory can be the link.
#[test]
fn spec_creation_creates_nothing_in_a_spec_directory_linked_outside_the_project() {
    for command in SPEC_CREATORS {
        let fx = project();
        fs::create_dir_all(fx.outside.join("elsewhere")).unwrap();
        symlink("../../outside/elsewhere", fx.root.join("specs/billing")).unwrap();

        let output = fx.run(&[command, "billing"]);

        assert!(
            names(&fx.outside.join("elsewhere")).is_empty(),
            "{command} wrote into a directory outside the project: {:?}",
            names(&fx.outside.join("elsewhere"))
        );
        assert_eq!(output.status.code(), Some(1), "{command}");
        assert!(
            stderr(&output).contains("specs/billing is a symbolic link that leaves the project"),
            "{command}: {}",
            stderr(&output)
        );
    }
}

/// Verifies REQ-cmd-scaffold-007. A spec directory linked to a directory inside the project is
/// still used.
#[test]
fn spec_creation_writes_through_a_spec_directory_link_that_stays_inside() {
    for command in SPEC_CREATORS {
        let fx = project();
        fs::create_dir_all(fx.root.join("kept/billing")).unwrap();
        symlink("../kept/billing", fx.root.join("specs/billing")).unwrap();

        let output = fx.run(&[command, "billing"]);

        assert!(output.status.success(), "{command}: {}", stderr(&output));
        assert!(
            fx.root.join("kept/billing/billing.spec.md").is_file(),
            "{command}"
        );
        assert!(is_link(&fx.root.join("specs/billing")), "{command}");
    }
}

/// Verifies REQ-generator-010. A companion was guarded by `exists()` too.
#[test]
fn scaffold_refuses_a_dangling_companion_link_and_creates_the_others() {
    let fx = project();
    fx.link_out("specs/billing/tasks.md", "planted.md");

    let output = fx.run(&["scaffold", "billing"]);

    assert!(
        names(&fx.outside).is_empty(),
        "scaffold created the target of a dangling companion link: {:?}",
        names(&fx.outside)
    );
    assert!(
        stderr(&output)
            .contains("specs/billing/tasks.md is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );
    assert!(fx.root.join("specs/billing/billing.spec.md").is_file());
    assert!(fx.root.join("specs/billing/context.md").is_file());
    assert!(fx.root.join("specs/billing/requirements.md").is_file());
    assert!(is_link(&fx.root.join("specs/billing/tasks.md")));
}

/// Verifies REQ-registry-003 and REQ-cmd-scaffold-007. `scaffold` registers the new module when
/// the project has a registry, and the registry it appends to can be the link: the 4.x file
/// `.specsync/registry.toml`, or the 3.x file at the root.
#[test]
fn scaffold_refuses_a_registry_linked_outside_the_project() {
    let registry = "[registry]\nname = \"elsewhere\"\n\n[specs]\n";

    let fx = project();
    fs::write(fx.root.join("specsync-registry.toml"), registry).unwrap();
    let victim = fx.victim("registry.toml", registry);
    fx.link_out(".specsync/registry.toml", "registry.toml");
    let output = fx.run(&["scaffold", "billing"]);
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        registry,
        "scaffold appended to a registry outside the project"
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output)
            .contains(".specsync/registry.toml is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );

    let fx = project();
    let victim = fx.victim("registry.toml", registry);
    fx.link_out("specsync-registry.toml", "registry.toml");
    let output = fx.run(&["scaffold", "billing"]);
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        registry,
        "scaffold appended to a registry outside the project"
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output)
            .contains("specsync-registry.toml is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );
}

// ─── migrate ─────────────────────────────────────────────────────────────

/// Verifies REQ-cmd-migrate-005. `.specsync/.gitignore` was guarded by `exists()`, which a
/// dangling link passes. The refusal comes before the first step, so nothing has moved.
#[test]
fn migrate_refuses_a_dangling_gitignore_link_before_its_first_write() {
    let fx = v3_project();
    fx.link_out(".specsync/.gitignore", "planted.txt");

    let output = fx.run(&["migrate", "--no-backup"]);

    assert!(
        names(&fx.outside).is_empty(),
        "migrate created the target of a dangling .specsync/.gitignore link: {:?}",
        names(&fx.outside)
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(".specsync/.gitignore is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );
    assert!(
        fx.root.join("specsync.json").is_file(),
        "the old config is still in place"
    );
    assert!(
        !fx.root.join(".specsync/config.toml").exists(),
        "no migration step ran"
    );
    assert!(
        fs::read_to_string(fx.root.join("specs/auth/auth.spec.md"))
            .unwrap()
            .contains("lifecycle_log:"),
        "the spec is not rewritten"
    );
}

/// Verifies REQ-cmd-migrate-005. `migrate` finds specs with a walker that follows links, and
/// rewrites what it finds.
#[test]
fn migrate_refuses_a_spec_linked_outside_the_project() {
    let fx = v3_project();
    let spec = fx.root.join("specs/auth/auth.spec.md");
    let before = fs::read_to_string(&spec).unwrap();
    let victim = fx.victim("auth.spec.md", &before);
    fs::remove_file(&spec).unwrap();
    fx.link_out("specs/auth/auth.spec.md", "auth.spec.md");

    let output = fx.run(&["migrate", "--no-backup"]);

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        before,
        "migrate rewrote a spec outside the project through a link"
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("specs/auth/auth.spec.md is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );
    assert!(fx.root.join("specsync.json").is_file());
}

/// Verifies REQ-cmd-migrate-005.
#[test]
fn migrate_creates_nothing_in_a_specsync_directory_linked_outside_the_project() {
    let fx = v3_project();
    fs::create_dir_all(fx.outside.join("elsewhere")).unwrap();
    symlink("../outside/elsewhere", fx.root.join(".specsync")).unwrap();

    let output = fx.run(&["migrate", "--no-backup"]);

    assert!(
        names(&fx.outside.join("elsewhere")).is_empty(),
        "migrate wrote into a directory outside the project: {:?}",
        names(&fx.outside.join("elsewhere"))
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(".specsync is a symbolic link that leaves the project"),
        "{}",
        stderr(&output)
    );
    assert!(fx.root.join("specsync.json").is_file());
}

/// Verifies REQ-cmd-migrate-005. A 3.x project with no link still migrates.
#[test]
fn migrate_still_migrates_a_project_with_no_link() {
    let fx = v3_project();
    let output = fx.run(&["migrate"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(fx.root.join(".specsync/config.toml").is_file());
    assert!(fx.root.join(".specsync/.gitignore").is_file());
    assert!(fx.root.join(".specsync/version").is_file());
    assert!(fx.root.join(".specsync/backup-3x/manifest.json").is_file());
    assert!(
        !fs::read_to_string(fx.root.join("specs/auth/auth.spec.md"))
            .unwrap()
            .contains("lifecycle_log:")
    );
}
