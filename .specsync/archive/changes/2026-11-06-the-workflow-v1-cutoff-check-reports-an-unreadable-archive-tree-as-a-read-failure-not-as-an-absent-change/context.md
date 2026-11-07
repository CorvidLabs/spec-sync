---
change: the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change
artifact: context
---

# Context

Issue #939, a Codex P2 on #918 that Leif deferred and then scheduled as its own fix. #918's cutoff-tree check (REQ-change-136) lists `<cutoff>:<archive root>` with `ls-tree` and reads the listing only when that call succeeds. Any failure, including a tree this clone doesn't hold (an offline `--filter=tree:0` partial clone), leaves only the active path as a candidate. The archived workflow-v1 change is then refused as "not present at the trusted pre-v2 cutoff". It fails closed (a refusal, never a false pass), but it names the wrong cause: the history walk it replaced surfaced the read failure.

Leif captured SPEED-1.a.3 for it on 2026-11-06 ("Capture both", relayed by orc t18): "When SpecSync cannot read the history it needs, it says it could not read it, rather than saying the change was not there." The orc assigned REQ-change-145.
