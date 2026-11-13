---
change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
artifact: testing
---

# Testing

| Behaviour | Evidence |
|---|---|
| A never-committed baseline is absent in a pathspec-magic project directory | `change::tests::a_never_committed_baseline_is_not_read_as_deleted_in_a_pathspec_magic_project_directory` (red before the fix, green after) |
| Deleted committed baselines are still refused | the existing workflow-v2 baseline tests |

Locally: `cargo test --bin specsync baseline`; hosted CI runs the full suite at the archived head.
