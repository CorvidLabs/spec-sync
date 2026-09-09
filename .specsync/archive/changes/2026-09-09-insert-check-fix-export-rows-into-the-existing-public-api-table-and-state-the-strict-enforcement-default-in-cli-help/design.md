---
change: insert-check-fix-export-rows-into-the-existing-public-api-table-and-state-the-strict-enforcement-default-in-cli-help
artifact: design
---

# Design

Two small helpers in `src/commands/check.rs`: `is_table_row` and `table_rows_end(block)` returning the offset just past the last table row. Both existing insertion branches use it when a table exists; the third branch and the no-Public-API branch are untouched. The help text change is a doc comment on the `--enforcement` argument.
