---
id: 2026-11-19-specsync-change-approve-id-dry-run-runs-the-approval-s-own
title: specsync change approve <id> --dry-run runs the approval's own checks
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T19:50:24Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-1091-approve-dry-run.md
  - .specsync/archive/changes/2026-11-19-change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing/
record: .chough/archive/sessions/2026-11-19-1091-approve-dry-run.md
actor:
  - agent: claude-code
    terminal: t59
---
# specsync change approve \<id\> --dry-run runs the approval's own checks

specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)

Why: 0xLeif decided "Intent approval of #1091 (CHANGE-15 and its three sub-criteria)": "OK, with (a) fixed" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition; relayed by hc t4c through ss-lead (hc t58).

Verified: At a768326c (the lock-create fix, REQ-change-163's lock-check wording under Leif's standing OK, the change re-checked), fledge lanes run verify passed 10/10 steps in 20m55s, run alone, including the…

Read more: [the story](../../.chough/archive/sessions/2026-11-19-1091-approve-dry-run.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing/)
