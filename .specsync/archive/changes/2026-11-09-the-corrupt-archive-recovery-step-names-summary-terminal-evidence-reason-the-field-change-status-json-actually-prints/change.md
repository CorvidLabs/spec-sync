---
id: the-corrupt-archive-recovery-step-names-summary-terminal-evidence-reason-the-field-change-status-json-actually-prints
state: archived
type: bug_fix
base_commit: 75df08756cafeb096dbf03a6208db17c8c8a31fa
---

# The corrupt-archive recovery step names summary.terminal_evidence.reason, the field change status --json actually prints (#966)

## Intent

The corrupt-archive recovery step names summary.terminal_evidence.reason, the field change status --json actually prints (#966)

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- When a corrupt archive's package still matches the commit that archived it, the recovery step change status prints names summary.terminal_evidence.reason, and reading that exact path from change status --json gives a non-empty reason (#966, CHANGE-7.a.2)

## No-spec Rationale

Not applicable
