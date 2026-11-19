//! What a repository carries is input, not something to trust: a committed hash cache never
//! replays a pass (CHECK-1.f), configured directories stay inside the project, and history and
//! imported files are read within a bound (SETUP-7).

use crate::helpers::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Output};
use tempfile::TempDir;

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

fn git_repo(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git_in(root, &["init", "-q", "-b", "main"]);
    git_in(root, &["config", "user.email", "inputs@example.invalid"]);
    git_in(root, &["config", "user.name", "Inputs Test"]);
    git_in(root, &["config", "commit.gpgsign", "false"]);
}

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run(root: &Path, args: &[&str]) -> Output {
    specsync()
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

/// A project with one valid `auth` spec and its source, beside an `outside/` directory.
fn project() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(outside.join("specs/stray")).unwrap();
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
    fs::write(
        outside.join("specs/stray/stray.spec.md"),
        valid_spec("stray", &["src/stray.ts"]),
    )
    .unwrap();
    (tmp, root, outside)
}

fn toml_config(root: &Path, body: &str) {
    fs::write(root.join(".specsync/config.toml"), body).unwrap();
}

// ─── a hash cache Git tracks (F4) ──────────────────────────────────────

/// Verifies REQ-hash-cache-009 and REQ-cmd-check-021. A `.specsync/hashes.json` that Git tracks
/// came with the repository: `check` neither replays nor rewrites it, says so once with the fix,
/// and validates every spec.
#[test]
fn check_never_uses_a_hash_cache_git_tracks() {
    let (_tmp, root, _outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    git_repo(&root);
    let first = run(&root, &["check"]);
    assert!(
        root.join(".specsync/hashes.json").exists(),
        "{}",
        said(&first)
    );
    git_in(&root, &["add", "-A"]);
    git_in(&root, &["add", "-f", ".specsync/hashes.json"]);
    git_in(&root, &["commit", "-q", "-m", "commit the cache"]);
    let cache = fs::read(root.join(".specsync/hashes.json")).unwrap();

    let output = run(&root, &["check"]);

    let text = said(&output);
    assert!(
        text.contains("hash cache not used") && text.contains("tracked by Git"),
        "{text}"
    );
    assert!(text.contains("git rm --cached"), "{text}");
    assert!(
        !text.contains("Skipped"),
        "a tracked cache was replayed: {text}"
    );
    assert_eq!(
        fs::read(root.join(".specsync/hashes.json")).unwrap(),
        cache,
        "a tracked cache was rewritten"
    );
}

/// Verifies REQ-hash-cache-009. `rehash` refuses a cache Git tracks, as it refuses any cache
/// location it will not write.
#[test]
fn rehash_refuses_a_hash_cache_git_tracks() {
    let (_tmp, root, _outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    git_repo(&root);
    fs::write(root.join(".specsync/hashes.json"), "{}").unwrap();
    git_in(&root, &["add", "-A"]);
    git_in(&root, &["add", "-f", ".specsync/hashes.json"]);
    git_in(&root, &["commit", "-q", "-m", "commit a cache"]);

    let output = run(&root, &["rehash"]);

    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("tracked by Git"),
        "{}",
        said(&output)
    );
    assert_eq!(
        fs::read_to_string(root.join(".specsync/hashes.json")).unwrap(),
        "{}"
    );
}

// ─── configured directories (F5) ───────────────────────────────────────

/// Verifies REQ-config-017. `specs_dir`, `source_dirs` and `schema_dir` name directories inside
/// the project: an absolute path or a `..` is a config error, and nothing outside is walked.
#[test]
fn a_configured_directory_outside_the_project_is_a_config_error() {
    let (_tmp, root, outside) = project();
    let absolute = outside.join("specs").to_string_lossy().into_owned();
    for (field, config) in [
        (
            "specs_dir",
            "specs_dir = \"../outside/specs\"\nsource_dirs = [\"src\"]\n".to_string(),
        ),
        (
            "specs_dir",
            format!("specs_dir = \"{absolute}\"\nsource_dirs = [\"src\"]\n"),
        ),
        (
            "specs_dir",
            "specs_dir = \"specs/../../outside/specs\"\nsource_dirs = [\"src\"]\n".to_string(),
        ),
        (
            "source_dirs",
            "specs_dir = \"specs\"\nsource_dirs = [\"src\", \"../outside\"]\n".to_string(),
        ),
        (
            "schema_dir",
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nschema_dir = \"../outside\"\n"
                .to_string(),
        ),
    ] {
        toml_config(&root, &config);

        let output = run(&root, &["deps"]);

        assert!(!output.status.success(), "{config}: {}", said(&output));
        assert!(said(&output).contains(field), "{config}: {}", said(&output));
        assert!(
            !said(&output).contains("stray"),
            "{config}: {}",
            said(&output)
        );
    }
}

/// Verifies REQ-validator-050. A committed `specs` link that leaves the project is not walked;
/// the command says why instead of listing what it found there.
#[cfg(unix)]
#[test]
fn a_specs_link_that_leaves_the_project_is_not_walked() {
    let (_tmp, root, _outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    fs::remove_dir_all(root.join("specs")).unwrap();
    std::os::unix::fs::symlink("../outside/specs", root.join("specs")).unwrap();

    let output = run(&root, &["deps"]);

    let text = said(&output);
    assert!(!output.status.success(), "{text}");
    assert!(
        text.contains("spec discovery stopped") && text.contains("leaves the project"),
        "{text}"
    );
    assert!(!text.contains("stray"), "the walk went outside: {text}");
}

/// Verifies REQ-validator-050. Spec discovery has the same depth budget as coverage, so a tree
/// deeper than it stops the command with a reason instead of walking on.
#[test]
fn spec_discovery_stops_at_the_depth_budget() {
    let (_tmp, root, _outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    let mut deep = root.join("specs/auth");
    for _ in 0..260 {
        deep.push("d");
    }
    fs::create_dir_all(&deep).unwrap();

    let output = run(&root, &["deps"]);

    assert!(!output.status.success(), "{}", said(&output));
    assert!(
        said(&output).contains("spec discovery stopped"),
        "{}",
        said(&output)
    );
}

// ─── history (F6) ──────────────────────────────────────────────────────

/// Verifies REQ-changelog-002. A spec blob in history larger than the spec bound is skipped with
/// one warning naming the spec and the ref, and the rest of the changelog still prints.
#[test]
fn changelog_skips_a_spec_blob_over_the_bound() {
    let (_tmp, root, _outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    fs::create_dir_all(root.join("specs/big")).unwrap();
    let mut big = valid_spec("big", &["src/big.ts"]);
    big.push_str(&"x".repeat(9 * 1024 * 1024));
    fs::write(root.join("specs/big/big.spec.md"), big).unwrap();
    git_repo(&root);
    git_in(&root, &["add", "-A"]);
    git_in(&root, &["commit", "-q", "-m", "big"]);
    fs::write(
        root.join("specs/big/big.spec.md"),
        valid_spec("big", &["src/big.ts"]),
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/billing")).unwrap();
    fs::write(
        root.join("specs/billing/billing.spec.md"),
        valid_spec("billing", &["src/billing.ts"]),
    )
    .unwrap();
    git_in(&root, &["add", "-A"]);
    git_in(&root, &["commit", "-q", "-m", "small"]);

    let output = run(&root, &["changelog", "HEAD~1..HEAD"]);

    let text = said(&output);
    assert!(output.status.success(), "{text}");
    assert!(
        text.contains("billing"),
        "the rest of the changelog: {text}"
    );
    assert!(
        text.contains("specs/big/big.spec.md") && text.contains("larger than"),
        "{text}"
    );
}

// ─── import --from-dir (F7) ────────────────────────────────────────────

const SECRET: &str = "SECRET-OUTSIDE-THE-REPOSITORY";

fn imported_text(root: &Path) -> String {
    let mut text = String::new();
    let mut stack = vec![root.join("specs")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(content) = fs::read_to_string(&path) {
                text.push_str(&content);
            }
        }
    }
    text
}

/// Verifies REQ-cmd-import-004. A directory inside the project is read by the project rule: an
/// entry linked outside the repository is refused, and nothing it names reaches a spec draft.
#[cfg(unix)]
#[test]
fn import_from_dir_never_imports_through_a_link_out() {
    let (_tmp, root, outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    fs::write(outside.join("secret.md"), format!("# Secret\n\n{SECRET}\n")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/notes.md"), "# Notes\n\nPlain notes.\n").unwrap();
    std::os::unix::fs::symlink("../../outside/secret.md", root.join("docs/secret.md")).unwrap();

    let output = run(&root, &["import", "--from-dir", "docs"]);

    assert!(!imported_text(&root).contains(SECRET), "{}", said(&output));
    assert!(
        said(&output).contains("leaves the project"),
        "{}",
        said(&output)
    );
    assert!(root.join("specs/notes").exists(), "{}", said(&output));
}

/// Verifies REQ-cmd-import-004. A directory outside the project is read within itself: a link
/// inside it is followed, a link out of it is refused.
#[cfg(unix)]
#[test]
fn import_from_an_outside_dir_stays_inside_that_dir() {
    let (tmp, root, outside) = project();
    toml_config(&root, "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n");
    let named = outside.join("docs");
    fs::create_dir_all(&named).unwrap();
    fs::write(named.join("notes.md"), "# Notes\n\nPlain notes.\n").unwrap();
    std::os::unix::fs::symlink("notes.md", named.join("again.md")).unwrap();
    fs::write(
        tmp.path().join("secret.md"),
        format!("# Secret\n\n{SECRET}\n"),
    )
    .unwrap();
    std::os::unix::fs::symlink("../../secret.md", named.join("secret.md")).unwrap();

    let output = run(&root, &["import", "--from-dir", named.to_str().unwrap()]);

    assert!(!imported_text(&root).contains(SECRET), "{}", said(&output));
    assert!(root.join("specs/notes").exists(), "{}", said(&output));
    assert!(root.join("specs/again").exists(), "{}", said(&output));
}
