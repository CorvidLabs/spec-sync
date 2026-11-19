---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: requirements
---

# Requirements

The deltas add REQ-change-159 and REQ-change-160 (`deltas/change.md`), REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037, REQ-cli-args-026 and REQ-cli-014.

They modify three existing requirements that described a Change Log row and a `version:` bump, because the issue removes both: REQ-change-021 (the row's column schema now governs how `specsync history` renders a fragment), REQ-change-092 (the #741 convergence: a module's record of the change is its fragment, or a row an earlier SpecSync wrote) and REQ-change-123 (send-back lists and refuses on that record, and names the fragment to delete). REQ-change-131's open-task gate and the #843 stale MODIFIED refusal (REQ-change-122) are unchanged. Intent: MERGE-1 (two changes to one module merge with no conflict on `version:` or the change-log row), here on GitHub's merge button. MERGE-4: no gate is loosened.
