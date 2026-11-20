---
kind: evidence
id: 2026-11-20-change-9-the-four-red-tests-an-oversized-and-a-non-utf-8-205d35
aliases: []
claim: "Change 9: the four red tests (an oversized and a non-UTF-8 CLAUDE.md, a linked hooks folder, a linked user-cache hashes.json) failed on 7527da3a and pass at 852ec92a; the two controls pass; full suites green (3158 unit, 730 integration), clippy clean, check --strict 100%"
result: passed
evidence_kind: product-test
revision: 852ec92a75fbb28b95f0332b2976674003ea2df1
content: 4f67ad1f4b5c35f97f144581d664204c9661d2fff64de6c5d054c9b642a985c2
environment: macOS arm64 verify lane and Linux rust:1.89-bookworm non-root, through one-lane, at 852ec92a
criteria: [SETUP-6, SETUP-7]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T16:30:17Z
actor:
  - agent: claude-code
    terminal: t5r
---
**Change 9: the four red tests (an oversized and a non-UTF-8 CLAUDE.md, a linked hooks folder, a linked user-cache hashes.json) failed on 7527da3a and pass at 852ec92a; the two controls pass; full suites green (3158 unit, 730 integration), clippy clean, check --strict 100%.**

Recorded as passed in macOS arm64 verify lane and Linux rust:1.89-bookworm non-root, through one-lane, at 852ec92a at revision 852ec92a75fb, observed 2026-11-20T16:30:17Z.
