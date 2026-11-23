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

## Acceptance Criteria

- With zero active changes, audit reports each vanished canonical requirement, its original file and a concrete restore command. An approved removal archived on this branch accounts for its own ID. An older archive cannot excuse a new deletion. A full-history CI checkout rejects the loss. A missing default reference is disclosed and an unreadable merge base gives actionable fetch guidance.

## No-spec Rationale

Not applicable
