---
kind: session
id: 2026-11-13-432-adopt-dry-run-validates-source
aliases: [432-adopt-dry-run-validates-source]
objective: change adopt --dry-run refuses an unsupported import source as adoption does (#432)
intent_scope: ""
modules: [change]
stage: done
stage_status: active
iteration: 1
callers: []
native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
pr: "https://github.com/CorvidLabs/spec-sync/pull/1151"
criteria: [SETUP-5.b]
requirements: []
cases: []
design: []
files: [src/change/historical_definition_digest_matches.rs, tests/integration/change.rs]
observations:
  - 2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
  - 2026-11-19-432-red-control-on-main-3bebe8b0
  - 2026-11-19-432-verify-lane-at-f8b341c5
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Merge PR #1151 when the archived revision passes the required checks and review is complete."
---
# change adopt --dry-run refuses an unsupported import source as adoption does (#432)

## Goal

change adopt --dry-run refuses an unsupported import source as adoption does (#432)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 with #475 and #1012 on fix/cli-432-475-1012; no conflict; a CHANGELOG Fixed line added; re-checked; red control on main and the verify lane recorded

## Decisions and why

- decided: authorization is Leif's own scope approval ('#432 adopt dry run', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers
- 0xLeif said in the November 23 SpecSync 6.4.2 release conversation: I inspected the remaining PRs and accept archival when qualified, including both ID renewals and #1114. Native review is recorded in .specsync/archive/changes/2026-11-23-change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does/review.json. Withdraw the duplicated session gate after native finalization.

## What was verified

- [At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does.](evidence/2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails. The failure is the defect #432 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)
- [Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails: change adopt --source bogus --dry-run exits 0 and prints an import plan. This is the defect #432 reports, fixed by this change; not a new bug.](evidence/2026-11-19-432-red-control-on-main-3bebe8b0.md)
- [At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does.](evidence/2026-11-19-432-verify-lane-at-f8b341c5.md)

## Open threads

## Timeline

- 2026-11-13T16:59:29Z · start · intent → intent · change adopt --dry-run refuses an unsupported import source as adoption does (#432) ^t-75f6d6
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:33Z · advance · intent → design ·  ^t-16eb68
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:33Z · skip · design → plan · CLI or CI behaviour only: no visuals or interactions ^t-978630
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:37Z · advance · plan → build · native:verifying ^t-4dcaea
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T16:59:37Z · advance · build → test · native:verifying ^t-b9285b
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:33Z · advance · test → accept · native:verifying ^t-0f37ce
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - by: claude-code (hc t38p)
- 2026-11-13T17:51:33Z · gate-opened · accept → accept · Implementation review of change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does ^t-062cd2
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
- 2026-11-19T08:44:55Z · back · accept → build · Batch 10 refresh onto main 3bebe8b0 (ss-builder, hc t59): the change gets its CHANGELOG Fixed line and is re-checked on the merged branch ^t-1bae5e
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: d8a3ba6c77873600e87b269625bd2d4d4c57613d
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T08:59:48Z · advance · build → test · native:verifying ^t-9e97a9
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-19T08:59:49Z · advance · test → accept · native:verifying ^t-0f22db
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:verifying
  - revision: f8b341c574e0b61113983999a8dad19506f9eb24
  - by: claude-code (hc t59)
- 2026-11-24T00:12:23Z · gate-closed · accept → accept · Implementation review of change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does ^t-fe94f0
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
- 2026-11-24T00:12:27Z · advance · accept → done · Native scoped review and finalization for PR #1151 are recorded in the archive. ^t-de797d
  - native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
  - gate: native:archived
  - revision: da57656d17af44e7945dbb4fa1c2267402838647
  - by: unknown agent (hc t14v)
