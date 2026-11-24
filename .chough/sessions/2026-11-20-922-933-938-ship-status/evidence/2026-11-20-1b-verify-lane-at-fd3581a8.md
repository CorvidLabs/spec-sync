---
kind: evidence
id: 2026-11-20-1b-verify-lane-at-fd3581a8
aliases: []
claim: "At fd3581a8 (checked and committed), with this tree's release build first on PATH: fledge lanes run verify 10/10 in 8m27s through one-lane (unit 3108, 2 ignored; integration 648, the 4 binary tests and a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it, which the first lane at a596265d caught and 48e448ce fixed; spec check 62/62, 167/167 files). change audit --strict fails only on tests/integration.rs and tests/integration/ship_status_reopen_and_resume.rs, which the definition does not declare."
result: passed
evidence_kind: product-test
revision: fd3581a813e40d0c30b41680472573f199585809
content: 109e14be19f7fac5e45d2c23b481f1fb8c4b84daa7725404fa2c8e12a9b8996a
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:e5d1ca1bac0493123034aa3ebcdd2ec553b221b75ed12dad97c1a6ea9041053e
external: ext-dfe06e
criteria: [SHIP-4.a, SHIP-5, SHIP-9.a]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md, specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T13:11:33Z
actor:
  - agent: claude-code
    terminal: t59
---
**At fd3581a8 (checked and committed), with this tree's release build first on PATH: fledge lanes run verify 10/10 in 8m27s through one-lane (unit 3108, 2 ignored; integration 648, the 4 binary tests and a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it, which the first lane at a596265d caught and 48e448ce fixed; spec check 62/62, 167/167 files). change audit --strict fails only on tests/integration.rs and tests/integration/ship_status_reopen_and_resume.rs, which the definition does not declare.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision fd3581a813e4, observed 2026-11-20T13:11:33Z.
