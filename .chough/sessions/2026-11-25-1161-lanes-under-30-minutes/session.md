---
kind: session
id: 2026-11-25-1161-lanes-under-30-minutes
aliases: [1161-lanes-under-30-minutes]
objective: "Every test lane finishes within 30 minutes: split the macOS release-candidate qualification into parallel jobs, start fewer Git processes for each lifecycle operation, and set each lane's limit there so growth fails instead of creeping (#1161)"
intent_scope: SPEED-2.b
modules: []
stage: design
stage_status: active
iteration: 1
callers: []
native_change: ""
pr: ""
criteria: [SPEED-2.b.2, SPEED-2.b.3, SPEED-2.b.4, SPEED-2.b.5]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
summary:
  - "Intent for #1161 captured on Leif's yes: SPEED-2.b.3 (30-minute max), SPEED-2.b.4 (a slower lane fails at its limit, the limit is not raised) and SPEED-2.b.5 (a release candidate does not fail on time), beside SPEED-2.b.1; SPEED-2.b.2 (no test dropped or weakened) bounds the work. Measured: macOS qualification 78m32s, Ubuntu 22m31s, CI coverage 22m24s, CI test 10m27s; four round-robin test slices put each macOS job near 23 minutes."
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
