---
change: CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance
artifact: context
---

# Context

Three review findings exposed gaps in the audited reopen path. Reacceptance compared a compatible legacy definition digest by exact equality, no-spec accepted workspaces could be treated as later canonical governance, and effective-contract validation excluded reopened canonical-applied changes while they were verifying. The normal ancestry path remains unchanged; all cited rollout verification commits are ancestors of the current head.
