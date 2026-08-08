## ADDED

### REQUIREMENT REQ-github-010

GitHub helpers SHALL expose an in-process check-run summary for a commit SHA so ship
readiness can report live CI trust without spawning `gh`.

Acceptance Criteria

- `fetch_commit_check_summary` requires `GITHUB_TOKEN`, uses in-process REST, and never
  spawns a `gh` process.
- Responses aggregate check-runs into overall status `green`, `pending`, `failed`, or
  `empty`.
- Failure conclusions (`failure`, `cancelled`, `timed_out`, `action_required`,
  `startup_failure`, `stale`) yield overall `failed`.
- Incomplete check-run status yields overall `pending` when no failure is present.
- Auth tokens are redacted from surfaced REST error messages.
