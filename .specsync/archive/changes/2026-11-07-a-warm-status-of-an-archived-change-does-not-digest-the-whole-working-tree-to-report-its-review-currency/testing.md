---
change: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
artifact: testing
---

# Testing

- `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (new; fails today: 89 Git processes with no extra files, 196 with 3,000).
- `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` (new; passes today and must stay passing). Full check vs summary: finalized and uncommitted gives true; archive committed, after a later change, after an unrelated working-tree edit, and for a moved archive all give false.
- Unchanged: the warm and cold archived-read guards, `fast_finalize`, the lifecycle integration tests, and the verify lane.
- On demand: Raven warm `change status` processes and time, before and after.

## Results

| Guard | Before | After |
|---|---|---|
| `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (0 vs 3000 extra files) | 89 vs 196, failed (`a5b54557`) | 70 vs 70 |
| `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (2 vs 8) | 91 vs 91 | 72 vs 72 |
| `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` | passes | passes (true right after finalize while uncommitted; false in the four committed cases) |
| `an_ignored_just_finalized_archive_reports_the_full_review_currency` (from review) | n/a | failed first (`f85d4a35`: summary false, full check true), passes |

Raven (150 archives, 5,629 tracked files; release builds; load average about 56), warm `change status <archived-id> --json`, three runs each:

| Build | Git processes | Wall |
|---|---|---|
| main after #979 | 183 | 6.2-13.1 s |
| This change | 65 | 1.8-1.9 s |

The JSON output is identical.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-147 | change tests `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (failing first, `a5b54557`), `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` and `an_ignored_just_finalized_archive_reports_the_full_review_currency` (failing first, `f85d4a35`) |
