---
kind: evidence
id: 2026-11-13-at-9786779ec-verify-step-5-release-candidate-test-failed-3
aliases: []
claim: "At 9786779ec, verify step 5 release-candidate-test failed 3 tests: this change's docs example used CorvidLabs/spec-sync@<commit>, and the release validator requires every site Action ref to be @v6.3.0. Fixed in 7f98a979a in this session: the example is a with: fragment fenced as text, and the page says @v6.3.0 lacks the input. No bug record: found and fixed before review."
result: failed
evidence_kind: product-test
revision: 7f98a979abe8422e8e33ea4f54e27ebb28a8c734
content: 69a234022f1e0b53c28d0d99db33e19551c35ead25f096a65aa57fba6165755b
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:2fcbaf91c0fcefb669caf565486f02bccc6ee84d675969e7125ce74097142476
external: ext-32250f
criteria: [CI-1.b]
requirements: [REQ-github-031]
contracts: []
limitations: []
observed: 2026-11-13T18:39:07Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 9786779ec, verify step 5 release-candidate-test failed 3 tests: this change's docs example used CorvidLabs/spec-sync@<commit>, and the release validator requires every site Action ref to be @v6.3.0. Fixed in 7f98a979a in this session: the example is a with: fragment fenced as text, and the page says @v6.3.0 lacks the input. No bug record: found and fixed before review.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 7f98a979abe8, observed 2026-11-13T18:39:07Z.
