## ADDED

### REQUIREMENT REQ-cmd-change-check-scoped-001
`specsync change check` SHALL run scoped verification for one change and SHALL NOT invoke full archive terminal-evidence revalidation.

Acceptance Criteria
- Text success ends with a verified marker and a Next action when possible.
- JSON emits verification only (not a full project archive evidence dump).
- Failure exits non-zero with actionable Next guidance.

### REQUIREMENT REQ-cmd-change-audit-001
`specsync change audit` SHALL report active-workspace and living-spec integrity and exit non-zero when the report contains errors.

Acceptance Criteria
- Output does not dump authenticated-history lines for archived changes.
- Checked count reflects active changes in scope.
