---
change: a-failed-verification-records-the-findings-that-failed-it
artifact: context
---

# Context

Issue #640: a failed `CommandEvidence` carried only command, success and exit code, so a flake and a
regression could not be told apart without re-running the check. This change re-cuts PR #1027 (three
review rounds) as one fresh change on REQ-change-153, because #1027's archived rounds held
REQ-change-151, which coordinator t370 allocated to #1041 (2026-11-11). It adds requirement text, so
its definition approval is Leif's.
