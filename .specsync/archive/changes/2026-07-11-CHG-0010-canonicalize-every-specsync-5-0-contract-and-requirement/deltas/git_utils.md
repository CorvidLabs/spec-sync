## ADDED

### REQUIREMENT REQ-git-utils-001

Git utility helpers SHALL execute bounded, non-interactive Git queries and SHALL distinguish valid empty results from unavailable comparisons.

Acceptance Criteria
- `git_last_commit_hash(root, file)` returns `Some(full SHA)` for a tracked file and `None` for an untracked/never-committed file
- `git_commits_since(root, spec_commit, source_file)` returns the count of commits touching `source_file` in the range `spec_commit..HEAD`, and `0` when the range is empty or the commit ref is invalid
- `is_git_repo(root)` returns `true` inside a git work tree and `false` otherwise
- `StaleInfo` carries `spec_path`, `module_name`, `max_commits_behind`, and `source_details: Vec<(String, usize)>`
- All commands run with `current_dir(root)`
- Any git command that fails to spawn or returns unparseable output yields the safe default (`None` / `0` / `false`)
