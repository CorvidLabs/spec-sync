---
kind: evidence
id: 2026-11-19-on-linux-at-e60fbf8e-change-5-a-clean-copy-in-a-local-rust-1
aliases: []
claim: "On Linux at e60fbf8e (change 5), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3120 passed, 0 failed, 2 ignored and the integration suite at 692 passed, 0 failed."
result: passed
evidence_kind: product-test
method: local rust:1.89-bookworm container through one-lane, a clean copy of e60fbf8e (plus three uncommitted Chough record files, no code), run as a normal user with jq installed
revision: e60fbf8e02faa643254590c1b4b5f5a659aa54d9
content: dd77399afb5305b4ad7830b7c1e438ea2c983073c6b5c5fe56d57ae0c641d405
environment: local-linux-arm64
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
contracts: []
limitations:
  - arm64, not x86_64; Debian 12, not the Ubuntu image CI uses; the private fork runs no hosted CI
observed: 2026-11-19T20:38:01Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at e60fbf8e (change 5), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3120 passed, 0 failed, 2 ignored and the integration suite at 692 passed, 0 failed.**

Recorded as passed in local-linux-arm64 at revision e60fbf8e02fa, observed 2026-11-19T20:38:01Z.
