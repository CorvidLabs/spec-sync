---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: 81877688e5eb0e3749d3db30e55fa7f0d1347302
content: 66f48f900b24f56ef8e63696fbd06379614040e7d87d9be76423f81f3339dfbe
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:e59482574dfd6474faf45cc03d6d339098e08a67919e7447ac4ed230cc14a2e7
external: ext-99d3f2
criteria: [SHIP-9.a]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-13T22:39:02Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, change::ship_status_exits_one_when_it_reports_a_blocker fails: ship-status printed its Blocker line and exited 0. The failure is the defect #654 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 81877688e5eb, observed 2026-11-13T22:39:02Z.
