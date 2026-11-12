---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: testing
---

# Testing

| Check | Expected |
|---|---|
| Relayed approve on a v2 change | `state.json` is at version 3 with origin 2, and approval and digest are as before |
| Relayed review on a v2 change after `check --commit` | `state.json` at 3; the earlier verification still binds; `ship` archives with version 3 in `state.json` and `accepted-state.json` |
| Approve and review without a relay | version stays 2, bytes as today |
| Relay on a workflow-v1 change | refused, nothing written |
| History: 2 to 3 | accepted between commits and between `HEAD` and the working tree |
| History: 3 to 2, origin changed, (3, None) | refused |
| A rewrite of a v2 change whose ledgers hold a relay | raises it to 3 |
| Version 4 | still "written by a newer SpecSync" |
| `test-relay-gate-compat.sh` | real 6.2.0 (sha256-pinned) refuses the gated change's lifecycle commands with the newer-SpecSync message and leaves `.specsync/` byte-identical; after archive, `change new` and `change audit` refuse the same way and `check --strict` passes. Negative control: the same records without the raise let 6.2.0 rewrite them and drop `relayed_by` |

Locally: the targeted unit and integration tests, the compat script, fmt, and clippy on the touched code. The full suite runs in hosted CI.
