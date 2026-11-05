## ADDED

### REQUIREMENT REQ-change-125

Lifecycle commands SHALL stay within a tested ceiling of git process spawns per command, and the ceiling SHALL NOT grow with unrelated archived changes. The integration test `eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci` SHALL spawn at most 40% of its 15,563-process baseline measured at 67aa7dc2.

#### Acceptance Criteria

- A test counts git spawns for `change new`, `answer`, `approve`, `check`, `status`, `review` and `finalize` and asserts a ceiling for each.
- The slowest lifecycle integration test spawns at most 6,225 git processes.
- No assertion of an existing lifecycle test is removed or loosened to meet the ceiling.
