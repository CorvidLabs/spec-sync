## ADDED

### REQUIREMENT REQ-cmd-change-015

The change CLI SHALL accept a scoped-review `--reviewer` who is the definition approver, and SHALL NOT report an independent-review rejection solely for that reason. Next-action copy SHALL name `--reviewer <human>`.

Acceptance Criteria

- `specsync change review --reviewer <approver>` succeeds when other review gates pass.
- Status and finalize diagnostics use `--reviewer <human>` rather than requiring a second identity.
- A blocking verdict still blocks finalization.
