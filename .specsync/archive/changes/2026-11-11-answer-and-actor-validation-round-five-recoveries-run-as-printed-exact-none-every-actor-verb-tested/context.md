---
change: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested
artifact: context
---

# Context

PR #1026 round 5. The second independent review, at `6c579737`, approved at about 85% with should-fixes:

- Owner recovery printed `answer affected_specs core` for a configured owner with no spec, which the round-3 existence check refuses.
- Empty `constraints` lost its `none` hint.
- Registered-but-missing specs and unreadable registries got misleading or no messages.
- `change new --spec` and the bootstrap exemption were not in the spec text.
- No tests covered the other actor verbs.
- `none` could collide with a real id or module.
- U+200B, U+FEFF and U+200E/F were accepted.

All are fixed here. The review found no data-loss or ledger-corruption path. Leif's standing bug-fix rule applies, and REQ-change-150 is this PR's own requirement.
