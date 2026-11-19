---
change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
artifact: context
---

# Context

Issue #1071. `change audit`, and every command that validates delta files, refused an approved
`## ADDED` requirement as "cannot add existing block … with different content" when the living
`requirements.md` had a `##` section right after it, though the requirement text was unchanged.
`living_requirement_conflicts` found the block with a raw substring search and ended it only at
the next `\n### `, so a following `## Notes` section counted as part of the requirement. The
materializer and `delta_item_is_applied` read the block with `markdown_block_range`, which ends
it at the next heading of level 3 or shallower, so the applier called the same delta applied.

The fix reads the living block through `markdown_block_range`, the one block reader #741 made
the applier and the applied check share. That also matches the header by the shared
requirement-heading reader, so a longer id no longer prefix-matches. REQ-change-048 and
REQ-change-049 already require this; no requirement text changes.
