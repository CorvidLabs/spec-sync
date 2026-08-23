---
change: db-tables-without-schema-dir-must-not-gate-a-release-on-advice-the-reader-cannot-take
artifact: testing
---

# Testing

## The adopter's shape, before and after

    before:  specsync check --strict  ->  exit 1, "1 warning(s) treated as errors"
    after:   specsync check --strict  ->  exit 0
             ⊘ DB table validation skipped: `db_tables` is declared but `schema_dir` is not configured

Both halves matter. The gate is gone AND the disclosure is still printed — a fix that silenced the
message would be a regression in visibility, which is what the original code comment was protecting.

## Discriminator and control

| test | baseline `f94ff7e4` | fixed | role |
|---|---|---|---|
| `db_tables_without_schema_dir_is_a_notice_not_a_warning` | FAILS | passes | true discriminator |
| `a_missing_db_table_is_still_an_error_when_schema_dir_is_configured` | PASSES | PASSES | true vacuity control |

The control passes on BOTH binaries. A fix that simply stopped checking `db_tables` would pass the
discriminator and fail the control.

## The control caught a bad fixture, which is worth recording

The control initially failed on the fixed tree. Cause: it set `schema_dir` but never created the
directory, so the schema was UNREADABLE — and #672's fix correctly degrades an unreadable schema to
"unknown" rather than claiming the table is missing. The fixture was wrong, not the code.

That is a live demonstration that the two fixes compose correctly: "no schema_dir" is a notice,
"schema_dir set but unreadable" is unknown, and "schema_dir readable, table absent" is an error.
Three distinct states, three distinct verdicts.

## Live controls

    CONTROL A  schema_dir set, `events` absent from a readable schema
               -> ✗ DB table not found in schema: events, exit 1     (unchanged)
    CONTROL B  schema_dir set, `events` present
               -> ✓ All DB tables exist in schema, strict exit 0     (unchanged, no notice)

## Suite

`cargo test`: 2354 unit + 405 integration, 0 failed. `cargo fmt --check` clean. `specsync check
--strict` on this repository: 0 warnings, 106/106 files.

## Not covered

Whether the accepted-change pile on the adopting repository actually clears once this lands. That
requires running `specsync change archive` on a live repository with 23 active changes, which is
the repository owner's decision and is deliberately not assumed here.
