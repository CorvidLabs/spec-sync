---
kind: evidence
id: 2026-11-13-at-84f77a405-fix-985-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At 84f77a405 (fix/985 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including git_utils::tests::batched_reads_accept_sha256_object_ids."
result: passed
evidence_kind: product-test
revision: 84f77a405718e452597582493300b184d3f3f9af
content: 1a73555c8c7c3606300fabdc868d0631454b8f49386e6f726a4896f8c43cc715
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:24e3adf5546a12b4ccfa88f9c4ce79b756ba86f36fa11c56d12fa047b64e46dc
external: ext-589608
criteria: [SPEED-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T16:57:47Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 84f77a405 (fix/985 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including git_utils::tests::batched_reads_accept_sha256_object_ids.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 84f77a405718, observed 2026-11-13T16:57:47Z.
