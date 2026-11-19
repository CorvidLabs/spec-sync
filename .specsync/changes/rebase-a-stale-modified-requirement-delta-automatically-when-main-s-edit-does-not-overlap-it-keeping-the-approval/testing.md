---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-161 | `change::tests::a_branch_side_edit_is_never_rebased_into_the_delta` and `change::tests::a_rebase_whose_main_block_the_default_branch_does_not_hold_is_refused` (fable-lead's review; both fail on 2c06cef4), `rebase_stale_delta::a_branch_side_edit_to_the_requirement_is_refused_not_rebased`, `change::tests::a_non_overlapping_main_edit_rebases_the_delta_and_keeps_the_approval`, `change::tests::an_overlapping_main_edit_is_refused_and_writes_nothing`, `change::tests::a_rebase_record_that_does_not_recompute_is_refused`, the out-of-scope controls, and the word-merge unit tests |
| REQ-change-161 (merge-base reading; the tester's, Codex's and ss-lead's reviews of 0d31218a) | `change::tests::a_materialized_change_that_merged_main_is_rebased_whichever_side_the_merge_kept` and `change::tests::a_second_main_edit_after_a_materialized_rebase_is_rebased_again` (the tester's regression on 0d31218a), `change::tests::an_overlapping_main_edit_after_a_rebase_is_refused_even_when_the_branch_kept_its_text`, `change::tests::a_rebase_record_naming_the_wrong_main_commit_is_refused_and_named`, `change::tests::a_word_one_side_deleted_is_never_put_back_by_the_other`, `change::tests::a_rebase_past_the_size_or_count_bound_keeps_the_refusal`, `change::tests::the_rebase_hint_names_the_module_with_the_stale_requirement`, `change::tests::a_side_branch_text_its_merge_did_not_keep_is_never_main_s_block`, `change::tests::an_unreadable_default_branch_text_refuses_the_rebase`, `rebase_stale_delta::a_change_squash_merged_before_finalize_is_never_rebased_over_its_own_text` (#654) |
| REQ-change-122 | the existing `stale_modified_delta` tests, still refusing overlapping edits |
| REQ-cmd-change-038 | an integration test of text and JSON `change status` after a rebase, and `rebase_stale_delta::text_status_names_a_refused_rebase_record` |

## Checks

| Check | Expected |
|---|---|
| Non-overlapping main edit | rebased, both edits in the canonical requirement, approval valid, one record |
| Overlapping edit | refused as today, nothing written |
| Tampered record | refused, naming it |
| Section, REMOVED, v1, CI, unrecoverable base | today's refusal |
| Materialized, then merged main (clean, or resolved to either side) | rebased to the merge |
| Second main edit after a materialized rebase | rebased again over the last main block |
| Squash-merged before finalize, checked on main | no refusal, no record |

Locally: the full `cargo test`, clippy with `-D warnings`, fmt, the strict spec check (the verify lane) and the pre-push gate.
