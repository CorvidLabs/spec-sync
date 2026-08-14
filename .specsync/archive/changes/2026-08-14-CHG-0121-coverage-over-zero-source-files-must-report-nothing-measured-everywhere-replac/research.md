---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: research
---

# Research

Nine computing sites and fourteen consumers were enumerated before any edit, by
grepping for both the expression shape (`total_source_files == 0`, `total_loc ==
0`, `unwrap_or(100)`, `checked_div`) and the field names. The two counts differ
in kind: the expressions each re-derive the number, while the consumers inherit
it. Only removing the fields addresses the second group.

Prior art in this codebase: the same shape was found and fixed in `stale.rs` for
#558 and left in `report.rs` and `check.rs`, which is now #572. Fixing a
computation at one site while parallel implementations survive is this
repository's most repeated defect, and the type change is the only remedy tried
so far that the compiler enforces.
