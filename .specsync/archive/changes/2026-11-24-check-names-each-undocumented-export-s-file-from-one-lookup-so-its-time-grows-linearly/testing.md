---
change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
artifact: testing
---

# Testing

| Behaviour | Evidence |
|---|---|
| check time grows linearly in undocumented exports (SPEED-1.b) | `check_scaling::check_time_grows_linearly_with_undocumented_exports` (red: ratio 9.0; green after) |
| warnings unchanged | the existing fix:: and check:: integration tests |

The timing test runs under the shared build lock locally; hosted CI runs it in the full suite.
