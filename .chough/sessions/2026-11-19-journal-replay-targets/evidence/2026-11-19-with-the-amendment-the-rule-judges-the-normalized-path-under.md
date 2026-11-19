---
kind: evidence
id: 2026-11-19-with-the-amendment-the-rule-judges-the-normalized-path-under
aliases: []
claim: "With the amendment (the rule judges the normalized path; under specs_dir only spec files, requirements.md and history fragments), all 8 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3106 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check, the gate scripts."
result: passed
evidence_kind: product-test
method: fledge lanes run verify with an isolated git config (10 steps, 19 min); rustc 1.89.0
revision: 75dddd7695acbbb2106feeb9889d315cd64297f8
content: 20744325f75306e5ec7793e16af2ddbec0d9746bbf22d486aa48c9c37cab24e0
environment: local-macos-arm64
criteria: [SETUP-8]
requirements: []
contracts: []
limitations:
  - macOS arm64; Linux at the pre-amendment tip 75dddd76 was green (unit 3108/0, integration 669/0); Linux on the amended tip follows.
observed: 2026-11-19T17:00:32Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the amendment (the rule judges the normalized path; under specs_dir only spec files, requirements.md and history fragments), all 8 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3106 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check, the gate scripts.**

Recorded as passed in local-macos-arm64 at revision 75dddd7695ac, observed 2026-11-19T17:00:32Z.
