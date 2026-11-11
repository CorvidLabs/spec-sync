---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: delta
module: cmd_change
---

## MODIFIED

### REQUIREMENT REQ-cmd-change-008

`specsync change ship [ID]` SHALL run ship preflight for one change and, when
`ready_to_finalize` is true, perform finalize. When not ready it SHALL exit
non-zero and print blockers and the next stage without mutating state.
Ship preflight, including `--dry-run`, SHALL run the canonical-ownership check
finalize runs over the change's delivery inputs. A production-source input that
no declared module owns SHALL be a blocker naming the path, the module that owns
it, and the recovery (`specsync change send-back <id>`, then
`specsync change answer <id> affected_specs <modules>`); `change ship-status`
SHALL report the same blocker and SHALL NOT report the change ready to finalize
(#986, #956). An unowned input the change has not modified since its base
commit SHALL instead be named with a narrowing recovery
(`specsync change send-back <id>`, then
`specsync change answer <id> affected_paths <paths>` naming only what the change
edits), because declaring its owner would pull an untouched module into the
change (#1000).

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
- A no-spec-change change whose directory scope delivers production source a
  module owns is not ready: `change ship-status` names the blocker and the
  recovery, `change ship --dry-run` exits non-zero, and running the named
  recovery ships it (#986, #956).
- When a directory scope covers an unowned file the change never modified, the
  recovery names that file and says to narrow `affected_paths`; a modified
  unowned file in the same scope keeps the declare-the-owner recovery (#1000).
