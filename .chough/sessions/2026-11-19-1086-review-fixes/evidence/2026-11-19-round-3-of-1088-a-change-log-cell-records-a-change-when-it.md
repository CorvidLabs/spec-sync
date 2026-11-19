---
kind: evidence
id: 2026-11-19-round-3-of-1088-a-change-log-cell-records-a-change-when-it
aliases: []
claim: "Round 3 of #1088: a Change Log cell records a change when it begins with <id>: as written, or just inside the one Markdown marker that opens the cell where that marker closes right after the id or at the cell's end, so a whole cell in emphasis (**my-change: Title**) is a row again and *hotfix: Hot stays the row of *hotfix only; an id's leading spaces are cell padding, so a fragment for ' add-passkeys' beside the row '|  add-passkeys: Passkeys |' is a double record (the tester's case of Codex's P2 at 8266e6c6). New asserts fail on 8266e6c6 and c983f2e9 respectively and pass. REQ-change-159's clause and criterion re-approved under Leif's relayed 'Approve' (round 3). fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3086 unit and 627 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed."
result: passed
evidence_kind: product-test
revision: cf3c98081b29122b23f05ef3a1961fd825be0972
content: baf68a3153fda8c4991941a158f2ac1d1fcac689cae43415f76e01e6e2e6cb5e
environment: local macOS 26, rust 1.89.0, branch 1084-review-fixes at cf3c9808
criteria: [MERGE-1]
requirements: [REQ-change-159]
contracts: []
limitations: []
observed: 2026-11-19T06:03:41Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Round 3 of #1088: a Change Log cell records a change when it begins with <id>: as written, or just inside the one Markdown marker that opens the cell where that marker closes right after the id or at the cell's end, so a whole cell in emphasis (**my-change: Title**) is a row again and *hotfix: Hot stays the row of *hotfix only; an id's leading spaces are cell padding, so a fragment for ' add-passkeys' beside the row '|  add-passkeys: Passkeys |' is a double record (the tester's case of Codex's P2 at 8266e6c6). New asserts fail on 8266e6c6 and c983f2e9 respectively and pass. REQ-change-159's clause and criterion re-approved under Leif's relayed 'Approve' (round 3). fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3086 unit and 627 integration tests, strict specs 62/62; change audit --strict, fledge trust verify and pre-push passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1084-review-fixes at cf3c9808 at revision cf3c98081b29, observed 2026-11-19T06:03:41Z.
