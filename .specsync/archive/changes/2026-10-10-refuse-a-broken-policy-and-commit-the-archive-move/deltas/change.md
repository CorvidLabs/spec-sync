## ADDED

### REQUIREMENT REQ-change-108
The system SHALL refuse to open a change when `.specsync/sdd.json` exists and cannot be parsed. The error SHALL say the SDD policy is invalid. A missing policy file SHALL still open a workflow-v2 change.

Acceptance Criteria
- `create_change` fails with `invalid SDD policy` when the policy file is `{`.
- `configured_principles_file` returns that same error for the same file.
- A missing policy file is not reported as an invalid policy.
