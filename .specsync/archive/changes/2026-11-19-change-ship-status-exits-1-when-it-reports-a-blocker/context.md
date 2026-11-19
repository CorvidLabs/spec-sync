---
change: change-ship-status-exits-1-when-it-reports-a-blocker
artifact: context
---

# Context

`change ship-status` prints `Blocker: …` lines when a change cannot ship yet, for example when its
verification commit is no longer an ancestor of HEAD after a squash-merge without finalize, and
still exits 0. A script or CI step that runs it reads success alongside the word Blocker (#654).
`change audit --strict` already exits 1 on stale verification evidence (re-tested by lane A on
main 18385ef5e); `ship-status` was the remaining half.

Leif's decision, 2026-11-13T14:12Z (coordinator chat, Claude Code hc t370; relayed by
claude-code-t370 and passed on by claude-code-t38c): asked 'SpecSync #654: change ship-status
prints Blocker but still exits 0. Change that?', he chose "Exit 1 on a blocker (Recommended)".
