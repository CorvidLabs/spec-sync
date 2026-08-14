## ADDED

### REQUIREMENT REQ-cmd-check-010

`check --stale` SHALL refuse when staleness cannot be measured.

Acceptance Criteria
- An explicitly requested measurement that cannot be taken exits non-zero rather than emitting zero warnings.
- JSON reports `null`, never an empty list.
- Plain `check` without `--stale` is unaffected.
