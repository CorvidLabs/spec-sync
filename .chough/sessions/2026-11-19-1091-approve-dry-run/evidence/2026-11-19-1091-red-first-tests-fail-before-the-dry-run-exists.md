---
kind: evidence
id: 2026-11-19-1091-red-first-tests-fail-before-the-dry-run-exists
aliases: []
claim: "Red first: on main 3bebe8b0 with the approved #1091 change and only the new tests, all 8 tests of tests/integration/approve_dry_run.rs fail with 'unexpected argument --dry-run found', and cli::tests::change_approve_accepts_a_dry_run fails to parse the flag. The feature does not exist yet; this is the expected failure, not a bug."
result: failed
evidence_kind: product-test
revision: 1f5c09de27b3b4022f86588b84844e9e1b7a1238
content: ed60283a75524a4d3d6f0fd432d70664a1f7ae5dc17dea11535fdbe42abb4f2a
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:66535e3d86976a17593dcd96879777e84add9a707f33c173b0decf326aa2cdcb
external: ext-50c462
criteria: [CHANGE-15, CHANGE-15.a, CHANGE-15.b, CHANGE-15.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T05:58:19Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first: on main 3bebe8b0 with the approved #1091 change and only the new tests, all 8 tests of tests/integration/approve_dry_run.rs fail with 'unexpected argument --dry-run found', and cli::tests::change_approve_accepts_a_dry_run fails to parse the flag. The feature does not exist yet; this is the expected failure, not a bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 1f5c09de27b3, observed 2026-11-19T05:58:19Z.
