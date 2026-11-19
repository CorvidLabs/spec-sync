---
kind: session
id: 2026-11-13-978-cutoff-verdicts-before-prefetch
aliases: [978-cutoff-verdicts-before-prefetch]
objective: Prefetch workers inherit the cutoff verdicts the command thread checked and persisted (#978)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
pr: ""
criteria: [SPEED-1.c]
requirements: []
cases: []
design: []
files: [src/change/read_cache.rs, src/change_tests/fast_archived_reads.rs]
observations:
  - 2026-11-13-at-db5f6c2e3-978-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
    holds: accept
    drives: review
    opened: 2026-11-13T21:51:42Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Prefetch workers inherit the cutoff verdicts the command thread checked and persisted (#978)

## Goal

Prefetch workers inherit the cutoff verdicts the command thread checked and persisted (#978)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#978 keep verdicts cached', coordinator chat 2026-11-13T14:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At db5f6c2e3 (978 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::prefetch_workers_leave_every_cutoff_verdict_in_the_read_cache.](evidence/2026-11-13-at-db5f6c2e3-978-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, change::tests::prefetch_workers_leave_every_cutoff_verdict_in_the_read_cache fails. Workers re-read the cutoff tree and kept nothing in the persistent read cache. The failure is the defect #978 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T20:56:17Z · start · intent → intent · Prefetch workers inherit the cutoff verdicts the command thread checked and persisted (#978) ^t-e2e1f4
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:21Z · advance · intent → design ·  ^t-71cb44
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:21Z · skip · design → plan · Read-path performance only: no visuals or interactions ^t-41425a
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:25Z · advance · plan → build · native:verifying ^t-1f532d
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - gate: native:verifying
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:25Z · advance · build → test · native:verifying ^t-876b55
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - gate: native:verifying
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:42Z · advance · test → accept · native:verifying ^t-81a362
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - gate: native:verifying
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:42Z · gate-opened · accept → accept · Implementation review of prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted ^t-7f679d
  - native_change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
  - revision: db5f6c2e39c427478737c36f68814001d60126b6
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
