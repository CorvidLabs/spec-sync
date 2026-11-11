---
change: delta-materializer-matches-titled-requirement-headers
artifact: context
---

# Context

Issue #1024, found while landing CorvidLabs/chough#203 with SpecSync 6.2.0: a
`## MODIFIED` delta item for a requirement whose canonical header carries a
title suffix (`### REQ-cli-006 — Bound local input`) fails with
`cannot modify/remove missing block REQ-cli-006`.

Both requirement parsers — `requirement_heading_id` in `src/merge_driver.rs`
and in `src/validator/requirement_ids.rs` — already take the first
whitespace-separated token, so a title suffix is legal everywhere a
requirement is *read*. Only the materializer's block matcher
(`markdown_block_range` / `markdown_block_matches` in
`src/change/collect_requirement_ids.rs`) demands exact line equality, so a
titled header is invisible at *write* time.

Constraint: the tolerance must apply only to requirement blocks
(`### REQ-…`); a `## ` section key like `Change Log` must never match
`## Change Log Archive`. On write the canonical (untitled) header is emitted,
which keeps re-application convergent.
