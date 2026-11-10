---
change: deps-reports-one-missing-dependency-as-one-error
artifact: tasks
---

# Tasks

- [x] `validate_deps` skips a second error for an edge whose entry already reported it
- [x] Unit test: one error for a bare and for a path entry, with `missing_deps` intact
- [x] Flip the #606 pin in `regression_w1.rs` to one occurrence
