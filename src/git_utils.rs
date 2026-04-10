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

/// Count commits that touched `source_file` since `spec_file` was last modified.
pub fn git_commits_between(root: &Path, spec_file: &str, source_file: &str) -> usize {
    let spec_commit = match git_last_commit_hash(root, spec_file) {
        Some(h) => h,
        None => return 0,
    };

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
