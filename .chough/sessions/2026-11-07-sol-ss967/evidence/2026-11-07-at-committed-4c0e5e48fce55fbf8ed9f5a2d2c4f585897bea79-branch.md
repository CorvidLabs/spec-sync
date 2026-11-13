---
kind: evidence
id: 2026-11-07-at-committed-4c0e5e48fce55fbf8ed9f5a2d2c4f585897bea79-branch
aliases: [OBS-structural-67bff9125d00d3cb05a0df22]
claim: "At committed 4c0e5e48fce55fbf8ed9f5a2d2c4f585897bea79, branch-built change audit --strict exited 1: cmd_change paths were outside the archived change-only scope. Pre-push and Trust passed separately. This failure blocks push until corrected and rechecked."
result: failed
evidence_kind: structural
method: automated
revision: 4c0e5e48fce55fbf8ed9f5a2d2c4f585897bea79
content: 611014ce6ac352c000943f818740fdd91710ef7e93ce50402e21a723372ca2d1
environment: local-macos-arm64
fingerprint: sha256:a7a5aec33ab817f0c58ffe242c7776025eaf10a32f06c987c3dcef4b2a6247bd
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
contracts: [specs/change/change.spec.md]
limitations:
  - "Retrospective import: executed input is the full SHA in the summary and receipt; observation automatic identity describes the recording tree."
observed: 2026-11-07T07:09:21Z
---
**At committed 4c0e5e48fce55fbf8ed9f5a2d2c4f585897bea79, branch-built change audit --strict exited 1: cmd_change paths were outside the archived change-only scope. Pre-push and Trust passed separately. This failure blocks push until corrected and rechecked.**

Recorded as failed in local-macos-arm64 at revision 4c0e5e48fce5, observed 2026-11-07T07:09:21Z.
