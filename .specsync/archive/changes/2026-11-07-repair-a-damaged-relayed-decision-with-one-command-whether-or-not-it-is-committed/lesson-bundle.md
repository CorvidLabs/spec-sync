# Lesson bundle: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Repair a damaged relayed decision with one command, whether or not it is committed
- **Kind**: BugFix
- **Specs**: change, cmd_change, cli_args
- **Paths**: src/change/relayed_decision.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change_tests/relayed_decision.rs, src/cli.rs, src/commands/change.rs, tests/integration/relayed_decision.rs, specs/change, specs/cmd_change, specs/cli_args, hi/change.md, INTENT.md, CHANGELOG.md, site/src/content/docs/workflow.md, site/src/content/docs/cli.md
- **Acceptance**: When one of those records is damaged, I am told one command that repairs it, whether or not it has been committed yet.
- **Acceptance**: That repair puts the original back when a copy of it still exists, and otherwise removes only the damaged record, never the approval or review it belongs to.
- **Acceptance**: The repair tells me what it put back and what it removed.

## Evidence

- Verification commit: `07f1739eb4b1647c8cfc702557be829fb7ced289`
- Base commit: `bc1859c4cfe1ee13fbfa54ad766a14fa7a2350c4`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

Issue #904, from a Codex thread on #900 (comment 4489111584 thread `PRRT_kwDORqhp_860YaI6`). #900 refuses a damaged `relayed_by` on read and every refusal names `git checkout HEAD -- <file>`. `change check --commit` commits `approvals.json` before review, but `change review` writes `review.json` and `review-attempts.json` afterwards, and nothing may be committed between review and ship. In that window `HEAD` holds neither file, so the named command fails. After a block, a fix and a new review, `HEAD` holds an older `review-attempts.json`, and `git checkout HEAD` would throw away the uncommitted attempts written since, one of which can be a block (REQ-change-046).

Leif approved CHANGE-13.e, 13.f and 13.g as drafted, the `specsync change repair-relays` command, and removing only the damaged relay when no copy survives, on 2026-11-06 between 06:10 and 06:19 MST in the Herdr orc tab, relayed by orc t18: "Ok yes I approve all".

Ruled out: refusing when no copy survives (the change stays unreadable, and the only way out deletes review files or hand-edits them); `git checkout` as the active-change recovery (fails in the window, and can erase newer attempts later); recording repairs in a history file (no gate reads a relay, so output is enough).

This branch is stacked on #900 and ships after it. If #900's REQ-change-relayed-decision-001 text or Public API rows change before it lands, these MODIFIED deltas are rebased onto them (CHANGE-11) and re-approved.

## From the change's design.md

# Design

- `with_relayed_read_recovery(root, record, path, message)` takes the change record. For an active change it names `specsync change repair-relays <id>` and says what it does; for an archived change it keeps `git checkout HEAD -- <file>`. Its callers in `load_approvals` and `load_scoped_review` pass the record they already hold.
- `repair_relayed_decisions(root, id) -> Result<RelayRepairReport, String>` takes the project lock and loads the change. An archived change is refused naming `git checkout HEAD -- <archived package>`. It reads `approvals.json` (records under `approvals`, decider `actor`), `review-attempts.json` (`reviews`, `reviewer`) and `review.json` (one record, `reviewer`) as JSON values, plus each file's copy at `HEAD` when there is one. A file that is not JSON is refused, naming `git checkout HEAD -- <file>` when `HEAD` holds it and otherwise saying no copy exists.
- A relay is damaged when `RelayedDecision::try_from` refuses it or its `decided_by` is not the record's decider. For each damaged one, the original is the relay of the first candidate that is the same record (every other field equal) and whose relay reads cleanly: the record at the same position at `HEAD`, then the twin (`review.json` ↔ the last attempt, both taken before any repair). An original without a relay puts back "no relay". With no candidate the damaged relay is removed and kept in the report.
- Each repaired file is converted back through its typed ledger and written with `json_content`, so a put-back from `HEAD` is byte-identical. All files are written once through `write_prepared_files`. If a repaired file still does not read, nothing is written. With no damage nothing is written.
- `RelayRepair { file, record, action, from, removed }` and `RelayRepairReport { change, repairs }` are `Serialize`; `from` and `removed` are omitted when absent.
- CLI: `ChangeAction::RepairRelays { id }`. Text prints `No relayed decision in <id> needs repair.` or one line per repair (`<place>: put back from <copy>`, or `<place>: removed, because no copy of the original exists; it was <escaped value>`), then a note that every approval, attempt and verdict was kept. `--json` prints the report.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
