---
kind: evidence
id: 2026-11-19-with-the-fix-at-e60fbf8e-all-24-new-integration-tests-both-new
aliases: []
claim: "With the fix at e60fbf8e, all 24 new integration tests, both new hash_cache unit tests and the 7 new util unit tests pass (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004, REQ-git-utils-012/013, REQ-agents-013 and the write-scan requirements), the write and read scans find no direct project file write or read outside the helpers, and the full verify lane passes: unit 3116 passed, 0 failed, 2 ignored; integration 689 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts."
result: passed
evidence_kind: product-test
method: fledge lanes run verify through one-lane at e60fbf8e (fmt, clippy on all targets, cargo test, release build, strict spec check, gate scripts)
revision: e60fbf8e02faa643254590c1b4b5f5a659aa54d9
content: dd77399afb5305b4ad7830b7c1e438ea2c983073c6b5c5fe56d57ae0c641d405
environment: local-macos-arm64
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements:
  - REQ-util-004
  - REQ-util-005
  - REQ-change-163
  - REQ-config-017
  - REQ-validator-050
  - REQ-validator-051
  - REQ-changelog-002
contracts: []
limitations: []
observed: 2026-11-19T20:33:49Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix at e60fbf8e, all 24 new integration tests, both new hash_cache unit tests and the 7 new util unit tests pass (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004, REQ-git-utils-012/013, REQ-agents-013 and the write-scan requirements), the write and read scans find no direct project file write or read outside the helpers, and the full verify lane passes: unit 3116 passed, 0 failed, 2 ignored; integration 689 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.**

Recorded as passed in local-macos-arm64 at revision e60fbf8e02fa, observed 2026-11-19T20:33:49Z.
