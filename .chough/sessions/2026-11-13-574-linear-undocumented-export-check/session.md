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
  - 2026-11-19-574-verify-lane-at-76a66032
  - 2026-11-19-574-red-on-main-release
  - 2026-11-19-574-red-on-main-debug
  - 2026-11-19-574-green-at-the-tip-release
  - 2026-11-19-574-green-at-the-tip-debug
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
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
- The lead's and the tester's review points on f4de84fc are in (f52d7f3c, 6ffa715e): #574's timing test is interleaved and resized so it is red on main in release and debug; #1050's and #1060's CHANGELOG entries say what an upgrade does and what is left out; re-checked; lane 10/10 at 76a66032

## Decisions and why

- decided: authorization is Leif's own scope approval ('#574 fast check', coordinator chat 2026-11-13T14:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 2ef24f40e (574 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including check_scaling::check_time_grows_linearly_with_undocumented_exports.](evidence/2026-11-13-at-2ef24f40e-574-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)
- [Red-first control: on main 3bebe8b0 with only the new tests, check_scaling::check_time_grows_linearly_with_undocumented_exports fails: four times the undocumented exports took 9.3 times as long (0.84s against 7.83s on a busy machine); linear would be about 4. This is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-19-574-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check_scaling::check_time_grows_linearly_with_undocumented_exports.](evidence/2026-11-19-574-verify-lane-at-c2b19a91.md)
- [At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)](evidence/2026-11-19-574-verify-lane-at-76a66032.md)
- [Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, release build: 14.3 times as long (0.56s, 7.95s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. The first sizes, 8,000 against 32,000, were red in debug only (the tester measured 5.2 times in release).](evidence/2026-11-19-574-red-on-main-release.md)
- [Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, debug build: 14.9 times as long (7.06s, 104.9s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug.](evidence/2026-11-19-574-red-on-main-debug.md)
- [The resized timing test at the batch 3 tip, idle machine, release build: 3.0 times as long (0.12s, 0.35s), under the bound of 7.](evidence/2026-11-19-574-green-at-the-tip-release.md)
- [The resized timing test at the batch 3 tip, idle machine, debug build: 3.8 times as long (0.65s, 2.45s), under the bound of 7.](evidence/2026-11-19-574-green-at-the-tip-debug.md)
- 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44

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
- 2026-11-19T07:23:20Z · back · accept → build · ss-lead's review of the batch 3 branch at f4de84fc: make the timing test harder for machine load to flip (interleaved runs, fastest of three), same 4x sizes and same bound ^t-539258
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: f4de84fc72a02214003261bb9e01648dbd9916c6
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 4 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T08:41:59Z · advance · build → test · native:verifying ^t-26ae16
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
- 2026-11-19T08:42:02Z · advance · test → accept · native:verifying ^t-2eca65
  - native_change: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
