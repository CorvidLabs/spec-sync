---
spec: git_utils.spec.md
---

## Tasks

- No open tasks.

- [x] Return from a Git child's output when that child exits, even if a descendant still holds the pipe (#854, #855, SPEED-1.e).

## Done

- [x] Extract shared git helpers from stale/report behavior
- [x] Return safe defaults when git is unavailable or files are untracked
- [x] Replace `git_commits_between` with `git_commits_since` (takes a precomputed spec commit hash) to eliminate N+1 `git log` calls
- [x] Add a `#[cfg(test)]` module covering hash lookup, commit counting, invalid-commit degradation, and repo detection
- [x] Add `tracked_by_git` and capped child output (REQ-git-utils-012, REQ-git-utils-013). Evidence: `capped_output_keeps_the_cap_and_reports_more` and the tracked-cache tests.
- [x] Match `tracked_by_git`'s path literally with a clean environment (REQ-git-utils-012). Evidence: `a_project_folder_named_like_a_glob_is_still_confined_to_itself`.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
