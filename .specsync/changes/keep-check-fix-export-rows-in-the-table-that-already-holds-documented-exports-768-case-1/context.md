---
change: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
artifact: context
---

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
