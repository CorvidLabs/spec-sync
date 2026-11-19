---
kind: session
id: 2026-11-13-887-memo-keeps-definite-answers
aliases: [887-memo-keeps-definite-answers]
objective: The repository-fact memo keeps definite answers and never a failed Git query, so a transient failure is retried (#887)
intent_scope: ""
modules: [git_utils]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
pr: ""
criteria: [SPEED-1.c]
requirements: []
cases: []
design: []
files: [src/git_utils.rs, src/change/validate_loaded_change.rs]
observations:
  - 2026-11-13-at-3810ab929-fix-887-with-origin-main-merged-in-the-verify
  - 2026-11-13-at-1898dc4bb-after-the-fix-fledge-lanes-run-verify-passed-10
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
    holds: accept
    drives: review
    opened: 2026-11-13T21:09:09Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# The repository-fact memo keeps definite answers and never a failed Git query, so a transient failure is retried (#887)

## Goal

The repository-fact memo keeps definite answers and never a failed Git query, so a transient failure is retried (#887)

## Where it stands

- Merged main, caught and fixed a spawn regression, reran the full verify lane, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#887 retry failed git', coordinator chat 2026-11-13T14:01Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: a show of a path the revision does not hold is a definite answer when the batch reader reports it missing or Git names the path, because both name the absence; a process 'bad revision' stays a failure since it is the same for a missing commit

## What was verified

- [At 3810ab929 (fix/887 with origin/main merged in), the verify lane's cargo test failed one test: cold_archived_status_prefetch_workers_reuse_repository_facts, 237 Git processes with six archives against main's 222, because a show <rev>:<path> of a path the revision does not hold was no longer memoized. Fixed in 1898dc4bb in this session by keeping the batch reader's missing answer, and Git's 'does not exist in', as definite; found and fixed before review, so no bug record.](evidence/2026-11-13-at-3810ab929-fix-887-with-origin-main-merged-in-the-verify.md)
- [At 1898dc4bb, after the fix, fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed, including the REQ-change-127 spawn-ceiling tests), release build, strict spec check, and the CI script tests. The cold archived status test reads 227 with six archives; a_failed_fact_is_not_memoized_but_a_definite_absence_is and a_missing_path_answer_is_definite_but_a_bad_revision_is_not pass.](evidence/2026-11-13-at-1898dc4bb-after-the-fix-fledge-lanes-run-verify-passed-10.md)
- [Red-first control: on main 18385ef5e with only the new test, git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is fails: a rev-parse that failed with 128 was memoized. The failure is the defect #887 reports, fixed by this change; not a new bug.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git.md)

## Open threads

## Timeline

- 2026-11-13T20:09:12Z · start · intent → intent · The repository-fact memo keeps definite answers and never a failed Git query, so a transient failure is retried (#887) ^t-04f68f
  - revision: 3810ab92980dd92611f0fc920810cfdc415742da
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:14Z · advance · intent → design ·  ^t-308995
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - revision: 3810ab92980dd92611f0fc920810cfdc415742da
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:14Z · skip · design → plan · Build, cache and CLI behaviour only: no visuals or interactions ^t-5717f5
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - revision: 3810ab92980dd92611f0fc920810cfdc415742da
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:16Z · advance · plan → build · native:verifying ^t-c8dd5f
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - gate: native:verifying
  - revision: 3810ab92980dd92611f0fc920810cfdc415742da
  - by: claude-code (hc t38p)
- 2026-11-13T20:09:16Z · advance · build → test · native:verifying ^t-e39116
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - gate: native:verifying
  - revision: 3810ab92980dd92611f0fc920810cfdc415742da
  - by: claude-code (hc t38p)
- 2026-11-13T21:09:09Z · advance · test → accept · native:verifying ^t-7954a9
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - gate: native:verifying
  - revision: 1898dc4bb4b2d11bbe740b52a94493b150ad50f9
  - by: claude-code (hc t38p)
- 2026-11-13T21:09:09Z · gate-opened · accept → accept · Implementation review of the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query ^t-62d2c4
  - native_change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
  - revision: 1898dc4bb4b2d11bbe740b52a94493b150ad50f9
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
