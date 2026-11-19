---
kind: evidence
id: 2026-11-19-1038-verify-lane-at-5f5cdf8d
aliases: []
claim: "At 5f5cdf8d (PR #1093 after the archive-side fix e54bc636, each change re-checked there), fledge lanes run verify passed 10/10 steps in 9m42s: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes the #1038 tests: the archive tip of a reopened change on its real shape, deleted package files beside a modified archive file, a package still present, the status table, the agreement with CI's classifier script (run with bash and jq), and the two archived review-stage tests."
result: passed
evidence_kind: product-test
revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
content: bf638a95bab9091bab5a1880c42b353e868c47abad1ea4614ebc0ac89c46416e
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:9c02647bf5c0ab235eddbf4832713a82e16bc50fc24ceaca983648aaf36cd305
external: ext-4dc414
criteria: [SHIP-8]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md, specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T06:20:36Z
actor:
  - agent: claude-code
    terminal: t59
---
**At 5f5cdf8d (PR #1093 after the archive-side fix e54bc636, each change re-checked there), fledge lanes run verify passed 10/10 steps in 9m42s: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes the #1038 tests: the archive tip of a reopened change on its real shape, deleted package files beside a modified archive file, a package still present, the status table, the agreement with CI's classifier script (run with bash and jq), and the two archived review-stage tests.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 5f5cdf8d41eb, observed 2026-11-19T06:20:36Z.
