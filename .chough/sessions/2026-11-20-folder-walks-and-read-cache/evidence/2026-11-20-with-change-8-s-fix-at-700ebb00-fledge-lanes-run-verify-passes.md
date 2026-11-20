---
kind: evidence
id: 2026-11-20-with-change-8-s-fix-at-700ebb00-fledge-lanes-run-verify-passes
aliases: []
claim: "With change 8's fix at 700ebb00, fledge lanes run verify passes all 10 steps: unit 3131 passed, 0 failed, 2 ignored; integration 703 passed, 0 failed; clippy, the release build, the strict spec check at 62 specs and 100% coverage, the gate scripts. All 6 new unit tests and the scaffold CLI test pass; configured folders that lead out are refused, not walked, and the read cache is in the user cache folder."
result: passed
evidence_kind: product-test
revision: 700ebb00cbf2b9a79a1fd43b008381e5939e1ed1
content: c661cc4880af0229c6efb480fd25ae19f34619e9dba3aa647afdf7c94e5cbb6c
environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 700ebb00
fingerprint: sha256:1b5257703bc4a049de8feb0a030825eee4b285511041f56539bb2da07d96d951
external: ext-138ea2
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-009, REQ-util-010, REQ-change-164]
contracts: []
limitations: []
observed: 2026-11-20T05:19:40Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With change 8's fix at 700ebb00, fledge lanes run verify passes all 10 steps: unit 3131 passed, 0 failed, 2 ignored; integration 703 passed, 0 failed; clippy, the release build, the strict spec check at 62 specs and 100% coverage, the gate scripts. All 6 new unit tests and the scaffold CLI test pass; configured folders that lead out are refused, not walked, and the read cache is in the user cache folder.**

Recorded as passed in macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 700ebb00 at revision 700ebb00cbf2, observed 2026-11-20T05:19:40Z.
