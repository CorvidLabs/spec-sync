---
kind: evidence
id: 2026-11-13-at-1898dc4bb-after-the-fix-fledge-lanes-run-verify-passed-10
aliases: []
claim: "At 1898dc4bb, after the fix, fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed, including the REQ-change-127 spawn-ceiling tests), release build, strict spec check, and the CI script tests. The cold archived status test reads 227 with six archives; a_failed_fact_is_not_memoized_but_a_definite_absence_is and a_missing_path_answer_is_definite_but_a_bad_revision_is_not pass."
result: passed
evidence_kind: product-test
revision: 1898dc4bb4b2d11bbe740b52a94493b150ad50f9
content: ded734ebf635232c49a9d58035421dcccab754fb30318f9780b063175a205127
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:662046c9ba95dfeb4dcd7788218243b5a2055ce232900448a46895ffe5e5a083
external: ext-60046c
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T21:09:05Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 1898dc4bb, after the fix, fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed, including the REQ-change-127 spawn-ceiling tests), release build, strict spec check, and the CI script tests. The cold archived status test reads 227 with six archives; a_failed_fact_is_not_memoized_but_a_definite_absence_is and a_missing_path_answer_is_definite_but_a_bad_revision_is_not pass.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 1898dc4bb4b2, observed 2026-11-13T21:09:05Z.
