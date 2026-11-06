---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: requirements
---

# Requirements

- REQ-hash-cache-005 (added): a validation snapshot is replayed only by the build that recorded it. Its input digest frames a fingerprint of every source and manifest the binary was compiled from, so a snapshot from another build is re-validated. That includes one reporting the same package version, and one from before #878.
- REQ-change-134 (added): the persistent read cache is reused only by the build that wrote it. Its identity carries the same fingerprint, so another build of the same version that shares the Git common directory, such as a sibling worktree, recomputes the verdicts.
- Traces to HI CHECK-1.d (approved by Leif on 2026-11-05, relayed by the orc) and SPEED-1.c; issue #878; Leif's choice of approach B, "Both in one".
- REQ-change-133 belongs to ss-docs (#883), so this change takes REQ-change-134, as the orc's registry assigns.
