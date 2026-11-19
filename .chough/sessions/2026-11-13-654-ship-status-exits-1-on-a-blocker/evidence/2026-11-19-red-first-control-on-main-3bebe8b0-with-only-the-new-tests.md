---
kind: evidence
id: 2026-11-19-red-first-control-on-main-3bebe8b0-with-only-the-new-tests
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the new tests, change::ship_status_exits_one_when_it_reports_a_blocker fails (ship-status printed its Blocker line and exited 0), and the three existing tests that now expect exit 1 from a blocked ship-status fail the same way. The failure is the defect #654 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: 4d439b395c78eb20987f11815fd46ee859bde087
content: 1b34863182f9a288aa5a2899fb3dad4140972adb3abe0d9516b44c695af88683
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:ed377209c613487d1e02dfb5d820168fad6abee797c88d85415adca933101b26
external: ext-213309
criteria: [SHIP-9.a]
requirements: [REQ-cmd-change-039]
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T04:55:17Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the new tests, change::ship_status_exits_one_when_it_reports_a_blocker fails (ship-status printed its Blocker line and exited 0), and the three existing tests that now expect exit 1 from a blocked ship-status fail the same way. The failure is the defect #654 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 4d439b395c78, observed 2026-11-19T04:55:17Z.
