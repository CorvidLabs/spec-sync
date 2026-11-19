---
kind: evidence
id: 2026-11-19-998-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), repair_puts_back_a_damaged_reopening_relay_from_head and repair_removes_a_damaged_correction_relay_with_no_committed_copy fail with an empty repairs list, and a_nested_repair_names_its_place fails: it reads 'approvals.json approval 2' where 'approvals.json reopening 2 superseded approval' is expected. This is the defect #998 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
content: 34f38b9e1b232742f0ff71cb8b2ba7423b674b99002a4ecf6eeacfe19549f007
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:10bbc70c68b81697eb3a09b6bb654417f08c64270d9c887f268a3695fcdd5d84
external: ext-0313a4
criteria: [CHANGE-13.e]
requirements: [REQ-change-relay-repair-001]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T07:53:32Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), repair_puts_back_a_damaged_reopening_relay_from_head and repair_removes_a_damaged_correction_relay_with_no_committed_copy fail with an empty repairs list, and a_nested_repair_names_its_place fails: it reads 'approvals.json approval 2' where 'approvals.json reopening 2 superseded approval' is expected. This is the defect #998 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision bf0014ada677, observed 2026-11-19T07:53:32Z.
