---
id: 2026-11-19-fix-the-four-1086-review-findings-on-history-fragments
title: "Fix the four #1086 review findings on history fragments"
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T14:07:59Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-1086-review-fixes.md
  - .specsync/archive/changes/2026-11-19-fix-the-four-pr-1086-review-findings-on-history-fragments/
record: .chough/archive/sessions/2026-11-19-1086-review-fixes.md
actor:
  - agent: claude-code
    terminal: t4t
---
# Fix the four #1086 review findings on history fragments

Fix the four #1086 review findings on history fragments: quote and unescape change:, bind history entry types in the cache digest, refuse Change Log delta items, one boundary-aware row matcher; each with a red-first test (#1084 follow-up; Leif chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c)

Why: Authorization: Leif approved spec-sync #1084 on 2026-11-18 (written on the issue) and, on PR #1086's review findings, chose 'Merge, fix all 4 in a follow-up before release', relayed by hc t4c on 2026-11-18; scope is exactly the four Codex findings on #1086; limits: no release, tag or publish, no merge without his relayed yes

Verified: Codex's P2 at 0cdd7f41 (the tester's case): a Change Log row now splits on unescaped pipes only, with an escaped pipe part of its cell, so the id a|b with the legacy cell 'a\|b: Title' and history/a|b.md is a double record; the unescaped 'a|b: Title' is two…

Read more: [the story](../../.chough/archive/sessions/2026-11-19-1086-review-fixes.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-fix-the-four-pr-1086-review-findings-on-history-fragments/)
