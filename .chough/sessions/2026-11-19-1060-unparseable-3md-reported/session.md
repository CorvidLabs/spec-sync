---
kind: session
id: 2026-11-19-1060-unparseable-3md-reported
aliases: [1060-unparseable-3md-reported]
objective: check reports a module's .3md lesson file that does not parse (#1060)
intent_scope: THREE
modules: [validator]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
pr: ""
criteria: [THREE-1]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-1060-red-control-on-main-3bebe8b0
  - 2026-11-19-1060-verify-lane-at-c2b19a91
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of check-reports-a-module-s-3md-lesson-file-that-does-not-parse
    holds: accept
    drives: review
    opened: 2026-11-19T07:14:39Z
run_consents: []
next: Pushed on fix/validation-1071-1076-1050-1060-574, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# check reports a module's .3md lesson file that does not parse (#1060)

## Goal

check reports a module's .3md lesson file that does not parse (#1060)

## Where it stands

- Merged the 11-13 branch onto main 3bebe8b0 on branch fix/validation-1071-1076-1050-1060-574
- Refreshed onto main 3bebe8b0 with the other four; the cmd_check and validator spec files kept as main has them and re-materialized the #1086 way: REQ-cmd-check-018 (text unchanged), the validator's Error Cases row and two history fragments, no version bump or Change Log row; the validator conflict kept main's history-fragment check and this change's .3md warning; re-checked; red control and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own decision ('#1060 broken .3md reported', coordinator chat 2026-11-13T03:28Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 3 refresh

## What was verified

- [Red-first control: on main 3bebe8b0 with only the new tests, check::an_unparseable_3md_is_a_check_warning_and_fails_strict fails: check prints no warning for a .3md with two planes at one z, and check --strict passes. This is the defect #1060 reports, fixed by this change; not a new bug.](evidence/2026-11-19-1060-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check::an_unparseable_3md_is_a_check_warning_and_fails_strict (REQ-cmd-check-018).](evidence/2026-11-19-1060-verify-lane-at-c2b19a91.md)

## Open threads

## Timeline

- 2026-11-19T06:56:58Z · start · intent → intent · check reports a module's .3md lesson file that does not parse (#1060) ^t-1fc3b7
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:57:05Z · advance · intent → design ·  ^t-515bd7
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:57:05Z · skip · design → plan · No visuals or interactions: validation and CLI output only ^t-658a0c
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:57:12Z · advance · plan → build · native:verifying ^t-afdac6
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T07:14:38Z · advance · build → test · native:verifying ^t-fb810b
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - by: claude-code (hc t59)
- 2026-11-19T07:14:39Z · advance · test → accept · native:verifying ^t-a985e4
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - by: claude-code (hc t59)
- 2026-11-19T07:14:39Z · gate-opened · accept → accept · Implementation review of check-reports-a-module-s-3md-lesson-file-that-does-not-parse ^t-e4d1da
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-19T07:40:28Z · back · accept → build · tester's design note on the batch 3 branch: the CHANGELOG entry says plainly that the briefing does not name the skipped file ^t-9ce832
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: 0d0840fd24cc099b68f4d5c9b4b009640429109b
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
