---
kind: evidence
id: 2026-11-07-test-first-recovery-refinement-on
aliases: [OBS-product-test-d753cf8983ee0af03f0458f9]
claim: "Test-first recovery refinement on 082f4c7377565db465b6ffba64991182d6958c62: the write-boundary matrix fails because its final-component refusal says to create an in-project directory, which would block retry. The tightened test requires leaving the dated destination absent, then executes that retry for real and dangling final links."
result: failed
evidence_kind: product-test
revision: 082f4c7377565db465b6ffba64991182d6958c62
content: d7f5e8209916152a3fa751103616c7e518c555355bb66c03959c598babe43cd6
environment: local-macos-arm64
fingerprint: sha256:bd90d9880244aadcd1cec3309f42624c4cc493abf16cb01f52992ddc96f25363
criteria: [SHIP-12]
requirements: [REQ-change-143]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-07T06:57:22Z
---
**Test-first recovery refinement on 082f4c7377565db465b6ffba64991182d6958c62: the write-boundary matrix fails because its final-component refusal says to create an in-project directory, which would block retry. The tightened test requires leaving the dated destination absent, then executes that retry for real and dangling final links.**

Recorded as failed in local-macos-arm64 at revision 082f4c737756, observed 2026-11-07T06:57:22Z.
