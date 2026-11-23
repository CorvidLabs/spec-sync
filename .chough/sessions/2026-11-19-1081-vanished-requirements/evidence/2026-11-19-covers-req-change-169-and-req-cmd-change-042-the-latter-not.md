---
kind: evidence
id: 2026-11-19-covers-req-change-169-and-req-cmd-change-042-the-latter-not
aliases: []
claim: "Covers REQ-change-169 and REQ-cmd-change-042 (the latter not linked: chough 0.6.0 looks it up under specs/cmd-change). Red first: a_branch_that_lost_requirements_no_removed_delta_names_is_refused (the starling shape) failed on main 3bebe8b0, committed at 5a7b6d68 before the fix. Now 9 unit tests (starling truncation with refused attempt and audit; accounted removals: same change, another active change, an archived change; merged-in vs cherry-picked main removal; move vs renumber with the permanence hint; deleted file and legacy .req.md; base_commit fallback, skip note, unreadable merge-base) and an integration test (ship-status blocker, ship --dry-run, finalize, re-check) pass. Real repos: on CorvidLabs/starling at PR #43's last open commit 3b1dca0 (origin/main at b68d547), released 6.3.0's change check passes (exit 0) and this build refuses (exit 1) naming all 26 lost REQ-StarlingCore ids, the file and the way out; starling main db97480 and Chough main 9d260310 audit clean (exit 0) under both builds; spec-sync's own check on this branch passed. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3085 unit and 628 integration tests (including the git-spawn ceiling test), strict specs 62/62, 100% file and LOC coverage; fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
content: 84255a8de476f145e555f54ce7f8c6c60e44a5f3cdca87bb0de36ee03527721f
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at d8612448
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-19T15:09:37Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Covers REQ-change-169 and REQ-cmd-change-042 (the latter not linked: chough 0.6.0 looks it up under specs/cmd-change). Red first: a_branch_that_lost_requirements_no_removed_delta_names_is_refused (the starling shape) failed on main 3bebe8b0, committed at 5a7b6d68 before the fix. Now 9 unit tests (starling truncation with refused attempt and audit; accounted removals: same change, another active change, an archived change; merged-in vs cherry-picked main removal; move vs renumber with the permanence hint; deleted file and legacy .req.md; base_commit fallback, skip note, unreadable merge-base) and an integration test (ship-status blocker, ship --dry-run, finalize, re-check) pass. Real repos: on CorvidLabs/starling at PR #43's last open commit 3b1dca0 (origin/main at b68d547), released 6.3.0's change check passes (exit 0) and this build refuses (exit 1) naming all 26 lost REQ-StarlingCore ids, the file and the way out; starling main db97480 and Chough main 9d260310 audit clean (exit 0) under both builds; spec-sync's own check on this branch passed. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3085 unit and 628 integration tests (including the git-spawn ceiling test), strict specs 62/62, 100% file and LOC coverage; fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at d8612448 at revision d8612448bca0, observed 2026-11-19T15:09:37Z.
