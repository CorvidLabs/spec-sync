---
module: change
---

## ADDED

### REQUIREMENT REQ-change-150

When a delta item targets a requirement whose canonical header carries a title
suffix (`### REQ-<module>-<number> — Title`), the materializer SHALL match the
block by its requirement id and apply the delta; the materialized block SHALL
carry the canonical untitled header. Section deltas (`## ` targets) SHALL keep
exact heading matching.

#### Acceptance Criteria

- A MODIFIED delta naming `REQ-x-NNN` applies to a canonical block headed
  `### REQ-x-NNN — Any Title` and the result verifies.
- The materialized header is `### REQ-x-NNN` (the title is normalized away),
  so re-application converges.
- A `## ` section delta whose key is a strict prefix of an existing section
  heading does not match that section.
