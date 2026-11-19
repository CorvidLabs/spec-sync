---
kind: evidence
id: 2026-11-19-998-verify-lane-at-bf0014ad
aliases: []
claim: "At bf0014ad (#998, #972 and #974 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 19m36s on a machine also running another lane: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 627 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes repair_puts_back_a_damaged_reopening_relay_from_head, repair_removes_a_damaged_correction_relay_with_no_committed_copy and a_nested_repair_names_its_place; the site's tests (48), lint and build pass for the edited cli.md (site-b6.log)."
result: passed
evidence_kind: product-test
revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
content: 34f38b9e1b232742f0ff71cb8b2ba7423b674b99002a4ecf6eeacfe19549f007
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:2c30d575c65d88b625a1db28a9b22d2468bb4f48c3c4bf929f470772c26ee56e
external: ext-d214f1
criteria: [CHANGE-13.e]
requirements: [REQ-change-relay-repair-001]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T08:25:34Z
actor:
  - agent: claude-code
    terminal: t59
---
**At bf0014ad (#998, #972 and #974 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 19m36s on a machine also running another lane: fmt, lint and check-types, cargo test (unit 3082 passed, 2 ignored; integration 627 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes repair_puts_back_a_damaged_reopening_relay_from_head, repair_removes_a_damaged_correction_relay_with_no_committed_copy and a_nested_repair_names_its_place; the site's tests (48), lint and build pass for the edited cli.md (site-b6.log).**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision bf0014ada677, observed 2026-11-19T08:25:34Z.
