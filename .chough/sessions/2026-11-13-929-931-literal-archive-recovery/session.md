---
kind: session
id: 2026-11-13-929-931-literal-archive-recovery
aliases: [929-931-literal-archive-recovery]
objective: "Corrupt-archive recovery prints restore commands Git and SpecSync read literally, with control bytes shown as text (#929, #930, #931)"
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
pr: ""
criteria: [CHANGE-7.a.2, CHANGE-7.b]
requirements: []
cases: []
design: []
files:
  - src/change/historical_definition_digest_matches.rs
  - src/change_tests/literal_guidance.rs
  - tests/integration/change.rs
  - hi/change.md
observations:
  - 2026-11-13-at-ea28c3e99-fix-929-931-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests
  - 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-all-five
  - 2026-11-19-refreshed-onto-main-3bebe8b0-with-990-and-1074-materialized
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
    holds: accept
    drives: review
    opened: 2026-11-13T19:22:20Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Corrupt-archive recovery prints restore commands Git and SpecSync read literally, with control bytes shown as text (#929, #930, #931)

## Goal

Corrupt-archive recovery prints restore commands Git and SpecSync read literally, with control bytes shown as text (#929, #930, #931)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own approval ('Approve (Recommended)' for #929-931 as one PR, coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370); REQ-change-093 wording aligned; recorded natively as 0xLeif in relay form; file list grew under the standing file-list rule (04:35Z)
- decided: no new lesson for change.3md, because the change is narrow and an existing lesson covers it
- Refreshed onto main 3bebe8b0 (batch 4) under Leif's relayed answer of 2026-11-18 that the 2026-11-13 approvals stand, 'Yes, they stand' (from his planning session hc t4c, relayed through ss-lead t58); no approved sentence changed, and no review, approval or gate close was recorded

## What was verified

- [At ea28c3e99 (fix/929-931 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3075 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including the literal-guidance tests in src/change_tests/literal_guidance.rs.](evidence/2026-11-13-at-ea28c3e99-fix-929-931-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new tests, four literal-guidance tests fail (leading-hyphen id, control byte in a directory name, package probe as one literal path, glob characters in the restore). These are the defects #929, #930 and #931 report, fixed by this change; not new bugs.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests.md)
- [Red-first control for the refresh onto main 3bebe8b0: all five literal_guidance tests fail (glob characters in the restore, the archiving-commit lookup, the package probe, the leading-hyphen id, a control byte in a directory name), and so do the four updated unit tests (strict_check_reports_standalone_unprovable_archived_history, handoff_corrupt_archive_is_not_yet_and_names_the_restore, corrupt_archive_restore_quotes_every_interpolated_word, corrupt_archive_restore_names_the_package_only_when_the_package_changed) and the integration test a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it. These are the defects #929, #930 and #931 report.](evidence/2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-all-five.md)
- [Refreshed onto main 3bebe8b0 with #990 and #1074, materialized again the #1086 way (history fragment; REQ-change-093 byte for byte the 11-13 text; no version bump or Change Log row). The ten red tests pass at the tip. fledge lanes run verify at 17e42ce1 with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10 of 10 steps in 8m40s, 3084 unit and 627 integration tests passed, strict specs 62/62, file and LOC coverage 100%; change audit --strict and pre-push passed.](evidence/2026-11-19-refreshed-onto-main-3bebe8b0-with-990-and-1074-materialized.md)

## Open threads

## Timeline

- 2026-11-13T18:40:08Z · start · intent → intent · Corrupt-archive recovery prints restore commands Git and SpecSync read literally, with control bytes shown as text (#929, #930, #931) ^t-352e71
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:10Z · advance · intent → design ·  ^t-2ea313
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:10Z · skip · design → plan · CLI text and Git read behaviour only: no visuals or interactions ^t-c66c15
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:12Z · advance · plan → build · native:verifying ^t-65016d
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - gate: native:verifying
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T18:40:12Z · advance · build → test · native:verifying ^t-acdeff
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - gate: native:verifying
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:20Z · advance · test → accept · native:verifying ^t-96a9a7
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - gate: native:verifying
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - by: claude-code (hc t38p)
- 2026-11-13T19:22:20Z · gate-opened · accept → accept · Implementation review of print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text ^t-d9bdd4
  - native_change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
  - revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
