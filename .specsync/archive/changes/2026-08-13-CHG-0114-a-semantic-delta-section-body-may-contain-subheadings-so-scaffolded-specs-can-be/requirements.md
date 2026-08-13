---
change: CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be
artifact: requirements
---

# Requirements

## REQ-change-065

A semantic delta SHALL accept subheadings within an item's body, and SHALL identify its own
items by keyword rather than by heading depth.

Acceptance Criteria
- A subheading met while an item is open is treated as that item's content.
- The spec sections a scaffold generates are accepted verbatim as delta content, without editing the spec first.
- A subheading appearing before any item is opened remains an error, because it cannot be attached to anything.
- That error names both valid item forms so the required shape is discoverable from the message.
