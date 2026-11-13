---
change: the-persistent-read-cache-stays-bounded
artifact: context
---

# Context

The persistent read cache (`.git/specsync/read-cache-v1.json`, in the Git common directory) never
dropped anything. `archive_introductions` kept one entry per `<commit>\\n<pathspec>` ever walked,
with every introduced package's `approvals.json` text inline, so it grew with history and with
every branch or worktree that read the archive. Every write rewrote it whole and every read scope
parsed it whole (#850).

Leif approved the scope at 2026-11-13T03:49Z (coordinator chat, Claude Code hc t370, relayed by
claude-code-t370): asked about four features, he selected "#850 cache size cap" among them. The
design had no open choice.
