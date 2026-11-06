---
change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
artifact: testing
---

# Testing

CLI tests in `tests/integration/relayed_decision.rs` drive the built binary. Each refusal test runs the command the refusal names and checks the result. They were written first and fail against `83db636b` (#900's head): five because the refusal names `git checkout HEAD`, three because `repair-relays` does not exist. The archived guard and #900's ten tests pass there.

| Criterion | Test |
| --- | --- |
| CHANGE-13.e, REQ-change-relay-repair-001 (window) | `a_damaged_review_relay_in_the_review_to_ship_window_is_repaired_by_the_named_command`: status refuses naming `repair-relays`, not `git checkout HEAD`; repair puts `review.json` back from attempt 1 byte for byte; `HEAD` is unchanged and `change finalize` succeeds |
| CHANGE-13.f (twin) | `a_damaged_relay_on_the_last_review_attempt_is_put_back_from_review_json` |
| CHANGE-13.f (HEAD) | `a_malformed_relay_record_is_refused_naming_repair_relays_and_it_puts_the_commit_back`: unknown key, `null`, wrong decider; bytes equal the commit |
| CHANGE-13.f (no copy), REQ-change-046 | `a_damaged_relay_with_no_surviving_copy_is_removed_and_every_attempt_and_verdict_stays`: the block attempt and its verdict stay; JSON report names the removed value |
| CHANGE-13.f (approval, no copy) | `a_damaged_approval_relay_before_its_first_commit_is_removed_and_the_approval_stays` |
| CHANGE-13.g, REQ-cmd-change-relay-repair-001 | the text and JSON assertions in the tests above, and `repair_relays_with_nothing_damaged_says_so_and_writes_nothing` |
| Archived | `an_archived_damaged_relay_still_names_git_checkout_and_that_command_recovers` (guard) and `repair_relays_on_an_archived_change_names_git_checkout_and_that_command_recovers` |
| Not JSON | `repair_relays_on_a_committed_file_that_is_not_json_names_git_checkout_and_it_recovers` |
| REQ-cli-args-relay-repair-001 | grammar test in `src/cli.rs` |

A unit test in `src/change_tests/relayed_decision.rs` covers record matching and that a clean record is never touched. `fledge lanes run verify` runs before review.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-relay-repair-001 | integration tests a_damaged_review_relay_in_the_review_to_ship_window_is_repaired_by_the_named_command, a_damaged_relay_on_the_last_review_attempt_is_put_back_from_review_json, a_malformed_relay_record_is_refused_naming_repair_relays_and_it_puts_the_commit_back, a_damaged_relay_with_no_surviving_copy_is_removed_and_every_attempt_and_verdict_stays, a_damaged_approval_relay_before_its_first_commit_is_removed_and_the_approval_stays, repair_relays_with_nothing_damaged_says_so_and_writes_nothing, repair_relays_on_an_archived_change_names_git_checkout_and_that_command_recovers, repair_relays_on_a_committed_file_that_is_not_json_names_git_checkout_and_it_recovers; unit test a_relay_is_put_back_only_from_a_clean_copy_of_the_same_record |
| REQ-change-relayed-decision-001 | the read-refusal bullet: a_read_relay_must_name_its_own_record_decider (an active change names repair-relays, not git checkout HEAD) and an_archived_damaged_relay_still_names_git_checkout_and_that_command_recovers; the other bullets keep #900's tests in src/change_tests/relayed_decision.rs and tests/integration/relayed_decision.rs |
| REQ-cmd-change-relay-repair-001 | the text and --json assertions in the integration tests above; the refusals exit non-zero and the named command recovers |
| REQ-cli-args-relay-repair-001 | unit test change_repair_relays_takes_only_the_change_id in src/cli.rs |
