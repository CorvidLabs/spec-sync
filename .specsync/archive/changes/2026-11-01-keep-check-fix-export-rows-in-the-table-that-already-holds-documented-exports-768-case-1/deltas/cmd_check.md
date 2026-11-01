## ADDED

### REQUIREMENT REQ-cmd-check-017

`check --fix` SHALL insert every new export row for one Public API block into exactly one target table, chosen as follows, and SHALL take the row's column count from that same table: the last table in the block that holds a documented export row; otherwise the first table whose first-column header is export-like and which is not a parameter or field table; otherwise the last table in the block, as before.

Acceptance Criteria
- A block is one `###` export subsection, or the flat `## Public API` section when it has no `###` subsections. A table is a maximal run of consecutive table-row lines outside fenced or indented code; the fence and indentation rules are unchanged.
- A documented export row is a data row below the separator from which `get_spec_symbols` reads a symbol: the first non-empty backticked name in its first cell.
- An export-like first-column header is `Name`, `Function`, `Export` or `Symbol`, singular or plural, trimmed and compared ignoring case. A parameter or field table is one whose header has a column other than the first named `Type`, or whose first-column header is `Parameter`, `Param`, `Argument` or `Field`.
- Width and insertion point never come from two different tables.
- An export table `| Name | Kind | Description |` holding `a`, followed by prose and `| Parameter | Description |` (with or without a parameters table before it, under a heading, a bold label or no label, and with `Function` in place of `Name`) receives a 3-cell row for `b` directly after the `a` row (#768 case 1).
- `| Parameter | Description |` followed by `| Function | Description |` holding `a` receives a 2-cell row for `b` directly after the `a` row.
- `| Function | Description |` holding `a`, followed by prose and `| Name | Type | Description |`, receives a 2-cell row for `b` directly after the `a` row.
- In each case every other byte of the file is unchanged, `check --strict` passes, and a second `check --fix` changes nothing.

## MODIFIED

### REQUIREMENT REQ-cmd-check-015

`check --fix` SHALL insert each new export row immediately after the last row of the target Public API table that REQ-cmd-check-017 selects in the matching section or subsection, never after trailing prose, and SHALL append rows at the end of the section only when that section contains no table.

Acceptance Criteria
- A table under a `### Heading`, a `**Bold**` label, or no label followed by prose receives the row inside the table and the prose is preserved after it.
- A `## Public API` section with no table still receives rows at its end, as before.
- `check --strict` passes on the rewritten spec without further edits.
