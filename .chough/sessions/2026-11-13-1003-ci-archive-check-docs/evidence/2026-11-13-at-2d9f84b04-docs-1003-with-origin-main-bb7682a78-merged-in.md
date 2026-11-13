---
kind: evidence
id: 2026-11-13-at-2d9f84b04-docs-1003-with-origin-main-bb7682a78-merged-in
aliases: []
claim: "At 2d9f84b04 (docs/1003 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including archived_upstream::the_documented_ci_archive_check_passes_an_intact_archive_and_fails_a_tampered_one, which runs the page's own check (REQ-cmd-change-038; chough cannot resolve requirement ids of the cmd_change module, so the id is named here)."
result: passed
evidence_kind: product-test
revision: af09c3dc58ccd08160660cccd3bf55ec27cb42ab
content: 5f03fdee74f2435e676baeb0daf98dc77986930c2857cfade549bf2ce4cbc53b
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:5deff0b599c703c5e9fad46985eeccf994742a2e16a65c49efb9a44cdb1747b8
external: ext-ccd517
criteria: [CI-1]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-13T18:38:46Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 2d9f84b04 (docs/1003 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3070 passed, 2 ignored; integration 620 passed), release build, strict spec check, and the CI script tests, including archived_upstream::the_documented_ci_archive_check_passes_an_intact_archive_and_fails_a_tampered_one, which runs the page's own check (REQ-cmd-change-038; chough cannot resolve requirement ids of the cmd_change module, so the id is named here).**

Recorded as passed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision af09c3dc58cc, observed 2026-11-13T18:38:46Z.
