---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now
artifact: context
---

# Context

An accepted change goes stale the moment a later change edits one of its signed inputs, because
only an accepted or archived successor covers the edit. While the later change is in flight, every
accepted change that signed the same file fails `change audit --strict` at once, and a team
reopened six changes four times, about 20 minutes of re-verification each round (#657).

Leif approved building it at 2026-11-13T03:49Z ("#657 fewer re-verifies") and chose at 04:04Z
"It may archive (Recommended)": its archive records which change covered the shared file
(coordinator chat, Claude Code hc t370, relayed by claude-code-t370).
