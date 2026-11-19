---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, git_utils::tests::batched_reads_accept_sha256_object_ids fails. The failure is the defect #985 reports, fixed by this change; it is not a new bug."
result: failed
evidence_kind: product-test
revision: 84f77a405718e452597582493300b184d3f3f9af
content: 1a73555c8c7c3606300fabdc868d0631454b8f49386e6f726a4896f8c43cc715
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:73b3917a54f504cc1d97104954c16b511e136e774967e4da8d51db32a3056179
external: ext-19695d
criteria: [SPEED-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T16:57:49Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, git_utils::tests::batched_reads_accept_sha256_object_ids fails. The failure is the defect #985 reports, fixed by this change; it is not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 84f77a405718, observed 2026-11-13T16:57:49Z.
