---
kind: session
id: 2026-11-13-965-archive-job-sees-default-branch
aliases: [965-archive-job-sees-default-branch]
objective: The Archive CI job fetches full history, so it sees the default branch and never tells a stale copy of an upstream archive to ship (#965)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
pr: ""
criteria: [CI-2]
requirements: []
cases: []
design: []
files:
  - .github/workflows/ci.yml
  - .github/workflows/trust.yml
  - .github/scripts/test-required-ci-gate.py
observations:
  - 2026-11-13-at-1085663ce-fix-965-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
    holds: accept
    drives: review
    opened: 2026-11-13T17:51:45Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The Archive CI job fetches full history, so it sees the default branch and never tells a stale copy of an upstream archive to ship (#965)

## Goal

The Archive CI job fetches full history, so it sees the default branch and never tells a stale copy of an upstream archive to ship (#965)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#965 CI archive check', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers

## What was verified

- [At 1085663ce (fix/965 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including ArchiveGateHistoryTests.test_every_archive_gate_job_checks_out_full_history in .github/scripts/test-required-ci-gate.py.](evidence/2026-11-13-at-1085663ce-fix-965-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, ArchiveGateHistoryTests.test_every_archive_gate_job_checks_out_full_history in .github/scripts/test-required-ci-gate.py fails. The failure is the defect #965 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T16:59:37Z · start · intent → intent · The Archive CI job fetches full history, so it sees the default branch and never tells a stale copy of an upstream archive to ship (#965) ^t-1b90f2
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:41Z · advance · intent → design ·  ^t-6628d0
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:41Z · skip · design → plan · CLI or CI behaviour only: no visuals or interactions ^t-5c49bd
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:45Z · advance · plan → build · native:verifying ^t-73c4b1
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - gate: native:verifying
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:45Z · advance · build → test · native:verifying ^t-4f4c46
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - gate: native:verifying
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:45Z · advance · test → accept · native:verifying ^t-35add2
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - gate: native:verifying
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:45Z · gate-opened · accept → accept · Implementation review of the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship ^t-4df9a3
  - native_change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
  - revision: 1085663ced25f270aaf7f88ababe2f28df59424e
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
