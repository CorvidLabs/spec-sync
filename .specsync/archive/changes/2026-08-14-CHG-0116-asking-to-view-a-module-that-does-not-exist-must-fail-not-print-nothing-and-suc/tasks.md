---
change: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
artifact: tasks
---

# Tasks

- [x] Count rendered specs and render failures rather than discarding both
- [x] Report an unknown module by name and exit non-zero
- [x] Suggest a near match, falling back to listing available modules
- [x] Make a render failure gate the exit code
- [x] Confirm an existing module still renders at exit zero
- [x] Confirm the unfiltered path is unchanged
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
