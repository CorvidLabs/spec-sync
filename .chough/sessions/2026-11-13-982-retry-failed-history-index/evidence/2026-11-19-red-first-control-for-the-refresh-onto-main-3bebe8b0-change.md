---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-change
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: change::tests::a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope fails (a failed index build stayed failed for later scopes), the defect #982 reports."
result: failed
evidence_kind: product-test
revision: 42dda911e039dd4b032fff5a278f1fc2eee391cb
content: 37d3903c9d3468a39a7fcfcac1aac4721a47c2b57679a3de8bad06b2e6b4f189
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b)"
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:44:28Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: change::tests::a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope fails (a failed index build stayed failed for later scopes), the defect #982 reports.**

Recorded as failed in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b) at revision 42dda911e039, observed 2026-11-19T08:44:28Z.

## Key failing lines

- a_failed_lifecycle_state_history_build_is_retried_in_the_next_read_scope FAILED
