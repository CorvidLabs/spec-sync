---
id: name-the-spec-to-edit-when-change-approve-refuses-an-unowned-affected-path
state: implementing
type: bug_fix
base_commit: 376d17cd44940bd1e241f1d15b64bb8542fe2e0a
---

# Name the spec to edit when change approve refuses an unowned affected path

## Intent

Name the spec to edit when change approve refuses an unowned affected path

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- When change approve refuses an affected path that no declared module owns, the error says which spec to edit for each such path: the undeclared spec that already lists it, or the declared spec to add it to.

## No-spec Rationale

Not applicable
