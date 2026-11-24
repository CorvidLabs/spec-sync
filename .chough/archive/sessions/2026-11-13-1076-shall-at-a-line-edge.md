---
kind: session
id: 2026-11-13-1076-shall-at-a-line-edge
aliases: [1076-shall-at-a-line-edge]
objective: A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)
modules: [change]
stage: done
stage_status: archived
iteration: 1
native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
criteria: [SPEC-1]
files: [src/change/collect_requirement_ids.rs, src/change_tests/approved_delta_requirements.rs]
observations:
  - 2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge
  - 2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a
  - 2026-11-19-1076-red-control-on-main-3bebe8b0
  - 2026-11-19-1076-verify-lane-at-c2b19a91
  - 2026-11-19-1076-verify-lane-at-76a66032
  - 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
closed: 2026-11-24
outcome: done
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
- 0xLeif decided "Implementation review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement": "I accept all four" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request headed 'Decision for you: do you accept each implementation?', which listed the four changes of the validation batch (#1071, #1076, #574 and #1060) with their native Verify lane at cb679a13, Trust, scoped checks and CLI dogfood against the 6.4.2 release binary. This change is the #1076 one of those four.); recorded by claude-code-t14v; a relayed claim, not authenticated; the scoped review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement was already current, so no review ran

## What was verified

- At ad36b0e10 (#1076 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_shall_at_a_line_edge_is_a_normative_statement. - 948e8da0cf94 ^2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge
- Red-first control: at the red commit affacaa99, change::tests::a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement wrapped with SHALL at the end of a line, with its Acceptance Criteria, is refused as having no normative SHALL. The failure is the defect #1076 reports, fixed by this change; not a new bug. - a4524e43fad9 ^2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a
- Red-first control: on main 3bebe8b0 with only the new tests, a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement whose SHALL ends a line is not read as a normative statement. This is the defect #1076 reports, fixed by this change; not a new bug. - cc0c01be6e2b ^2026-11-19-1076-red-control-on-main-3bebe8b0
- At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes a_shall_at_a_line_edge_is_a_normative_statement. - 7080ba2d1534 ^2026-11-19-1076-verify-lane-at-c2b19a91
- At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092) - 244325edaca8 ^2026-11-19-1076-verify-lane-at-76a66032
- At cb679a13 (#1071, #1076, #1060 and #574 merged onto main a275ab44, the 6.4.2 release candidate commit, each change re-checked there), fledge lanes run verify passed 10/10 steps in 34m20s: fmt, lint and check-types, cargo test (unit 3357 passed, 2 ignored; integration 825 passed), the release build, the forced strict spec check at 100 percent coverage, and the six CI contract suites. On real CLI paths with the release binary that lane built (sha256 5c4426e0), against the 6.4.2 release binary: change approve --dry-run accepts a requirement whose SHALL ends a line and one whose SHALL starts a line, where 6.4.2 refuses both with 'must contain a normative SHALL statement'; a requirement with no SHALL is still refused by both. - 4f138e01c8d7 ^2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44

## Bugs

None.

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
- 2026-11-24T20:57:50Z · direct-native-write · accept → accept · a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement's lifecycle records changed outside Chough: review.json added, review-attempts.json added; state stays verifying ^t-fd3c37
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - revision: 625ebffc454499a3c5260abdfb50ada38a23e486
  - direct_write:
    - change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
    - files:
      - review.json: added
      - review-attempts.json: added
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t14v)
- 2026-11-24T20:57:50Z · gate-closed · accept → accept · Implementation review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement ^t-237ade
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - revision: 625ebffc454499a3c5260abdfb50ada38a23e486
  - relayed_by:
    - decided_by: 0xLeif
    - words: I accept all four
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a request headed 'Decision for you: do you accept each implementation?', which listed the four changes of the validation batch (#1071, #1076, #574 and #1060) with their native Verify lane at cb679a13, Trust, scoped checks and CLI dogfood against the 6.4.2 release binary. This change is the #1076 one of those four.
    - recorded_by: claude-code-t14v
  - by: claude-code (hc t14v)
- 2026-11-24T20:57:52Z · advance · accept → done · native:archived ^t-1bad2a
  - native_change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
  - gate: native:archived
  - revision: 625ebffc454499a3c5260abdfb50ada38a23e486
  - by: claude-code (hc t14v)

Closed 2026-11-24: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge
    claim: "At ad36b0e10 (#1076 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_shall_at_a_line_edge_is_a_normative_statement."
    result: passed
    revision: ad36b0e102bd1605ed930e0e07b8eb576623934c
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: f0d92baff6800b434e1db8557c7d25d2a22ad397a9e192ec47f43fe3c4a3057e
    fingerprint: sha256:948e8da0cf94631a7fc3b4e0eaa045cd8cb0a6bad7ee4b8454d2d9d7467c919c
    criteria: SPEC-1
    observed: 2026-11-13T22:38:27Z
    actor: claude-code (hc t38p)
  - id: 2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a
    claim: "Red-first control: at the red commit affacaa99, change::tests::a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement wrapped with SHALL at the end of a line, with its Acceptance Criteria, is refused as having no normative SHALL. The failure is the defect #1076 reports, fixed by this change; not a new bug."
    result: failed
    revision: ad36b0e102bd1605ed930e0e07b8eb576623934c
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: f0d92baff6800b434e1db8557c7d25d2a22ad397a9e192ec47f43fe3c4a3057e
    fingerprint: sha256:a4524e43fad9181c5167593bda221038d822e2e3dd1e1d30b30884b04eaf6e0d
    criteria: SPEC-1
    observed: 2026-11-13T22:38:27Z
    actor: claude-code (hc t38p)
  - id: 2026-11-19-1076-red-control-on-main-3bebe8b0
    claim: "Red-first control: on main 3bebe8b0 with only the new tests, a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement whose SHALL ends a line is not read as a normative statement. This is the defect #1076 reports, fixed by this change; not a new bug."
    result: failed
    revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
    fingerprint: sha256:cc0c01be6e2b1a79519259666ff6c3a25aa4df90cfd6da474bc8bd28c7663b46
    criteria: SPEC-1
    observed: 2026-11-19T07:14:06Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-1076-verify-lane-at-c2b19a91
    claim: "At c2b19a91 (#1071, #1076, #1050, #1060 and #574 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m28s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes a_shall_at_a_line_edge_is_a_normative_statement."
    result: passed
    revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
    fingerprint: sha256:7080ba2d15341fbedda3803c248adfb3483329540dcdd1b6dedc63bf9649e810
    criteria: SPEC-1
    observed: 2026-11-19T07:14:07Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-1076-verify-lane-at-76a66032
    claim: "At 76a66032 (the two small commits f52d7f3c and 6ffa715e from the lead's and the tester's reviews, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m09s: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. A run of the same head beside another lane failed once on archived_upstream::the_printed_cannot_tell_status_recovery_keeps_the_selected_project_root, a test from main that races Git's background packing (#1097); this run was alone. The lane's processes ran with a neutral global git config (#1092)"
    result: passed
    revision: 76a6603257a1efec6ba25d83c603617ce7434c46
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
    fingerprint: sha256:244325edaca85ce1dc0b4afe3c4feda6ea8378863e0a1c5dbf8ef4de18476554
    criteria: SPEC-1
    observed: 2026-11-19T08:41:50Z
    actor: claude-code (hc t59)
  - id: 2026-11-24-at-cb679a13-1071-1076-1060-and-574-merged-onto-main-a275ab44
    claim: "At cb679a13 (#1071, #1076, #1060 and #574 merged onto main a275ab44, the 6.4.2 release candidate commit, each change re-checked there), fledge lanes run verify passed 10/10 steps in 34m20s: fmt, lint and check-types, cargo test (unit 3357 passed, 2 ignored; integration 825 passed), the release build, the forced strict spec check at 100 percent coverage, and the six CI contract suites. On real CLI paths with the release binary that lane built (sha256 5c4426e0), against the 6.4.2 release binary: change approve --dry-run accepts a requirement whose SHALL ends a line and one whose SHALL starts a line, where 6.4.2 refuses both with 'must contain a normative SHALL statement'; a requirement with no SHALL is still refused by both."
    result: passed
    revision: cb679a13f1433afbfc0989666a234d05ae822f66
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
    content: a309099a545c98c834a7d5a3bf4a0e3c684e607bb7325f90d97bed3d80c78d19
    fingerprint: sha256:4f138e01c8d72be3d4e938e9ffc65bee1cabd32f39df9b6c22c17b2470e05870
    criteria: SPEC-1
    observed: 2026-11-24T20:49:24Z
    actor: claude-code (hc t14v)
```
