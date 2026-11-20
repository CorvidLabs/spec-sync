---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: testing
---

# Testing

Failing first: on main, the S1 and S2 fixtures read `corrupt-history` for a change whose own
evidence is intact, as tester2 found.

- `src/change_tests/terminal_evidence_isolation.rs`
  - S1: a change archived before another change is left open and archived at one commit, then
    cleaned up. It reads `authenticated-history`, through the scoped read (warm cache) and the
    unscoped one.
  - S2: a change created and archived after that commit and its cleanup. The same.
  - An accepted change whose inputs are unchanged reads `exact` beside the broken change.
  - The broken change's own status still refuses, naming its failure.
  - A broken change that declares an accepted change's changed input: `stale`, naming it.
  - An unreadable `state.json`: counted in a successor search, not otherwise.
  - A retired change whose replacement fails: its own refusal, as today.
- `tests/integration/`: through the binary on the S1 fixture, `change status`, `change list`,
  `change ship-status` and `change audit --archives`; `check`, `review` and `finalize` of a third
  change, with main's exit codes and messages.
- The whole suite, since every status read goes through `terminal_evidence_summary`.
