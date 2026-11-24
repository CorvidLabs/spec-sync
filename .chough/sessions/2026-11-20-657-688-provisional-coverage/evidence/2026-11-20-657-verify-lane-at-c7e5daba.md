---
kind: evidence
id: 2026-11-20-657-verify-lane-at-c7e5daba
aliases: []
claim: "At c7e5daba (checked and committed), with this tree's release build first on PATH: fledge lanes run verify passed 10/10 in 7m40s through one-lane (unit 3111 passed, 2 ignored, provisional_coverage 11 of 11; integration 644; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes. change audit's exit status is report.errors.is_empty() with or without --strict, so the unit test's no-error-one-warning audit is also --strict's exit 0."
result: passed
evidence_kind: product-test
revision: c7e5daba2b794e31975863df7ed90d1f2c152e01
content: d94588e6d0f6595361b1100b1fa6c55ef793d3486e3a31e0f12e5c5ddc8c9173
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:5c5389a611090f6685d218ea7d8ba10f4d55dd366efd2522f0c491b0174172cb
external: ext-7ad97a
criteria: [SHIP-13, SHIP-13.a, SHIP-13.b, SHIP-13.c, SHIP-13.d]
requirements: []
contracts: [specs/change/change.spec.md, specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-20T07:30:04Z
actor:
  - agent: claude-code
    terminal: t59
---
**At c7e5daba (checked and committed), with this tree's release build first on PATH: fledge lanes run verify passed 10/10 in 7m40s through one-lane (unit 3111 passed, 2 ignored, provisional_coverage 11 of 11; integration 644; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes. change audit's exit status is report.errors.is_empty() with or without --strict, so the unit test's no-error-one-warning audit is also --strict's exit 0.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c7e5daba2b79, observed 2026-11-20T07:30:04Z.
