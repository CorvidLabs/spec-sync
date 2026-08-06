---
id: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
state: archived
type: bug_fix
base_commit: 2a9375f4fceb0d22766f1172bed9c7f152399a47
---

# Resolve canonical ownership at approve and free never-closed changes

## Intent

Resolve canonical ownership at approve and free never-closed changes

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- approve rejects declared paths no declared module owns, reporting all of them together; a never-closed verifying change corrects an owner without a reopen

## No-spec Rationale

Not applicable
