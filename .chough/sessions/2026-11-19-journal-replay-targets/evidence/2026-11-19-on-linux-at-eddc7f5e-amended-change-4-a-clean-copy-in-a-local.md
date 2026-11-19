---
kind: evidence
id: 2026-11-19-on-linux-at-eddc7f5e-amended-change-4-a-clean-copy-in-a-local
aliases: []
claim: "On Linux at eddc7f5e (amended change 4), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3110 passed, 0 failed, 2 ignored and the integration suite at 669 passed, 0 failed."
result: passed
evidence_kind: product-test
method: docker run rust:1.89-bookworm, source mounted read-only and copied, non-root user, rustc 1.89.0
revision: eddc7f5efa074c41abc452d09ec8c4dcc2fd6811
content: 20744325f75306e5ec7793e16af2ddbec0d9746bbf22d486aa48c9c37cab24e0
environment: local-linux-arm64-container
criteria: [SETUP-8]
requirements: []
contracts: []
limitations:
  - The copy carried one uncommitted Chough session record, which holds no code. arm64 on Debian 12, not x86_64 or the CI Ubuntu image; no hosted CI.
observed: 2026-11-19T17:04:35Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at eddc7f5e (amended change 4), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3110 passed, 0 failed, 2 ignored and the integration suite at 669 passed, 0 failed.**

Recorded as passed in local-linux-arm64-container at revision eddc7f5efa07, observed 2026-11-19T17:04:35Z.
