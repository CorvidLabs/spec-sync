## ADDED

### REQUIREMENT REQ-git-utils-002

Git helpers SHALL expose whether a repository has any history, distinctly from whether a
path is a work tree.

Acceptance Criteria
- A repository with at least one commit reports that it has history.
- A repository with an unborn HEAD reports that it does not, while still reporting as a work tree.
- A path that is not a repository reports neither.

## MODIFIED

### SPEC SECTION Public API
**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `git_last_commit_hash` | `root: &Path, file: &str` | `Option<String>` | Get the SHA hash of the last commit that touched a file |
| `git_commits_since` | `root: &Path, spec_commit: &str, source_file: &str` | `usize` | Count commits to source_file since a precomputed spec commit hash |
| `is_git_repo` | `root: &Path` | `bool` | Check if a directory is inside a git work tree |
| `has_commits` | `root: &Path` | `bool` | Whether the repository has any history. An unborn `HEAD` is a work tree by every other test, but nothing can be newer or older than a history that does not exist |

**Exported Types**

| Type | Kind | Description |
|------|------|-------------|
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details |

