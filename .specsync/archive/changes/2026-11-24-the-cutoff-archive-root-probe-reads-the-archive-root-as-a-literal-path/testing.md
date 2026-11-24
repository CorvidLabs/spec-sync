---
change: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-145 | `change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory` (red before the fix, green after) and the existing `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` |
| REQ-change-136 | `cutoff_tree_verdict_equals_the_history_walk_verdict` still passes |

## How it runs

`change check` runs only the in-process spec-code sync. The unit tests run locally with
`cargo test --bin specsync cutoff`, and in hosted CI's full suite at the archived head.
