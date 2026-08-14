## ADDED

### REQUIREMENT REQ-cmd-comment-005

`comment` SHALL exit with the verdict it renders.

Acceptance Criteria
- A rendered failure verdict exits non-zero; a rendered pass verdict exits zero.
- The exit status agrees with `check` over the same project and flags.
- `--require-coverage N` gates `comment` as it gates `check`, `score`, `report` and `deps`.
- The comment body and the conditions for posting it are unchanged.
