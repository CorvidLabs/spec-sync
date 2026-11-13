---
kind: session
id: 2026-11-13-982-retry-failed-history-index
aliases: [982-retry-failed-history-index]
objective: A failed lifecycle-state history index build is retried in the next read scope instead of staying failed (#982)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
pr: ""
criteria: [SPEED-1.c]
requirements: []
cases: []
design: []
files:
  - src/change/sdd_version.rs
  - src/change/validate_loaded_change.rs
  - src/change_tests/fast_archived_reads.rs
observations:
  - 2026-11-13-at-e4e0e229a-fix-982-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
    holds: accept
    drives: review
    opened: 2026-11-13T18:38:04Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# A failed lifecycle-state history index build is retried in the next read scope instead of staying failed (#982)

## Goal

A failed lifecycle-state history index build is retried in the next read scope instead of staying failed (#982)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#982 retry history index', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form; file list grew under the standing file-list rule (04:35Z)
- decided: no new lesson for change.3md, because the change is narrow and an existing lesson covers it

## What was verified

- [At e4e0e229a (fix/982 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope.](evidence/2026-11-13-at-e4e0e229a-fix-982-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, change::tests::a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope fails: a failed index build stayed failed for later read scopes. The failure is the defect #982 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T17:52:29Z · start · intent → intent · A failed lifecycle-state history index build is retried in the next read scope instead of staying failed (#982) ^t-6393f2
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:31Z · advance · intent → design ·  ^t-893389
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:31Z · skip · design → plan · Docs, CI and CLI behaviour only: no visuals or interactions ^t-088b58
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:33Z · advance · plan → build · native:verifying ^t-eb41e3
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - gate: native:verifying
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T17:52:33Z · advance · build → test · native:verifying ^t-172408
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - gate: native:verifying
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T18:38:04Z · advance · test → accept · native:verifying ^t-ec8ead
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - gate: native:verifying
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - by: claude-code (hc t38p)
- 2026-11-13T18:38:04Z · gate-opened · accept → accept · Implementation review of a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope ^t-a37f7c
  - native_change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
  - revision: e4e0e229a0e3d4398b6949bf50b1288a3493b1bc
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
