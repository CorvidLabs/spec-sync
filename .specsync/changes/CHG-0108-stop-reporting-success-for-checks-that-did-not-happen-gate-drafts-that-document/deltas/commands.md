## ADDED

### REQUIREMENT REQ-commands-006

A spec with `status: draft` SHALL be reported when it skips validation it could have
performed, and SHALL NOT be reported otherwise.

Acceptance Criteria
- A draft spec produces a warning when at least one mapped source file was present and readable AND its Public API names at least one symbol.
- A draft spec whose mapped files do not exist yet produces no such warning and continues to pass strict validation.
- A draft spec whose Public API names no symbol produces no such warning.
- Bare `specsync check` remains exit 0 in every draft case; only strict mode gates.
