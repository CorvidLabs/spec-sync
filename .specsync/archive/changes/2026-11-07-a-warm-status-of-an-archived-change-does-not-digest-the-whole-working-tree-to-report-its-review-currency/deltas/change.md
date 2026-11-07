## ADDED

### REQUIREMENT REQ-change-147

A change summary of an archived change SHALL NOT take the whole-tree project-input digest to report `scoped_review_current` once its archived package is committed; it SHALL report `false`, which is what the full check gives for a committed archive. While the archived package has uncommitted changes, the summary SHALL compute it as before. A warm `change status` of an archived change SHALL NOT spawn more Git processes as the project tracks more files (SPEED-1.a). Summaries of changes before archive are unchanged.

Acceptance Criteria
- `warm_archived_status_git_processes_do_not_grow_with_tracked_files`: 3000 more tracked files add fewer than 3 Git processes to a warm archived status.
- `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives`: the summary reports what the full check gives, right after finalize (uncommitted), once committed, after later commits, after an unrelated edit and for a moved archive.
