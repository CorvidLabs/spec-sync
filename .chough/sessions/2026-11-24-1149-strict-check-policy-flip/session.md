---
kind: session
id: 2026-11-24-1149-strict-check-policy-flip
aliases: [1149-strict-check-policy-flip]
objective: "check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)"
intent_scope: CHANGE
modules: [cmd_check, change]
stage: accept
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
observations:
  - 2026-11-24-1149-policy-flip-red-at-a8b31fe8
  - 2026-11-24-1149-policy-flip-green-with-the-fix
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
    holds: accept
    drives: review
    opened: 2026-11-24T14:12:15Z
run_consents: []
next: Commit, run change check --commit and the full native Verify lane, then ship on 0xLeif's recorded decision
---
# check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)

## Goal

check --strict keeps the lost-requirements comparison of #1149 when a branch disables or deletes the SDD policy its comparison base has enabled, as change audit does (Codex P1 on #1149, CHANGE-17.d)

## Where it stands

- Drafted the native change: interview answered, six artifacts written, MODIFIED deltas for cmd_check (REQ-cmd-check-004) and change (REQ-change-archived-requirement-loss-001); chough change ready reports nothing missing and approve --dry-run would approve
- Added the failing CLI cases and the disabled-at-base control, then the fix: check --strict asks the change module whether SDD governs the branch (working tree, then the trusted comparison base audit reads). Updated both contracts, testing companions, task lists and the cmd_check lesson.

## Decisions and why

- 0xLeif chose 'Fix as a successor change (Recommended)' when asked what to do with the Codex P1 on #1149 (Claude Code release chat, 2026-11-24 about 04:42 UTC). The option approved the scope shown with it (cmd_check and change; check.rs, one helper in change, the vanished-requirements tests, specs/cmd_check/, specs/change/; MODIFIED REQ-cmd-check-004) and archival once the full native Verify passes. Not chosen: a plain fix commit, deferring as an issue, holding #1149 out of 6.4.2.
- 0xLeif decided "Scope approval of keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off": "Approve both deltas (Recommended)" (Claude Code release-lead chat, 2026-11-24 UTC, a multiple-choice answer recorded by the agent he answered. The option read: 'Scope approval of the full definition: MODIFIED REQ-cmd-check-004 as you saw it, plus MODIFIED REQ-change-archived-requirement-loss-001 as in the preview, and archival once the full native Verify passes.' It followed his answer 'Fix as a successor change (Recommended)' to the question on the Codex P1 on #1149.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off as 0xLeif

## What was verified

- 2026-11-24-1149-policy-flip-red-at-a8b31fe8
- [With the fix on the working tree over 2e2ddab1 (REQ-cmd-check-004, REQ-change-archived-requirement-loss-001): strict_check_keeps_the_comparison_when_the_branch_turns_sdd_off failed before the fix at check --strict and passes now; the disabled-at-base control passes both times. All 27 tests of the vanished_requirements integration module pass. Through the binary, the two policy-flip cases now exit 1 and name the loss in both check --strict and change audit --strict, uncommitted and committed, and the three controls are unchanged.](evidence/2026-11-24-1149-policy-flip-green-with-the-fix.md)

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
- 2026-11-24T14:07:18Z · gate-opened · plan → plan · Scope approval of keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off ^t-e781b2
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - revision: 3c837718a02eb323bbfd7df8fab019bf6a99b297
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-24T14:07:26Z · gate-closed · plan → plan · Scope approval of keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off ^t-0376a6
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - revision: 3c837718a02eb323bbfd7df8fab019bf6a99b297
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve both deltas (Recommended)
    - where: Claude Code release-lead chat, 2026-11-24 UTC, a multiple-choice answer recorded by the agent he answered. The option read: 'Scope approval of the full definition: MODIFIED REQ-cmd-check-004 as you saw it, plus MODIFIED REQ-change-archived-requirement-loss-001 as in the preview, and archival once the full native Verify passes.' It followed his answer 'Fix as a successor change (Recommended)' to the question on the Codex P1 on #1149.
    - recorded_by: claude-code-t14v
    - native: specsync change approve keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-24T14:07:39Z · advance · plan → build · native:approved ^t-14d15b
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - gate: native:approved
  - revision: af9ee3727c15ef74371508dddabdaf168e540459
  - by: claude-code (hc t14v)
- 2026-11-24T14:11:18Z · advance · build → test · native:approved ^t-bb6d1b
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - gate: native:approved
  - revision: af9ee3727c15ef74371508dddabdaf168e540459
  - by: claude-code (hc t14v)
- 2026-11-24T14:11:19Z · advance · test → accept · native:approved ^t-e2b86d
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - gate: native:approved
  - revision: af9ee3727c15ef74371508dddabdaf168e540459
  - by: claude-code (hc t14v)
- 2026-11-24T14:12:15Z · gate-opened · accept → accept · Implementation review of keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off ^t-f945a4
  - native_change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
  - revision: af9ee3727c15ef74371508dddabdaf168e540459
  - holds: accept
  - drives: review
  - by: claude-code (hc t14v)
