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

REQ-cmd-change-023: `status_and_show_json_report_approved_delta_requirements` in `tests/integration/change.rs` drives `change status --json`, `change show --json`, `change depend --json` and `change list --json` through the built binary. It requires the `approved_deltas` key for a draft (`[]`) and an approved change, CRLF to stay intact, an edited body to report only `delta_intact: false`, `materialized: true` after `change check`, unchanged text output and no key in `change list --json`. It also requires the top-level and `summary` key sets of the draft, approved and `change new --json` outputs to equal the sets a80748f prints for the same fixture, apart from the new key, and `change new --json` to carry `approved_deltas == []`. It fails on a80748f, where the key is absent. Run `cargo test --test integration change::status_and_show_json_report_approved_delta_requirements`.
