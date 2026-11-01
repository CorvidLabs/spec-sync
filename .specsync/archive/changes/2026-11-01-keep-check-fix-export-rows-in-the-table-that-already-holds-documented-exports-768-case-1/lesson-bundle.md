# Lesson bundle: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep check --fix export rows in the table that already holds documented exports (#768 case 1)
- **Kind**: BugFix
- **Specs**: cmd_check
- **Paths**: src/commands/check.rs, tests/integration/fix.rs, specs/cmd_check
- **Acceptance**: `check --fix` inserts all new export rows for one Public API block (one `###` export subsection, or the flat `## Public API` section when it has no `###` subsections) into exactly one target table, immediately after that table's last row. A table is a maximal run of consecutive table-row lines outside fenced or indented code, with the fence and indentation rules unchanged.
- **Acceptance**: Target selection, in order: (1) the last table in the block that holds a documented export row, meaning a data row below the separator from which the parser reads a documented symbol (the first non-empty backticked name in its first cell, as `get_spec_symbols` reads it); (2) otherwise the first table whose first-column header, trimmed and compared ignoring case, is `Name`, `Function`, `Export` or `Symbol` (singular or plural) and which is not a parameter or field table, meaning one whose header has a column, other than the first, named `Type`, or whose first-column header is `Parameter`, `Param`, `Argument` or `Field`; (3) otherwise the table main used before this change (the last table in the block).
- **Acceptance**: The new row's column count is the header column count of that same target table. Width and insertion point never come from two different tables.
- **Acceptance**: Layout (a), #768 case 1: an export table `| Name | Kind | Description |` holding `a`, then prose, then `| Parameter | Description |`, under a `### Exported Functions` heading, a `**Functions**` label or no label, with and without a parameters table before the export table, and the same with `Function` in place of `Name`: the row for new export `b` has 3 cells and directly follows the `a` row.
- **Acceptance**: Layout (c): `| Parameter | Description |` followed by an export table `| Function | Description |` holding `a`: the 2-cell `b` row directly follows the `a` row, as on origin/main (#815 inserted it into the parameters table).
- **Acceptance**: Layout (d): an export table `| Function | Description |` holding `a`, then prose, then a parameters table `| Name | Type | Description |`: the 2-cell `b` row directly follows the `a` row (origin/main and #815 both put it in the parameters table; #815 also used 3 cells).
- **Acceptance**: Each of (a), (c) and (d) is an integration test in `tests/integration/fix.rs` that reuses #815's `assert_fix_768_table` helper, generalized to take the existing export row and the expected width: the rewritten file equals the input with only the new row added, `check --strict --force` passes, and a second `check --fix` leaves the file byte for byte unchanged. (a) and (d) fail on origin/main 709ab2bd; (c) and (d) fail on #815's code (PR head 80658a63, whose src and test change is carried onto this branch unchanged, authored by corvid-agent, right after scope approval); all pass with the fix.
- **Acceptance**: #768 case 2 (an indented literal fence marker hides the following table from symbol discovery) is not fixed. It is recorded as an open task in `specs/cmd_check/tasks.md` with the reason: fixing only the insertion scanner is insufficient because parser symbol discovery also treats the marker as a fence and can duplicate existing exports.
- **Acceptance**: No other `check` behavior changes. Sections with a single table, sections with no table (rows appended at the section end), the #615 trailing-prose rule, REQ-cmd-check-016 fence handling, header renames, duplicate suppression, `--dry-run`, `--backup` and JSON output behave as on origin/main, and the existing `fix_` integration tests pass unchanged.

## Evidence

- Verification commit: `2d87128f70258018dc164e2a798577192b243490`
- Base commit: `709ab2bd030540dcc03912449d176ec146ce1170`
- Verified by: `specsync check --spec cmd_check`

## From the change's context.md

# Context

Issue #768 lists three `check --fix` edge cases left over from the Codex review of #766. Case 3
(a fenced heading before the table) was fixed by #770. This change covers case 1 only: with two
real tables in one export block, `table_column_count` takes the width from the first table while
`table_rows_end` inserts after the last table, so the new row can land in a parameters table and
use the export table's width.

## Credit: #815 by corvid-agent

corvid-agent opened #815 (branch `fix/768-contiguous-export-table`, head 80658a63) for case 1. Its
idea is the basis of this change: group rows into contiguous tables and take width and insertion
offset from one table. Its code and its `fix_768_uses_one_contiguous_export_table` test with the
`assert_fix_768_table` helper (exact preservation, `check --strict`, idempotence) are carried onto
this branch unchanged, in a commit authored by corvid-agent, as the first step after scope
approval. They are not in the draft because `change audit` refuses changed source paths that no
approved change covers. Its `.specsync` lifecycle records and its hand edits to `specs/cmd_check`
are not carried.

This draft supersedes #815 for two reasons found in review:

1. **Regression (c).** #815 picks the first table whose header contains `Name`, otherwise the
   first table. For `| Parameter | Description |` followed by an export table
   `| Function | Description |`, origin/main correctly appends to the Function table, while #815
   inserts into the Parameters table.
2. **Archive cannot be reused.** #815's native change was reviewed and archived with
   corvid-agent as both author and reviewer, so it has no independent human review. Its spec
   change log also says v35 while the version moved from 34 to 36 (a hand bump and a
   materialization bump). Here the spec version and Change Log are left to materialization.

## Reproduction

A harness writes a spec with a given Public API body plus `src/auth/service.ts` exporting `a` and
`b`, runs `check --fix`, and prints the section. With a release build of origin/main 709ab2bd:

- (c) `| Parameter | Description |` then `| Function | Description |` holding `a`: `b` is
  appended to the Function table. Correct on main; the reviewer observed #815 putting it in the
  Parameters table.
- (d) `| Function | Description |` holding `a`, prose, then `| Name | Type | Description |`: `b`
  is appended to the parameters table with 2 cells. Wrong on main; wrong on #815 too, which
  prefers the `Name` table and uses 3 cells.
- (a) is #768 case 1 as #815's test states it: wrong on main whenever a parameters table follows
  the export table.

## Why "the table that holds exports" first

A `Name` header does not mark an export table: parameter tables commonly use
`| Name | Type | Description |`. The signal the checker itself trusts is a row it reads as a
documented symbol (`get_spec_symbols`: the first backticked name in a table row). A table that
already holds such a row is the export table by the checker's own definition, so the fixer and
the checker agree. Choosing the last such table keeps origin/main's placement whenever the last
table in the block is an export table, which is every layout the existing `fix_` tests use. The
header rule only decides when no table holds a documented row yet, and the final fallback is
origin/main's behavior, now with width from the same table.

## Out of scope

- #768 case 2 (indented literal fence marker). #815 found that fixing only the insertion scanner
  is not enough: parser symbol discovery also treats the marker as a fence, hides the following
  table and can duplicate existing exports. It stays an open task in `specs/cmd_check/tasks.md`.
- Routing new exports to a table by kind (functions vs types). Behavior there is unchanged.
- `####` method sub-tables, parser changes, and every other `check` behavior.

## From the change's testing.md

# Testing

Labels:

- A DISCRIMINATOR fails on the named base and passes with the fix.
- Bases: origin/main 709ab2bd, and #815's code (PR head 80658a63; the same src and test change
  is carried onto this branch first after approval). Labels are confirmed against builds of both before
  review and recorded here; they are not assumed.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cmd-check-017` | `fix_768_uses_one_contiguous_export_table` (layout a), `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table` (layout c), `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table` (layout d), all in `tests/integration/fix.rs` |
| `REQ-cmd-check-015` | the existing `fix_` suite in `tests/integration/fix.rs` and `tests/integration.rs`, unchanged and passing |

## Integration tests (`tests/integration/fix.rs`)

All three reuse #815's `assert_fix_768_table`, generalized to take the existing export row and
the expected cell count. It asserts that the `b` row has that width, directly follows the
existing row, that the file equals the input plus that one row, that `check --strict --force`
passes, and that a second `check --fix` is a no-op.

- DISCRIMINATOR on both `fix_768_uses_one_contiguous_export_table` (layout a, from #815):
  headed, bold-labelled and flat sections; with and without a leading parameters table, each
  with `Name` and `Function` headers. #815's version tried `Function` only without the leading
  parameters table; review asked for the combined case, which #815's code fails (it takes the
  width from the parameters table).
- DISCRIMINATOR on #815 `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table`
  (layout c): `| Parameter | Description |` then `| Function | Description |` holding `a`. Passes
  on origin/main; that is the point: it pins main's correct behavior that #815 broke.
- DISCRIMINATOR on both `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table`
  (layout d): `| Function | Description |` holding `a`, prose, then
  `| Name | Type | Description |`.

Header-rule and fallback coverage (only reached when no table holds a documented row), added on
approval:

- DISCRIMINATOR on both `fix_768_header_rule_picks_the_export_table_when_none_holds_a_row`:
  `| Name | Type | Description |` parameters table, an empty `| Function | Description |`
  table, prose, then `| Field | Description |`. Rows for `a` and `b` (2 cells) follow the Function
  table's separator.
- DISCRIMINATOR on both `fix_768_falls_back_to_the_last_table_with_its_own_width`:
  `| Parameter | Type | Description |`, prose, then an empty `| Item | Description |` table. Rows
  (2 cells) follow the last table's separator.

Each test runs under a `### Exported Functions` heading, a `**Functions**` label and no label.

## Regression lanes

- `cargo test --test integration fix_` (every existing `--fix` test passes unchanged).
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- `specsync check --strict --require-coverage 100 --force` and `fledge lanes run pre-push` before
  any push.

## Results

Labels confirmed by running `cargo test --locked --test integration fix_768` with the current
tests against each base's `src/commands/check.rs`:

| Test | origin/main 709ab2bd | #815 (carried, 40c6a4a0) | fix (2e36b08b) |
|------|----------------------|--------------------------|----------------|
| `fix_768_uses_one_contiguous_export_table` (a) | FAIL: rows directly follow the target table's last row | FAIL: must use the target table's width (`Function` header after a parameters table) | pass |
| `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table` (c) | pass | FAIL: rows directly follow the target table's last row | pass |
| `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table` (d) | FAIL: rows directly follow the target table's last row | FAIL: must use the target table's width | pass |
| `fix_768_header_rule_picks_the_export_table_when_none_holds_a_row` | FAIL: must use the target table's width | FAIL: must use the target table's width | pass |
| `fix_768_falls_back_to_the_last_table_with_its_own_width` | FAIL: must use the target table's width | FAIL: must use the target table's width | pass |

Regression lanes on the fix:

- `cargo test --locked --test integration fix`: 32 passed, 0 failed.
- `cargo fmt --check`: clean. `cargo clippy -- -D warnings`: clean.

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
