//! Files SpecSync writes into a repository carry no U+2013 (en dash) or U+2014 (em dash)
//! (REQ-hooks-005, REQ-agents-009, REQ-compact-001, REQ-cmd-init-008, REQ-cmd-migrate-004).
//!
//! Downstream projects lint their own Markdown for these characters, and some of what SpecSync
//! writes cannot be fixed by hand: the managed hook block says "do not edit inside". So the guard
//! here reads the bytes the built binary actually wrote, not a list of constants, and a new dash
//! in any of these writers fails it. Every fixture this file writes is itself dash-free, so any
//! dash found came from SpecSync.
//!
//! The lesson bundle that `change finalize` writes is covered in-process instead
//! (`lesson_bundle_heading_uses_a_colon_and_adds_no_dash`): a binary-driven finalize needs a
//! fully approved, checked and reviewed change, which this test cannot produce honestly.

use crate::helpers::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::TempDir;

const EN_DASH: char = '\u{2013}';
const EM_DASH: char = '\u{2014}';

fn git_init(root: &Path) {
    let status = StdCommand::new("git")
        .args(["init", "--quiet"])
        .arg(root)
        .status()
        .unwrap();
    assert!(status.success(), "git init failed");
}

/// Every file under `root`, skipping `.git/` except the pre-commit hook SpecSync installs.
fn written_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path == root.join(".git") {
                let hook = path.join("hooks").join("pre-commit");
                if hook.is_file() {
                    out.push(hook);
                }
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                walk(&path, root, out);
            } else {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// Fail naming every `path:line` that holds an en or em dash.
fn assert_no_dashes(root: &Path) {
    let files = written_files(root);
    assert!(
        !files.is_empty(),
        "nothing was written under {}",
        root.display()
    );
    let mut offenders = Vec::new();
    for path in &files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if line.contains(EN_DASH) || line.contains(EM_DASH) {
                let rel = path.strip_prefix(root).unwrap_or(path);
                offenders.push(format!("{}:{}: {line}", rel.display(), index + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "SpecSync wrote {} line(s) holding U+2013 or U+2014:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

fn run(root: &Path, args: &[&str]) {
    let output = specsync()
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "specsync {args:?} failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Append twelve dated rows to the spec's `## Change Log` table.
fn append_changelog_rows(spec: &Path, month: u32, two_columns: bool) {
    let mut content = fs::read_to_string(spec).unwrap();
    if two_columns {
        content = content.replace(
            "| Date | Author | Change |\n|------|--------|--------|",
            "| Date | Change |\n|------|--------|",
        );
    }
    assert!(content.contains("## Change Log"), "{content}");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    for day in 1..=12 {
        if two_columns {
            content.push_str(&format!("| 2026-{month:02}-{day:02} | Change {day} |\n"));
        } else {
            content.push_str(&format!(
                "| 2026-{month:02}-{day:02} | dev | Change {day} |\n"
            ));
        }
    }
    fs::write(spec, content).unwrap();
}

/// DISCRIMINATOR: on a80748ff `hooks install` (CLAUDE.md, AGENTS.md, copilot-instructions,
/// .cursorrules, the pre-commit hook), `agents install` (the Claude and Cursor create-spec
/// commands), `init` (`.specsync/.gitignore`) and `compact` (summary rows) all write U+2014.
#[test]
fn fresh_project_writers_emit_no_em_or_en_dash() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn widget() {}\n").unwrap();

    run(root, &["init"]);
    run(root, &["hooks", "install"]);
    run(root, &["agents", "install"]);
    run(root, &["add-spec", "widget"]);
    run(root, &["add-spec", "gadget"]);

    assert!(
        root.join(".git/hooks/pre-commit").is_file(),
        "hooks install must write the pre-commit hook this test inspects"
    );
    assert!(root.join(".claude/settings.json").is_file());

    let widget = root.join("specs/widget/widget.spec.md");
    let gadget = root.join("specs/gadget/gadget.spec.md");
    append_changelog_rows(&widget, 1, false);
    append_changelog_rows(&gadget, 2, true);
    run(root, &["compact", "--keep", "3"]);
    assert_no_dashes(root);

    let widget = fs::read_to_string(&widget).unwrap();
    assert!(
        widget.contains(
            "| 2026-01-01 to 2026-01-09 | - | Compacted: 9 entries <!-- specsync:compact:v1 --> |"
        ),
        "3-column summary row: {widget}"
    );
    let gadget = fs::read_to_string(&gadget).unwrap();
    assert!(
        gadget.contains(
            "| 2026-02-01 to 2026-02-09 | Compacted: 9 entries <!-- specsync:compact:v1 --> |"
        ),
        "2-column summary row: {gadget}"
    );
}

/// DISCRIMINATOR: on a80748ff `migrate` writes U+2014 into the `.specsync/.gitignore` header.
#[test]
fn migrate_writes_no_em_or_en_dash() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_v3_project(root);

    run(root, &["migrate"]);

    let gitignore = fs::read_to_string(root.join(".specsync/.gitignore")).unwrap();
    assert!(
        gitignore.starts_with("# spec-sync v4: generated by `specsync migrate`\n"),
        "{gitignore}"
    );
    assert_no_dashes(root);
}

/// The begin sentinel releases up to and including 6.1.1 wrote, frozen as a literal so the test
/// does not follow a constant that changes.
const BEGIN_6_1_1: &str =
    "<!-- specsync:hook:begin (managed by `specsync hooks` \u{2014} do not edit inside) -->";

/// CONTROL: passes before and after. A block a 6.1.1 install left behind is still removed
/// exactly, with the user's text on both sides kept.
#[test]
fn hooks_uninstall_through_the_binary_removes_a_6_1_1_block() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    git_init(root);
    let block = format!(
        "{BEGIN_6_1_1}\n# Spec-Sync Integration\n\nEach spec has companion files \u{2014} read them first.\n\n## Key commands\n\n- `specsync check` \u{2014} validate all specs\n<!-- specsync:hook:end -->"
    );
    for name in ["CLAUDE.md", "AGENTS.md"] {
        fs::write(
            root.join(name),
            format!("# My notes\n\nkeep before\n\n{block}\n\n## After\n\nkeep after\n"),
        )
        .unwrap();
    }

    run(root, &["hooks", "uninstall", "--claude", "--agents"]);

    for name in ["CLAUDE.md", "AGENTS.md"] {
        let content = fs::read_to_string(root.join(name)).unwrap();
        assert_eq!(
            content, "# My notes\n\nkeep before\n\n## After\n\nkeep after\n",
            "{name}"
        );
        assert!(!content.contains("specsync:hook"), "{name}: {content}");
    }
}
