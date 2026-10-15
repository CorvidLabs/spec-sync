## ADDED

### REQUIREMENT REQ-cmd-generate-003

Batch generation SHALL recognize an existing scoped module spec at the same leaf filename used by generation.

Acceptance Criteria
- A second batch request for @scope/demo reports already specced, preserves bytes and companions and never emits nested duplicates; flat module behavior is retained.
