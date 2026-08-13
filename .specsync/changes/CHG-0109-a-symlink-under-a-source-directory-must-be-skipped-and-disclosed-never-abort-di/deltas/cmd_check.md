## ADDED

### REQUIREMENT REQ-cmd-check-006

Machine-readable check output SHALL carry the skipped symlinked entries.

Acceptance Criteria
- The JSON payload includes the full list of skipped entries, not a truncated summary.
- The field is present whenever the payload reports a result.
