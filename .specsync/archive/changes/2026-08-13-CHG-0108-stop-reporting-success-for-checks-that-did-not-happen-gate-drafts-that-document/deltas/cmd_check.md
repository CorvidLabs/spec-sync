## ADDED

### REQUIREMENT REQ-cmd-check-005

`specsync check` SHALL report requirements drift and companion updates only for
classifications observed against a known baseline.

Acceptance Criteria
- A project with no hash cache reports no requirements-drift warnings in any output format.
- The warning count, the machine-readable staleness entries, and the review hint all follow the same condition.
- Spec selection is unaffected: the same specs are re-validated whether or not a baseline exists.
