## ADDED

### REQUIREMENT REQ-merge-004

The git merge driver SHALL merge two independent edits of one module spec so `version:`, the change-log rows, and `files:` do not conflict, and the version SHALL count every numeric bump from the common ancestor.

Acceptance Criteria
- Two branches that each bump the same integer version and append a different change-log row and `files:` entry merge in either order with one `version:` line, both rows, and both paths.
- The merged version is the ancestor plus both deltas, and it is never lower than either side.
- A one-sided edit is returned byte-for-byte.
- The marker resolver still uses `max(version)` and still leaves prose hunks manual.

## ADDED

### REQUIREMENT REQ-merge-005

Divergent prose in a spec section SHALL remain a conflict, and a requirements file SHALL keep both bodies when two branches add the same new requirement ID.

Acceptance Criteria
- A `## Purpose` paragraph edited differently on both sides stays a conflict and keeps both texts.
- Requirement blocks added under different IDs are both present after either merge order.
- The same new requirement ID with two different bodies is kept twice.
- An existing requirement edited differently on both sides stays a conflict.
