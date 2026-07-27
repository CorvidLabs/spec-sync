## ADDED

### REQUIREMENT REQ-change-042

Git candidate inspection SHALL deduplicate repeated stage-zero paths only when their normalized
mode and object identity are exact, while conflicting observations fail closed.

Acceptance Criteria

- A stage-zero path returned through overlapping bounded pathspec batches is represented once when
  every observed mode and normalized object ID is identical.
- A repeated path with a differing mode fails closed without replacing the first observation.
- A repeated path with a differing object ID fails closed without replacing the first observation.
- Parent-directory and exact-child candidate scopes remain valid across pathspec batch boundaries.
- Deterministic output bounds, unresolved-stage rejection, malformed metadata rejection, and
  out-of-scope path rejection remain unchanged.
