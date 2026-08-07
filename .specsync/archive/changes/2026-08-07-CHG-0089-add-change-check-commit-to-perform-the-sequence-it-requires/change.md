---
id: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
state: archived
type: feature
base_commit: 45b1c3436d17c3d9015c9afc7d959a1ed7b94d29
---

# Add change check --commit to perform the sequence it requires

## Intent

add change check --commit to perform the sequence it requires

## Affected Canonical Specs

- `cli_args`
- `cmd_change`

## Acceptance Criteria

- change check --commit materializes, re-verifies, and records verification evidence; --push requires --commit; failed first verification commits nothing

## No-spec Rationale

Not applicable
