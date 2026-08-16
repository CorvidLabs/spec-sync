## ADDED

### REQUIREMENT REQ-cmd-check-013

`check` SHALL produce identical findings on repeated runs over an unchanged tree.

Acceptance Criteria
- Two consecutive runs report the same findings in text and in JSON.
- A skipped spec is counted in `specs_checked` and its warnings are named.
- A genuinely clean spec still reports clean, so replay does not manufacture findings.
- `--force` and `--no-cache` are unaffected.
