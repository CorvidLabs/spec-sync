//! Project initialization, configuration discovery, and hook preservation.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── 4. specsync init ───────────────────────────────────────────────────

#[test]
fn init_creates_config_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .specsync/config.toml"));

    let config_path = root.join(".specsync/config.toml");
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("specs_dir"));
    assert!(content.contains("source_dirs"));
    assert!(content.contains("required_sections"));

    // Full v4 layout — version stamp, .gitignore, and state directories
    assert!(root.join(".specsync/version").exists());
    assert!(root.join(".specsync/.gitignore").exists());
    assert!(root.join(".specsync/lifecycle").is_dir());
    assert!(root.join(".specsync/changes").is_dir());
    assert!(root.join(".specsync/archive").is_dir());
}

#[test]
fn init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success();

    let policy: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".specsync/sdd.json")).unwrap())
            .unwrap();
    assert_eq!(policy["enabled"], false);
    assert_eq!(policy["require_change_for_meaningful_files"], false);

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stderr(predicate::str::contains("Legacy 3.x layout").not())
        .stdout(predicate::str::contains("active change").not())
        .stderr(predicate::str::contains("active change").not());
}

#[test]
fn check_does_not_walk_sdd_when_policy_is_on() {
    // Honest label: DISCRIMINATOR. On the unfixed binary `check` printed
    // "N active change(s)" and warned about unreadable workspaces because it
    // called `audit_project`. `change audit` is the CONTROL — it must still
    // see the corrupt workspace when SDD is enabled.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success();
    let policy_path = root.join(".specsync/sdd.json");
    let mut policy: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    policy["enabled"] = serde_json::json!(true);
    fs::write(&policy_path, serde_json::to_string_pretty(&policy).unwrap()).unwrap();
    let broken = root.join(".specsync/changes/broken-sibling");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("state.json"), "{ this is not json\n").unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("active change").not())
        .stderr(predicate::str::contains("unreadable").not())
        .stderr(predicate::str::contains("state.json").not());

    specsync()
        .arg("change")
        .arg("audit")
        .arg("--root")
        .arg(root)
        .assert()
        .stderr(
            predicate::str::contains("state.json")
                .or(predicate::str::contains("unreadable"))
                .or(predicate::str::contains("invalid")),
        );
}

#[test]
fn init_does_not_overwrite_existing_v4_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"custom\"\n",
    )
    .unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    let content = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(content.contains("custom"));
}

#[test]
fn init_does_not_overwrite_existing_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(root.join("specsync.json"), r#"{"specsDir":"custom"}"#).unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    // Original content preserved
    let content = fs::read_to_string(root.join("specsync.json")).unwrap();
    assert!(content.contains("custom"));
}

// ─── Auto-detect source directories ─────────────────────────────────────

#[test]
fn init_auto_detects_src_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with src/ containing source files
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: src"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"src\"]"));
}

#[test]
fn init_auto_detects_lib_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with lib/ containing source files
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/utils.py"), "def hello(): pass\n").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: lib"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"lib\"]"));
}

#[test]
fn init_auto_detects_multiple_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with both src/ and lib/
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.ts"), "export function main() {}").unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/helpers.ts"), "export function help() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Detected source directories: lib, src",
        ));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"lib\", \"src\"]"));
}

#[test]
fn init_ignores_node_modules_and_hidden_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create source in app/ and noise in node_modules/ and .cache/
    fs::create_dir_all(root.join("app")).unwrap();
    fs::write(root.join("app/index.ts"), "export default function() {}").unwrap();
    fs::create_dir_all(root.join("node_modules/some-pkg")).unwrap();
    fs::write(
        root.join("node_modules/some-pkg/index.js"),
        "module.exports = {}",
    )
    .unwrap();
    fs::create_dir_all(root.join(".cache")).unwrap();
    fs::write(root.join(".cache/data.js"), "const x = 1;").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: app"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"app\"]"));
}

#[test]
fn check_works_without_config_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with lib/ source and specs, but no specsync.json
    fs::create_dir_all(root.join("lib/auth")).unwrap();
    fs::write(
        root.join("lib/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = valid_spec("auth", &["lib/auth/service.ts"]);
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    // Should auto-detect lib/ and work without any config
    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success();
}

#[test]
fn init_falls_back_to_src_when_no_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Empty project with only a README
    fs::write(root.join("README.md"), "# My Project").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: src"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"src\"]"));
}

// ─── specsync hooks ─────────────────────────────────────────────────────

#[test]
fn hooks_uninstall_preserves_user_content_after_block() {
    // Regression: `hooks uninstall` used to delete from the managed block to EOF,
    // wiping any content the user added after it (and the whole file if spec-sync
    // created it). It must now remove only the managed block.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    specsync()
        .current_dir(root)
        .args(["hooks", "install", "--claude"])
        .assert()
        .success();

    let claude = root.join("CLAUDE.md");
    let mut content = fs::read_to_string(&claude).unwrap();
    content.push_str("\n## Deploy Notes\nDO NOT DELETE THIS LINE\n");
    fs::write(&claude, content).unwrap();

    specsync()
        .current_dir(root)
        .args(["hooks", "uninstall", "--claude"])
        .assert()
        .success();

    assert!(claude.exists(), "CLAUDE.md must not be deleted");
    let after = fs::read_to_string(&claude).unwrap();
    assert!(
        after.contains("DO NOT DELETE THIS LINE"),
        "content added after the managed block must survive uninstall:\n{after}"
    );
    assert!(
        !after.contains("Spec-Sync"),
        "the managed block must be removed:\n{after}"
    );
}

#[test]
fn check_scalar_inline_comment_does_not_hide_specs() {
    // Regression (#6): an inline comment on `specs_dir` used to be kept in the
    // value (`"specs" # note`), mis-resolving the specs dir so every spec became
    // invisible and `check` silently passed. The spec must be discovered.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("mydocs")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"mydocs\" # where specs live\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("mydocs/a.spec.md"),
        "---\nmodule: a\nstatus: stable\nfiles:\n  - src/a.ts\n---\n# A\n## Purpose\nx\n",
    )
    .unwrap();

    // The spec is now discovered (output names it) rather than "No spec files found".
    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stdout(predicate::str::contains("a.spec.md"));
}

#[test]
fn config_warns_on_unreadable_config_file() {
    // Regression: a config file that exists but can't be read as UTF-8 silently
    // reverted to built-in defaults, downgrading enforcement with no signal.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    // A config whose keys are valid ASCII but whose tail is invalid UTF-8.
    let mut bad = b"specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n".to_vec();
    bad.extend_from_slice(&[0xFF, 0xFE]);
    fs::write(root.join(".specsync.toml"), bad).unwrap();

    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stderr(predicate::str::contains("exists but could not be read"));
}

// ─── hooks install: claude-code-hook must not clobber user settings ──────

#[test]
fn hooks_install_claude_code_hook_preserves_user_settings() {
    // Regression (H3): `hooks install --claude-code-hook` used to overwrite the
    // whole `hooks` object in .claude/settings.json, destroying the user's own
    // hooks. It must deep-merge instead.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(
        root.join(".claude/settings.json"),
        "{\n  \"permissions\": { \"allow\": [\"Bash(ls:*)\"] },\n  \"hooks\": {\n    \"PreToolUse\": [\n      { \"matcher\": \"Bash\", \"hooks\": [{ \"type\": \"command\", \"command\": \"audit.sh\" }] }\n    ]\n  }\n}\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["hooks", "install", "--claude-code-hook"])
        .assert()
        .success();

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(
        parsed["permissions"]["allow"][0], "Bash(ls:*)",
        "unrelated settings must survive"
    );
    assert_eq!(
        parsed["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "audit.sh",
        "the user's own hooks must survive"
    );
    assert!(
        parsed["hooks"]["PostToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("specsync"),
        "specsync's hook must be added"
    );
}
