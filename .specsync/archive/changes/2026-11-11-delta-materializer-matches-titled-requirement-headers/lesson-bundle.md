# Lesson bundle: delta-materializer-matches-titled-requirement-headers

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Delta materializer matches titled requirement headers
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests.rs, src/change_tests/
- **Acceptance**: a MODIFIED delta naming REQ-X-NNN applies to a canonical block headed '### REQ-X-NNN — Title', and the materialized tree verifies

## Evidence

- Verification commit: `03639a9ae79c3d152fad433abc95482abb5146be`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

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

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/`: a MODIFIED delta applies to a titled
requirement header; the title is normalized away on write; a `## ` section
delta does not match a section whose heading merely starts with the key;
`delta_item_is_applied` recognizes the already-applied titled block.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-150 | change_tests unit tests exercising MODIFIED against titled and untitled headers |

## Where these lessons go

- `specs/change/change.3md`
