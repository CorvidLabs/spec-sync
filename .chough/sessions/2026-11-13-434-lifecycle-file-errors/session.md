---
kind: session
id: 2026-11-13-434-lifecycle-file-errors
aliases: [434-lifecycle-file-errors]
objective: Errors about a change's lifecycle files name the file, and a file a newer SpecSync wrote is refused alike everywhere, audit included (#434)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
pr: ""
criteria: [CHECK-9]
requirements: []
cases: []
design: []
files:
  - src/change/validate_loaded_change.rs
  - src/change/historical_definition_digest_matches.rs
  - src/change/relayed_decision.rs
  - tests/integration/lifecycle_file_errors.rs
observations:
  - 2026-11-13-at-791158b3e-feat-434-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-against-main-s-code-the-three-lifecycle-file
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
    holds: accept
    drives: review
    opened: 2026-11-13T20:55:17Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Errors about a change's lifecycle files name the file, and a file a newer SpecSync wrote is refused alike everywhere, audit included (#434)

## Goal

Errors about a change's lifecycle files name the file, and a file a newer SpecSync wrote is refused alike everywhere, audit included (#434)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own decision ('Yes (Recommended)', option text 'Small feature, clearer errors.', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370); REQ-change-164 and REQ-change-165 wording aligned by claude-code-t370; recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 791158b3e (feat/434 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 622 passed), release build, strict spec check, and the CI script tests, including the three lifecycle_file_errors integration tests.](evidence/2026-11-13-at-791158b3e-feat-434-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: against main's code the three lifecycle_file_errors tests fail: a type error in approvals.json did not name the file, and state.json and verification-attempts.json from a newer SpecSync were read or not refused alike. These are the gaps #434 reports, fixed by this change; not new bugs.](evidence/2026-11-13-red-first-control-against-main-s-code-the-three-lifecycle-file.md)

## Open threads

## Timeline

- 2026-11-13T20:09:08Z · start · intent → intent · Errors about a change's lifecycle files name the file, and a file a newer SpecSync wrote is refused alike everywhere, audit included (#434) ^t-59a6ef
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:10Z · advance · intent → design ·  ^t-6d4522
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:10Z · skip · design → plan · Build, cache and CLI behaviour only: no visuals or interactions ^t-00ef32
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:12Z · advance · plan → build · native:verifying ^t-53649c
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - gate: native:verifying
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:12Z · advance · build → test · native:verifying ^t-8834e1
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - gate: native:verifying
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:55:17Z · advance · test → accept · native:verifying ^t-f90881
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - gate: native:verifying
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - by: claude-code (hc t38p)
- 2026-11-13T20:55:17Z · gate-opened · accept → accept · Implementation review of errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere ^t-045a9b
  - native_change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
  - revision: 791158b3e0981a215936c139dbe69a848db28015
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
