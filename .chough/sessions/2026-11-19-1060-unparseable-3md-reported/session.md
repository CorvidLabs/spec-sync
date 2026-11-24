---
kind: session
id: 2026-11-19-1060-unparseable-3md-reported
aliases: [1060-unparseable-3md-reported]
objective: check reports a module's .3md lesson file that does not parse (#1060)
intent_scope: THREE
modules: [validator]
stage: accept
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
  - 2026-11-19-1060-verify-lane-at-76a66032
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
  - 2026-11-24-the-change-read-module-3md-with-a-plain-read-written-before-6
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
- The lead's and the tester's review points on f4de84fc are in (f52d7f3c, 6ffa715e): #574's timing test is interleaved and resized so it is red on main in release and debug; #1050's and #1060's CHANGELOG entries say what an upgrade does and what is left out; re-checked; lane 10/10 at 76a66032

## Decisions and why

- decided: authorization is Leif's own decision ('#1060 broken .3md reported', coordinator chat 2026-11-13T03:28Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 3 refresh

## What was verified

- [Red-first control: on main 3bebe8b0 with only the new tests, check::an_unparseable_3md_is_a_check_warning_and_fails_strict fails: check prints no warning for a .3md with two planes at one z, and check --strict passes. This is the defect #1060 reports, fixed by this change; not a new bug.](evidence/2026-11-19-1060-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes check::an_unparseable_3md_is_a_check_warning_and_fails_strict (REQ-cmd-check-018).](evidence/2026-11-19-1060-verify-lane-at-c2b19a91.md)
- [At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)](evidence/2026-11-19-1060-verify-lane-at-76a66032.md)
- [At cb679a13 (#1071, #1076, #1060 and #574 merged onto main a275ab44, the 6.4.2 release candidate commit, each change re-checked there), fledge lanes run verify passed 10/10 steps in 34m20s: fmt, lint and check-types, cargo test (unit 3357 passed, 2 ignored; integration 825 passed), the release build, the forced strict spec check at 100 percent coverage, and the six CI contract suites. On real CLI paths with the release binary that lane built (sha256 5c4426e0), against the 6.4.2 release binary: a lesson file with two planes at one z is a warning naming the file, the parser's error and context.md, and check --strict exits 1; 6.4.2 reports nothing and exits 0. A parsing lesson file adds no finding on either.](evidence/2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44.md)
- [The change read <module>.3md with a plain read, written before 6.4.2 put every project read behind the bounded reader. Red control at 30283539 with only the new test added: a_3md_over_the_read_bound_is_reported_and_not_read failed (the file was read in full, nothing reported) and no_project_file_is_read_outside_the_bounded_read failed naming src/validator/consumed_by_re.rs:2288. With read_project_string at the spec-file bound (a0013747) both pass, with the 128 tests of the check, check_scaling, project_reads, project_links and project_writes groups. A lesson file that is a link is refused earlier by coverage discovery under either read, shown on the CLI for check, score and report.](evidence/2026-11-24-the-change-read-module-3md-with-a-plain-read-written-before-6.md)

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
- 2026-11-19T08:42:12Z · advance · build → test · native:verifying ^t-c7f02d
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
- 2026-11-19T08:42:16Z · advance · test → accept · native:verifying ^t-dd8e09
  - native_change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
  - gate: native:verifying
  - revision: 76a6603257a1efec6ba25d83c603617ce7434c46
  - by: claude-code (hc t59)
