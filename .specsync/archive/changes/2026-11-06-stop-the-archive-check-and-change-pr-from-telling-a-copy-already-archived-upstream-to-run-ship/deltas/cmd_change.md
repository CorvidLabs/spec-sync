## MODIFIED

### REQUIREMENT REQ-cmd-change-029

`change status` and `change ship-status` SHALL report a change that is already archived upstream with the reference, the archive commit and the next step, in text and JSON, and SHALL NOT name finalize or ship as its next action. The pull-request sentence that `change status` and `change pr` print, and `.github/scripts/archive-readiness.sh` that the Archive check runs, SHALL NOT name `change ship` for such a change either, and SHALL name `change status` instead.

Acceptance Criteria
- Text output names the reference, the commit, the unarchived copy and the recovery: rebase onto that reference dropping the commit that added the copy.
- JSON readiness is not `safe`.
- With the remote default holding an archive of the change, the pull-request sentence names `specsync change status <id>` even when the stale copy carries a passing review at `HEAD` and a verification record (#902).
- `archive-readiness.sh` prints the same sentence when a remote default ref is present, using the same upstream-archive test; with no such ref, or an archive `HEAD` already contains, it keeps its classification.
