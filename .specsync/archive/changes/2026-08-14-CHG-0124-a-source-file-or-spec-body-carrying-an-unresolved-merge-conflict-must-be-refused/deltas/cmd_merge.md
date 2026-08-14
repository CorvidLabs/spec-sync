## ADDED

### REQUIREMENT REQ-cmd-merge-002

`merge` SHALL exit non-zero when its scan could not run.

Acceptance Criteria
- A scan that could not be performed exits non-zero rather than reporting that no conflicts need resolution.
- An unperformed scan is never reported as a pass.
