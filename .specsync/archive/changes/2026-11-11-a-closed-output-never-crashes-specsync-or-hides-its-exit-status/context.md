---
change: a-closed-output-never-crashes-specsync-or-hides-its-exit-status
artifact: context
---

# Context

Issue #1036 was filed from the spec-sync worker's report: `specsync change status <id> | head -1`
panicked with exit 101 on 6.2.0 and main, masking the real 0/1 verdict. REQ-cli already requires
exit 0 on success and 1 on errors, so this is a bug fix with no requirement text.
