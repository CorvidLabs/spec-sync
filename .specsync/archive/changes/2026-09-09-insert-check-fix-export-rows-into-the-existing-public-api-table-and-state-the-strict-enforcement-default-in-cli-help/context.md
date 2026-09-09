---
change: insert-check-fix-export-rows-into-the-existing-public-api-table-and-state-the-strict-enforcement-default-in-cli-help
artifact: context
---

# Context

Issue #615, reproduced with the 6.0 binary: `check --fix` computed its insert point as the end of the containing block (the matched `###` subsection, or the whole section when the label is bold or absent) and appended rows there, so any prose after the table received bare pipe rows outside every table and a following strict check still failed. The 6.0 pre-release doc sweep also found the `--enforcement` help summary still naming `warn` as the default while the enum and changelog say `strict`.

## Constraints

Minimal change in `auto_fix_specs`: locate the last table row in the block and insert there; keep the no-table fallback. No change to which table a row is routed to, to placeholder text, or to enforcement behaviour. The second defect in #615 (placeholder rows counting as documented) is out of scope.

## Implementation evidence

Three new integration tests in `tests/integration/fix.rs` cover bold label, heading subsection, and no-table cases; the 19 `fix_` integration tests pass; fmt and clippy are clean; strict spec validation passes at 100% coverage.
