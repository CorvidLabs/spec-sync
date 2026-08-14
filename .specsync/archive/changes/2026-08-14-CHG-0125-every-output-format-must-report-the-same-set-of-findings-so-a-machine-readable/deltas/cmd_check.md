## ADDED

### REQUIREMENT REQ-cmd-check-011

`check` SHALL name every finding in table and csv.

Acceptance Criteria
- csv emits one row per finding with stable columns; a clean tree emits a header and no rows, distinguishable from a run that never happened.
- table emits an aligned list.
- Staleness findings appear in every non-text format, not only the tabular pair — they drive the exit code, so a format that omits them exits non-zero while naming nothing.
