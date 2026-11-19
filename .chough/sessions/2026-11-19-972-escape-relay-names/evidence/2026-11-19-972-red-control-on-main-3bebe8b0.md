---
kind: evidence
id: 2026-11-19-972-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), the_decider_mismatch_error_escapes_both_names fails: the error for a relayed decision that names another decider prints a raw escape sequence from the name. This is the defect #972 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: bf0014ada6776d75cfeb2ebc6ba9e1601bc16585
content: 34f38b9e1b232742f0ff71cb8b2ba7423b674b99002a4ecf6eeacfe19549f007
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:10bbc70c68b81697eb3a09b6bb654417f08c64270d9c887f268a3695fcdd5d84
external: ext-e2303c
criteria: [CHANGE-13.d]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T07:53:34Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with the three 11-13 failing-test commits applied (890c154a, 5d86b391, 2e32795b: the tests, plus 890c154a's compile scaffolding, a 'within' field that is always empty and a 'place' method that keeps main's wording), the_decider_mismatch_error_escapes_both_names fails: the error for a relayed decision that names another decider prints a raw escape sequence from the name. This is the defect #972 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision bf0014ada677, observed 2026-11-19T07:53:34Z.
