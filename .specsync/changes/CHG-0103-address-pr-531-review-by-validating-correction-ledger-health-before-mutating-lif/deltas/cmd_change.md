## MODIFIED

### REQUIREMENT REQ-cmd-change-010

Lifecycle command adapters SHALL delegate existing-change mutations to domain operations that
validate correction-ledger integrity while holding the same project lock used for persistence.

Acceptance Criteria

- `answer`, `depend`, and `supersede` reject an invalid existing correction ledger before
  changing lifecycle files.
- A ledger that becomes invalid while a mutation waits for the project lock is rejected after lock
  acquisition and before persistence.
- Read-only text show, status, and list views retain their existing fail-closed behavior.
- Valid mutation and rendering behavior is unchanged.
- The `cmd_change` canonical contract version is incremented.
