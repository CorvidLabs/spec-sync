## ADDED

### REQUIREMENT REQ-cmd-coverage-003

`coverage --format json` SHALL carry the findings the text renderer reports.

Acceptance Criteria
- The payload includes the errors and warnings, not only the coverage figures.
- A failing tree is distinguishable from a passing one by the payload alone.
