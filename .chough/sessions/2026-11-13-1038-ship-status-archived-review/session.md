---
kind: session
id: 2026-11-13-1038-ship-status-archived-review
aliases: [1038-ship-status-archived-review]
objective: ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
pr: ""
criteria: [SHIP-8]
requirements: []
cases: []
design: []
files: [src/change/consider_accepted_evidence_anchor.rs, src/commands/change.rs]
observations:
  - 2026-11-13-at-f52c39ff3-fix-1038-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-at-f52c39ff3-verify-step-4-spec-check-strict-failed-archived
  - 2026-11-13-at-e1919d06f-after-the-public-api-row-spec-check-strict-62
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
    holds: accept
    drives: review
    opened: 2026-11-13T20:07:47Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)

## Goal

ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own decision ('Bug fix (Recommended)', option text 'For an archived change, report the review as done by finalize and suggest nothing.', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At f52c39ff3 (fix/1038 with origin/main bb7682a78 merged in), fledge lanes run verify steps 1 to 3 passed: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed), including the commands::change tests for an archived change's review stage and its archive-only tip, and the release build.](evidence/2026-11-13-at-f52c39ff3-fix-1038-with-origin-main-bb7682a78-merged-in.md)
- [At f52c39ff3, verify step 4 spec-check --strict failed: archived_review_is_settled_by_finalize, the crate-visible helper this change adds, had no Public API row in specs/change/change.spec.md. Fixed in e1919d06f in this session by adding the row; found and fixed before review, so no bug record.](evidence/2026-11-13-at-f52c39ff3-verify-step-4-spec-check-strict-failed-archived.md)
- [At e1919d06f, after the Public API row: spec-check --strict (62 specs, 100% file and LOC coverage), release-candidate-test, ci-gate-test, linux-baseline-test, ci-trigger-paths-test, ci-attest-and-summary-test and clippy-all-targets-test passed. Rust sources are unchanged since steps 1 to 3.](evidence/2026-11-13-at-e1919d06f-after-the-public-api-row-spec-check-strict-62.md)
- [Red-first control: on main 18385ef5e with only the new tests, an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review and an_archive_tip_that_removes_its_own_package_files_is_archive_only fail. These are the defects #1038 reports, fixed by this change; not new bugs.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an.md)

## Open threads

## Timeline

- 2026-11-13T19:23:28Z · start · intent → intent · ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038) ^t-c7c8f5
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:30Z · advance · intent → design ·  ^t-a0a1bc
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:30Z · skip · design → plan · CLI, CI and docs behaviour only: no visuals or interactions ^t-a59442
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:32Z · advance · plan → build · native:verifying ^t-b0b998
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:32Z · advance · build → test · native:verifying ^t-ac5b84
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T20:07:47Z · advance · test → accept · native:verifying ^t-5b933b
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: e1919d06f5ce03090fe954ecc8eb99d3064212b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:07:47Z · gate-opened · accept → accept · Implementation review of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only ^t-75dae6
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: e1919d06f5ce03090fe954ecc8eb99d3064212b6
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
