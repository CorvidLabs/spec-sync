---
kind: evidence
id: 2026-11-20-428-migrate-regression-at-c7d16a2b
aliases: []
claim: "tester2's regression on #1120 (P-428-guard-migrate-digests): red first at 768e71b6 (a reopening committed without the migrate 5.0 digests was refused as not a ledger); fixed, each committed version must be an object whose approvals, reopenings and send_backs are lists; checked at c7d16a2b. fledge lanes run verify 10/10 in 7m43s through one-lane (unit 3127 passed, 2 ignored, committed_approvals 27 of 27; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes."
result: passed
evidence_kind: product-test
revision: c7d16a2bb7565e04998dbb9753d3cacd3542f6f4
content: 0e26ffc723eeea01f6086cdd5ca592079771142646e82a940d3f81304e1dfc76
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:5e1b8dca9692759a642bf6864b8533908d52119715cfe02f7684f4fe3665ce23
external: ext-c5c8f7
criteria: [CHANGE-16, CHANGE-16.c]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T07:43:29Z
actor:
  - agent: claude-code
    terminal: t59
---
**tester2's regression on #1120 (P-428-guard-migrate-digests): red first at 768e71b6 (a reopening committed without the migrate 5.0 digests was refused as not a ledger); fixed, each committed version must be an object whose approvals, reopenings and send_backs are lists; checked at c7d16a2b. fledge lanes run verify 10/10 in 7m43s through one-lane (unit 3127 passed, 2 ignored, committed_approvals 27 of 27; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c7d16a2bb756, observed 2026-11-20T07:43:29Z.
