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
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
    holds: accept
    drives: review
    opened: 2026-11-13T22:38:50Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)

## Goal

A requirement whose SHALL sits at a line edge counts as a normative statement (#1076)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('Approve (Recommended)' for #1076, coordinator chat 2026-11-13T14:15Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At ad36b0e10 (#1076 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_shall_at_a_line_edge_is_a_normative_statement.](evidence/2026-11-13-at-ad36b0e10-1076-with-origin-main-bb7682a78-merged-in-fledge.md)
- [Red-first control: at the red commit affacaa99, change::tests::a_shall_at_a_line_edge_is_a_normative_statement fails: a requirement wrapped with SHALL at the end of a line, with its Acceptance Criteria, is refused as having no normative SHALL. The failure is the defect #1076 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-at-the-red-commit-affacaa99-change-tests-a.md)

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
