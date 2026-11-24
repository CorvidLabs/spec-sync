---
id: 2026-11-24-6-4-2-120-minutes-for-the-macos-candidate-lane
title: "6.4.2: 120 minutes for the macOS candidate lane"
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-24
time: 2026-11-24T18:47:27Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-24-1161-rc-qualify-120-minutes.md
  - .specsync/archive/changes/2026-11-24-give-the-macos-release-candidate-qualification-120-minutes/
record: .chough/archive/sessions/2026-11-24-1161-rc-qualify-120-minutes.md
actor:
  - agent: claude-code
    terminal: t14v
---
# 6.4.2: 120 minutes for the macOS candidate lane

v6.4.2-rc.1 passed all 4,176 tests on macOS and was cancelled at the 75-minute qualify limit, so the limit is 120 for v6.4.2-rc.2. One value in release.yml and a handoff note; no product, test or spec file changes.

The validators and all six contract suites pass, the parsed workflow is otherwise identical to main, and scoped check, pre-push and Trust pass.

Next: bring every test lane under 30 minutes (#1161).

Read more: [the story](../../.chough/archive/sessions/2026-11-24-1161-rc-qualify-120-minutes.md) · [the archived change](../../.specsync/archive/changes/2026-11-24-give-the-macos-release-candidate-qualification-120-minutes/)
