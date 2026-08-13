## ADDED

### REQUIREMENT REQ-validator-011

A mapped source file SHALL be recorded as present only when it resolved to a readable
file.

Acceptance Criteria
- A mapping that is missing, planned, a directory, unreadable, or rejected for escaping the project root is not recorded as present.
- The Public API symbol count is computed from the spec body regardless of status, so a skipped spec still reports whether it documents a contract.
