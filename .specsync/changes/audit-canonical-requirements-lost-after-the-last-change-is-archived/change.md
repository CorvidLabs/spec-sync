---
id: audit-canonical-requirements-lost-after-the-last-change-is-archived
state: approved
type: bug_fix
base_commit: 5314fb6ae13f7b9cc656cc2aef0fb875a59b2026
---

# Audit canonical requirements lost after the last change is archived

## Intent

Audit canonical requirements lost after the last change is archived

## Affected Canonical Specs

- `change`
- `cmd_check`

## Acceptance Criteria

- With no active changes on an SDD-enabled Git branch, both change audit and check --strict report each unexplained canonical requirement loss, its original file and a concrete restore command. Managed CI may run only strict check. Both commands pass on a separate no-loss control. A Git project that never enabled SDD keeps its usual strict drift checks. Approved branch removals and requirements moved under the same ID pass. A missing comparison ref is disclosed and incomplete comparison history gives fetch guidance.

## No-spec Rationale

Not applicable
