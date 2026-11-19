---
kind: evidence
id: 2026-11-19-1050-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the new tests, both fenced_requirement_example_delta tests fail: a MODIFIED delta replaces the fenced example instead of the real requirement, and a heading inside a fenced example ends the block. This is the defect #1050 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: c2b19a9115af3d6045d3f09447fcd7bc05907cae
content: d51119b1914763f95f18c5a96371c2c554353568b4863d3b69f2f0234962b4d8
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:cc0c01be6e2b1a79519259666ff6c3a25aa4df90cfd6da474bc8bd28c7663b46
external: ext-eff1a5
criteria: [CHANGE-5]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T07:14:08Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the new tests, both fenced_requirement_example_delta tests fail: a MODIFIED delta replaces the fenced example instead of the real requirement, and a heading inside a fenced example ends the block. This is the defect #1050 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c2b19a9115af, observed 2026-11-19T07:14:08Z.
