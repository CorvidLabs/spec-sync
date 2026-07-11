---
spec: cmd_stale.spec.md
---

## Tasks

- No open tasks.

## Done

- [x] Implement git-based stale-spec detection
- [x] Support text/table/csv, JSON, and markdown/github output
- [x] Sort results most-stale-first and exit non-zero when stale specs are found
- [x] Migrate to `git_commits_since` (one spec commit hash per spec) as part of the N+1 fix
- [x] Add integration coverage for non-git and fresh-repo behavior (`stale_outside_git_repo_fails_with_message`, `stale_outside_git_repo_json_reports_error`, `stale_in_fresh_repo_reports_all_up_to_date`)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
