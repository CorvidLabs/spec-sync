---
change: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
artifact: context
---

# Context

Both accepted CHG-0048 records own the GitHub Action integration guide from their independent
branches. The integrated guide truthfully contains the release-version guidance from one branch and
the musl artifact from main, so both exact acceptance manifests require audited refresh.

The sequence ledger also acknowledges these records as one immutable historical collision. Reopen
changes the active state to `verifying`, and the current collision validator mistakes that governed
delivery refresh for mutable history even though the prior accepted evidence and closing approval
remain preserved in the append-only reopen audit.
