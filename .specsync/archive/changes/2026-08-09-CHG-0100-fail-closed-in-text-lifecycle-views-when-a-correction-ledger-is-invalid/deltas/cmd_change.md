## ADDED

### REQUIREMENT REQ-cmd-change-009

Text `specsync change show`, `specsync change status <id>`, and aggregate
`specsync change status` SHALL fail closed before emitting a successful lifecycle projection
when an active change has invalid correction-ledger health.

Acceptance Criteria

- Each affected text command exits non-zero and emits the same safe correction-ledger integrity
  diagnostic.
- No successful identity, answer, next-action, or correction-count output precedes that diagnostic.
- JSON inspection retains its typed fail-closed behavior.
- Valid active changes retain their existing text output and exit status.
