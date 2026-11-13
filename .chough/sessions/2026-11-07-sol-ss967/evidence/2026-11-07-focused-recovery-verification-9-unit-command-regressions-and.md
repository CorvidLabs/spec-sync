---
kind: evidence
id: 2026-11-07-focused-recovery-verification-9-unit-command-regressions-and
aliases: [OBS-product-test-efd35a9f612ba24bb9c3ae66]
claim: "Focused recovery verification: 9 unit/command regressions and all 4 archived-upstream binary integration tests pass after retaining --root. Executes Git repair and final status from the project parent with quotes/spaces, checks archive/rebase, unchanged verifying state, no second archive. Shell argument test includes single/double quotes and literal command substitution. t1b2 exit 0."
result: passed
evidence_kind: product-test
method: automated
revision: 23fffcedee001dac6ede47e89ffe912b819b4552
content: 20b7917892b88fa7646d336f3c2f4172345c14ca0d6aa7031082a7300c9965ca
environment: local-macos-arm64
fingerprint: sha256:ecd94138afa4ef515b091175b730124f27cf44319b8add537474deea9fd4e186
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
contracts: [specs/change/change.spec.md]
limitations:
  - Focused run was on the uncommitted fix over red commit 23fffcedee001dac6ede47e89ffe912b819b4552; final committed full lane and hosted CI remain pending
observed: 2026-11-07T06:51:12Z
---
**Focused recovery verification: 9 unit/command regressions and all 4 archived-upstream binary integration tests pass after retaining --root. Executes Git repair and final status from the project parent with quotes/spaces, checks archive/rebase, unchanged verifying state, no second archive. Shell argument test includes single/double quotes and literal command substitution. t1b2 exit 0.**

Recorded as passed in local-macos-arm64 at revision 23fffcedee00, observed 2026-11-07T06:51:12Z.
