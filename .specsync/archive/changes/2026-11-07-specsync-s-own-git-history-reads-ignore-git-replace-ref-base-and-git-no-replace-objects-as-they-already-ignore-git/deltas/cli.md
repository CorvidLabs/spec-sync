## ADDED

### REQUIREMENT REQ-cli-013

At process start, before any thread runs and before any Git command, `specsync` SHALL remove `GIT_REPLACE_REF_BASE`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE` from its own environment, so every Git process it spawns walks the history its read-cache and lifecycle-index guards check (REQ-change-148).

Acceptance Criteria
- A `specsync` run with any of those variables set spawns Git processes that do not see them.
- `change status` under `GIT_REPLACE_REF_BASE` or `GIT_NO_REPLACE_OBJECTS` gives the plain read.
