## ADDED

### REQUIREMENT REQ-cmd-score-002

The `score` strict floor SHALL remain inclusive, and SHALL gate a directory mapping by that spec scoring zero rather than by changing the boundary.

Acceptance Criteria
- A total equal to the floor passes; only a total below it fails.
- A spec whose `files:` entry is a directory fails the floor because it scores zero, not because the comparison was made exclusive.
- `score` does not become a second `check`: it continues to report a metric for every spec it is given.
