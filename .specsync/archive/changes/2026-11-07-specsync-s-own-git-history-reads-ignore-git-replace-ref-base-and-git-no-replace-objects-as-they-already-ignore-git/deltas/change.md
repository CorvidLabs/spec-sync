## ADDED

### REQUIREMENT REQ-change-148

SpecSync's lifecycle reads SHALL walk only the history its read-cache and lifecycle-index guards check: the object graph with `refs/replace/` and `info/grafts`. Environment that points Git at other replacement or graft sources (`GIT_REPLACE_REF_BASE`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE`, `GIT_SHALLOW_FILE`) SHALL NOT change what a lifecycle read concludes, whether it answers fresh or from the persistent read cache (SPEED-1.a.2, CI-2).

Acceptance Criteria
- With a replacement under a custom `GIT_REPLACE_REF_BASE` that swaps HEAD for a copy whose archived record lost its workflow version, `change status` under that variable gives exactly the plain read, fresh and cached.
- `GIT_NO_REPLACE_OBJECTS=1` gives the plain read too.
