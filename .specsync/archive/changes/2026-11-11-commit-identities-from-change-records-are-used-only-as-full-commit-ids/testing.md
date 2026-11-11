---
change: commit-identities-from-change-records-are-used-only-as-full-commit-ids
artifact: testing
---

# Testing

- Unit `a_recorded_commit_that_is_not_a_full_commit_id_is_never_handed_to_git` (fails on main).
- Integration `coverage_uses_a_changes_recorded_base_only_when_it_is_a_full_commit_id` (fails on main
  with "unable to inspect changed paths").
- The targeted succession, shallow-history, retirement, scoped-review, status and ship-status tests
  still pass.
