---
kind: evidence
id: 2026-11-19-codex-s-p2-at-0cdd7f41-the-tester-s-case-a-change-log-row-now
aliases: []
claim: "Codex's P2 at 0cdd7f41 (the tester's case): a Change Log row now splits on unescaped pipes only, with an escaped pipe part of its cell, so the id a|b with the legacy cell 'a\\|b: Title' and history/a|b.md is a double record; the unescaped 'a|b: Title' is two cells and does not match. The new test fails on 0cdd7f41 and passes. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3087 unit and 627 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed."
result: passed
evidence_kind: product-test
revision: 2d5d5ff52087039264d8e23c5effb511b99e5ced
content: 64a830d840dfeef2086c93b0d8e9ccb3d8504b001d380c4a9884fd3334ecc5a4
environment: local macOS 26, rust 1.89.0, branch 1084-review-fixes at 2d5d5ff5
criteria: [MERGE-1]
requirements: [REQ-change-159]
contracts: []
limitations: []
observed: 2026-11-19T06:24:47Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Codex's P2 at 0cdd7f41 (the tester's case): a Change Log row now splits on unescaped pipes only, with an escaped pipe part of its cell, so the id a|b with the legacy cell 'a\|b: Title' and history/a|b.md is a double record; the unescaped 'a|b: Title' is two cells and does not match. The new test fails on 0cdd7f41 and passes. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3087 unit and 627 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1084-review-fixes at 2d5d5ff5 at revision 2d5d5ff52087, observed 2026-11-19T06:24:47Z.
