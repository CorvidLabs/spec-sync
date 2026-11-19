---
kind: session
id: 2026-11-13-985-sha256-batch-ids
aliases: [985-sha256-batch-ids]
objective: Batched Git reads accept SHA-256 object ids, so SHA-256 repositories keep the batch reader (#985)
intent_scope: ""
modules: [change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-batched-object-reader-accepts-sha-256-object-ids
pr: ""
criteria: [SPEED-1.a]
requirements: []
cases: []
design: []
files: [src/git_utils.rs]
observations:
  - 2026-11-13-at-84f77a405-fix-985-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git
  - 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-git-utils
  - 2026-11-19-refreshed-onto-main-3bebe8b0-with-887-981-and-982-batched
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-batched-object-reader-accepts-sha-256-object-ids
    holds: accept
    drives: review
    opened: 2026-11-13T16:57:54Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Batched Git reads accept SHA-256 object ids, so SHA-256 repositories keep the batch reader (#985)

## Goal

Batched Git reads accept SHA-256 object ids, so SHA-256 repositories keep the batch reader (#985)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#985 SHA-256 repos', coordinator chat 2026-11-13T03:49Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson for change.3md, because the fix is a narrow correction an existing lesson already covers
- Refreshed onto main 3bebe8b0 (batch 7a) under Leif's relayed answer of 2026-11-18 that the 2026-11-13 approvals stand, 'Yes, they stand' (from his planning session hc t4c, relayed through ss-lead t58); no approved sentence changed, and no review, approval or gate close was recorded

## What was verified

- [At 84f77a405 (fix/985 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including git_utils::tests::batched_reads_accept_sha256_object_ids.](evidence/2026-11-13-at-84f77a405-fix-985-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: on main 18385ef5e with only the new test, git_utils::tests::batched_reads_accept_sha256_object_ids fails. The failure is the defect #985 reports, fixed by this change; it is not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git.md)
- 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git
- 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-git-utils
- [Refreshed onto main 3bebe8b0 with #887, #981 and #982; batched_reads_accept_sha256_object_ids passes. fledge lanes run verify at 42dda911 with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10/10 steps, 3085 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage; lifecycle_commands_stay_within_git_spawn_ceilings passes; change audit --strict and pre-push passed.](evidence/2026-11-19-refreshed-onto-main-3bebe8b0-with-887-981-and-982-batched.md)

## Open threads

## Timeline

- 2026-11-13T16:16:48Z · start · intent → intent · Batched Git reads accept SHA-256 object ids, so SHA-256 repositories keep the batch reader (#985) ^t-97c8e2
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:01Z · advance · intent → design ·  ^t-fe6094
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:01Z · skip · design → plan · Internal Git read path only: no visuals or interactions ^t-181949
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:03Z · advance · plan → build · native:verifying ^t-a5c702
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - gate: native:verifying
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:17:03Z · advance · build → test · native:verifying ^t-21ec1b
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - gate: native:verifying
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:57:54Z · advance · test → accept · native:verifying ^t-ef6e45
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - gate: native:verifying
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - by: claude-code (hc t38p)
- 2026-11-13T16:57:54Z · gate-opened · accept → accept · Implementation review of the-batched-object-reader-accepts-sha-256-object-ids ^t-cc225f
  - native_change: the-batched-object-reader-accepts-sha-256-object-ids
  - revision: 84f77a405718e452597582493300b184d3f3f9af
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
