## MODIFIED

### REQUIREMENT REQ-cmd-change-008

`specsync change ship [ID]` SHALL run ship preflight for one change and, when
`ready_to_finalize` is true, perform finalize. When not ready it SHALL exit
non-zero and print blockers and the next stage without mutating state.

Optional orchestration flags after a successful finalize (or for already-archived
changes):

- `--push` SHALL commit the archive tip when the working tree is dirty and run
  `git push` for the current branch.
- `--wait` SHALL poll GitHub check-runs for HEAD (using the same in-process REST
  path as ship-status trust) until overall status is `green`, `failed`, timeout,
  or offline/`GITHUB_TOKEN` absent (reported as `local_guidance` without failing
  when no token).
- `--wait-timeout-secs` SHALL bound the wait (default 900).
- `--dry-run` SHALL refuse combination with `--push` or `--wait`.

Acceptance Criteria

- Exit code 0 only when preflight is clean and finalize succeeds (or the change
  is already archived and nothing remains), and optional push/wait succeed.
- Exit code non-zero when blockers remain, push fails, wait sees failed checks,
  or wait times out.
- Text and JSON outputs name the current tip class and next ship stage; JSON may
  include `push` and `wait` result objects when those flags are used.
- When sibling active changes remain after finalize, next guidance names them and
  requires their own check → review → ship cycle before merge.
