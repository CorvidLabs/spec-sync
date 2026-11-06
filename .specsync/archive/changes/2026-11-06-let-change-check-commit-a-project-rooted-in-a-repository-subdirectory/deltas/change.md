## ADDED

### REQUIREMENT REQ-change-128

Working-tree evidence capture SHALL read Git's modified-path output relative to the project root, so a project rooted in a subdirectory of its repository can complete `change check --commit`, and SHALL still refuse any path Git reports outside the project or outside the change's candidates.

Acceptance Criteria
- In a project at `app/`, `change check <id> --commit` commits the materialized spec and the verification evidence.
- A reported path without the repository prefix, or a prefixed path that is not a candidate, is refused as out of scope and named.
- A project at the repository root behaves as before.
