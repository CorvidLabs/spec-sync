---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: design
---

# Design

Delete, do not weaken. The removed code is a whole subsystem: the ancestry walk
(`verification_persistence_descendants`), the path allowlist
(`supported_verification_persistence_id`), the consistency check
(`ensure_verification_persistence_consistent`), and the commit-identity/ancestry arm of
`validate_verification_for_commit_binding` with its now-unused `current_commit` parameter.

`verification_commit_is_accepted_current` is deliberately left in place. It belongs to trusted
transitions, a later step; removing it here would be trimming across a subsystem boundary,
which is how both fixes recorded in `docs/GOAL-6-fixes.md` had to be reverted.

Requirement text is updated in this change rather than deferred to the reduction's
documentation step. A living requirement that describes deleted behaviour is precisely the
drift this tool exists to detect, and leaving `REQ-change-016` asserting that "normal
verification-commit ancestry remains mandatory proof" for several more steps would make the
spec lie about its own implementation.
