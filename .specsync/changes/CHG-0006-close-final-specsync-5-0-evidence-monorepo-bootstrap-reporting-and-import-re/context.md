---
change: CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re
artifact: context
---

# Context

The exact-commit review of PR #335 found ten additional gaps after the complete green matrix. They cluster around accepted-evidence freshness, archive safety, monorepo-relative Git behavior, canonical-spec coverage, bootstrap behavior, empty-spec PR reporting, contradictory no-spec declarations, and symlink-safe foreign imports.

CHG-0006 remains on the existing branch and PR. It also formalizes the observed archive-attribution edge: overlapping accepted changes must not allow a change to archive before its own delivery is absent from the comparison diff.

Independent evidence and security re-reviews found no blocker in the new behavior after ancestor-symlink preflight and required acceptance-input evidence. The three pre-field accepted workspaces must receive current scoped digests and new user-authorized acceptance approvals; the exact original self-adoption record remains a documented schema-v1 exception rather than a broad accepted-state bypass.
