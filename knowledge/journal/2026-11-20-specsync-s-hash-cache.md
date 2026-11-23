---
id: 2026-11-20-specsync-s-hash-cache
title: SpecSync's hash cache
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T06:21:15Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-state-outside-the-work-tree.md
  - .specsync/archive/changes/2026-11-20-keep-specsync-s-own-caches-and-journal-outside-the-work-tree/
record: .chough/archive/sessions/2026-11-20-state-outside-the-work-tree.md
actor:
  - agent: claude-code
    terminal: t5r
---
# SpecSync's hash cache

SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)

Why: 0xLeif decided "Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree": "a, all yes..

Verified: On Linux at 0187d498: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3126 passed, 0 failed, 2 ignored and integration 700 passed, 0 failed.

Read more: [the story](../../.chough/archive/sessions/2026-11-20-state-outside-the-work-tree.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-keep-specsync-s-own-caches-and-journal-outside-the-work-tree/)
