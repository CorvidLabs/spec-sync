---
kind: evidence
id: 2026-11-19-428-red-integration-on-main
aliases: []
claim: "Red first on main: finalize_ship_and_ship_status_refuse_a_ledger_the_working_tree_changed fails: ship-status calls a change ready to finalize after its approvals.json was edited in the working tree."
result: failed
evidence_kind: product-test
revision: b70c28033539420c937ddb44d64a12ddcb35b63d
content: 7570fb4cbbe75d814fbc3157c3728c0a93f0093c8e7f59d4fdfa22714208eb0d
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:07622771d5c8695f46eddfbd5a0e84d76152fa68a73b76ad80c99d46f1471b5a
external: ext-d12943
criteria: [CHANGE-16.a]
requirements: [REQ-cmd-change-041]
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T17:15:03Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first on main: finalize_ship_and_ship_status_refuse_a_ledger_the_working_tree_changed fails: ship-status calls a change ready to finalize after its approvals.json was edited in the working tree.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision b70c28033539, observed 2026-11-19T17:15:03Z.
