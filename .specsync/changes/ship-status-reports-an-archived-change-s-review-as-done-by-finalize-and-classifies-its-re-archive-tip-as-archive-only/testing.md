---
change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
artifact: testing
---

# Testing

| Behaviour | Evidence |
|---|---|
| An archived change's review is done by finalize, with no re-review | `commands::change::tests::an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review` (red: `stale`; green after) |
| A re-archive tip is archive_only; another change's package is product | `commands::change::tests::an_archive_tip_that_removes_its_own_package_files_is_archive_only` (red: `product`; green after) |
| A package the tip modified, or one still present, is product, as in CI | `commands::change::tests::an_archive_tip_whose_package_is_still_present_is_product` (red: `archive_only`; green after) |
| A workflow-v1 archive is not done by finalize and is told no review | `commands::change::tests::a_workflow_v1_archive_is_not_done_by_finalize_and_suggests_no_review` (red: `current`; green after) |
| An archived change lists no blocker, so ship-status exits 0 on it | the two archived-change tests above assert an empty `blockers` list |
| Existing tip and review semantics | `classify_paths_distinguishes_product_review_and_archive_tips` and the other ship-status tests |

Locally: `cargo test --bin specsync commands::change::`; hosted CI runs the full suite at the archived head.
