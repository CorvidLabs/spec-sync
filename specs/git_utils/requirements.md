---
spec: git_utils.spec.md
---

## User Stories

- As a developer, I want git-aware spec tooling so that staleness and freshness are tracked automatically
- As a module consumer, I want a clean API for git log queries without reimplementing git2 boilerplate

## Acceptance Criteria

- `commits_since` returns accurate commit counts for files since a given timestamp
- `last_commit_for_file` returns the most recent commit touching a specific file
- `changed_files_since` lists files modified since a reference point
- All functions handle missing repos, untracked files, and shallow clones gracefully

## Constraints

- Must not panic on expected error conditions — return Results
- Must use git2 (libgit2) for git operations, not shell commands
- Must not hold repository locks longer than necessary
