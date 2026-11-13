---
kind: evidence
id: 2026-11-13-red-first-control-on-main-bb7682a78-the-action-has-no-expected
aliases: []
claim: "Red-first control: on main bb7682a78 the Action has no expected-sha256 input and no verifier, so .github/scripts/test-verify-release-digest.sh fails at its first case. This change adds them; not a new bug."
result: failed
evidence_kind: product-test
revision: 7f98a979abe8422e8e33ea4f54e27ebb28a8c734
content: 69a234022f1e0b53c28d0d99db33e19551c35ead25f096a65aa57fba6165755b
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:9f00a374b59b9e11535fc28fc4a13bd723c13d963d444c3f030d763e156a6870
external: ext-494e70
criteria: [CI-1.b]
requirements: [REQ-github-031]
contracts: []
limitations: []
observed: 2026-11-13T18:39:08Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main bb7682a78 the Action has no expected-sha256 input and no verifier, so .github/scripts/test-verify-release-digest.sh fails at its first case. This change adds them; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 7f98a979abe8, observed 2026-11-13T18:39:08Z.
