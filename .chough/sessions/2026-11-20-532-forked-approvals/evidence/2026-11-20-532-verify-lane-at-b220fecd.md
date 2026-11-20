---
kind: evidence
id: 2026-11-20-532-verify-lane-at-b220fecd
aliases: []
claim: "At b220fecd: fledge lanes run verify 10/10 in 13m04s through one-lane (unit 3130, 2 ignored; integration 648; spec check 62/62, 169/169 files). The first lane at 0701ff5c failed #428's a_rewrite_on_a_branch_that_was_merged_in_is_refused_at_the_merge_commit; fixed at 45a4d289 by leaving a divergence that reached first-parent history to #428's rule (approved choice 4). change audit --strict fails only because #428's change is still active on this stacked branch (its evidence reads stale); that clears when #1120 merges and archives it."
result: passed
evidence_kind: product-test
revision: b220fecd3a50161f7fb44ebdf71c510feb05732e
content: a6d5de1311e931a220d14019971ea4a2e8fb58919c9502cb148e887a009c9870
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:c70c77671b12d938b303b462084d7d2fc26bb8d08c22466014ca38866e714bc8
external: ext-3c785c
criteria: [CHANGE-6, CHANGE-6.a]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T16:07:58Z
actor:
  - agent: claude-code
    terminal: t59
---
**At b220fecd: fledge lanes run verify 10/10 in 13m04s through one-lane (unit 3130, 2 ignored; integration 648; spec check 62/62, 169/169 files). The first lane at 0701ff5c failed #428's a_rewrite_on_a_branch_that_was_merged_in_is_refused_at_the_merge_commit; fixed at 45a4d289 by leaving a divergence that reached first-parent history to #428's rule (approved choice 4). change audit --strict fails only because #428's change is still active on this stacked branch (its evidence reads stale); that clears when #1120 merges and archives it.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision b220fecd3a50, observed 2026-11-20T16:07:58Z.
