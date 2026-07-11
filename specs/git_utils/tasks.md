---
spec: git_utils.spec.md
---

## Tasks

- No open tasks.

## Done

- [x] Extract shared git helpers from stale/report behavior
- [x] Return safe defaults when git is unavailable or files are untracked
- [x] Replace `git_commits_between` with `git_commits_since` (takes a precomputed spec commit hash) to eliminate N+1 `git log` calls
- [x] Add a `#[cfg(test)]` module covering hash lookup, commit counting, invalid-commit degradation, and repo detection

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
