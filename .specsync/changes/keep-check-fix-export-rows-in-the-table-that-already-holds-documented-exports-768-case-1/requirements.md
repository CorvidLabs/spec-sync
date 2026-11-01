---
change: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
artifact: requirements
---

# Requirements

The full requirement text is in `deltas/cmd_check.md`. Approval materializes it.

### REQ-cmd-check-017 (added)

`check --fix` puts every new export row for one Public API block into exactly one table and takes
the row width from that same table. The target is the last table that already holds a documented
export row; otherwise the first table with an export-like first-column header (`Name`,
`Function`, `Export`, `Symbol`) that is not a parameter or field table; otherwise the last table,
as before. Layouts (a), (c) and (d) from #768 and the #815 review are stated as criteria.

The id is the next free one. origin/main 709ab2bd ends at REQ-cmd-check-016, and neither open
draft (#822 release prep, #830 verification ignored paths) nor #815 adds a cmd_check id. #815
modified REQ-cmd-check-015 instead of adding one.

### REQ-cmd-check-015 (modified)

"The nearest Public API table" becomes "the target table that REQ-cmd-check-017 selects". The
statement is otherwise unchanged and every acceptance criterion is restated unchanged.

## Human-intent criteria

None is changed or claimed. No `hi/` criterion talks about where `check --fix` places a row.
