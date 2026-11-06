---
id: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
state: approved
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Stop the Archive check and change pr from telling a copy already archived upstream to run ship

## Intent

Stop the Archive check and change pr from telling a copy already archived upstream to run ship

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- With the remote default holding an archive of a change that this branch still holds unarchived, the pull-request sentence printed by `change status` and `change pr` names `specsync change status <id>`, never `specsync change ship <id>`, even when the stale copy carries a passing review whose `implementation_commit` equals `HEAD` and a verification record.
- `.github/scripts/archive-readiness.sh` reaches the same verdict and prints the same sentence when the remote default ref is present (`origin/HEAD`, else `origin/main`, else `origin/master`), using the same test as `upstream_archive_of`: a dated package of this change id on that ref whose `state.json` carries the id, added by a commit `HEAD` does not contain.
- An archive `HEAD` already contains (the reopen-and-rearchive path) and a repository with no remote default ref keep today's classification; the script never fetches, so the Archive job's single-commit checkout cannot see upstream archives, and that job still fails on any open change.
- A copy with no upstream archive and a passing review at `HEAD` is still told `specsync change ship <id>` by both.
- A Rust test (`an_upstream_archived_copy_is_not_told_to_ship` in `commands::change`) and a `test-archive-readiness.sh` case each fail on main cd8cab5e and pass with the fix.

## No-spec Rationale

Not applicable
