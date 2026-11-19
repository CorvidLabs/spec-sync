---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-161 | `src/change_tests/rebase_stale_delta.rs`: a non-overlapping main edit is written into the delta as a proposal and refused; after `change approve` the check applies it and the requirement carries both edits; the canonical file and `approvals.json` are untouched by the proposal; each working-tree text (main's, the change's, the merge) gets the same proposal; overlaps, moves, a lost repeated word, main's deletion, main's heading edit, branch-side text, an older main text, a side branch's text and an unreadable merge-base are refused naming the cause; the #654 no-op; no proposal in CI or without a default branch; the 4,000-word limit; `tests/integration/rebase_stale_delta.rs` through the binary |
| REQ-change-122 | the existing `stale_modified_delta` tests, still refusing overlapping edits |

## How it runs

Unit and integration tests locally (red, then green), the full verify lane, and the real-repo run on the open changes of chough, starling and the seven triples' repositories.
