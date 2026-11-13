---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: context
---

# Context

#476: the pinned Action downloads both the release archive and its `.sha256` sidecar from the same
mutable GitHub Release, so a release that replaced both would pass. Leif chose to build it on
2026-11-13 ("#476 Action checksum", coordinator chat 03:49Z) and chose one checksum per platform,
one per line ("One per platform (Recommended)", 04:04Z), both relayed by claude-code-t370. The
coordinator allocated REQ-github-031.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
