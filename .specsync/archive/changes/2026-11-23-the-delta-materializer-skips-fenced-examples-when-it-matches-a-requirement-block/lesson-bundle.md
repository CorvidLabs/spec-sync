# Lesson bundle: the-delta-materializer-skips-fenced-examples-when-it-matches-a-requirement-block

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The delta materializer skips fenced examples when it matches a requirement block
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests.rs, src/change_tests/fenced_requirement_example_delta.rs
- **Acceptance**: A MODIFIED delta for a requirement whose header also appears in a fenced example above it replaces the real block and keeps the example byte for byte; a heading inside a fenced example within the real block does not end the block (red then green)

## Evidence

- Verification commit: `04977932e2e8411d9d20a7ec0928d7217c045112`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change --strict`

## From the change's context.md

# Context

Issue #1050, from t384's review of #1049. `markdown_block_range` in
`src/change/collect_requirement_ids.rs` matches the delta target's header line by line with no
idea of fenced code. A fenced example in `requirements.md` that holds `### <id>` above the real
block is matched first, so a MODIFIED delta replaces the example and leaves the real requirement
as it was. A heading inside a fenced example within the real block also ends the block early.

REQ-change-152 has the materializer match a block by the shared requirement-heading reader, and
that reader (`validator::requirement_ids`) treats fenced examples as quoted text, never
requirements. So this is a bug fix with no requirement text. `parser::blank_fenced_code` already
blanks fenced lines while keeping every byte offset, so the scan can run on a blanked copy and
the range still indexes the original. The applier, the already-applied check and send-back all
read the block through this one function, so they stay in agreement.

## From the change's testing.md

# Testing

- Unit `a_modified_delta_skips_a_fenced_example_of_the_same_id`: a MODIFIED delta replaces the
  real block and keeps the fenced example above it byte for byte (fails on main: the example is
  replaced).
- Unit `a_fenced_heading_inside_the_block_does_not_end_it`: the whole real block, its fenced
  example included, is replaced (fails on main: the block ends at the fenced heading).
- The titled-header (#1024), stale-MODIFIED and send-back tests still pass.

## Where these lessons go

- `specs/change/change.3md`
