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

Codex review of #900 (`e5716ab2`): the rerun named for an incomplete relayed review now keeps `--verdict` (and `--note`/`--portable-5-0-1` for approve); both correction digests hash embedded approvals without `relayed_by`; and a relay read back must name its record's own actor or reviewer. Tests: `a_refused_blocking_review_names_a_rerun_that_keeps_the_verdict`, `correction_digests_leave_embedded_relays_out` and `a_read_relay_must_name_its_own_record_decider`, plus a decider-mismatch case in the malformed-relay CLI test, which runs the recovery. The two unit tests fail with their fix removed. The uncommitted-`review.json` recovery gap is #904.

Second Codex round (`29a18a80`): the write APIs validate every relay field; the decider check covers every review attempt; append-only review-history comparisons ignore `relayed_by`, so an older binary's rewrite of earlier attempts is not read as removed evidence. Tests: `the_write_apis_validate_every_relay_field`, `every_review_attempt_relay_must_name_its_reviewer`, `review_history_comparisons_ignore_relays_but_not_other_edits`; all three fail with their fix removed.
