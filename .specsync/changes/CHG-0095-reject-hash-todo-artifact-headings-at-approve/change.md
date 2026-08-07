---
id: CHG-0095-reject-hash-todo-artifact-headings-at-approve
state: approved
type: feature
base_commit: 547386ad362be171d430c4c2636392da6a789d6f
---

# Reject hash TODO artifact headings at approve

## Intent

reject hash TODO artifact headings at approve

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- artifact bodies that are only markdown TODO headings or bare TODO placeholders fail validate_artifacts and approve; HTML TODO still fails; real prose still passes; next_action lists incomplete artifacts

## No-spec Rationale

Sandbox #22 / product #495: plain markdown TODO headings pass approve while HTML TODO comments reject. Align placeholder detection.
