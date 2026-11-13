---
kind: session
id: 2026-11-13-654-ship-status-exits-1-on-a-blocker
aliases: [654-ship-status-exits-1-on-a-blocker]
objective: change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)
intent_scope: ""
modules: [cmd_change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: change-ship-status-exits-1-when-it-reports-a-blocker
pr: ""
criteria: [SHIP-9.a]
requirements: []
cases: []
design: []
files:
  - src/commands/change.rs
  - tests/integration/change.rs
  - site/src/content/docs/cli.md
  - docs/ADOPTING.md
observations:
  - 2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker
    holds: accept
    drives: review
    opened: 2026-11-13T22:39:05Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Goal

change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own decision ('Exit 1 on a blocker (Recommended)', option text 'So scripts and CI can act on it.', coordinator chat 2026-11-13T14:12Z, relayed by claude-code-t370 and passed on by claude-code-t38c); REQ-cmd-change-039 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 81877688e (#654 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039; chough cannot resolve cmd_change requirement ids, so the id is named here).](evidence/2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T21:52:39Z · start · intent → intent · change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654) ^t-f6681b
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:41Z · advance · intent → design ·  ^t-cdcc2d
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:41Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-4cd0d8
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:43Z · advance · plan → build · native:verifying ^t-527d9f
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:43Z · advance · build → test · native:verifying ^t-d862de
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 5a117c21912b244a25de00cce004d5c805105498
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:05Z · advance · test → accept · native:verifying ^t-c8ebc6
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - gate: native:verifying
  - revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:05Z · gate-opened · accept → accept · Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker ^t-671531
  - native_change: change-ship-status-exits-1-when-it-reports-a-blocker
  - revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
