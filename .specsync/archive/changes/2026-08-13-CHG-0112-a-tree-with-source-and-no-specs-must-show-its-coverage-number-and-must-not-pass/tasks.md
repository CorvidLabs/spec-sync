---
change: CHG-0112-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
artifact: tasks
---

# Tasks

- [x] Print the coverage line unconditionally in the no-specs branch
- [x] Gate strict validation when the tree contains source files
- [x] Confine the gate so an empty project still exits zero
- [x] Add the source-file count and coverage percent to the machine-readable payload
- [x] Verify the empty-project control is unchanged
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
