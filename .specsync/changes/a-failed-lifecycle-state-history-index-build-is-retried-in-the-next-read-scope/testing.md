---
change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-140 | `change::tests::a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope` (red before the fix, green after); `cold_archived_status_git_processes_do_not_grow_with_archives` and the other fast-archived-read guards still pass |

## How it runs

`change check` runs only the in-process spec-code sync. The unit tests run locally with
`cargo test --bin specsync change::tests::` filtered to the archived-read tests, and in hosted
CI's full suite at the archived head.
