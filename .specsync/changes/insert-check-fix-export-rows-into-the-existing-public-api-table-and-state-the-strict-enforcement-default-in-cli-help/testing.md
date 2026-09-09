---
change: insert-check-fix-export-rows-into-the-existing-public-api-table-and-state-the-strict-enforcement-default-in-cli-help
artifact: testing
---

# Testing

- `REQ-cmd-check-015`: `fix_inserts_row_into_table_under_bold_label_before_trailing_prose`, `fix_inserts_row_into_table_under_heading_subsection`, `fix_creates_rows_at_section_end_when_no_table_exists` in `tests/integration/fix.rs`; each asserts row placement and that `check --strict --force` passes afterwards.
- `REQ-cli-args-016`: `specsync check --help` output inspected; existing `cli::tests::` pass.
- Regression: `cargo test --test integration fix_` (19 passed), `cargo clippy -- -D warnings`, `cargo fmt --check`, `specsync check --strict --require-coverage 100 --force`.
