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
  - 2026-11-19-574-red-control-on-main-3bebe8b0
  - 2026-11-19-574-verify-lane-at-c2b19a91
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
    holds: accept
    drives: review
    opened: 2026-11-13T21:51:56Z
run_consents: []
next: Pushed on fix/validation-1071-1076-1050-1060-574, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Goal

check names each undocumented export's file from one lookup, so its time grows linearly (#574)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 with the other four; one conflict in tests/integration.rs, both test modules kept; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own scope approval ('#574 fast check', coordinator chat 2026-11-13T14:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 2ef24f40e (574 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including check_scaling::check_time_grows_linearly_with_undocumented_exports.](evidence/2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)
- [Red-first control: on main 3bebe8b0 with only the new tests, check_scaling::check_time_grows_linearly_with_undocumented_exports fails: four times the undocumented exports took 9.3 times as long (0.84s against 7.83s on a busy machine); linear would be about 4. This is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-19-574-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check_scaling::check_time_grows_linearly_with_undocumented_exports.](evidence/2026-11-19-574-verify-lane-at-c2b19a91.md)

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
