---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration fails: the fingerprint covered the sources but not the build configuration. The failure is the defect #981 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: f4f8a0e75dda470c85fe8fd657d1e00615b7bb73
content: 5757a8070d345b93828663724a048fe53040cd4fe38f85d126cf39be3f1e8358
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:99d5042b804281665a8710b54532300165a6171957674e1646e084d378cd81d6
external: ext-f3ab6e
criteria: [SPEED-1.c]
requirements: []
contracts: [specs/hash_cache/hash_cache.spec.md]
limitations: []
observed: 2026-11-13T20:54:49Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration fails: the fingerprint covered the sources but not the build configuration. The failure is the defect #981 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision f4f8a0e75dda, observed 2026-11-13T20:54:49Z.
