---
kind: evidence
id: 2026-11-13-at-1085663ce-fix-965-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At 1085663ce (fix/965 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including ArchiveGateHistoryTests.test_every_archive_gate_job_checks_out_full_history in .github/scripts/test-required-ci-gate.py."
result: passed
evidence_kind: product-test
revision: 1085663ced25f270aaf7f88ababe2f28df59424e
content: 60c0c9fa860da0cf179d0ee6db091c4ac659cd2bc6d36fa2adace0698f52023c
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:af4260b92b48f653772271a76f525093782fefc6761807d0e1a0ad74f43708ef
external: ext-d65266
criteria: [CI-2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T17:51:41Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 1085663ce (fix/965 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including ArchiveGateHistoryTests.test_every_archive_gate_job_checks_out_full_history in .github/scripts/test-required-ci-gate.py.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 1085663ced25, observed 2026-11-13T17:51:41Z.
