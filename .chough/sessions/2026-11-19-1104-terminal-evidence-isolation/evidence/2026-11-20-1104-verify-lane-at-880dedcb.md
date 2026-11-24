---
kind: evidence
id: 2026-11-20-1104-verify-lane-at-880dedcb
aliases: []
claim: "At 880dedcb (main a28e3042 merged in, sent back, reworded criterion 2 approved, checked and committed), with this tree's own release build (specsync 6.4.0) first on PATH: fledge lanes run verify passed 10/10 steps in 9m37s through one-lane (unit 3106 passed, 2 ignored, including #1104's six unit tests and the shared-constant test; integration 645, including the binary test's three audit assertions: plain change audit exits 1 with the error on a change open and archived at once and passes on S1, change audit --archives exits 1 naming it on S1; spec check 62/62 specs, 168/168 files), the neutral git config as a prefix on the lane only (#1092). specsync change audit --strict against origin/main a28e3042 passes."
result: passed
evidence_kind: product-test
revision: 880dedcbf94ccd8d8e88ae9d786d42b38c45044c
content: 5cafb6cceb18a3f12164612296a31f68ffcd279cf1dff0b68ef5ad8f45bdbc6f
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:f597adda7f4b2107d17b3c0d6c96832aff0a2e5cce01be4346d58a901d61e931
external: ext-8cfbba
criteria: [CHANGE-7, CHANGE-8]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T03:38:40Z
actor:
  - agent: claude-code
    terminal: t59
---
**At 880dedcb (main a28e3042 merged in, sent back, reworded criterion 2 approved, checked and committed), with this tree's own release build (specsync 6.4.0) first on PATH: fledge lanes run verify passed 10/10 steps in 9m37s through one-lane (unit 3106 passed, 2 ignored, including #1104's six unit tests and the shared-constant test; integration 645, including the binary test's three audit assertions: plain change audit exits 1 with the error on a change open and archived at once and passes on S1, change audit --archives exits 1 naming it on S1; spec check 62/62 specs, 168/168 files), the neutral git config as a prefix on the lane only (#1092). specsync change audit --strict against origin/main a28e3042 passes.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 880dedcbf94c, observed 2026-11-20T03:38:40Z.
