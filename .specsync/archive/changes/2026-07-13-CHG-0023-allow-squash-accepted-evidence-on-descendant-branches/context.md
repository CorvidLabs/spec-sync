---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: context
---

# Context

Squash merging replaces a feature commit with a new main-branch commit. The lifecycle already recognizes the committed accepted-state history when authorizing audited reopen, but closing validation only accepts an ancestor verification commit or a workspace byte-identical to the remote default branch. On the first descendant feature branch, HEAD is necessarily not an ancestor of remote main and later lifecycle audit records make the workspace non-identical, so otherwise-current accepted evidence fails every strict check.

The descendant fallback therefore requires the accepted state to exist on the remote default branch. Local-only accepted history, a missing remote default, changed delivery inputs, stale definitions, and mismatched closing approvals continue to fail closed.
