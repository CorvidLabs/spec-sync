---
kind: evidence
id: 2026-11-13-at-a5f25b136-fix-990-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At a5f25b136 (fix/990 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory."
result: passed
evidence_kind: product-test
revision: a5f25b13601af8cdde1c1aa64dab0dc367ccd5b8
content: 9621afcea4595afd9a3df9b88919d635b03e2e15c3757cd08246eb1f7129dc31
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:763d63cc932ea6be5ddc32ae764b132f37c158c8048fe13f0533704ee9553bb6
external: ext-ecb2b3
criteria: [SPEED-1.a.2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T16:58:11Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At a5f25b136 (fix/990 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including change::tests::a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision a5f25b13601a, observed 2026-11-13T16:58:11Z.
