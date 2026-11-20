---
id: 2026-11-20-writes-reads-and-caches-a-repository-can-steer-stay-inside-its
title: Writes, reads and caches a repository can steer stay inside its Git…
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T06:14:47Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-repository-steered-paths.md
  - .specsync/archive/changes/2026-11-20-keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree/
record: .chough/archive/sessions/2026-11-19-repository-steered-paths.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Writes, reads and caches a repository can steer stay inside its Git…

Writes, reads and caches a repository can steer stay inside its Git work tree, never reach the Git directory, and never hide a failure (F2-F9, R1, R2, G1 and point 6 of the independent review, private, with GHSA-7q5v-8jcc-rx54)

Why: 0xLeif decided "Intent approval of proposed criterion CHECK-1.f": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 16:00Z, after t4c summed up security/change5-scope.md: SETUP-6 and SETUP-7 cover points 1, 2, 3 and 5; the new intent line CHECK-1.f 'A result that a repository somebody else wrote carries along never hides a failure my check would report.'; choices 1-8 as recommended…

Verified: On Linux at e60fbf8e (change 5), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast…

Read more: [the story](../../.chough/archive/sessions/2026-11-19-repository-steered-paths.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree/)
