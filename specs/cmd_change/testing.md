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

REQ-cmd-change-016: report-level regressions assert a preserved-content squash completes the product stage, stale content with ancestor evidence does not, and missing evidence does not. The first two failed against the previous implementation. Existing report/finalize agreement checks continue to enforce stale and unavailable review behavior. Run `cargo test --bin specsync commands::change::tests`.

REQ-cmd-change-017: `check_commit_never_commits_an_unrelated_untracked_file` and `ship_push_never_commits_an_unrelated_untracked_file` drive `run_checked_commit` and `run_ship --push` (into a bare remote) over a tree holding `debug-dump.zip`, `.agents/scratch.md` and `exp2.sh`. They require every stray to be absent from all history, or from the remote's history for ship, and still untracked and unmodified. Both fail when staging is put back to `git add -A`. The ship test also fails when only the archive commit is put back, so it guards that path on its own. The controls in the same tests require the change's untracked workspace, the tracked delivery edit and the archive package to be committed, and the vacated workspace to be gone from the pushed tree. `staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits` pins the porcelain parse: a staged rename is one entry, and it fails if the rename's source field is read as an entry of its own. `a_lifecycle_scope_owns_its_subtree_and_not_a_prefix_sibling` pins the ownership boundary, and `the_left_out_warning_names_the_files_and_what_to_do` pins the warning's listing, its bound, the re-check guidance and once-per-run disclosure. REQ-cmd-change-012 is covered by `lifecycle_commit_raises_a_stale_ledger_before_staging_it`. Run `cargo test --bin specsync commands::change::tests`.
