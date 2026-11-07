---
change: the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change
artifact: testing
---

# Testing

- `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` (new; returns `Ok(false)` before the fix, `45ef2006`).
- Unchanged: `cutoff_tree_verdict_equals_the_history_walk_verdict` (a cutoff with legacy archives, a record added after the cutoff), `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`, the partial-clone tests, the integration test `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-145 | change test `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` (failing first, `45ef2006`); `cutoff_tree_verdict_equals_the_history_walk_verdict` unchanged |
