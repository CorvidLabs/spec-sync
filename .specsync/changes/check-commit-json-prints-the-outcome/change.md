---
id: check-commit-json-prints-the-outcome
state: draft
type: bug_fix
base_commit: d93cad4615c3b1de5dfd0b2fa19425f06a2bbad6
---

# Check commit json prints the outcome

## Intent

check commit json prints the outcome

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- check --commit --json prints one JSON outcome document carrying change, both commit ids, pushed, left_out, and configured_commands_not_run; a refusal prints the JSON error document with a non-zero exit

## No-spec Rationale

Not applicable
