---
kind: evidence
id: 2026-11-19-red-first-40f19ea4-s-tests-failed-on-main-m2-7d8370f4-the
aliases: []
claim: "Red first: 40f19ea4's tests failed on main (M2' 7d8370f4): the audit reported nothing for an active change whose approved delta targets the frozen Change Log, and reported the stale base instead of the Change Log item for a change with both; the archive case is pinned. Fix 134325a0: the audit asks refuse_change_log_delta_items where change check asks it. All 24 tests in stale_modified_delta.rs pass. Real repositories, change audit --strict, 6.3.0 against this build: Chough main, starling (PR #43's last commit), spec-sync main, Chough PRs #311 and #310, a blobless peck clone (67 changes) and NFTMix (with the #1083 probe edits) give identical error lines, none about the Change Log; no new refusal. fledge lanes run verify passed at b4c925f1 through one-lane (3097 unit, 629 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: b4c925f115139587a1f8680f23ac378889bde2f5
content: 6c739c2ec7b023027aa8afab1d7f95ed77aca832961d294a70470ba07ce3ccdf
environment: local macOS 26, rust 1.89.0, branch 1090-audit-change-log-delta at b4c925f1
criteria: [CHANGE-11.b]
requirements: [REQ-change-170]
contracts: []
limitations: []
observed: 2026-11-19T20:46:38Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red first: 40f19ea4's tests failed on main (M2' 7d8370f4): the audit reported nothing for an active change whose approved delta targets the frozen Change Log, and reported the stale base instead of the Change Log item for a change with both; the archive case is pinned. Fix 134325a0: the audit asks refuse_change_log_delta_items where change check asks it. All 24 tests in stale_modified_delta.rs pass. Real repositories, change audit --strict, 6.3.0 against this build: Chough main, starling (PR #43's last commit), spec-sync main, Chough PRs #311 and #310, a blobless peck clone (67 changes) and NFTMix (with the #1083 probe edits) give identical error lines, none about the Change Log; no new refusal. fledge lanes run verify passed at b4c925f1 through one-lane (3097 unit, 629 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1090-audit-change-log-delta at b4c925f1 at revision b4c925f11513, observed 2026-11-19T20:46:38Z.
