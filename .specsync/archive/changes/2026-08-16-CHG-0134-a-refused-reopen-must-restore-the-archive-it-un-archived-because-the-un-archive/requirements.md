---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: requirements
---

# Requirements

`REQ-change-00N` — a refused reopen SHALL leave the archive as finalize wrote it,
report that it did so, and be retryable to the same refusal.

Out of scope: reopen's anchor preflight, which still fails after a squash merge
and remains pinned by sandbox drill 008.
