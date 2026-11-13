---
kind: evidence
id: 2026-11-07-full-fledge-verify-passed-at-frozen-clean
aliases: [OBS-product-test-e75d3369395637c959915dd6]
claim: "Full fledge verify passed at frozen clean 995694aba251659a702bd9fd0d295849c4e67a0e: 2908 unit and 527 integration passes, 2 ignored benchmarks, all 8 steps in 18m58.599s. Exact start/end SHA and empty manifests; complete log retained with checksum in the PR #999 durable qualification receipts."
result: passed
evidence_kind: product-test
method: automated
revision: 995694aba251659a702bd9fd0d295849c4e67a0e
content: 979aee8f46cea64acfbfa4c5c945412538e4fcbb574a655b1685f0566c8d0578
environment: local
criteria: [SHIP-4.a, CHANGE-10]
requirements: [REQ-change-053, REQ-change-120, REQ-cmd-change-008]
contracts: [specs/change/change.spec.md, specs/cmd_change/cmd_change.spec.md]
limitations:
  - Two pre-existing benchmark tests remain ignored. Hosted full exact-head CI and final Trust qualification are still pending. This observation records an actual local run and conveys no human approval.
observed: 2026-11-07T08:21:51Z
---
**Full fledge verify passed at frozen clean 995694aba251659a702bd9fd0d295849c4e67a0e: 2908 unit and 527 integration passes, 2 ignored benchmarks, all 8 steps in 18m58.599s. Exact start/end SHA and empty manifests; complete log retained with checksum in the PR #999 durable qualification receipts.**

Recorded as passed in local at revision 995694aba251, observed 2026-11-07T08:21:51Z.
