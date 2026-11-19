---
kind: session
id: 2026-11-19-1081-vanished-requirements
aliases: [1081-vanished-requirements]
objective: check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)
intent_scope: ""
modules: [change, cmd_change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
pr: ""
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: []
cases: []
design: []
files: []
observations: [2026-11-19-covers-req-change-169-and-req-cmd-change-042-the-latter-not]
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of #1081"
    holds: accept
    drives: review
    opened: 2026-11-19T15:09:52Z
run_consents: []
next: Continue accept
---
# check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)

## Goal

check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names": "Approve as written" (Claude Code chat with his planning session hc t4c, 2026-11-19 about 14:17Z, on ss-lead's frozen gates/1081-definition.md (CHANGE-17 and .a to .c marked as the agent's draft, REQ-change-169, REQ-cmd-change-042, five choices, no CI-side guard); relayed through ss-lead t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names as 0xLeif

## What was verified

- [Covers REQ-change-169 and REQ-cmd-change-042 (the latter not linked: chough 0.6.0 looks it up under specs/cmd-change). Red first: a_branch_that_lost_requirements_no_removed_delta_names_is_refused (the starling shape) failed on main 3bebe8b0, committed at 5a7b6d68 before the fix. Now 9 unit tests (starling truncation with refused attempt and audit; accounted removals: same change, another active change, an archived change; merged-in vs cherry-picked main removal; move vs renumber with the permanence hint; deleted file and legacy .req.md; base_commit fallback, skip note, unreadable merge-base) and an integration test (ship-status blocker, ship --dry-run, finalize, re-check) pass. Real repos: on CorvidLabs/starling at PR #43's last open commit 3b1dca0 (origin/main at b68d547), released 6.3.0's change check passes (exit 0) and this build refuses (exit 1) naming all 26 lost REQ-StarlingCore ids, the file and the way out; starling main db97480 and Chough main 9d260310 audit clean (exit 0) under both builds; spec-sync's own check on this branch passed. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3085 unit and 628 integration tests (including the git-spawn ceiling test), strict specs 62/62, 100% file and LOC coverage; fledge trust verify passed.](evidence/2026-11-19-covers-req-change-169-and-req-cmd-change-042-the-latter-not.md)

## Open threads

## Timeline

- 2026-11-19T14:23:36Z · start · intent → intent · check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58) ^t-7d3cb6
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:23:42Z · advance · intent → design ·  ^t-7a0beb
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:25:51Z · skip · design → plan · Record and CLI behaviour only: no visuals or interactions ^t-08d095
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:27:20Z · gate-opened · plan → plan · Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names ^t-2be65a
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T14:27:20Z · gate-closed · plan → plan · Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names ^t-e7e7c5
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve as written
    - where: Claude Code chat with his planning session hc t4c, 2026-11-19 about 14:17Z, on ss-lead's frozen gates/1081-definition.md (CHANGE-17 and .a to .c marked as the agent's draft, REQ-change-169, REQ-cmd-change-042, five choices, no CI-side guard); relayed through ss-lead t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:43Z · direct-native-write · plan → plan · check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-6017c5
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - direct_write:
    - change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:43Z · advance · plan → build · native:verifying ^t-e79c5f
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:47Z · advance · build → test · native:verifying ^t-f87d4c
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:52Z · advance · test → accept · native:verifying ^t-aba92d
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:52Z · gate-opened · accept → accept · Leif's review of #1081 ^t-db1014
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
