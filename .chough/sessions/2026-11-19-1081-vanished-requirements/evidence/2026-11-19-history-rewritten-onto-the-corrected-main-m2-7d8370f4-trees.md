---
kind: evidence
id: 2026-11-19-history-rewritten-onto-the-corrected-main-m2-7d8370f4-trees
aliases: []
claim: "History rewritten onto the corrected main (M2' 7d8370f4); trees unchanged. It also folds in the two follow-ups made while commits were held: the tester's L2b shape pinned (cherry-pick -x, change created after it), and the merge named as the way out when the default branch removed a lost id too (failing first, then the fix). change check --commit re-recorded verification on the new history (65638bbe, 2d41147d). fledge lanes run verify passed at 2d41147d through one-lane (3108 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 2d41147d2863e8c1a6a808343cd87a17ef4502a9
content: 09cfe30c386d4ee096b3004fbf019bfafe28258322c94c6e7c057f666c0ed38a
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 2d41147d
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-19T20:05:56Z
actor:
  - agent: claude-code
    terminal: t4t
---
**History rewritten onto the corrected main (M2' 7d8370f4); trees unchanged. It also folds in the two follow-ups made while commits were held: the tester's L2b shape pinned (cherry-pick -x, change created after it), and the merge named as the way out when the default branch removed a lost id too (failing first, then the fix). change check --commit re-recorded verification on the new history (65638bbe, 2d41147d). fledge lanes run verify passed at 2d41147d through one-lane (3108 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 2d41147d at revision 2d41147d2863, observed 2026-11-19T20:05:56Z.
