---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: 2ef24f40efb50521c8c712fd9f842ab2c58d0612
content: 403b9ac0ba3a3c372edbc958342ac51185669a5fab2d0d2883ac4b8391d544dc
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:1b8b0888dcf6c22c142d40d96daba4a2db89ecbed36e9f2d1734c12c5bc335e5
external: ext-b16282
criteria: [SPEED-1.b]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T21:51:54Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, check_scaling::check_time_grows_linearly_with_undocumented_exports fails. On main 8,000 undocumented exports took 0.55 s and 32,000 took 4.9 s, ratio 9.0, over the linear bound of 7. The failure is the defect #574 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 2ef24f40efb5, observed 2026-11-13T21:51:54Z.
