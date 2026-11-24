---
kind: evidence
id: 2026-11-19-428-red-ways-out-at-4ebc7bea
aliases: []
claim: "Red first for tester2's interim findings at 999f55a3: at 4ebc7bea (the tests alone) three fail: an ignored ledger is told to run check --commit, which cannot commit it; a re-created ledger's way out names only the re-creation; a missing working ledger gives finalize a bare 'No such file or directory'. Fixed in 88cb2857."
result: failed
evidence_kind: product-test
revision: e869bf117949ca8f1454eb07832899f24b77940a
content: a12ae27cab638f1d03d70dd440b745d1884aa774fd7dd1cf38c56f9f78c42bd3
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:a10d974d3ee5c059f4abadedaf042930f0eae50b3379e20b44261a09502ce355
external: ext-4f462d
criteria: [CHANGE-16.a]
requirements: [REQ-change-166]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T19:43:08Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first for tester2's interim findings at 999f55a3: at 4ebc7bea (the tests alone) three fail: an ignored ledger is told to run check --commit, which cannot commit it; a re-created ledger's way out names only the re-creation; a missing working ledger gives finalize a bare 'No such file or directory'. Fixed in 88cb2857.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision e869bf117949, observed 2026-11-19T19:43:08Z.
