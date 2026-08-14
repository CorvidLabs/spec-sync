## ADDED

### REQUIREMENT REQ-output-006

One finding list SHALL back every output format.

Acceptance Criteria
- The set of findings a consumer sees does not depend on which `--format` was requested; only presentation differs.
- Findings whose severity affects the exit code are included regardless of which collection they were gathered into.
- CSV fields containing separators or quotes are quoted so one finding is one row.
