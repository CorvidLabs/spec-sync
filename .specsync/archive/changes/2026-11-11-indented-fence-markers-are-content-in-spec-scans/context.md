---
change: indented-fence-markers-are-content-in-spec-scans
artifact: context
---

# Context

#768 re-cut per t370: fence fix only (the table-selection reorder is dropped — t384 showed it contradicts REQ-cmd-check-017). Both scanners (export_table in check.rs, blank_fenced_code in parser.rs) trimmed the indent before the fence check; a 4-space-indented fence marker opened a phantom fence and the real table read as fenced, so --fix appended a duplicate row. Both now check indentation first. Case 1 of the issue stays open (needs a requirements decision).
