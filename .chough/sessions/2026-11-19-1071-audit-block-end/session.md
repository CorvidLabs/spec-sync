---
kind: session
id: 2026-11-19-1071-audit-block-end
aliases: [1071-audit-block-end]
objective: Delta validation reads a living requirement's block as the materializer does, so an unchanged ADDED requirement followed by a section is not a conflict (#1071)
intent_scope: CHANGE
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
pr: ""
criteria: [CHANGE-5]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-1071-red-control-on-main-3bebe8b0
  - 2026-11-19-1071-verify-lane-at-c2b19a91
  - 2026-11-19-1071-verify-lane-at-76a66032
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
    holds: accept
    drives: review
    opened: 2026-11-19T07:14:31Z
run_consents: []
next: Pushed on fix/validation-1071-1076-1050-1060-574, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# Delta validation reads a living requirement's block as the materializer does, so an unchanged ADDED requirement followed by a section is not a conflict (#1071)

## Goal

Delta validation reads a living requirement's block as the materializer does, so an unchanged ADDED requirement followed by a section is not a conflict (#1071)

## Where it stands

- Merged the 11-13 branch onto main 3bebe8b0 on branch fix/validation-1071-1076-1050-1060-574
- Refreshed onto main 3bebe8b0 with #1076, #1050, #1060 and #574; no conflict; re-checked; red control on main and the verify lane recorded
- The lead's and the tester's review points on f4de84fc are in (f52d7f3c, 6ffa715e): #574's timing test is interleaved and resized so it is red on main in release and debug; #1050's and #1060's CHANGELOG entries say what an upgrade does and what is left out; re-checked; lane 10/10 at 76a66032

## Decisions and why

- decided: authorization is Leif's own decision ('Approve (Recommended)', coordinator chat 2026-11-13T03:41Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main
- decided: this change had no Chough session because its 11-13 branch was cut before the Chough 0.6.0 migration (#1073) reached main; ss-builder (hc t59) opened this one on 2026-11-19 for the batch 3 refresh

## What was verified

- [Red-first control: on main 3bebe8b0 with only the new tests, added_requirement_block_end::an_unchanged_added_requirement_followed_by_a_section_is_not_a_conflict fails: the unchanged ADDED requirement is refused as 'cannot add existing block'. The control a_changed_added_requirement_followed_by_a_section_is_still_a_conflict passes there, as it should. This is the defect #1071 reports, fixed by this change; not a new bug.](evidence/2026-11-19-1071-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes both added_requirement_block_end tests.](evidence/2026-11-19-1071-verify-lane-at-c2b19a91.md)
- [At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)](evidence/2026-11-19-1071-verify-lane-at-76a66032.md)
- 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44

## Open threads

## Timeline

- 2026-11-19T06:56:29Z · start · intent → intent · Delta validation reads a living requirement's block as the materializer does, so an unchanged ADDED requirement followed by a section is not a conflict (#1071) ^t-cbdd9f
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:56:36Z · advance · intent → design ·  ^t-81926d
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:56:37Z · skip · design → plan · No visuals or interactions: validation and CLI output only ^t-cd4f76
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T06:56:44Z · advance · plan → build · native:verifying ^t-f95c48
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - gate: native:verifying
  - revision: f85496782b3381f9d4887b305f7e7874bbd1367d
  - by: claude-code (hc t59)
- 2026-11-19T07:14:28Z · advance · build → test · native:verifying ^t-bb37b8
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - gate: native:verifying
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - by: claude-code (hc t59)
- 2026-11-19T07:14:31Z · advance · test → accept · native:verifying ^t-c44b83
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - gate: native:verifying
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - by: claude-code (hc t59)
- 2026-11-19T07:14:31Z · gate-opened · accept → accept · Implementation review of delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does ^t-7b1569
  - native_change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
  - revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
