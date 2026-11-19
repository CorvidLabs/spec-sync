---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-change
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: change_correct_costs_less_than_one_unscoped_summary fails, 'change correct spawned 696 Git processes, more than the 641 of one summary read without a scope', the defect #921 reports."
result: failed
evidence_kind: product-test
revision: fde6e01b84481563d8d376aa9f40456a4a63cd76
content: 27b34fb76ccbc56a5108d4091dff7436c9e922f887facc0c9ed4510f2471451f
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76)"
criteria: [SPEED-1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T09:01:09Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: change_correct_costs_less_than_one_unscoped_summary fails, 'change correct spawned 696 Git processes, more than the 641 of one summary read without a scope', the defect #921 reports.**

Recorded as failed in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76) at revision fde6e01b8448, observed 2026-11-19T09:01:09Z.

## Key failing lines

- change correct spawned 696 Git processes, more than the 641 of one summary read without a scope
