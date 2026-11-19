---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-161 | `change::tests::a_branch_side_edit_is_never_rebased_into_the_delta` and `change::tests::a_rebase_whose_main_block_the_default_branch_does_not_hold_is_refused` (fable-lead's review; both fail on 2c06cef4), `rebase_stale_delta::a_branch_side_edit_to_the_requirement_is_refused_not_rebased`, `change::tests::a_non_overlapping_main_edit_rebases_the_delta_and_keeps_the_approval`, `change::tests::an_overlapping_main_edit_is_refused_and_writes_nothing`, `change::tests::a_rebase_record_that_does_not_recompute_is_refused`, the out-of-scope controls, and the word-merge unit tests |
| REQ-change-122 | the existing `stale_modified_delta` tests, still refusing overlapping edits |
| REQ-cmd-change-038 | an integration test of text and JSON `change status` after a rebase |

## Checks

| Check | Expected |
|---|---|
| Non-overlapping main edit | rebased, both edits in the canonical requirement, approval valid, one record |
| Overlapping edit | refused as today, nothing written |
| Tampered record | refused, naming it |
| Section, REMOVED, v1, CI, unrecoverable base | today's refusal |

Locally: the full `cargo test`, clippy with `-D warnings`, fmt, the strict spec check (the verify lane) and the pre-push gate.
