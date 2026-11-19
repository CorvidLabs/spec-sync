---
kind: evidence
id: 2026-11-19-red-control-weaker-than-the-others-as-on-11-13-on-main
aliases: []
claim: "Red control, weaker than the others, as on 11-13: on main 3bebe8b0 the failing-test commit 665f8a22 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers calls Arc::ptr_eq on the seed's facts, which main holds as a plain HashMap. No behavioural failure on the old code was demonstrated; the test pins the new sharing."
result: not-run
evidence_kind: product-test
revision: fde6e01b84481563d8d376aa9f40456a4a63cd76
content: 27b34fb76ccbc56a5108d4091dff7436c9e922f887facc0c9ed4510f2471451f
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76)"
criteria: [SPEED-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T09:01:11Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red control, weaker than the others, as on 11-13: on main 3bebe8b0 the failing-test commit 665f8a22 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers calls Arc::ptr_eq on the seed's facts, which main holds as a plain HashMap. No behavioural failure on the old code was demonstrated; the test pins the new sharing.**

Recorded as not-run in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (cb1588c1, a8c7d76e, 665f8a22, 64340c76) at revision fde6e01b8448, observed 2026-11-19T09:01:11Z.

## Key failing lines

- error[E0308]: Arc::ptr_eq expected &Arc, found &HashMap (src/git_utils.rs:1815)
