---
change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
artifact: context
---

# Context

Issue #1012, a Codex P2 on #1006 tracked under Leif's two-round cap. When a list union holds a
quoted item such as `"2026-11-07"`, `render_list_item` treats it as plain because
`yaml_reads_as_non_string` covers booleans, nulls and numbers but not timestamps, so the merged
list writes `2026-11-07` unquoted. YAML 1.1 reads that as a timestamp, so a clean merge changes
the item's type, against REQ-merge-006 ("means what they meant").

Only the date-only form gets through `render_list_item`'s plain-character check: a full
timestamp holds `:` or a space, and so does a YAML 1.1 sexagesimal number, so both are already
quoted. The fix adds the YAML 1.1 date form `[0-9]{4}-[0-9]{2}-[0-9]{2}` to
`yaml_reads_as_non_string`. No requirement text changes.
