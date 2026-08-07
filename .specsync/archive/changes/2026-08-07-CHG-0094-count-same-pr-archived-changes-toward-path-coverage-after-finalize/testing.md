---
change: CHG-0094-count-same-pr-archived-changes-toward-path-coverage-after-finalize
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|---------|
| REQ-change-012 | same_pr_archived_change_covers_delivery_paths_with_zero_actives; historical_archive_not_in_delivery_does_not_cover_unrelated_paths |

## Commands

`cargo test same_pr_archived_change_covers historical_archive_not_in_delivery`
