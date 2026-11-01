---
id: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
state: approved
type: bug_fix
base_commit: 709ab2bd030540dcc03912449d176ec146ce1170
---

# Keep check --fix export rows in the table that already holds documented exports (#768 case 1)

## Intent

Keep check --fix export rows in the table that already holds documented exports (#768 case 1)

## Affected Canonical Specs

- `cmd_check`

## Acceptance Criteria

- `check --fix` inserts all new export rows for one Public API block (one `###` export subsection, or the flat `## Public API` section when it has no `###` subsections) into exactly one target table, immediately after that table's last row. A table is a maximal run of consecutive table-row lines outside fenced or indented code, with the fence and indentation rules unchanged.
- Target selection, in order: (1) the last table in the block that holds a documented export row, meaning a data row below the separator from which the parser reads a documented symbol (the first non-empty backticked name in its first cell, as `get_spec_symbols` reads it); (2) otherwise the first table whose first-column header, trimmed and compared ignoring case, is `Name`, `Function`, `Export` or `Symbol` (singular or plural) and which is not a parameter or field table, meaning one whose header has a column, other than the first, named `Type`, or whose first-column header is `Parameter`, `Param`, `Argument` or `Field`; (3) otherwise the table main used before this change (the last table in the block).
- The new row's column count is the header column count of that same target table. Width and insertion point never come from two different tables.
- Layout (a), #768 case 1: an export table `| Name | Kind | Description |` holding `a`, then prose, then `| Parameter | Description |`, under a `### Exported Functions` heading, a `**Functions**` label or no label, with and without a parameters table before the export table, and the same with `Function` in place of `Name`: the row for new export `b` has 3 cells and directly follows the `a` row.
- Layout (c): `| Parameter | Description |` followed by an export table `| Function | Description |` holding `a`: the 2-cell `b` row directly follows the `a` row, as on origin/main (#815 inserted it into the parameters table).
- Layout (d): an export table `| Function | Description |` holding `a`, then prose, then a parameters table `| Name | Type | Description |`: the 2-cell `b` row directly follows the `a` row (origin/main and #815 both put it in the parameters table; #815 also used 3 cells).
- Each of (a), (c) and (d) is an integration test in `tests/integration/fix.rs` that reuses #815's `assert_fix_768_table` helper, generalized to take the existing export row and the expected width: the rewritten file equals the input with only the new row added, `check --strict --force` passes, and a second `check --fix` leaves the file byte for byte unchanged. (a) and (d) fail on origin/main 709ab2bd; (c) and (d) fail on #815's code (PR head 80658a63, whose src and test change is carried onto this branch unchanged, authored by corvid-agent, right after scope approval); all pass with the fix.
- #768 case 2 (an indented literal fence marker hides the following table from symbol discovery) is not fixed. It is recorded as an open task in `specs/cmd_check/tasks.md` with the reason: fixing only the insertion scanner is insufficient because parser symbol discovery also treats the marker as a fence and can duplicate existing exports.
- No other `check` behavior changes. Sections with a single table, sections with no table (rows appended at the section end), the #615 trailing-prose rule, REQ-cmd-check-016 fence handling, header renames, duplicate suppression, `--dry-run`, `--backup` and JSON output behave as on origin/main, and the existing `fix_` integration tests pass unchanged.

## No-spec Rationale

Not applicable
