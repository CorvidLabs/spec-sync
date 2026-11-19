---
kind: evidence
id: 2026-11-13-at-f4f8a0e75-fix-981-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At f4f8a0e75 (fix/981 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3073 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration."
result: passed
evidence_kind: product-test
revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
content: 5757a8070d345b93828663724a048fe53040cd4fe38f85d126cf39be3f1e8358
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:e2816553b332e3e1f80382c88f5eac3bf22bbda840ae778adf70202fed5cdfd9
external: ext-b94ca8
criteria: [SPEED-1.c]
requirements: []
contracts: [specs/hash_cache/hash_cache.spec.md]
limitations: []
observed: 2026-11-13T20:54:48Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At f4f8a0e75 (fix/981 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3073 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision f4f8a0e75dda, observed 2026-11-13T20:54:48Z.
