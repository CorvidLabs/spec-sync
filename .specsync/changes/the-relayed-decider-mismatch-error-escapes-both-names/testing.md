---
change: the-relayed-decider-mismatch-error-escapes-both-names
artifact: testing
---

# Testing

- Unit `the_decider_mismatch_error_escapes_both_names` in `src/change_tests/relayed_decision.rs`:
  on write, a relay whose `decided_by` holds `ESC[2J` and a newline is refused with both shown
  escaped; on read, a hand-edited `decided_by` holding `ESC[31m` and a newline is refused the
  same way (fails on main: the raw escape is in the error).
- The existing one-decider and repair-relays tests still pass.
