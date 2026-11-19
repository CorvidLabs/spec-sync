---
kind: session
id: 2026-11-13-921-change-correct-read-scope
aliases: [921-change-correct-read-scope]
objective: change correct reads its summary inside a change read scope, so it reuses memoized reads (#921)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: change-correct-reads-its-summary-inside-a-change-read-scope
pr: ""
criteria: [SPEED-1]
requirements: []
cases: []
design: []
files: [src/change/historical_definition_digest_matches.rs, src/change_tests/fast_archived_reads.rs]
observations:
  - 2026-11-13-at-60217e874-921-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-a
  - 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-change
  - 2026-11-19-refreshed-onto-main-3bebe8b0-with-942-955-and-978-the-shared
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of change-correct-reads-its-summary-inside-a-change-read-scope
    holds: accept
    drives: review
    opened: 2026-11-13T22:39:20Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# change correct reads its summary inside a change read scope, so it reuses memoized reads (#921)

## Goal

change correct reads its summary inside a change read scope, so it reuses memoized reads (#921)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#921 faster change correct', coordinator chat 2026-11-13T04:55Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- Refreshed onto main 3bebe8b0 (batch 7b) under Leif's relayed answer of 2026-11-18 that the 2026-11-13 approvals stand, 'Yes, they stand' (from his planning session hc t4c, relayed through ss-lead t58); no approved sentence changed, and no review, approval or gate close was recorded

## What was verified

- [At 60217e874 (#921 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::change_correct_costs_less_than_one_unscoped_summary.](evidence/2026-11-13-at-60217e874-921-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, a whole change correct spawned 696 Git processes against 641 for one summary read without a scope (279 with the fix). The failure is the defect #921 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-a.md)
- [Red-first control for the refresh onto main 3bebe8b0: change_correct_costs_less_than_one_unscoped_summary fails, 'change correct spawned 696 Git processes, more than the 641 of one summary read without a scope', the defect #921 reports.](evidence/2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-change.md)
- [Refreshed onto main 3bebe8b0 with #942, #955 and #978 (the shared test file's appended tests merged by hand, all kept); the red test passes. fledge lanes run verify at fde6e01b with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10/10 steps, 3081 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage; lifecycle_commands_stay_within_git_spawn_ceilings passes; change audit --strict and pre-push passed.](evidence/2026-11-19-refreshed-onto-main-3bebe8b0-with-942-955-and-978-the-shared.md)

## Open threads

## Timeline

- 2026-11-13T21:52:43Z · start · intent → intent · change correct reads its summary inside a change read scope, so it reuses memoized reads (#921) ^t-a890d0
  - revision: 72643f1963efe015f7db4a2ceec8763957d95aa7
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:45Z · advance · intent → design ·  ^t-7ce8e7
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - revision: 72643f1963efe015f7db4a2ceec8763957d95aa7
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:45Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-1d5c87
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - revision: 72643f1963efe015f7db4a2ceec8763957d95aa7
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:47Z · advance · plan → build · native:verifying ^t-27a5e6
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - gate: native:verifying
  - revision: 72643f1963efe015f7db4a2ceec8763957d95aa7
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:47Z · advance · build → test · native:verifying ^t-63473d
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - gate: native:verifying
  - revision: 72643f1963efe015f7db4a2ceec8763957d95aa7
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:20Z · advance · test → accept · native:verifying ^t-f4c17b
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - gate: native:verifying
  - revision: 60217e874c0ea566300e58dbb95e328ffd5bd487
  - by: claude-code (hc t38p)
- 2026-11-13T22:39:20Z · gate-opened · accept → accept · Implementation review of change-correct-reads-its-summary-inside-a-change-read-scope ^t-091f97
  - native_change: change-correct-reads-its-summary-inside-a-change-read-scope
  - revision: 60217e874c0ea566300e58dbb95e328ffd5bd487
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
