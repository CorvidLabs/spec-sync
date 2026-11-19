---
kind: evidence
id: 2026-11-19-1060-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the new tests, check::an_unparseable_3md_is_a_check_warning_and_fails_strict fails: check prints no warning for a .3md with two planes at one z, and check --strict passes. This is the defect #1060 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:ced0b3127d128007a5e43cf2d61e35b53e2f1230dd7a8ef05d4e0d6907a742ef
external: ext-1dbe42
criteria: [THREE-1]
requirements: []
contracts: [specs/cmd_check/cmd_check.spec.md]
limitations: []
observed: 2026-11-19T07:14:10Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the new tests, check::an_unparseable_3md_is_a_check_warning_and_fails_strict fails: check prints no warning for a .3md with two planes at one z, and check --strict passes. This is the defect #1060 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c2b19a9115af, observed 2026-11-19T07:14:10Z.
