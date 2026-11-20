---
kind: evidence
id: 2026-11-20-640-release-validators-at-6-4-0
aliases: []
claim: "At 68b76f27 and its checked head 7ff79527, the three release validators pass at Cargo 6.4.0: validate-release-version.py prints 'Validated release version 6.4.0 across current distribution surfaces', validate-workflow-runtime-pins.py passes, and test-validate-release-candidate.py runs 67 tests OK (neutral git config as a prefix only, #1092). The action.yml default, PUBLISHED_ACTION_DEFAULT and the docs default row stay at published 6.3.0, as the validator requires."
result: passed
evidence_kind: product-test
revision: 7ff79527093faf02d6d9d9370c673502f56e8501
content: 016054af00bb40e433cbc1ee4239b79fe0d437d5b8f2b4fd5bd7ed00a25b44df
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:1075b62286ac2269a85804ae67d85141b479604df0955184a3eab91c18f246cd
external: ext-2b260c
criteria: [CI-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T01:39:40Z
actor:
  - agent: claude-code
    terminal: t59
---
**At 68b76f27 and its checked head 7ff79527, the three release validators pass at Cargo 6.4.0: validate-release-version.py prints 'Validated release version 6.4.0 across current distribution surfaces', validate-workflow-runtime-pins.py passes, and test-validate-release-candidate.py runs 67 tests OK (neutral git config as a prefix only, #1092). The action.yml default, PUBLISHED_ACTION_DEFAULT and the docs default row stay at published 6.3.0, as the validator requires.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 7ff79527093f, observed 2026-11-20T01:39:40Z.
