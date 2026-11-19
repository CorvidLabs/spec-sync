---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: requirements
---

# Requirements

The deltas add REQ-change-159 and REQ-change-160 (`deltas/change.md`), REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037, REQ-cli-args-026 and REQ-cli-014.

They change no existing requirement. REQ-change-131's open-task gate, the #843 stale MODIFIED refusal and the #741 convergence rules are unchanged; only the evidence that a module records the change gains the fragment. Intent: MERGE-1 (two changes to one module merge with no conflict on `version:` or the change-log row), here on GitHub's merge button. MERGE-4: no gate is loosened.
