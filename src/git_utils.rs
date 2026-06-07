use std::path::Path;
use std::process::Command;

/// Get the last commit hash that touched a file.
pub fn git_last_commit_hash(root: &Path, file: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%H", "--", file])
        .current_dir(root)
        .output()
        .ok()?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() { None } else { Some(hash) }
}

/// Count commits that touched `source_file` since the given `spec_commit`.
///
/// Takes a precomputed spec commit hash (from [`git_last_commit_hash`]) so
/// callers iterating over a spec's source files spawn one `git rev-list` per
/// file instead of also re-resolving the spec's commit each time.
pub fn git_commits_since(root: &Path, spec_commit: &str, source_file: &str) -> usize {
    let output = match Command::new("git")
        .args([
            "rev-list",
            "--count",
            &format!("{spec_commit}..HEAD"),
            "--",
            source_file,
        ])
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(_) => return 0,
    };

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
}

/// Check if the current directory is inside a git repository.
pub fn is_git_repo(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Staleness info for a single spec relative to its source files.
#[derive(Debug, Clone)]
pub struct StaleInfo {
    /// Relative path to the spec file.
    pub spec_path: String,
    /// Module name from frontmatter.
    pub module_name: String,
    /// Maximum commits behind across all source files.
    pub max_commits_behind: usize,
    /// Per-source-file commit distances (file, commits_behind).
    pub source_details: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Run a git command in `root`, asserting it succeeds.
    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command should run");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Initialize an empty git repo with a deterministic identity.
    fn init_repo() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        git(root, &["init"]);
        git(root, &["config", "user.email", "test@test.com"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "commit.gpgsign", "false"]);
        tmp
    }

    /// Write a file (creating parent dirs), stage everything, and commit.
    fn commit_file(root: &Path, rel: &str, contents: &str, message: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-m", message]);
    }

    #[test]
    fn last_commit_hash_returns_none_for_untracked_file() {
        let tmp = init_repo();
        assert_eq!(
            git_last_commit_hash(tmp.path(), "specs/missing.spec.md"),
            None
        );
    }

    #[test]
    fn last_commit_hash_returns_hash_for_tracked_file() {
        let tmp = init_repo();
        commit_file(tmp.path(), "specs/auth.spec.md", "spec v1", "add spec");

        let hash = git_last_commit_hash(tmp.path(), "specs/auth.spec.md")
            .expect("tracked file should have a commit hash");
        assert_eq!(
            hash.len(),
            40,
            "expected a full 40-char SHA-1, got {hash:?}"
        );
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn commits_since_is_zero_when_source_unchanged_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "src/auth.rs", "fn login() {}", "add source");
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");

        let spec_commit = git_last_commit_hash(root, "specs/auth.spec.md").unwrap();
        // Source has not changed since the spec was committed.
        assert_eq!(git_commits_since(root, &spec_commit, "src/auth.rs"), 0);
    }

    #[test]
    fn commits_since_counts_source_changes_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");
        let spec_commit = git_last_commit_hash(root, "specs/auth.spec.md").unwrap();

        // Three commits touch the source file after the spec was last committed.
        commit_file(root, "src/auth.rs", "v1", "change 1");
        commit_file(root, "src/auth.rs", "v2", "change 2");
        commit_file(root, "src/auth.rs", "v3", "change 3");

        assert_eq!(git_commits_since(root, &spec_commit, "src/auth.rs"), 3);
        // A different, untouched file reports zero.
        assert_eq!(git_commits_since(root, &spec_commit, "src/other.rs"), 0);
    }

    #[test]
    fn commits_since_returns_zero_for_invalid_commit() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "src/auth.rs", "v1", "add source");
        // A bogus commit ref makes `git rev-list` fail; we degrade to zero.
        assert_eq!(
            git_commits_since(
                root,
                "0000000000000000000000000000000000000000",
                "src/auth.rs"
            ),
            0
        );
    }

    #[test]
    fn is_git_repo_detects_repo_and_non_repo() {
        let repo = init_repo();
        assert!(is_git_repo(repo.path()));

        let plain = TempDir::new().unwrap();
        assert!(!is_git_repo(plain.path()));
    }
}
