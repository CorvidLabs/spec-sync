---
kind: evidence
id: 2026-11-20-1b-red-through-the-binary
aliases: []
claim: "Through the binary, on the code before the fix (the fix set aside, the tests of db652292): #922 told to reopen after unrelated drift and with an unreadable archived verification.json; #933 a finalization stopped before the archive move (its destination pre-made) not ready. The #938 output guard passes before and after. cargo test --test integration through one-lane."
result: failed
evidence_kind: product-test
revision: fd3581a813e40d0c30b41680472573f199585809
content: 109e14be19f7fac5e45d2c23b481f1fb8c4b84daa7725404fa2c8e12a9b8996a
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:8d3aadfdf057d65835f091e3e6ac5f2c8c8a6975a412533f111d122ad15544bd
external: ext-1e0937
criteria: [SHIP-4.a, SHIP-5, SHIP-9.a]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-20T13:11:32Z
actor:
  - agent: claude-code
    terminal: t59
---
**Through the binary, on the code before the fix (the fix set aside, the tests of db652292): #922 told to reopen after unrelated drift and with an unreadable archived verification.json; #933 a finalization stopped before the archive move (its destination pre-made) not ready. The #938 output guard passes before and after. cargo test --test integration through one-lane.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision fd3581a813e4, observed 2026-11-20T13:11:32Z.
