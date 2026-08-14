## ADDED

### REQUIREMENT REQ-commands-011

A `--require-coverage` gate SHALL fail when coverage could not be measured.

Acceptance Criteria
- An unmeasurable tree fails the gate rather than being compared against a substituted value.
- The gate's verdict and the percentage printed by the same run never disagree.
