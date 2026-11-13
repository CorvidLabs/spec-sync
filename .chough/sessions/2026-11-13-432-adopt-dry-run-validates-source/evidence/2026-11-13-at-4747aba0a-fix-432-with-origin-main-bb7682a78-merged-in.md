---
kind: evidence
id: 2026-11-13-at-4747aba0a-fix-432-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does."
result: passed
evidence_kind: product-test
revision: 4747aba0a318c01b7d06e5baef9b435e83b4b11c
content: e8d35f3032f532d31ed0d9f99906516cd10e5346803504d2d2e113049082ddec
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:27d78b1371d971da7c4921ad61fa9d09618e614439a6c67f9cd675d2b93850c3
external: ext-01e71a
criteria: [SETUP-5.b]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T17:51:30Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 4747aba0a (fix/432 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 4747aba0a318, observed 2026-11-13T17:51:30Z.
