---
kind: evidence
id: 2026-11-19-covers-req-change-159-160-and-req-hash-cache-006-not-linked
aliases: []
claim: "Covers REQ-change-159, -160 and REQ-hash-cache-006 (not linked: chough 0.6.0 looks up specs/hash-cache). Red first on 979c3fe9: the four new tests (quoted change id round-trip, boundary-aware matcher, Change Log delta refusal, symlink-swapped fragment cache replay) all failed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3081 unit and 627 integration tests, release build, strict specs 62/62, CI script tests). An older test that approved a MODIFIED Change Log delta now asserts the check-time refusal instead. change audit --strict and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: de6a78796022977c87002ea9610fb0911a406f8e
content: c44939b2e1b03baa00e64a209036fb1624c938a52665b216d2f46f66add3deae
environment: local macOS 26, rust 1.89.0, branch 1084-review-fixes at de6a7879 (rebased on main 3bebe8b0)
criteria: [MERGE-1]
requirements: [REQ-change-159, REQ-change-160]
contracts: []
limitations: []
observed: 2026-11-19T02:36:25Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Covers REQ-change-159, -160 and REQ-hash-cache-006 (not linked: chough 0.6.0 looks up specs/hash-cache). Red first on 979c3fe9: the four new tests (quoted change id round-trip, boundary-aware matcher, Change Log delta refusal, symlink-swapped fragment cache replay) all failed. Now fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null (fmt, clippy -D warnings, 3081 unit and 627 integration tests, release build, strict specs 62/62, CI script tests). An older test that approved a MODIFIED Change Log delta now asserts the check-time refusal instead. change audit --strict and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1084-review-fixes at de6a7879 (rebased on main 3bebe8b0) at revision de6a78796022, observed 2026-11-19T02:36:25Z.
