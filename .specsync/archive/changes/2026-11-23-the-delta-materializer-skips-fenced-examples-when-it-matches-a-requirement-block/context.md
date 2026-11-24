---
change: the-delta-materializer-skips-fenced-examples-when-it-matches-a-requirement-block
artifact: context
---

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
