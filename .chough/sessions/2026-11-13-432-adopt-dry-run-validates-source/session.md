---
kind: session
id: 2026-11-13-432-adopt-dry-run-validates-source
aliases: [432-adopt-dry-run-validates-source]
objective: change adopt --dry-run refuses an unsupported import source as adoption does (#432)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
pr: ""
criteria: [SETUP-5.b]
requirements: []
cases: []
design: []
files: [src/change/historical_definition_digest_matches.rs, tests/integration/change.rs]
observations:
  - 2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
    holds: accept
    drives: review
    opened: 2026-11-13T17:51:33Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# change adopt --dry-run refuses an unsupported import source as adoption does (#432)

## Goal

change adopt --dry-run refuses an unsupported import source as adoption does (#432)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#432 adopt dry run', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers

## What was verified

- [At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does.](evidence/2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails. The failure is the defect #432 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test.md)

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
