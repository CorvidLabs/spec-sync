---
change: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
artifact: testing
---

# Testing

- Unit tests in `src/change_tests/relayed_decision.rs`: a damaged nested reopening relay is put
  back from `HEAD`; a damaged correction-embedded relay with no committed copy is removed, and the
  correction stays; the report names `within` and the printed place (fail on main: nothing is
  repaired).
- The existing repair-relays tests still pass.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-relay-repair-001 | `src/change_tests/relayed_decision.rs`: the nested reopening and correction repair tests |
| REQ-cmd-change-relay-repair-001 | `src/change_tests/relayed_decision.rs`: the report's `within` and the printed place for nested records |
