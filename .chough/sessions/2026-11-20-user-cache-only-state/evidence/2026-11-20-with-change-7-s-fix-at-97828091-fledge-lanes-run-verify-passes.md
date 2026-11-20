---
kind: evidence
id: 2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes
aliases: []
claim: "With change 7's fix at 97828091, fledge lanes run verify passes all 10 steps: unit 3125 passed, 0 failed, 2 ignored; integration 702 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 3 new unit tests and 7 new integration tests pass (also REQ-git-utils-012 and REQ-cmd-check-021); the state folder is in the user cache folder, a copy of a project with its .git validates every spec, a cache folder inside the project is refused, a folder named * stays confined, a registry entry naming .env opens no replay, a local core.hooksPath outside makes no folder, and the scans find nothing unmarked."
result: passed
evidence_kind: product-test
revision: 97828091ad38ab5bdc7c4ab49f1d9554cfae8be3
content: 2c8e78ca4b990e9ef05fe352e68d6be0504a860db9de22e3267d1f77e5762688
environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 97828091 (change 7's code 18d07160)
fingerprint: sha256:5ae2654c32d3deef0339bc2d1923acb1ee4682d911c105472e915f4dec0cc9a1
external: ext-b21440
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-006, REQ-util-008, REQ-change-162, REQ-hooks-009]
contracts: []
limitations: []
observed: 2026-11-20T04:12:03Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With change 7's fix at 97828091, fledge lanes run verify passes all 10 steps: unit 3125 passed, 0 failed, 2 ignored; integration 702 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 3 new unit tests and 7 new integration tests pass (also REQ-git-utils-012 and REQ-cmd-check-021); the state folder is in the user cache folder, a copy of a project with its .git validates every spec, a cache folder inside the project is refused, a folder named * stays confined, a registry entry naming .env opens no replay, a local core.hooksPath outside makes no folder, and the scans find nothing unmarked.**

Recorded as passed in macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 97828091 (change 7's code 18d07160) at revision 97828091ad38, observed 2026-11-20T04:12:03Z.
