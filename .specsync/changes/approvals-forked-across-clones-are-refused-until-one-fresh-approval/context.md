---
change: approvals-forked-across-clones-are-refused-until-one-fresh-approval
artifact: context
---

# Context

Two clones can approve the same change at once: the project lock covers one working tree, so both
`change approve` commands succeed with different actors, and after the branches merge history
holds two approvals ledgers that cannot both be authoritative (#532, sandbox drill 026).

Leif approved building it at 2026-11-13T03:49Z ("#532 two-clone approvals") and chose the
recovery at 04:04Z: "One fresh approval (Recommended)", recording one new approval on the merged
branch (coordinator chat, Claude Code hc t370, relayed by claude-code-t370).
