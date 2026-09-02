---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: requirements
---

# Requirements

REQ-change-046 currently requires a reviewer identity distinct from the definition approver. That SHALL change: scoped review records a stable reviewer claim and a pass/block verdict; the reviewer MAY be the same actor as the definition approver. GitHub required-review settings remain the merge authority. SpecSync MUST NOT refuse a solo ship that GitHub would merge.
