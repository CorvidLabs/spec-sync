---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-git-utils
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is fails (a failed rev-parse was memoized), the defect #887 reports. Spawn count for eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci by the 11-13 PATH shim: 6,400 on main."
result: failed
evidence_kind: product-test
revision: 42dda911e039dd4b032fff5a278f1fc2eee391cb
content: 37d3903c9d3468a39a7fcfcac1aac4721a47c2b57679a3de8bad06b2e6b4f189
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b)"
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:44:25Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is fails (a failed rev-parse was memoized), the defect #887 reports. Spawn count for eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci by the 11-13 PATH shim: 6,400 on main.**

Recorded as failed in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b) at revision 42dda911e039, observed 2026-11-19T08:44:25Z.

## Key failing lines

- a_failed_fact_is_not_memoized_but_a_definite_absence_is FAILED
