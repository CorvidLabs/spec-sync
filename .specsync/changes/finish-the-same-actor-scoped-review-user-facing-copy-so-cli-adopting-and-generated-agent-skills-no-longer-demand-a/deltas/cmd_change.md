## MODIFIED

### REQUIREMENT REQ-cmd-change-015

The change CLI SHALL accept a scoped-review `--reviewer` who is the definition approver, and SHALL NOT report an independent-review rejection solely for that reason. Next-action copy SHALL name `--reviewer <human>`. Ship-status, status, and finalize diagnostics SHALL describe a scoped human review and SHALL NOT tell the operator to invent a second identity.

Acceptance Criteria

- `specsync change review --reviewer <approver>` succeeds when other review gates pass.
- Status, ship-status, and finalize diagnostics use `--reviewer <human>` rather than `--reviewer <other>`.
- Next-action and ship-status stage copy name scoped review, not a required second person.
- `docs/ADOPTING.md` examples use `--reviewer "<human>"` and do not instruct adopters to pass `"<someone else>"`.
- A blocking verdict still blocks finalization.
