---
kind: evidence
id: 2026-11-19-428-verify-lane-at-d8ad737f
aliases: []
claim: "At d8ad737f (main 2cc0897f merged in; the fix, the rule tests and the existing-test commits), fledge lanes run verify passed 10/10 steps in 8m29s, run alone: unit 3110 passed (2 ignored), integration 630 passed (ship-status exit 1 on the ledger blocker included), strict specs 62/62. The neutral git config was a prefix on the lane command only (#1092)"
result: passed
evidence_kind: product-test
revision: b70c28033539420c937ddb44d64a12ddcb35b63d
content: 7570fb4cbbe75d814fbc3157c3728c0a93f0093c8e7f59d4fdfa22714208eb0d
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:a1e3ec68b6f785c6072f70b958d571f6bd29486c779fa2879318f80abf77bd11
external: ext-5e3d0b
criteria: [CHANGE-16]
requirements: [REQ-change-166]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T17:15:03Z
actor:
  - agent: claude-code
    terminal: t59
---
**At d8ad737f (main 2cc0897f merged in; the fix, the rule tests and the existing-test commits), fledge lanes run verify passed 10/10 steps in 8m29s, run alone: unit 3110 passed (2 ignored), integration 630 passed (ship-status exit 1 on the ledger blocker included), strict specs 62/62. The neutral git config was a prefix on the lane command only (#1092).**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision b70c28033539, observed 2026-11-19T17:15:03Z.
