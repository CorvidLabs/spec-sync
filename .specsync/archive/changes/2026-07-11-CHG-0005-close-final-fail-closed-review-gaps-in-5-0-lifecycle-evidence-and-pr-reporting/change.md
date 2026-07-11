---
id: CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting
state: archived
type: bug_fix
base_commit: 8196db41a6d9149f0551ced078d691cf5eeea2d5
---

# Close final fail-closed review gaps in 5.0 lifecycle evidence and PR reporting

## Intent

Close final fail-closed review gaps in 5.0 lifecycle evidence and PR reporting

## Affected Canonical Specs

- `change`
- `cmd_comment`

## Acceptance Criteria

- Persisted change IDs and spec scopes cannot escape their workspaces; corrupt approvals or tombstone history fail closed; CI requires fresh verification evidence; PR comments report SDD failures; focused and full cross-platform gates pass.

## No-spec Rationale

Not applicable
