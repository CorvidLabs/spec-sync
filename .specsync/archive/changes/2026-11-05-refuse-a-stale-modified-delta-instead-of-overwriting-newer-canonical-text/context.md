---
change: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
artifact: context
---

# Context

Issue #843. While merging main into CorvidLabs/chough PR #77, `specsync change check` applied #77's approved REQ-cli-007 MODIFIED body over the version #71 had just landed on main, deleting #71's paragraph and acceptance criterion. It exited 0, and `change audit` passed. It was caught only by grepping before and after.

Leif confirmed the criteria (CHANGE-11 to CHANGE-11.c) in a Grok Bot widget on 2026-11-04 at 7:34 AM MST. He chose to "Protect only changes approved from now on", so approvals written before this field are not re-judged.
