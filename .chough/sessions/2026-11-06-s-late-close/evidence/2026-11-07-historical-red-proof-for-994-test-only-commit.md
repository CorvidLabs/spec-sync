---
kind: evidence
id: 2026-11-07-historical-red-proof-for-994-test-only-commit
aliases: [OBS-product-test-8a9f5a47598253cb2208e85c]
claim: "Historical red proof for #994: test-only commit 9532a256b4b2e6c39fee88e39ebb212bb3125e18 on ba934e74ef8997fea35ca6e1f3987b10e7c68379 implementation; write-boundary unit failed and all four CLI regressions failed. Linked ancestor/leaf destinations move source packages outside; dangling links pass dry-run."
result: failed
evidence_kind: product-test
revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
content: 0077e9b4ea007f052780fd94a72ffb328f842dac0bcaa7ea970301d71212dc9a
environment: local-macos-arm64
fingerprint: sha256:543adef8c4df0f0c645d9a66e2d01d8cb1883f591e337ffe43ee54e6fe00da1d
criteria: [SHIP-12]
requirements: [REQ-change-143]
contracts: [specs/change/change.spec.md]
limitations:
  - Receipt imported after source repair; tested input is the named red commit, not the current working content identity.
observed: 2026-11-07T06:51:18Z
---
**Historical red proof for #994: test-only commit 9532a256b4b2e6c39fee88e39ebb212bb3125e18 on ba934e74ef8997fea35ca6e1f3987b10e7c68379 implementation; write-boundary unit failed and all four CLI regressions failed. Linked ancestor/leaf destinations move source packages outside; dangling links pass dry-run.**

Recorded as failed in local-macos-arm64 at revision 9532a256b4b2, observed 2026-11-07T06:51:18Z.
