---
spec: cmd_change.spec.md
---

# Testing

CLI integration coverage validates creation, JSON schema, rationale errors, adoption dry runs, initialization, and the complete stale accepted → reopen → verify → reaccept flow. Reopen assertions cover deterministic audit JSON and preserved approval/reopen ledger history. Domain transitions are covered by change-module unit tests.

`REQ-cmd-change-002` is covered by the accepted → correct → approve → verify → reaccept CLI integration flow. It asserts equivalent text and JSON original/effective projections, correction history, added-artifact next actions, and persisted append-only evidence.

`REQ-cmd-change-003` is covered by the reopened → correct-owner CLI integration flow. It asserts deterministic JSON persistence, equivalent human output, required audit inputs, exact path/spec ownership validation, next-gate guidance, and transactional rejection.

`REQ-cmd-change-004` is covered by the batch correct-owner CLI integration flow. It asserts repeated-path batch success, atomic rejection when any entry is invalid, and deterministic JSON persistence of every appended correction.

`REQ-cmd-change-010` is covered by an invalid-ledger CLI regression that invokes answer, depend,
and supersede, requires the safe integrity diagnostic, and compares every lifecycle file
byte-for-byte before and after each rejected mutation. The change-domain lock-race regression
proves an answer mutation revalidates after acquiring the persistence lock when the ledger becomes
invalid while it waits. A command-unit regression then corrupts the ledger after a successful
mutation and proves both text and JSON rendering succeed from the transaction snapshot instead of
reporting a false failure after persistence. It also proves a live summary becomes invalid after
corruption while both transaction-captured normal/strict summaries remain valid. CodeQL enforces
that correction-ledger-derived values remain confined to the JSON branch; text-only counts come
from an independent state reload.

REQ-cmd-change-021: `status_reads_the_same_broken_policy_error` writes `{` to `.specsync/sdd.json` and requires `configured_principles_file` to fail with `invalid SDD policy`. `change status` returns that error before it prints a briefing.

REQ-cmd-change-022: `finalize_commits_the_archive_move` finalizes a reviewed change through `cmd_change` and requires `git log -1` to be `chore(lifecycle): archive <id>` and porcelain to no longer show `.specsync/changes/<id>`.

REQ-cmd-change-016: report-level regressions assert a preserved-content squash completes the product stage, stale content with ancestor evidence does not, and missing evidence does not. The first two failed against the previous implementation. Existing report/finalize agreement checks continue to enforce stale and unavailable review behavior. Run `cargo test --bin specsync commands::change::tests`.

REQ-cmd-change-017: `check_commit_never_commits_an_unrelated_untracked_file` and `ship_push_never_commits_an_unrelated_untracked_file` drive `run_checked_commit` and `run_ship --push` (into a bare remote) over a tree holding `debug-dump.zip`, `.agents/scratch.md` and `exp2.sh`. They require every stray to be absent from all history, or from the remote's history for ship, and still untracked and unmodified. Both fail when staging is put back to `git add -A`. The ship test also fails when only the archive commit is put back, so it guards that path on its own. The controls in the same tests require the change's untracked workspace, the tracked delivery edit and the archive package to be committed, and the vacated workspace to be gone from the pushed tree. `staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits` pins the porcelain parse: a staged rename is one entry, and it fails if the rename's source field is read as an entry of its own. `a_lifecycle_scope_owns_its_subtree_and_not_a_prefix_sibling` pins the ownership boundary, and `the_left_out_warning_names_the_files_and_what_to_do` pins the warning's listing, its bound, the re-check guidance and once-per-run disclosure. REQ-cmd-change-012 is covered by `lifecycle_commit_raises_a_stale_ledger_before_staging_it`. Run `cargo test --bin specsync commands::change::tests`.

REQ-cmd-change-005 (approval currency in text `Next:`): `text_next_action_names_reapproval_when_an_approved_delta_was_edited` and `text_next_action_names_reapproval_when_the_approved_scope_changed` are DISCRIMINATORS that fail on f240d37, where text said `change check` for both. The first also requires text and JSON to name the same action and `change check` to refuse the edit. `text_next_action_names_check_for_a_current_approval` is the CONTROL for a current approval, with and without a delta. The text half of the integration test `status_json_and_text_report_an_edited_approved_delta` checks the printed `Next:` and `Handoff: not yet` lines. Run `cargo test --bin specsync commands::change::tests`.

REQ-cmd-change-024 (`verification_ignored_paths` disclosure): `ship_status_blocks_on_an_invalid_verification_ignored_paths_entry` is a DISCRIMINATOR; on a80748ff the blockers said "verification evidence is stale" and "scoped review is stale" for a list of `["src/"]`. `status_and_check_text_render_the_list_only_when_non_empty` pins the two text lines and cannot compile on a80748ff. Through the binary, `listed_tool_records_do_not_stale_review_through_the_cli` checks `summary.verification_ignored_paths` and the text of `change status` and `change check --commit`, `invalid_verification_ignored_paths_are_refused_through_the_cli` checks the status next action and the ship-status blocker, and `absent_list_keeps_issue_825_behaviour` checks that nothing extra is printed without the key. Run `cargo test --bin specsync commands::change::tests` and `cargo test --test integration change::`.

REQ-cmd-change-023: `status_and_show_json_report_approved_delta_requirements` in `tests/integration/change.rs` drives `change status --json`, `change show --json`, `change depend --json` and `change list --json` through the built binary. It requires the `approved_deltas` key for a draft (`[]`) and an approved change, CRLF to stay intact, an edited body to report only `delta_intact: false`, `materialized: true` after `change check`, unchanged text output and no key in `change list --json`. It also requires the top-level and `summary` key sets of the draft, approved and `change new --json` outputs to equal the sets a80748f prints for the same fixture, apart from the new key, and `change new --json` to carry `approved_deltas == []`. It fails on a80748f, where the key is absent. Run `cargo test --test integration change::status_and_show_json_report_approved_delta_requirements`.


## Historical reconciliation evidence

REQ-cmd-change-025: Targeted lifecycle tests plus actual consumer CLI status/finalize JSON inspection verify explicit unproven-history disclosure; normal command output stays unchanged.

REQ-cmd-change-026: `change send-back` text, `--json` (`change` and `audit`) and non-zero draft refusal were inspected with the built binary on a scratch project; the command only delegates to `change::send_back_change`.

## Strict audit evidence

REQ-cmd-change-027: `change audit` passes its `--archives` flag to `change::audit_project_with_archives`. strict_audit_still_refuses_a_hidden_downgrade_in_another_archive (src/change_tests/fast_finalize.rs) proves the report names the archived change whose history hides a downgrade, and that the non-strict report equals `audit_project`'s.

## Recorded claims evidence

REQ-cmd-change-relayed-decision-001: status_shows_the_approval_and_review_claims_with_the_relay_and_its_boundary; a_change_with_no_approval_shows_no_claim; an_incomplete_relay_is_refused_before_anything_is_written_and_the_named_rerun_records_it (it runs the named re-run for approve and review); a_blank_or_oversized_relay_detail_is_refused_and_named; a_relayed_approval_needs_an_explicit_actor; relayed_words_are_escaped_in_text_status; a_refused_blocking_review_names_a_rerun_that_keeps_the_verdict, which runs the named re-run and finds a block, not a pass (#900 review) (tests/integration/relayed_decision.rs).

REQ-cmd-change-relay-repair-001 (#904): the text and `--json` assertions in a_damaged_review_relay_in_the_review_to_ship_window_is_repaired_by_the_named_command, a_damaged_relay_on_the_last_review_attempt_is_put_back_from_review_json, a_malformed_relay_record_is_refused_naming_repair_relays_and_it_puts_the_commit_back, a_damaged_relay_with_no_surviving_copy_is_removed_and_every_attempt_and_verdict_stays, a_damaged_approval_relay_before_its_first_commit_is_removed_and_the_approval_stays (the removed value is shown) and repair_relays_with_nothing_damaged_says_so_and_writes_nothing; the refusals in repair_relays_on_an_archived_change_names_git_checkout_and_that_command_recovers and repair_relays_on_a_committed_file_that_is_not_json_names_git_checkout_and_it_recovers exit non-zero and the named command recovers (tests/integration/relayed_decision.rs).


REQ-cmd-change-answer-hint-001 (#958): `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` and `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers` in `tests/integration/documented_workflows.rs` check text `Next:` and JSON `summary.next_action` after every approval refusal. Both name the exact `change answer` command the refusal named, including `constraints none`; the harness runs that command without rewriting a single-choice answer. Both failed first on db641967 and pass after text status recomputes the draft's open questions and both status forms use the domain helper.
