---
id: 2026-11-19-ship-status-reports-an-archived-change-s-review-as-done-by
title: ship-status reports an archived change's review as done by finalize…
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T14:41:20Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-13-1038-ship-status-archived-review.md
  - .specsync/archive/changes/2026-11-19-ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only/
record: .chough/archive/sessions/2026-11-13-1038-ship-status-archived-review.md
actor:
  - agent: claude-code
    terminal: t59
---
# ship-status reports an archived change's review as done by finalize…

ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)

Why: authorization is Leif's own decision ('Bug fix (Recommended)', option text 'For an archived change, report the review as done by finalize and suggest nothing.', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form

Verified: At 9564c7a3 (after the send-back, the new criterion and Leif's relayed re-approval of #1038's change, with all three changes re-checked), fledge lanes run verify passed 10/10 steps in 11m60s, run alone: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests.

Read more: [the story](../../.chough/archive/sessions/2026-11-13-1038-ship-status-archived-review.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only/)
