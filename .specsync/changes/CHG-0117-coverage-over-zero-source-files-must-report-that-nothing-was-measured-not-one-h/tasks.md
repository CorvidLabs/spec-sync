---
change: CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h
artifact: tasks
---

# Tasks

- [x] Report a zero file denominator as nothing measured rather than 100%
- [x] Report a zero line denominator the same way
- [x] Suppress the two affirmative lines that are vacuous over an empty set
- [x] Name the likely cause in their place
- [x] Confirm a project with source files is byte-for-byte unchanged
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
