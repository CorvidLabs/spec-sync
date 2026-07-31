## ADDED

### REQUIREMENT REQ-cli-change-audit-001
The CLI SHALL expose `specsync change audit` as a first-class `ChangeAction` alongside `change check`.

Acceptance Criteria
- Help text distinguishes scoped check from active-only audit.
- Parsing accepts `change audit` with no change id.
