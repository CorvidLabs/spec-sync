---
kind: session
id: 2026-11-13-432-adopt-dry-run-validates-source
aliases: [432-adopt-dry-run-validates-source]
objective: change adopt --dry-run refuses an unsupported import source as adoption does (#432)
modules: [change]
stage: done
stage_status: archived
iteration: 1
native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
pr: "https://github.com/CorvidLabs/spec-sync/pull/1151"
criteria: [SETUP-5.b]
files: [src/change/historical_definition_digest_matches.rs, tests/integration/change.rs]
observations:
  - 2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
  - 2026-11-19-432-red-control-on-main-3bebe8b0
  - 2026-11-19-432-verify-lane-at-f8b341c5
closed: 2026-11-24
outcome: done
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

- At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does. - 27d78b1371d9 ^2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
- Red-first control: on main 18385ef5e with only the new test, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails. The failure is the defect #432 reports, fixed by this change; it is not a new bug. - 7796efa837e6 ^2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
- Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails: change adopt --source bogus --dry-run exits 0 and prints an import plan. This is the defect #432 reports, fixed by this change; not a new bug. - 8c2f548e6caf ^2026-11-19-432-red-control-on-main-3bebe8b0
- At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does. - 0a29bac949e3 ^2026-11-19-432-verify-lane-at-f8b341c5

## Bugs

None.

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

Closed 2026-11-24: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
    claim: "At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does."
    result: passed
    revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: e8d35f3032f532d31ed0d9f99906516cd10e5346803504d2d2e113049082ddec
    fingerprint: sha256:27d78b1371d971da7c4921ad61fa9d09618e614439a6c67f9cd675d2b93850c3
    criteria: SETUP-5.b
    observed: 2026-11-13T17:51:30Z
    actor: claude-code (hc t38p)
  - id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
    claim: "Red-first control: on main 18385ef5e with only the new test, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails. The failure is the defect #432 reports, fixed by this change; it is not a new bug."
    result: failed
    revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
    environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
    content: e8d35f3032f532d31ed0d9f99906516cd10e5346803504d2d2e113049082ddec
    fingerprint: sha256:7796efa837e6e40918df972bedb6c1c1ae54105a97119f65225598b5847b0263
    criteria: SETUP-5.b
    observed: 2026-11-13T17:51:31Z
    actor: claude-code (hc t38p)
  - id: 2026-11-19-432-red-control-on-main-3bebe8b0
    claim: "Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails: change adopt --source bogus --dry-run exits 0 and prints an import plan. This is the defect #432 reports, fixed by this change; not a new bug."
    result: failed
    revision: f8b341c574e0b61113983999a8dad19506f9eb24
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: 2f700bb9bf88dfae0d444c6fbdcd38ff8216f3e8fa75c8fb11d52fee2272fa99
    fingerprint: sha256:8c2f548e6caf6a14658d1b898bb0b744ac3e52017991c8fb043f498675da8520
    criteria: SETUP-5.b
    observed: 2026-11-19T08:47:14Z
    actor: claude-code (hc t59)
  - id: 2026-11-19-432-verify-lane-at-f8b341c5
    claim: "At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does."
    result: passed
    revision: f8b341c574e0b61113983999a8dad19506f9eb24
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: 2f700bb9bf88dfae0d444c6fbdcd38ff8216f3e8fa75c8fb11d52fee2272fa99
    fingerprint: sha256:0a29bac949e33f5328f56c77e0c6e88020756362371115873b8dd9c196f97c10
    criteria: SETUP-5.b
    observed: 2026-11-19T08:59:45Z
    actor: claude-code (hc t59)
```
