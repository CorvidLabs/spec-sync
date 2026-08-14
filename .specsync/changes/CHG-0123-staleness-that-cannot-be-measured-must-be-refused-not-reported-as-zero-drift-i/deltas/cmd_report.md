## ADDED

### REQUIREMENT REQ-cmd-report-003

`report` SHALL refuse when staleness cannot be measured.

Acceptance Criteria
- Both unmeasurable states exit non-zero and name which one applies.
- JSON reports `null` for staleness, never `0` or `false`.
- The refusal is placed after the coverage computation, so an inconclusive coverage input still reports itself.
- A healthy repository is unchanged.
