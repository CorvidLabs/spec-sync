---
kind: evidence
id: 2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli
aliases: []
claim: "With the fix, the 6 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3104 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts."
result: passed
evidence_kind: product-test
method: fledge lanes run verify with an isolated git config (10 steps, 13 min); rustc 1.89.0
revision: cf5284076db2b692d8e8df8953eb6525d5952690
content: e47aa656f3078a178d8a6bad65550d5234b468f37f1b0273affc434e1a03699f
environment: local-macos-arm64
criteria: [SETUP-8]
requirements: []
contracts: []
limitations:
  - macOS arm64; the Linux container run follows. The first lane run failed one older test that planted a bare-array journal by hand; it now writes the durable form, as two other older tests do.
observed: 2026-11-19T16:35:48Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix, the 6 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3104 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.**

Recorded as passed in local-macos-arm64 at revision cf5284076db2, observed 2026-11-19T16:35:48Z.
