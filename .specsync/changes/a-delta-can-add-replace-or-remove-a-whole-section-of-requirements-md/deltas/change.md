## ADDED

### REQUIREMENT REQ-change-162

A semantic delta SHALL accept a third item kind, `### REQUIREMENTS SECTION <name>`, that targets the `## <name>` section of the module's `requirements.md`, such as `## Acceptance Criteria` or `## Constraints`. It SHALL add, replace or remove that whole section as `### SPEC SECTION <name>` does for the canonical spec: the same operations, convergence, approved-base digest for `## MODIFIED` and `## REMOVED`, and send-back restoration. The section SHALL end at the next heading of level 2 or shallower or at the next requirement heading, so a requirement block is never part of a section, and an item whose content holds a requirement heading SHALL be refused, naming `### REQUIREMENT`.

Acceptance Criteria
- A `## MODIFIED` item replaces the whole list and leaves every requirement block and every other section byte-identical; re-applying converges.
- `## ADDED` appends a missing section; one already present with the same content is applied, and one with other content is refused, naming `## MODIFIED`.
- `## REMOVED` drops only that section.
- A section followed by requirement blocks ends at the first of them.
- An item whose content holds `### REQ-...`, outside a fenced example, is refused before approval.
- A stale `## MODIFIED` item, whose section main rewrote after approval, is refused as for `### SPEC SECTION`.
- `### requirements section` parses as `### REQUIREMENTS SECTION`.
