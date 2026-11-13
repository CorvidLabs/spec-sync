---
change: a-delta-can-add-replace-or-remove-a-whole-section-of-requirements-md
artifact: design
---

# Design

`DeltaTarget` gains `RequirementsSection`. The parser recognizes `### REQUIREMENTS SECTION <name>`
case-insensitively, and the item-heading refusal names all three forms. Every place that maps a
target to its file and heading prefix maps it to `requirements.md` and `## `. The block range
for it is `markdown_block_range` with one more stop: a requirement heading, as the shared reader
(`validator::requirement_heading_id`) reads it, outside fenced code. So application, the
already-applied check, the approved-base digest, send-back restoration and the stale-MODIFIED
refusal all read the same section. Delta validation refuses an item whose content holds a
requirement heading outside a fence, naming `### REQUIREMENT`.
