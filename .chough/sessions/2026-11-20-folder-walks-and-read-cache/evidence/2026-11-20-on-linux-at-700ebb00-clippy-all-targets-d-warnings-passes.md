---
kind: evidence
id: 2026-11-20-on-linux-at-700ebb00-clippy-all-targets-d-warnings-passes
aliases: []
claim: "On Linux at 700ebb00: clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3135 passed, 0 failed, 2 ignored and integration 706 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI."
result: passed
evidence_kind: product-test
revision: 700ebb00cbf2b9a79a1fd43b008381e5939e1ed1
content: c661cc4880af0229c6efb480fd25ae19f34619e9dba3aa647afdf7c94e5cbb6c
environment: Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 700ebb00, as a normal user with jq, through one-lane
fingerprint: sha256:072ee4148190fbc2da04649bd20e62726c9e17828b684b86aed81e0b2e09234a
external: ext-34029e
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-009, REQ-util-010, REQ-change-164]
contracts: []
limitations: []
observed: 2026-11-20T05:19:42Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at 700ebb00: clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3135 passed, 0 failed, 2 ignored and integration 706 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI.**

Recorded as passed in Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 700ebb00, as a normal user with jq, through one-lane at revision 700ebb00cbf2, observed 2026-11-20T05:19:42Z.
