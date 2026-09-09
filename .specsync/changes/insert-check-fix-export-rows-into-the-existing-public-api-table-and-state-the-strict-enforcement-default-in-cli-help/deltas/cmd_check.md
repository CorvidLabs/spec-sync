## ADDED

### REQUIREMENT REQ-cmd-check-015

`check --fix` SHALL insert each new export row immediately after the last row of the nearest Public API table in the matching section or subsection, never after trailing prose, and SHALL append rows at the end of the section only when that section contains no table.

Acceptance Criteria
- A table under a `### Heading`, a `**Bold**` label, or no label followed by prose receives the row inside the table and the prose is preserved after it.
- A `## Public API` section with no table still receives rows at its end, as before.
- `check --strict` passes on the rewritten spec without further edits.
