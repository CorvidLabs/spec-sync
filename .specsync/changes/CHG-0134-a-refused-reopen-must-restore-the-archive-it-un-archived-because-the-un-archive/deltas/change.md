## ADDED

### REQUIREMENT REQ-change-067

A refused reopen SHALL leave the archive as finalize wrote it.

Acceptance Criteria
- The dated archive package remains at its original path, with no orphan in the active workspace and the record still archived.
- The refusal states that the archive was restored, so a user whose reopen failed knows the package survived; if the restore itself fails, the message names the path to move back by hand.
- Retrying reproduces the same refusal rather than a different one, because the first attempt consumed nothing.
- A reopen that legitimately succeeds still un-archives, so the restore cannot be satisfied by never moving anything.
