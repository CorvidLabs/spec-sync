---
kind: evidence
id: 2026-11-19-1104-verify-lane-at-b3c2cc0b
aliases: []
claim: "At b3c2cc0b (checked and committed), fledge lanes run verify passed 10/10 steps in 11m42s through one-lane: unit 3099 passed (2 ignored), integration 630, spec check 166/166 files. Includes the S1 integration test through the binary (status, change list, ship-status, audit --archives, a third change shipped). The neutral git config was a prefix on the lane command only (#1092)."
result: passed
evidence_kind: product-test
revision: 38b2dbfb2ffd1cc5db7ce186f15041c29be1601a
content: 4536f5d9e339c8c46fd9a45f49e7b89839ccc5fc3b324fbc5cba75dad01f9d5e
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:c270e62ec6fabd5d1cf7335da4c4188e3b911529d4c39eb4dadff17bf5d71561
external: ext-73b29b
criteria: [CHANGE-7, CHANGE-8]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T20:15:39Z
actor:
  - agent: claude-code
    terminal: t59
---
**At b3c2cc0b (checked and committed), fledge lanes run verify passed 10/10 steps in 11m42s through one-lane: unit 3099 passed (2 ignored), integration 630, spec check 166/166 files. Includes the S1 integration test through the binary (status, change list, ship-status, audit --archives, a third change shipped). The neutral git config was a prefix on the lane command only (#1092).**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 38b2dbfb2ffd, observed 2026-11-19T20:15:39Z.
