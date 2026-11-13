---
change: a-malformed-relay-in-a-correction-gets-the-repair-relays-recovery
artifact: testing
---

# Testing

- Unit `a_malformed_correction_relay_is_refused_with_its_recovery` in
  `src/change_tests/relayed_decision.rs`: a malformed embedded relay, then one naming someone other
  than its approval's actor, are each refused naming `repair-relays`, which then fixes them (fails
  without the fix: no recovery is named).
- The existing correction and relay tests still pass.
