---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: context
---

# Context

Issue #1043, found while working PR #1037: one send-back by the installed 6.2.0 removed four relayed
approval records from an active change. Coordinator t370 (2026-11-11) chose preserve over refuse,
because refusing contradicts hi CHANGE-13.h, and asked for one change that also fixes #434 part 1.
The delta adds requirement text, so the change stops at READY for Leif's definition approval.
