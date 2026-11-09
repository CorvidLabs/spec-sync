## ADDED

### REQUIREMENT REQ-cli-args-recovery-retirement-001

The CLI SHALL parse change retire with an id and required actor, reason, replacement and authority-source arguments.

Acceptance Criteria
- Missing required arguments refuse before mutation; existing change command grammar remains unchanged.


- Optional --source-kind accepts accepted-receipt (default) or committed-audited-reopening-unproven; unknown values refuse before mutation.
