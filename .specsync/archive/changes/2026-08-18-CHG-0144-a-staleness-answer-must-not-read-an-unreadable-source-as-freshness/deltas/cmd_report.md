## ADDED

### REQUIREMENT REQ-cmd-report-005

`report` SHALL decline to state a staleness it could not measure.

Acceptance Criteria
- A module whose cited files are all absent or are directories reports its staleness as unmeasured rather than as false with zero commits behind.
- Such a module is counted by the existing unmeasured-staleness total rather than by the stale or the current total.
- The run-level inconclusive flag is set whenever any module was unmeasured, for any reason, so it cannot disagree with that total.
