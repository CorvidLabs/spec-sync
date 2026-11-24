---
kind: session
id: 2026-11-13-1076-shall-at-a-line-edge
aliases: [1076-shall-at-a-line-edge]
objective: A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
pr: ""
criteria: [SPEC-1]
requirements: []
cases: []
design: []
files: [src/change/collect_requirement_ids.rs, src/change_tests/approved_delta_requirements.rs]
observations:
  - 2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a
  - 2026-11-19-1076-red-control-on-main-3bebe8b0
  - 2026-11-19-1076-verify-lane-at-c2b19a91
  - 2026-11-19-1076-verify-lane-at-76a66032
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
    holds: accept
    drives: review
    opened: 2026-11-13T22:38:50Z
run_consents: []
next: Pushed on fix/validation-1071-1076-1050-1060-574, no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record it with chough change ship and his words. Record no review before that.
---
# A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)

## Goal

A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 with #1071, #1050, #1060 and #574; no conflict; re-checked; red control on main and the verify lane recorded
- The lead's and the tester's review points on f4de84fc are in (f52d7f3c, 6ffa715e): #574's timing test is interleaved and resized so it is red on main in release and debug; #1050's and #1060's CHANGELOG entries say what an upgrade does and what is left out; re-checked; lane 10/10 at 76a66032

## Decisions and why

- decided: authorization is Leif's own scope approval ('Approve (Recommended)' for #1076, coordinator chat 2026-11-13T14:15Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At ad36b0e10 (#1076 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_shall_at_a_line_edge_is_a_normative_statement.](evidence/2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: at the red commit affacaa99, change::tests::a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement wrapped with SHALL at the end of a line, with its Acceptance Criteria, is refused as having no normative SHALL. The failure is the defect #1076 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a.md)
- [Red-first control: on main 3bebe8b0 with only the new tests, a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement whose SHALL ends a line is not read as a normative statement. This is the defect #1076 reports, fixed by this change; not a new bug.](evidence/2026-11-19-1076-red-control-on-main-3bebe8b0.md)
- [At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes a_shall_at_a_line_edge_is_a_normative_statement.](evidence/2026-11-19-1076-verify-lane-at-c2b19a91.md)
- [At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)](evidence/2026-11-19-1076-verify-lane-at-76a66032.md)
- 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44

## Open threads

## Timeline

- 2026-11-13T21:52:35Z · start · intent → intent · A requirement whose SHALL sits at a line edge counts as a normative statement (#1076) ^t-fb4542
  - revision: 890b0cf36e2f3af883cc977cdb2317795b621f4c
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:37Z · advance · intent → design ·  ^t-d2696e
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - revision: 890b0cf36e2f3af883cc977cdb2317795b621f4c
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:37Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-dd6733
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - revision: 890b0cf36e2f3af883cc977cdb2317795b621f4c
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:39Z · advance · plan → build · native:verifying ^t-c0b7f4
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - gate: native:verifying
  - revision: 890b0cf36e2f3af883cc977cdb2317795b621f4c
  - by: claude-code (hc t38p)
- 2026-11-13T21:52:39Z · advance · build → test · native:verifying ^t-c84e5f
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - gate: native:verifying
  - revision: 890b0cf36e2f3af883cc977cdb2317795b621f4c
  - by: claude-code (hc t38p)
- 2026-11-13T22:38:50Z · advance · test → accept · native:verifying ^t-7b4727
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - gate: native:verifying
  - revision: ad36b0e102bd1605ed930e0e07b8eb576623934c
  - by: claude-code (hc t38p)
- 2026-11-13T22:38:50Z · gate-opened · accept → accept · Implementation review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement ^t-b96e52
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - revision: ad36b0e102bd1605ed930e0e07b8eb576623934c
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
