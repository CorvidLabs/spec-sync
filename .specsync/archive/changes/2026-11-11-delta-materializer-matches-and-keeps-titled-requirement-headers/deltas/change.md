---
module: change
---

## ADDED

### REQUIREMENT REQ-change-152

When a delta item targets a requirement whose canonical header carries a title
suffix (`### REQ-<module>-<number> — Title`, `### REQ-x-001: Title`), the
materializer SHALL match the block by the shared requirement-heading parser —
accepting every form the parser accepts, including multi-segment module ids
and up to three spaces of indent — and the block-end scan SHALL apply the same
ATX indent rule, so a matched indented block ends at the next heading. The
materialized block SHALL keep the matched header line unchanged. Section
deltas (`## ` targets) SHALL keep exact heading matching.

#### Acceptance Criteria

- A MODIFIED delta applies to a titled header, including a hyphenated module
  id (`REQ-cmd-change-012 — Title`) and a colon title (`REQ-x-001: Title`).
- A MODIFIED or REMOVED on an indented titled header leaves the following
  indented requirement block intact.
- The matched header line is preserved byte-for-byte in the materialized
  tree, and re-application converges.
- A `## ` section delta whose key is a strict prefix of an existing section
  heading does not match that section.
