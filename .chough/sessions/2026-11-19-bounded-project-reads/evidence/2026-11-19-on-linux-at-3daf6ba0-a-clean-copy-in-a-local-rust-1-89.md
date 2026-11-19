---
kind: evidence
id: 2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89
aliases: []
claim: "On Linux at 3daf6ba0, a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3102 passed, 0 failed, 2 ignored and the integration suite at 668 passed, 0 failed."
result: passed
evidence_kind: product-test
method: docker run rust:1.89-bookworm, source mounted read-only and copied to a working directory, non-root user, rustc 1.89.0
revision: 3daf6ba003c3ba4e9aef64c4a9d5767fb5bb0b5b
content: 8919ddb2458082958a65bf2d700905bea3cf3ecc6af23487346d667aa41bd582
environment: local-linux-arm64-container
criteria: [SETUP-7]
requirements: []
contracts: []
limitations:
  - arm64 on Debian 12, not x86_64 and not the Ubuntu image CI uses; no hosted CI because the private fork runs no Actions.
observed: 2026-11-19T15:21:03Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at 3daf6ba0, a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3102 passed, 0 failed, 2 ignored and the integration suite at 668 passed, 0 failed.**

Recorded as passed in local-linux-arm64-container at revision 3daf6ba003c3, observed 2026-11-19T15:21:03Z.
