---
kind: evidence
id: 2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings
aliases: []
claim: "On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI."
result: passed
evidence_kind: product-test
revision: 97828091ad38ab5bdc7c4ab49f1d9554cfae8be3
content: 2c8e78ca4b990e9ef05fe352e68d6be0504a860db9de22e3267d1f77e5762688
environment: Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 97828091, as a normal user with jq, through one-lane
fingerprint: sha256:4c9675140e3d4ef96b418c3c526f81b146cdd2f972925814080a8396c2ecd10d
external: ext-ee95da
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-006, REQ-util-008, REQ-change-162, REQ-hooks-009]
contracts: []
limitations: []
observed: 2026-11-20T04:12:05Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI.**

Recorded as passed in Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 97828091, as a normal user with jq, through one-lane at revision 97828091ad38, observed 2026-11-20T04:12:05Z.
