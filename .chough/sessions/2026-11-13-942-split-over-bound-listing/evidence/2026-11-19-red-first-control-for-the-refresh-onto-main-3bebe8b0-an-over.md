---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-an-over
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: an_over_bound_archive_introduction_listing_is_split_not_read_per_commit fails ('a split listing equals the whole one': left None), the defect #942 reports."
result: failed
evidence_kind: product-test
revision: fde6e01b84481563d8d376aa9f40456a4a63cd76
content: 27b34fb76ccbc56a5108d4091dff7436c9e922f887facc0c9ed4510f2471451f
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76)"
criteria: [SPEED-1.a.1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T09:01:10Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: an_over_bound_archive_introduction_listing_is_split_not_read_per_commit fails ('a split listing equals the whole one': left None), the defect #942 reports.**

Recorded as failed in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76) at revision fde6e01b8448, observed 2026-11-19T09:01:10Z.

## Key failing lines

- a split listing equals the whole one; left: None
