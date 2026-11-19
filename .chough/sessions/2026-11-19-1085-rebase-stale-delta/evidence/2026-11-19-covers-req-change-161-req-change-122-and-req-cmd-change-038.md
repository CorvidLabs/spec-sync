---
kind: evidence
id: 2026-11-19-covers-req-change-161-req-change-122-and-req-cmd-change-038
aliases: []
claim: "Covers REQ-change-161, REQ-change-122 and REQ-cmd-change-038 (the last not linked: chough 0.6.0 looks up specs/cmd-change). Red first on 99f61b2b: tests/integration/rebase_stale_delta.rs's discriminator failed (change check refused the non-overlapping stale delta); the overlapping control passed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3084 unit and 630 integration tests, release build, strict specs 62/62, CI script tests): 3 integration and 7 unit tests cover the rebase, status text and JSON, a tampered record, re-approval and acceptance after a rebase, and the refusing controls (overlap, spec section, workflow v1, no history, CI). change audit --strict and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 101edcf006e0b8f366a031bf22d89452d6c89daa
content: 5ef2f7c8e080e5ca39ad61be55ae099357ed236900fad4f1b039fec5fd5b90fa
environment: local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 101edcf0 (rebased on main 3bebe8b0)
criteria: [CHANGE-6.a, CHANGE-6.c]
requirements: [REQ-change-161, REQ-change-122]
contracts: []
limitations: []
observed: 2026-11-19T02:47:30Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Covers REQ-change-161, REQ-change-122 and REQ-cmd-change-038 (the last not linked: chough 0.6.0 looks up specs/cmd-change). Red first on 99f61b2b: tests/integration/rebase_stale_delta.rs's discriminator failed (change check refused the non-overlapping stale delta); the overlapping control passed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3084 unit and 630 integration tests, release build, strict specs 62/62, CI script tests): 3 integration and 7 unit tests cover the rebase, status text and JSON, a tampered record, re-approval and acceptance after a rebase, and the refusing controls (overlap, spec section, workflow v1, no history, CI). change audit --strict and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 101edcf0 (rebased on main 3bebe8b0) at revision 101edcf006e0, observed 2026-11-19T02:47:30Z.
