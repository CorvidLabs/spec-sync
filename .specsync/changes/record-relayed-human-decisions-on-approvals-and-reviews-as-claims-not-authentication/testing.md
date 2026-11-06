---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: testing
---

# Testing

CLI tests in `tests/integration/relayed_decision.rs` drive the built binary through approve, check, review and finalize. Unit tests in `src/change_tests/relayed_decision.rs` cover the record type and the review digest.

| Criterion | Test |
| --- | --- |
| CHANGE-13, REQ-change-relayed-decision-001 | a relayed approval records all four fields, with `decided_by` taken from `--actor` |
| CHANGE-13.a | an approval and a review without the flags omit `relayed_by` |
| CHANGE-13.b, 13.c, REQ-cmd-change-relayed-decision-001 | status text and JSON show the approval and review claims, the relay and the boundary sentence |
| CHANGE-13.d | one or two flags are refused naming the missing ones, nothing is written, and the named re-run succeeds; a relay without `--actor` is refused |
| SHIP-2.e | a relayed review records the relay, finalizes, and shows it in status |
| CHANGE-6.d, REQ-change-001 | status labels the approval's actor a recorded claim, not authenticated |
| Review digest ("Keep it out") | finalization's `review_digest` equals the digest of the review with `relayed_by` removed |
| Read-side refusal | a hand-edited relay (`null`, unknown key, blank) is refused naming `git checkout`, and running that command recovers |
| REQ-cli-args-relayed-decision-001 | grammar test for the three flags on approve and review |

The tests are written first and fail against `b179a4f2` (unknown flags, missing record). `fledge lanes run verify` runs before review.
