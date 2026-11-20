---
id: 2026-11-20-every-walk-of-a-configured-folder-goes-through-the-confined
title: Every walk of a configured folder goes through the confined walk
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T14:22:48Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-folder-walks-and-read-cache.md
  - .specsync/archive/changes/2026-11-20-walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder/
record: .chough/archive/sessions/2026-11-20-folder-walks-and-read-cache.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Every walk of a configured folder goes through the confined walk

Every walk of a configured folder goes through the confined walk, so a committed specs, source or schema link that leaves the project is refused, not walked; the change read cache lives in the user's cache folder, never in a Git directory an archive can ship (items 2 and 4 of Leif's 04:09Z round, private, with GHSA-7q5v-8jcc-rx54)

Why: 0xLeif decided "Scope approval of walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder": "continue with 95% confidence" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 04:09Z, answering t4c's five-item list with…

Verified: On Linux at 700ebb00: clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3135 passed, 0 failed, 2 ignored and integration 706 passed, 0 failed.

Read more: [the story](../../.chough/archive/sessions/2026-11-20-folder-walks-and-read-cache.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder/)
