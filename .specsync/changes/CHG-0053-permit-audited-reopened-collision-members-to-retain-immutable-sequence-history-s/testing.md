---
change: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
artifact: testing
---

# Testing

- `REQ-change-035`: an acknowledged collision remains valid through a structurally and
  definition-valid audited reopen.
- Tampered, missing, unapplied, wrong-state, failed-verification, stale-definition, and
  closing-digest-mismatched reopen evidence remains mutable and fails collision validation.
- Both real CHG-0048 records complete full unit/integration and release-validator verification.
- Final `specsync check --strict --require-coverage 100 --force` and `fledge trust verify` pass.
