---
kind: evidence
id: 2026-11-20-428-review-round-1-at-a0b997db
aliases: []
claim: "#1120's first review round (Codex's five, corvid-agent's review) and tester2's way-out finding: red first at b00511c5 (5 of 5 fail); fixed at 5f57f347 and 41fa601f; checked at a0b997db. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 9m03s through one-lane (unit 3126 passed, 2 ignored, committed_approvals 26 of 26; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes."
result: passed
evidence_kind: product-test
revision: a0b997db0a467a51cbd0180430a135c2080fa261
content: a6bc33814aebecbc92cfedb28596242069548008df8fe8a844d5ad80d7139967
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:bb7b8b05b0fa4cf45c457ab618eb69d78c82077e45cbd9878336ceee29252db2
external: ext-7e1979
criteria: [CHANGE-16, CHANGE-16.a, CHANGE-16.c]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T06:38:40Z
actor:
  - agent: claude-code
    terminal: t59
---
**#1120's first review round (Codex's five, corvid-agent's review) and tester2's way-out finding: red first at b00511c5 (5 of 5 fail); fixed at 5f57f347 and 41fa601f; checked at a0b997db. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 9m03s through one-lane (unit 3126 passed, 2 ignored, committed_approvals 26 of 26; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision a0b997db0a46, observed 2026-11-20T06:38:40Z.
