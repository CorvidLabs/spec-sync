---
id: 2026-11-20-specsync-keeps-its-cache
title: SpecSync keeps its cache
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T14:16:13Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-user-cache-only-state.md
  - .specsync/archive/changes/2026-11-20-keep-specsync-s-own-state-only-in-the-user-cache-folder/
record: .chough/archive/sessions/2026-11-20-user-cache-only-state.md
actor:
  - agent: claude-code
    terminal: t5r
---
# SpecSync keeps its cache

SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)

Why: 0xLeif decided "Scope approval of keep-specsync-s-own-state-only-in-the-user-cache-folder": "A" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 03:27Z, answering t4c's question after sec-review2's re-check of change 6 (security/review2/recheck-change6.md): 'A (recommended): one more small change, about…

Verified: On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed.

Read more: [the story](../../.chough/archive/sessions/2026-11-20-user-cache-only-state.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-keep-specsync-s-own-state-only-in-the-user-cache-folder/)
