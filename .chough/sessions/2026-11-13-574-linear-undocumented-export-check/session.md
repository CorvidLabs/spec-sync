---
kind: session
id: 2026-11-13-574-linear-undocumented-export-check
aliases: [574-linear-undocumented-export-check]
objective: check names each undocumented export's file from one lookup, so its time grows linearly (#574)
intent_scope: ""
modules: [validator]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
pr: ""
criteria: [SPEED-1.b]
requirements: []
cases: []
design: []
files: [src/validator/consumed_by_re.rs, tests/integration/check_scaling.rs]
observations:
  - 2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
    holds: accept
    drives: review
    opened: 2026-11-13T21:51:56Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Goal

check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#574 fast check', coordinator chat 2026-11-13T14:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 2ef24f40e (574 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including check_scaling::check_time_grows_linearly_with_undocumented_exports.](evidence/2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

## Open threads

## Timeline

- 2026-11-13T20:56:25Z · start · intent → intent · check names each undocumented export's file from one lookup, so its time grows linearly (#574) ^t-b1c6d3
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:29Z · advance · intent → design ·  ^t-004196
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:29Z · skip · design → plan · Read-path performance only: no visuals or interactions ^t-3f8943
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:33Z · advance · plan → build · native:verifying ^t-d67c28
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T20:56:33Z · advance · build → test · native:verifying ^t-2f65c1
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:56Z · advance · test → accept · native:verifying ^t-2de051
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - by: claude-code (hc t38p)
- 2026-11-13T21:51:56Z · gate-opened · accept → accept · Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly ^t-0855bf
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
