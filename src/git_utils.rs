use std::path::Path;
use std::process::Command;

/// Why a tree cannot be asked how far a spec has fallen behind its source.
///
/// Both variants mean the same thing to a caller measuring drift: there is no
/// history, so the distance is UNKNOWN — never zero. They differ only in what
/// a human should be told to do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingHistory {
    /// `root` is not inside a git repository.
    NotARepository,
    /// A git repository whose `HEAD` is unborn — `git init` and nothing since.
    NoCommits,
}

impl MissingHistory {
    /// Lowercase reason, for machine payloads and mid-sentence use.
    pub fn reason(self) -> &'static str {
        match self {
            Self::NotARepository => "not a git repository",
            Self::NoCommits => "repository has no commits",
        }
    }

    /// Sentence-cased reason, for the head of a human-readable error line.
    pub fn sentence(self) -> &'static str {
        match self {
            Self::NotARepository => "Not a git repository",
            Self::NoCommits => "Repository has no commits",
        }
    }
}

/// Whether `root` has committed history to measure staleness against.
///
/// `None` means history is usable. `Some(..)` means every spec-behind-source
/// distance in this tree is unknown, and any caller about to report `0` or
/// `false` is reporting an answer to a question git could not be asked.
pub fn missing_history(root: &Path) -> Option<MissingHistory> {
    if !is_git_repo(root) {
        return Some(MissingHistory::NotARepository);
    }
    if !has_commits(root) {
        return Some(MissingHistory::NoCommits);
    }
    None
}

/// The commit a spec's staleness is measured from.
///
/// This type exists to make the bug in #572 unspellable. The old
/// `git_last_commit_hash -> Option<String>` collapsed two unrelated absences
/// into one `None`: "history exists, this spec simply is not in it" (drift is
/// genuinely zero) and "there is no history at all" (drift is UNKNOWN). Four
/// call sites read that `None` as the former and silently reported everything
/// up to date on a tree that had no `.git` — `report`, `check --stale`,
/// `score`, and the lifecycle `no_stale` guard, while only `stale` got it
/// right. Splitting the absence forces every caller — including the next one
/// written — through an exhaustive `match` the compiler checks, instead of
/// relying on each author to remember a guard helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecBaseline {
    /// The commit that last touched the spec. Drift is measurable from here.
    Commit(String),
    /// History exists, but git has no record of this spec file yet. There is
    /// nothing for the spec to be behind, so drift is genuinely zero.
    Untracked,
    /// There is no history to ask. Drift is unknown, and MUST NOT be reported
    /// as zero.
    Missing(MissingHistory),
}

/// Resolve the commit a spec's staleness is measured from.
///
/// The `missing_history` probe runs only when no commit came back, so a healthy
/// repository still costs exactly one `git log` per spec.
pub fn spec_baseline(root: &Path, spec_file: &str) -> SpecBaseline {
    match last_commit_hash(root, spec_file) {
        Some(hash) => SpecBaseline::Commit(hash),
        // A hash would have proven history exists; none did, so ask why.
        None => match missing_history(root) {
            Some(missing) => SpecBaseline::Missing(missing),
            None => SpecBaseline::Untracked,
        },
    }
}

/// Get the last commit hash that touched a file.
///
/// Deliberately private: its `None` is ambiguous, and every caller outside this
/// module must go through [`spec_baseline`], which disambiguates it.
fn last_commit_hash(root: &Path, file: &str) -> Option<String> {
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
    // Content first: add-then-revert commit pairs leave the file byte-identical
    // to its state at `spec_commit`, and pure commit counting would report
    // phantom drift. If the working-tree content matches the content at
    // `spec_commit`, there is nothing to catch up on.
    if let Ok(diff) = Command::new("git")
        .args(["diff", "--quiet", spec_commit, "--", source_file])
        .current_dir(root)
        .status()
        && diff.success()
    {
        return 0;
    }
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

/// Whether the repository has at least one commit.
///
/// A repository with an unborn `HEAD` is a git repository by every other test,
/// but there is no history for anything to be newer or older than. Callers that
/// decide staleness from history must treat it the same way they treat "not a
/// git repository" rather than reporting everything current (#558) — it is the
/// state `git init` leaves behind, which is where the quick start begins.
pub fn has_commits(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
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

    /// The commit a tracked spec resolves to, for tests that only need the hash.
    fn commit_of(root: &Path, spec: &str) -> String {
        match spec_baseline(root, spec) {
            SpecBaseline::Commit(hash) => hash,
            other => panic!("{spec} should be tracked, got {other:?}"),
        }
    }

    #[test]
    fn baseline_is_untracked_for_a_file_git_has_no_record_of() {
        let tmp = init_repo();
        // The repo HAS history, so "no commit for this path" genuinely means
        // there is nothing for the spec to be behind.
        commit_file(tmp.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(
            spec_baseline(tmp.path(), "specs/missing.spec.md"),
            SpecBaseline::Untracked
        );
    }

    #[test]
    fn baseline_is_a_commit_for_a_tracked_file() {
        let tmp = init_repo();
        commit_file(tmp.path(), "specs/auth.spec.md", "spec v1", "add spec");

        let hash = match spec_baseline(tmp.path(), "specs/auth.spec.md") {
            SpecBaseline::Commit(hash) => hash,
            other => panic!("tracked file should resolve to a commit, got {other:?}"),
        };
        assert_eq!(
            hash.len(),
            40,
            "expected a full 40-char SHA-1, got {hash:?}"
        );
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// The distinction the whole type exists for (#572): a tree with no
    /// history must NOT resolve to the same value as a tracked-but-uncommitted
    /// spec, because the first means "unknown" and the second means "zero".
    #[test]
    fn baseline_separates_no_history_from_no_commit_for_this_path() {
        let plain = TempDir::new().unwrap();
        assert_eq!(
            spec_baseline(plain.path(), "specs/auth.spec.md"),
            SpecBaseline::Missing(MissingHistory::NotARepository)
        );

        let unborn = init_repo();
        assert_eq!(
            spec_baseline(unborn.path(), "specs/auth.spec.md"),
            SpecBaseline::Missing(MissingHistory::NoCommits)
        );

        let healthy = init_repo();
        commit_file(healthy.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(
            spec_baseline(healthy.path(), "specs/auth.spec.md"),
            SpecBaseline::Untracked
        );
    }

    #[test]
    fn missing_history_reports_usable_history_as_none() {
        let plain = TempDir::new().unwrap();
        assert_eq!(
            missing_history(plain.path()),
            Some(MissingHistory::NotARepository)
        );

        let unborn = init_repo();
        assert_eq!(
            missing_history(unborn.path()),
            Some(MissingHistory::NoCommits)
        );

        let healthy = init_repo();
        commit_file(healthy.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(missing_history(healthy.path()), None);
    }

    #[test]
    fn commits_since_is_zero_when_source_unchanged_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "src/auth.rs", "fn login() {}", "add source");
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");

        let spec_commit = commit_of(root, "specs/auth.spec.md");
        // Source has not changed since the spec was committed.
        assert_eq!(git_commits_since(root, &spec_commit, "src/auth.rs"), 0);
    }

    #[test]
    fn commits_since_counts_source_changes_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");
        let spec_commit = commit_of(root, "specs/auth.spec.md");

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
