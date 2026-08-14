---
spec: cmd_stale.spec.md
---

## Key Decisions

- **Git history as source of truth**: Staleness is based on commits to source files since the spec's last commit, not file modification time.
- **One spec commit, many source files**: the spec's last commit hash is resolved once via `spec_baseline`, then each source file's divergence is counted with `git_commits_since(root, spec_commit, source_file)` — this is the post-N+1-fix shape (`git_commits_between` was removed).
- **Most stale first**: Results sort by maximum commits behind so the highest-risk specs appear first.
- **Skip untracked history**: Specs not yet committed (no commit hash) and specs with empty `files` are counted as fresh to avoid false failures during bootstrap.
- **Non-zero on drift**: the command exits 1 when any stale spec is found so CI can gate on it.

## Files to Read First

- `src/commands/stale.rs` — CLI implementation, status filtering, and output formatting.
- `src/git_utils.rs` — `spec_baseline`, `git_commits_since`, `is_git_repo`, and the `StaleInfo` struct.

## Current Status

Stable. Implemented across all output formats with integration coverage for non-git and fresh-repo cases.

## Notes

- Use `specsync report` when stale information should be combined with coverage and validation status.
