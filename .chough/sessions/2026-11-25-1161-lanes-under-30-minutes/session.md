---
kind: session
id: 2026-11-25-1161-lanes-under-30-minutes
aliases: [1161-lanes-under-30-minutes]
objective: "Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)"
intent_scope: SPEED-2.b
modules: []
stage: plan
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
observations: []
bugs: []
blockers: []
waiting_on_human:
  - gate: Scope approval of release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
    holds: plan
    drives: approve
    opened: 2026-11-25T03:29:59Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
summary:
  - "Defined the first #1161 change: qualification as ten parts (four test slices and one release build on each platform) within 30 minutes each, platform evidence built from the part records, a rehearsal without a candidate, and every workflow job limited to 30 minutes. Verified with the pinned cargo-nextest that four slices are disjoint and cover all 4,187 tests. The approval dry run passes; the requirement words wait for Leif."
---
# Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)

## Goal

Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)

## Where it stands

## Decisions and why

## What was verified

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
