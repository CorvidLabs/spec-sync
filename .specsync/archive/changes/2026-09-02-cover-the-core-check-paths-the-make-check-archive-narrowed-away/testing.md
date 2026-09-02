---
change: cover-the-core-check-paths-the-make-check-archive-narrowed-away
artifact: testing
---

# Testing

No new assertions: this record changes no file. The paths it covers are already exercised —
`tests/integration/commands.rs` is run by `cargo test --test integration` (409 integration
tests, 0 failures, on this branch; `full-test.log`), and the `cmd_check` / `cmd_init` specs
that own the two production files pass `specsync check --strict` (62/62).

The gate this record exists for is `specsync change audit --strict`: it exits 1 on the branch
tip before this record and must exit 0 after the record is archived. That command is what
`.github/workflows/trust.yml` runs, so the same verdict lands in CI.
