---
change: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
artifact: requirements
---

# Requirements

REQ-change-140 (new, change): when a remote default exists and its archived state records cannot be read, the upstream-archive lookup reports that it cannot tell, and review, finalize, status and ship-status treat it as a refusal with a named recovery, never as no upstream archive (SHIP-3.c).
