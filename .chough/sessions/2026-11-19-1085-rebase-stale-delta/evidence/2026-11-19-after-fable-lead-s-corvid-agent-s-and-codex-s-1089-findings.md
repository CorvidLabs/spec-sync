---
kind: evidence
id: 2026-11-19-after-fable-lead-s-corvid-agent-s-and-codex-s-1089-findings
aliases: []
claim: "After fable-lead's, corvid-agent's and Codex's #1089 findings: main's block must be text the default branch held more recently than the approval-time base; rebases are staged and written only when no stale item is left; the merge anchors only on words unique in all three texts; status lists only rebases that recompute; rebases are keyed by approval index, time and digest; the stale refusal names git show origin/main:<file>. New tests (branch-side edit before the first check, after materialization and in a merge resolution; no origin; origin moved back; an older main text; the mixed two-item refusal byte-identical; repeated words; same-second re-approval; status of a non-recomputing record; an end-to-end branch-edit refusal) fail on 2c06cef4 and pass. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3091 unit and 631 integration tests, strict specs 62/62. Re-approved under Leif's relayed 'Approve' of the new REQ-change-161 and REQ-cmd-change-038 wording; change audit --strict and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 5b2dfce4e6edb1404a303d3e9167ee759c806382
content: 199810bc90ac24c1dd0e6bbc01b71dc8f325f4ac41666aaad95699b248ada632
environment: local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 5b2dfce4
criteria: [CHANGE-6.a]
requirements: [REQ-change-161]
contracts: []
limitations: []
observed: 2026-11-19T04:03:31Z
actor:
  - agent: claude-code
    terminal: t4t
---
**After fable-lead's, corvid-agent's and Codex's #1089 findings: main's block must be text the default branch held more recently than the approval-time base; rebases are staged and written only when no stale item is left; the merge anchors only on words unique in all three texts; status lists only rebases that recompute; rebases are keyed by approval index, time and digest; the stale refusal names git show origin/main:<file>. New tests (branch-side edit before the first check, after materialization and in a merge resolution; no origin; origin moved back; an older main text; the mixed two-item refusal byte-identical; repeated words; same-second re-approval; status of a non-recomputing record; an end-to-end branch-edit refusal) fail on 2c06cef4 and pass. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3091 unit and 631 integration tests, strict specs 62/62. Re-approved under Leif's relayed 'Approve' of the new REQ-change-161 and REQ-cmd-change-038 wording; change audit --strict and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 5b2dfce4 at revision 5b2dfce4e6ed, observed 2026-11-19T04:03:31Z.
