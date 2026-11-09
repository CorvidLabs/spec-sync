---
change: the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id
artifact: testing
---

# Testing

- `cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff` (new; fails before the fix at "the cutoff-tree check must accept what the history walk accepts", `14903d0b`).
- Unchanged: `cutoff_tree_verdict_equals_the_history_walk_verdict`, `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`, `cached_archived_history_refuses_a_hidden_downgrade_in_a_renamed_archive`, the integration test `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-146 | change test `cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff` (failing first, `14903d0b`); `cutoff_tree_verdict_equals_the_history_walk_verdict` unchanged |
