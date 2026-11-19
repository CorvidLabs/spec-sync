---
kind: evidence
id: 2026-11-13-at-bff847366-fix-937-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At bff847366 (fix/937 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps in 14m56s under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests. It includes archived_upstream::a_sequence_ledger_freeze_outranks_the_verification_ignored_paths_repair_in_text_status."
result: passed
evidence_kind: product-test
revision: bff847366cbe88ce91144ee3352983dbbe906b5f
content: 0aa47cb570da7ccd4a41d8ad949b503868b34a751782e84a5af35b9a1247e3f2
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:0a3c32a28eab8001eddb6329f5f7abe209a8d5fc3fefc62a6d9e72f64cf331c8
external: ext-c9655e
criteria: [SHIP-9.a]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-13T16:15:18Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At bff847366 (fix/937 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps in 14m56s under the shared heavy-test lock: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests. It includes archived_upstream::a_sequence_ledger_freeze_outranks_the_verification_ignored_paths_repair_in_text_status.**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision bff847366cbe, observed 2026-11-13T16:15:18Z.
