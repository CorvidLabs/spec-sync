## ADDED

### REQUIREMENT REQ-cmd-change-010

Lifecycle commands that mutate an existing change and then render its record SHALL validate
correction-ledger integrity before persisting the mutation.

Acceptance Criteria

- `answer`, `depend`, and `supersede` reject an invalid existing correction ledger before
  changing lifecycle files.
- Read-only text show, status, and list views retain their existing fail-closed behavior.
- Valid mutation and rendering behavior is unchanged.
- The `cmd_change` canonical contract version is incremented.
