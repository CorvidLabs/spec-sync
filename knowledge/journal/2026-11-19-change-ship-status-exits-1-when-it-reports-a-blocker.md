---
id: 2026-11-19-change-ship-status-exits-1-when-it-reports-a-blocker
title: change ship-status exits 1 when it reports a blocker
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T14:38:05Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-13-654-ship-status-exits-1-on-a-blocker.md
  - .specsync/archive/changes/2026-11-19-change-ship-status-exits-1-when-it-reports-a-blocker/
record: .chough/archive/sessions/2026-11-13-654-ship-status-exits-1-on-a-blocker.md
actor:
  - agent: claude-code
    terminal: t59
---
# change ship-status exits 1 when it reports a blocker

change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

Why: authorization is Leif's own decision ('Exit 1 on a blocker (Recommended)', option text 'So scripts and CI can act on it.', coordinator chat 2026-11-13T14:12Z, relayed by claude-code-t370 and passed on by claude-code-t38c); REQ-cmd-change-039 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form

Verified: At 9564c7a3 (after the send-back, the new criterion and Leif's relayed re-approval of #1038's change, with all three changes re-checked), fledge lanes run verify passed 10/10 steps in 11m60s, run alone: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092)

Read more: [the story](../../.chough/archive/sessions/2026-11-13-654-ship-status-exits-1-on-a-blocker.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-change-ship-status-exits-1-when-it-reports-a-blocker/)
