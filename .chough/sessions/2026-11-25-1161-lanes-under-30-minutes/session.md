---
kind: session
id: 2026-11-25-1161-lanes-under-30-minutes
aliases: [1161-lanes-under-30-minutes]
objective: "Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)"
intent_scope: SPEED-2.b
modules: []
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
pr: ""
criteria: [SPEED-2.b.2, SPEED-2.b.3, SPEED-2.b.4, SPEED-2.b.5]
requirements: [REQ-github-007, REQ-github-031, REQ-github-032, REQ-github-033]
cases: []
design: []
files: []
observations:
  - 2026-11-25-before-the-implementation-at-25d015b0-with-only-the-new
  - 2026-11-25-at-a6c79b5a-under-the-native-lane-lock-one-real-slice-through
  - 2026-11-25-the-rehearsal-workflow-ran-the-ten-parts-of-the-release
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
    holds: accept
    drives: review
    opened: 2026-11-25T04:33:26Z
run_consents: []
next: Continue accept
summary:
  - "Defined the first #1161 change: qualification as ten parts (four test slices and one release build on each platform) within 30 minutes each, platform evidence built from the part records, a rehearsal without a candidate, and every workflow job limited to 30 minutes. Verified with the pinned cargo-nextest that four slices are disjoint and cover all 4,187 tests. The approval dry run passes; the requirement words wait for Leif."
---
# Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)

## Goal

Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit": "I approve" (Claude Code release-lead chat, 2026-11-25 UTC, typed by him to the agent recording it. It answered a message headed 'Decision for you: approve these requirement words?' that quoted the three added requirement statements in full (REQ-github-031 parallel parts and whole test set, REQ-github-032 a 30-minute limit on every workflow job, REQ-github-033 a rehearsal without a candidate) and described the two amended lines of REQ-github-007. The message named two things beyond the scope he had said yes to earlier in the same chat, the rehearsal on pull requests that change the lane and the weekly rehearsal, and said he could strike the weekly one; he did not. The acceptance criteria were in the committed definition at 42be91f6 and were summarized, not quoted.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit as 0xLeif

## What was verified

- [Before the implementation, at 25d015b0 with only the new contract tests added: 61 of 98 tests in test-validate-release-candidate.py failed (the validator had no parts or part-record mode, the slice script and the rehearsal workflow did not exist, and the qualify job was one job per platform at 120 minutes), and 4 of 6 in test-workflow-time-limits.py failed, naming the jobs over 30 minutes or with no limit.](evidence/2026-11-25-before-the-implementation-at-25d015b0-with-only-the-new.md)
- [At a6c79b5a, under the native lane lock: one real slice through the new lane and slice script with the pinned cargo-nextest 0.9.151 (slice 1 of 4: 1047 tests run, 1047 passed, counts file 1047 of 4187), then fledge lanes run verify, 11 steps in 34m14s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check, and the seven CI contract suites (98 release-candidate tests, 7 workflow time-limit tests). No src, tests, .github, Cargo or fledge.toml path changed between that commit and a29b4a43, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 45) pass. No test was removed, ignored or changed in what it asserts about the product.](evidence/2026-11-25-at-a6c79b5a-under-the-native-lane-lock-one-real-slice-through.md)
- [The rehearsal workflow ran the ten parts of the release-candidate lane at a6c79b5a, each under its 30-minute limit, and the whole run took 22m23s. macOS: tests 1 of 4 18m32s, 2 of 4 21m37s, 3 of 4 22m11s, 4 of 4 21m22s, build 15m45s; all five macOS jobs started together. Ubuntu: 4m43s, 4m30s, 4m08s, 4m47s, build 5m51s. The slices-add-up job passed: macOS slices 1047, 1047, 1047 and 1046 are its whole list of 4187, and Ubuntu's 1049, 1049, 1048 and 1048 are its whole list of 4194. The macOS qualification of v6.4.2-rc.2 took 78m32s as one job.](evidence/2026-11-25-the-rehearsal-workflow-ran-the-ten-parts-of-the-release.md)

## Open threads

## Timeline

- 2026-11-25T02:58:40Z · start · intent → intent · Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161) ^t-868c25
  - revision: 778a111a7caccd31e0f9fb52f61601828d84f006
  - by: claude-code (hc t14v)
- 2026-11-25T03:23:11Z · advance · intent → design ·  ^t-0036d3
  - revision: 778a111a7caccd31e0f9fb52f61601828d84f006
  - by: claude-code (hc t14v)
- 2026-11-25T03:29:59Z · skip · design → plan · Workflow, validator and documentation changes. No visuals or interactions. ^t-91d1bf
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - revision: 39e7e17be6c6614f8bd49604de9c599939b9fa30
  - by: claude-code (hc t14v)
- 2026-11-25T03:29:59Z · gate-opened · plan → plan · Scope approval of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit ^t-827c5c
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - revision: 39e7e17be6c6614f8bd49604de9c599939b9fa30
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-25T03:31:54Z · direct-native-write · plan → plan · release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state draft → approved ^t-737bff
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - revision: 42be91f60b7928696dfe03ecd043d44790855df2
  - direct_write:
    - change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: draft
    - after_state: approved
  - by: claude-code (hc t14v)
- 2026-11-25T03:31:54Z · gate-closed · plan → plan · Scope approval of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit ^t-0747de
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - revision: 42be91f60b7928696dfe03ecd043d44790855df2
  - relayed_by:
    - decided_by: 0xLeif
    - words: I approve
    - where: Claude Code release-lead chat, 2026-11-25 UTC, typed by him to the agent recording it. It answered a message headed 'Decision for you: approve these requirement words?' that quoted the three added requirement statements in full (REQ-github-031 parallel parts and whole test set, REQ-github-032 a 30-minute limit on every workflow job, REQ-github-033 a rehearsal without a candidate) and described the two amended lines of REQ-github-007. The message named two things beyond the scope he had said yes to earlier in the same chat, the rehearsal on pull requests that change the lane and the weekly rehearsal, and said he could strike the weekly one; he did not. The acceptance criteria were in the committed definition at 42be91f6 and were summarized, not quoted.
    - recorded_by: claude-code-t14v
    - native: specsync change approve release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-25T03:31:57Z · advance · plan → build · native:approved ^t-58772e
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - gate: native:approved
  - revision: 42be91f60b7928696dfe03ecd043d44790855df2
  - by: claude-code (hc t14v)
- 2026-11-25T04:33:22Z · advance · build → test · native:verifying ^t-1672e1
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - gate: native:verifying
  - revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
  - by: claude-code (hc t14v)
- 2026-11-25T04:33:25Z · advance · test → accept · native:verifying ^t-b6db71
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - gate: native:verifying
  - revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
  - by: claude-code (hc t14v)
- 2026-11-25T04:33:26Z · gate-opened · accept → accept · Implementation review of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit ^t-6d23e2
  - native_change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
  - revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
  - holds: accept
  - drives: review
  - by: claude-code (hc t14v)
