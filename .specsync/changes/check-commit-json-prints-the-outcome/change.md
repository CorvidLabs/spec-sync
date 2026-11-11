---
id: check-commit-json-prints-the-outcome
state: implementing
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Check commit json prints the outcome

## Intent

check commit json prints the outcome

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- change check --commit --json prints one JSON document with the change id, both commit ids, pushed and left_out; a refusal prints the JSON error document with a non-zero exit

## No-spec Rationale

the --json path must answer with one document (REQ-change-004: equivalent human and machine workflows; cli invariant 5: --json switches all output to JSON) — the commit path printing nothing was the bug
