---
kind: evidence
id: 2026-11-13-red-first-control-with-the-test-and-no-page-archived-upstream
aliases: []
claim: "Red-first control: with the test and no page, archived_upstream::the_documented_ci_archive_check_passes_an_intact_archive_and_fails_a_tampered_one fails because the page is absent. The page this change adds is the fix; not a new bug."
result: failed
evidence_kind: product-test
revision: af09c3dc58ccd08160660cccd3bf55ec27cb42ab
content: 5f03fdee74f2435e676baeb0daf98dc77986930c2857cfade549bf2ce4cbc53b
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:79c3aaf76cfba8e6056d6b8bb821257d157ecbe3dcf3be45317704ff00e971fd
external: ext-bae8c4
criteria: [CI-1]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-13T18:38:47Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: with the test and no page, archived_upstream::the_documented_ci_archive_check_passes_an_intact_archive_and_fails_a_tampered_one fails because the page is absent. The page this change adds is the fix; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision af09c3dc58cc, observed 2026-11-13T18:38:47Z.
