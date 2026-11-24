---
kind: session
id: 2026-11-24-1149-strict-check-policy-flip
aliases: [1149-strict-check-policy-flip]
objective: "check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)"
intent_scope: CHANGE
modules: [cmd_check, change]
stage: plan
stage_status: active
iteration: 1
callers: []
native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
pr: "1149"
criteria: [CHANGE-17.d]
requirements: [REQ-cmd-check-004, REQ-change-archived-requirement-loss-001]
cases: []
design: []
files: []
observations: [2026-11-24-1149-policy-flip-red-at-a8b31fe8]
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Close the scope gate with 0xLeif's recorded decision, then add the failing CLI cases
---
# check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)

## Goal

check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)

## Where it stands

- Drafted the native change: interview answered, six artifacts written, MODIFIED deltas for cmd_check (REQ-cmd-check-004) and change (REQ-change-archived-requirement-loss-001); chough change ready reports nothing missing and approve --dry-run would approve

## Decisions and why

- 0xLeif chose 'Fix as a successor change (Recommended)' when asked what to do with the Codex P1 on #1149 (Claude Code release chat, 2026-11-24 about 04:42 UTC). The option approved the scope shown with it (cmd_check and change; check.rs, one helper in change, the vanished-requirements tests, specs/cmd_check/, specs/change/; MODIFIED REQ-cmd-check-004) and archival once the full native Verify passes. Not chosen: a plain fix commit, deferring as an issue, holding #1149 out of 6.4.2.

## What was verified

- 2026-11-24-1149-policy-flip-red-at-a8b31fe8

## Open threads

## Timeline

- 2026-11-24T04:44:50Z · start · intent → intent · check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d) ^t-5ddbfa
  - revision: 2e2ddab1af06a6b8e877a3efd6fb40801786a2fc
  - by: claude-code (hc t14v)
- 2026-11-24T04:45:13Z · advance · intent → design ·  ^t-1a2afc
  - revision: 2e2ddab1af06a6b8e877a3efd6fb40801786a2fc
  - by: claude-code (hc t14v)
- 2026-11-24T04:45:13Z · skip · design → plan · CLI behaviour only: one policy read in check --strict. No visuals or interactions change. ^t-41d20c
  - revision: 2e2ddab1af06a6b8e877a3efd6fb40801786a2fc
  - by: claude-code (hc t14v)
