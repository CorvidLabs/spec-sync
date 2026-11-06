## ADDED

### REQUIREMENT REQ-cmd-change-028

`change status` and `change ship-status` SHALL report a change that is already archived upstream with the reference, the archive commit and the next step, in text and JSON, and SHALL NOT name finalize or ship as its next action.

Acceptance Criteria
- Text output names the reference, commit and the merge or rebase step.
- JSON readiness is not `safe`.
