---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: context
---

# Context

Issue #844. After a spec split in CorvidLabs/chough (#71), the verifying change of chough#69 needed a different scope. `change answer` only edits drafts, `change correct` only fixes accepted metadata, and `change reopen` only reopens accepted or archived work, so the only way out was deleting the workspace by hand. That loses its audit history, and while it stays open it blocks Archive.

Leif chose "Both A and B" in a Grok Bot widget on 2026-11-04 at 7:34 AM MST. That confirms option A (send back for re-scoping, CHANGE-12 to CHANGE-12.c), which this change implements. Option B (withdraw plus supersede) is a separate change and is not in this scope.
