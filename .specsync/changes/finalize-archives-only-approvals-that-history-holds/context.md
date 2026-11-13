---
change: finalize-archives-only-approvals-that-history-holds
artifact: context
---

# Context

Two gaps in archived approvals evidence (#428). First, finalize archived an `approvals.json`
edited after its commit, such as an actor rewritten to `mallory`, so the dated archive held a
ledger no commit contains. Second, earlier records in the ledger, including `reopenings[]`, could
be rewritten in a later commit with nothing noticing; only the current closing approval is
authenticated, and no digest covers an approval's actor.

Leif approved building it at 2026-11-13T03:49Z ("#428 refuse edited approvals") and chose
"Every committed version (Recommended)" at 04:04Z: the append-only proof runs through every
committed version of the change's own approvals file (coordinator chat, Claude Code hc t370,
relayed by claude-code-t370).
