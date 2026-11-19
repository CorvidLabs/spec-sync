---
kind: evidence
id: 2026-11-19-covers-req-change-159-160-req-validator-048-req-hash-cache-006
aliases: []
claim: "Covers REQ-change-159, -160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037 (the last two are not linked: chough 0.6.0 looks them up under specs/hash-cache and specs/cmd-change). Failing first on 65864de8: 7 of 8 tests in tests/integration/spec_history.rs failed, and main's release binary made two branches (ADDED REQ-auth-002; MODIFIED Invariants) that conflict under plain git merge in specs/auth/auth.spec.md on the two Change Log rows. Now, with GIT_CONFIG_GLOBAL=/dev/null (this Mac's global tag.gpgsign breaks changelog::tests' lightweight tags; they do not touch this change): fledge lanes run verify passed (fmt, clippy --all-targets -D warnings, check, 3077 unit and 627 integration tests, release build, specsync check --strict --require-coverage 100 62/62, CI script tests). The same two branches merge with plain git, no driver; version: and the Change Log stay byte for byte; both fragments are kept; specsync history renders them. This change's own change check --commit wrote six fragments and bumped no version. change audit --strict and fledge trust verify (augur review, risk 39) passed."
result: passed
evidence_kind: product-test
revision: d2fa5b6bc8686320333be8463b0d2fa913d836d5
content: aad27e38a54cc3cf876bb6b7a8059ae041e8b7a242e2864f515b6b1a41e2089f
environment: local macOS 26, rust 1.89.0 (rust-toolchain.toml), branch 1084-history-fragments at d2fa5b6b
criteria: [MERGE-1]
requirements: [REQ-change-159, REQ-change-160, REQ-validator-048]
contracts: []
limitations: []
observed: 2026-11-19T01:30:14Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Covers REQ-change-159, -160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037 (the last two are not linked: chough 0.6.0 looks them up under specs/hash-cache and specs/cmd-change). Failing first on 65864de8: 7 of 8 tests in tests/integration/spec_history.rs failed, and main's release binary made two branches (ADDED REQ-auth-002; MODIFIED Invariants) that conflict under plain git merge in specs/auth/auth.spec.md on the two Change Log rows. Now, with GIT_CONFIG_GLOBAL=/dev/null (this Mac's global tag.gpgsign breaks changelog::tests' lightweight tags; they do not touch this change): fledge lanes run verify passed (fmt, clippy --all-targets -D warnings, check, 3077 unit and 627 integration tests, release build, specsync check --strict --require-coverage 100 62/62, CI script tests). The same two branches merge with plain git, no driver; version: and the Change Log stay byte for byte; both fragments are kept; specsync history renders them. This change's own change check --commit wrote six fragments and bumped no version. change audit --strict and fledge trust verify (augur review, risk 39) passed.**

Recorded as passed in local macOS 26, rust 1.89.0 (rust-toolchain.toml), branch 1084-history-fragments at d2fa5b6b at revision d2fa5b6bc868, observed 2026-11-19T01:30:14Z.
