---
change: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
artifact: testing
---

# Testing

Unit tests in `src/change_tests/relayed_decision.rs`, written first (fe9615c1); each failed on `main` b3ca5f97.

| Criterion | Test |
| --- | --- |
| CHANGE-13.h, REQ-change-relay-compare-001 (#926) | `approval_succession_ignores_a_relay_an_older_binary_dropped`: an older binary's reopen without `relayed_by` is a lawful successor; an edited approval or superseded approval is not |
| CHANGE-13.h, REQ-change-relay-compare-001 (#927) | `the_definition_digest_leaves_relays_in_corrections_out`: the digest is the same with a correction's relays present, edited or removed, and changes for another edit |
| CHANGE-13.h, REQ-change-relay-compare-001 (#971) | `trusted_correction_history_ignores_a_relay_an_older_binary_dropped`: a ledger rewritten without the embedded relay is the same history; an edited reason is divergence |

`fledge lanes run verify` runs before review.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-relay-compare-001 | unit tests approval_succession_ignores_a_relay_an_older_binary_dropped, the_definition_digest_leaves_relays_in_corrections_out and trusted_correction_history_ignores_a_relay_an_older_binary_dropped |
