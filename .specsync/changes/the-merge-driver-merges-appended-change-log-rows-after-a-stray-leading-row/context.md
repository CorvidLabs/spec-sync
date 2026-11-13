---
change: the-merge-driver-merges-appended-change-log-rows-after-a-stray-leading-row
artifact: context
---

# Context

Issue #895, found merging main into #848 with the #865 driver. `specs/change/change.spec.md`'s
Change Log opened with a stray, headerless row before the real table (since #480).
`parse_changelog_table` took the first `|...|` line as the header; the line after it was blank,
not a separator, so the table did not parse and the merge fell back to a text merge, which left a
diff3 conflict around the two appended rows. Leif decided (orc relay, 2026-11-06): the driver
tolerates a stray leading row and merges both rows; repairing that file's stray row is separate.
The fix takes the header as the first table row a separator row follows, so a stray row before it
stays in the prefix, which is already merged as text. REQ-merge-004 already requires this.
