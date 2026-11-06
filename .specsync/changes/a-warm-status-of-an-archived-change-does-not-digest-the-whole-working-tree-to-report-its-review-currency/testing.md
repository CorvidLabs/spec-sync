---
change: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
artifact: testing
---

# Testing

- `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (new; fails today: 89 Git processes with no extra files, 196 with 3,000).
- `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` (new; passes today and must stay passing). Full check vs summary: finalized and uncommitted gives true; archive committed, after a later change, after an unrelated working-tree edit, and for a moved archive all give false.
- Unchanged: the warm and cold archived-read guards, `fast_finalize`, the lifecycle integration tests, and the verify lane.
- On demand: Raven warm `change status` processes and time, before and after.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-147 | change tests `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (failing first, `a5b54557`) and `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` |
