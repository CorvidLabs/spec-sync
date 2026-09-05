---
id: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
state: archived
type: bug_fix
base_commit: 6b1717038edb467d95bb483861f0c076da76deb5
---

# Archive preflight lets the package being closed cover the legacy change it supersedes, and stale-input diagnostics name the refused successor

## Intent

Archive preflight lets the package being closed cover the legacy change it supersedes, and stale-input diagnostics name the refused successor

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- `specsync change finalize` of a workflow-v2 change that supersedes inputs of a legacy accepted change succeeds: the archive post-move preflight authenticates the package being closed with its closing token and reads its succession entries from the working tree, so the legacy change is successor-covered before and after the archive commit
- A stale delivery-input diagnostic names every successor that claimed the input and was refused, with the reason (sorted by successor ID), instead of reporting that no successor covers it
- When the stale predecessor is workflow v1 and a refused claiming successor is workflow v2, the diagnostic directs the operator to finish that successor and does not offer `specsync change reopen` of the legacy change
- Regression tests: finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change and stale_accepted_change_error_names_the_successor_rejected_for_failed_authentication fail on ac796b8 and pass with the fix

## No-spec Rationale

Not applicable
