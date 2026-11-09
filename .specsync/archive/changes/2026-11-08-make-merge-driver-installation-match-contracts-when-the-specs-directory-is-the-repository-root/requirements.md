---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: requirements
---

# Requirements

The confirmed human intent, unchanged:

- MERGE-1: Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's `.spec.md` (`version:`, change-log row, `files:`) or on `requirements.md`.

The orc confirmed on 2026-11-06 that no new HI wording is needed and registered REQ-cmd-merge-004 for this fix. REQ-merge-007 was explicitly set aside because the requirement belongs to `cmd_merge`.

REQ-cmd-merge-004 requires `merge --install` to produce Git attributes that actually select the SpecSync driver when the configured specs directory is the repository root. Both `.` and `./` must work for root and nested `.spec.md` and `requirements.md` files, without changing existing non-root installation behavior.
