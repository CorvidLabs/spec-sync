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
  - CHANGELOG.md
observations:
  - 2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
  - 2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests
  - 2026-11-19-654-verify-lane-at-4d439b39
  - 2026-11-19-654-verify-lane-at-612d255d
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of change-ship-status-exits-1-when-it-reports-a-blocker
    holds: accept
    drives: review
    opened: 2026-11-13T22:39:05Z
run_consents: []
next: Wait for Leif's review of the batch PR, relayed by hc t4c through ss-lead; then chough change ship with his words. Record no review before that.
---
# change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Goal

change ship-status exits 1 when it reports a blocker, so scripts and CI can act on it (#654)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 on branch fix/ship-status-654-1038-937 with #1038 and #937: merged the 11-13 branch, kept main's cmd_change spec files, and let change check apply REQ-cmd-change-039 and write specs/cmd_change/history/change-ship-status-exits-1-when-it-reports-a-blocker.md instead of the old version bump and Change Log row; added the CHANGELOG line; red control on main and the verify lane recorded
- PR #1093 opened; re-checked at 612d255d after the review fixes for #1038, with the verify lane recorded there

## Decisions and why

- decided: authorization is Leif's own decision ('Exit 1 on a blocker (Recommended)', option text 'So scripts and CI can act on it.', coordinator chat 2026-11-13T14:12Z, relayed by claude-code-t370 and passed on by claude-code-t38c); REQ-cmd-change-039 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- decided: the approved REQ-cmd-change-039 text is unchanged and the 11-13 approval stands because Leif said 'Yes, they stand' (relayed by his planning session hc t4c through ss-lead hc t58, 2026-11-18)
- decided: one review answer from Leif on the batch PR counts as the scoped review of each change in it because he said 'Yes, one answer per batch' (relayed by hc t4c through ss-lead hc t58, 2026-11-18); nothing is recorded until it comes

## What was verified

- [At 81877688e (#654 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039; chough cannot resolve cmd_change requirement ids, so the id is named here).](evidence/2026-11-13-at-81877688e-654-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)
- 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- [Red-first control: on main 3bebe8b0 with only the new tests, change::ship_status_exits_one_when_it_reports_a_blocker fails (ship-status printed its Blocker line and exited 0), and the three existing tests that now expect exit 1 from a blocked ship-status fail the same way. The failure is the defect #654 reports, fixed by this change; not a new bug.](evidence/2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests.md)
- [At 4d439b39 (#654, #1038 and #937 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m07s: fmt, lint and check-types, cargo test (unit 3079 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests, including change::ship_status_exits_one_when_it_reports_a_blocker and the three tests that now expect exit 1 from a blocked ship-status (REQ-cmd-change-039). The lane's processes ran with a neutral global git config: with the machine's own config, which signs tags, four changelog unit tests wait on an editor for a tag message.](evidence/2026-11-19-654-verify-lane-at-4d439b39.md)
- [At 612d255d (PR #1093 after the review fixes for #1038 in c697bbde, each change re-checked there), fledge lanes run verify passed 10/10 steps in 29m34s on a busy machine: fmt, lint and check-types, cargo test (unit 3081 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::ship_status_exits_one_when_it_reports_a_blocker (REQ-cmd-change-039); this change's source is unchanged since 4d439b39.](evidence/2026-11-19-654-verify-lane-at-612d255d.md)

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
