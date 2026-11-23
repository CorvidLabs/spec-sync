---
id: audit-canonical-requirements-lost-after-the-last-change-is-archived
state: draft
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

- With no active changes, both change audit and check --strict report each unexplained canonical requirement loss, its original file and a concrete restore command. The same commands pass on a separate no-loss control. Removals authorized on the branch and requirements moved under the same ID pass. Both entry points disclose a missing comparison ref and give fetch guidance when the known comparison history is incomplete.

## No-spec Rationale

Not applicable
