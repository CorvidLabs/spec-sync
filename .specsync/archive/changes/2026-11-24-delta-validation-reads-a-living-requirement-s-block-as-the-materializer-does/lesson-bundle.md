# Lesson bundle: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Delta validation reads a living requirement's block as the materializer does
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests.rs, src/change_tests/added_requirement_block_end.rs
- **Acceptance**: An ADDED requirement whose text is unchanged in the living requirements.md validates when a ## section follows it, and the applier reads the same block as applied; a different body under the same id is still refused as cannot add existing block (red then green)

## Evidence

- Verification commit: `0410342358c06367344266e404f9ba6e21cc7c7e`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change`

## From the change's context.md

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

## From the change's testing.md

# Testing

- Unit `an_unchanged_added_requirement_followed_by_a_section_is_not_a_conflict`: validation
  accepts it and the applier reads it as applied (fails on main with "cannot add existing block").
- Unit `a_changed_added_requirement_followed_by_a_section_is_still_a_conflict`: a different body
  is still refused.
- The existing ADDED-conflict, titled-header (#1024) and send-back tests still pass.

## Where these lessons go

- `specs/change/change.3md`
