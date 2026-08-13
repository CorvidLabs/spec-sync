---
change: CHG-0115-a-fix-request-that-could-not-be-applied-must-be-reported-not-reported-as-succes
artifact: tasks
---

# Tasks

- [x] Return what the fix could not do, not only how many it did
- [x] Report a write failure with its path and the OS error
- [x] Report a read failure the same way instead of skipping the spec
- [x] Emit failures on stderr in every format, not only text
- [x] Exit non-zero when any requested fix was not applied
- [x] Confirm a writable spec is still repaired and still exits zero
- [x] Confirm a dry run against an unwritable spec still exits zero
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
