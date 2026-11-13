---
kind: evidence
id: 2026-11-07-at-test-only-commit-23fffcedee001dac6ede47e89ffe912b819b4552
aliases: [OBS-product-test-2d6da6e25125f8c0fc3cf3a8]
claim: "At test-only commit 23fffcedee001dac6ede47e89ffe912b819b4552 the complete printed recovery failed at status: selected project root was lost, change not found; t19r exit 1"
result: failed
evidence_kind: product-test
method: automated
revision: 23fffcedee001dac6ede47e89ffe912b819b4552
content: 20b7917892b88fa7646d336f3c2f4172345c14ca0d6aa7031082a7300c9965ca
environment: local-macos-arm64
fingerprint: sha256:f67a7520140a2784fd10a6b1b44f77508528d6e78a762e49ac9f2d223a055327
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
contracts: [specs/change/change.spec.md]
limitations:
  - "Retrospective receipt import: the observation revision/content identity describes the recording tree; the executed red subject is the exact SHA in the summary and artifact"
observed: 2026-11-07T06:50:48Z
---
**At test-only commit 23fffcedee001dac6ede47e89ffe912b819b4552 the complete printed recovery failed at status: selected project root was lost, change not found; t19r exit 1.**

Recorded as failed in local-macos-arm64 at revision 23fffcedee00, observed 2026-11-07T06:50:48Z.
