# Lesson bundle — insert-check-fix-export-rows-into-the-existing-public-api-table-and-state-the-strict-enforcement-default-in-cli-help

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Insert check --fix export rows into the existing Public API table and state the strict enforcement default in CLI help
- **Kind**: BugFix
- **Specs**: cmd_check, cli_args
- **Paths**: src/commands/check.rs, src/cli.rs, tests/integration/fix.rs, specs/cmd_check, specs/cli_args, SCOPE.md
- **Acceptance**: check --fix inserts a new export row directly after the last row of the nearest Public API table in the section or subsection, under a heading, a bold label, or no label, and preserves trailing prose; a section with no table still receives rows at its end; a subsequent check --strict passes on the fixed spec; the --enforcement help summary names strict as the default; targeted fix integration tests, clippy, fmt, and strict spec validation pass

## Evidence

- Verification commit: `133f717ca1f0b54da10c4ec197d00045476f1457`
- Base commit: `38354c51ad3a4ba2408b8daf906dd1a9a0d5659f`
- Verified by: `specsync check --spec cli_args --spec cmd_check`

## From the change's context.md

# Context

Issue #615, reproduced with the 6.0 binary: `check --fix` computed its insert point as the end of the containing block (the matched `###` subsection, or the whole section when the label is bold or absent) and appended rows there, so any prose after the table received bare pipe rows outside every table and a following strict check still failed. The 6.0 pre-release doc sweep also found the `--enforcement` help summary still naming `warn` as the default while the enum and changelog say `strict`.

## Constraints

Minimal change in `auto_fix_specs`: locate the last table row in the block and insert there; keep the no-table fallback. No change to which table a row is routed to, to placeholder text, or to enforcement behaviour. The second defect in #615 (placeholder rows counting as documented) is out of scope.

## Implementation evidence

Three new integration tests in `tests/integration/fix.rs` cover bold label, heading subsection, and no-table cases; the 19 `fix_` integration tests pass; fmt and clippy are clean; strict spec validation passes at 100% coverage.

## From the change's design.md

# Design

Two small helpers in `src/commands/check.rs`: `is_table_row` and `table_rows_end(block)` returning the offset just past the last table row. Both existing insertion branches use it when a table exists; the third branch and the no-Public-API branch are untouched. The help text change is a doc comment on the `--enforcement` argument.

## From the change's testing.md

# Testing

- `REQ-cmd-check-015`: `fix_inserts_row_into_table_under_bold_label_before_trailing_prose`, `fix_inserts_row_into_table_under_heading_subsection`, `fix_creates_rows_at_section_end_when_no_table_exists` in `tests/integration/fix.rs`; each asserts row placement and that `check --strict --force` passes afterwards.
- `REQ-cli-args-016`: `specsync check --help` output inspected; existing `cli::tests::` pass.
- Regression: `cargo test --test integration fix_` (19 passed), `cargo clippy -- -D warnings`, `cargo fmt --check`, `specsync check --strict --require-coverage 100 --force`.

## Where these lessons go

- `specs/cmd_check/context.md`
- `specs/cli_args/context.md`
