# Lesson bundle: finalize-validates-only-the-change-it-archives

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Finalize validates only the change it archives
- **Kind**: BugFix
- **Specs**: change, cmd_change, cli_args
- **Paths**: src/change/validate_loaded_change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/historical_reconciliation.rs, src/change/read_cache.rs, src/change_tests/fast_finalize.rs, src/commands/change.rs, src/cli.rs, specs/change, specs/cmd_change, specs/cli_args, docs/ADOPTING.md, site/src/content/docs/cli.md
- **Acceptance**: One finalize spawns no more Git processes when more archived changes exist; every refusal finalize issues for the change it archives is unchanged; change audit --strict still names a hidden workflow-version downgrade in another archive; a Raven-sized finalize finishes in seconds.

## Evidence

- Verification commit: `1a739858ae5ea8ef43068494a96aeb759958e4ad`
- Base commit: `0723bdca6c7d592baff6fdd8dba84b2de07c875a`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

Issue #856. `change finalize` and `change ship` re-validated every other archived change's workflow-version history each time they archived one change, so their cost grew with the number of archives and with the length of history. On this repository (about 130 archives) `change ship` took 3 to 11 minutes while `change check --commit` took about 30 seconds, and sampling showed back-to-back `git` processes from the archive walk. A downstream consumer's CI Archive job grew with the commits in the PR and timed out at 91 commits. #840 (read cache) and #842 (finalize memo, REQ-change-121) made the walk cheaper but kept it. The walk lives in `list_all_changes_uncached`, which calls `validate_workflow_version_history` (and the parallel prefetch) on every archived record.

This branch stacks on #842 (`perf/fast-finalize`). Leif asked for it in 6.2.0. Already-archived changes were validated when they were archived, so the per-archive walk protects nothing new during a finalize. The issue moves it to `change audit --archives` and keeps every refusal for the change being archived.

## From the change's design.md

# Design

1. `list_all_changes_structural` loads the same roster as `list_all_changes_checked`: active records through `list_changes_checked` (their history still validated), each archived `state.json` parsed and passed through `validate_loaded_change`, husks and tombstones skipped, duplicate IDs refused. It skips only `validate_workflow_version_history` and the history prefetch for archived records. Both loaders share one body, `list_all_changes_loaded(root, walk_archived_history)`, so the structural checks cannot drift apart.
2. `AcceptedInputValidationMemo::authenticate` validates an archived record's own workflow-version history before authenticating its accepted evidence. That is the one choke point through which an accepted-input graph pass relies on another archive: a semantic successor in `successor_covers_input`, or a sequence owner in `later_sequence_owner_covers_historical_input`. On the full roster this cannot fail, because loading the roster already proved the same verdict (and memoized it in a read scope). On the structural roster it proves exactly the archives the pass leans on, and no others.
3. `audit_project_with_archives(root, strict)`: under `strict` it adds `archived_workflow_history_errors`, which prefetches and validates every archived record's workflow-version history and reports each failure by change ID. `change audit` passes the global `--strict` flag. `audit_project` and the non-strict audit are unchanged.

## Per-caller decisions for `list_all_changes_checked`

Switched to the structural roster:

- `archive_change_with_options`, the `simulated` roster, used by both `change finalize` and `change archive`. It is the roster for the post-move preflight over accepted candidates and for duplicate-ID detection. The change being archived is validated in full: `load_change` walks its own history, `validate_archived_integrity_closing` checks its package, and the projection is that record itself. Other archives appear only as successor candidates, and any one relied on proves its history through (2).
- `ensure_closing_approval_valid` as called from `archive_change_with_options`, through a new `ensure_archiving_closing_approval_valid`. Same reasoning: it validates the target's accepted inputs against the roster. The other callers of `ensure_closing_approval_valid` (`correct_interview_metadata` and the status and handoff summaries) keep the full roster.

Kept on the full roster, each because an existing contract or a non-finalize path depends on it:

- `ensure_no_sequence_collision` (approve, audit): REQ-change-049 requires approval and `change audit` to refuse same-base ordinal collisions. It returns before loading anything for slug IDs and is not on the finalize path.
- `validate_workflow_v1_records_at_adoption_cutoff` (`change adopt`): adoption is rare and its contract is the full roster.
- `terminal_evidence_summary` and `archived_terminal_evidence_in_scope` (status, show, list, ship-status): REQ-change-119 requires read output identical to a live read, including the first error.
- `load_legacy_archive_baseline_context`: reached only for legacy archives with no accepted snapshot, never for a workflow-v2 target.
- `reopen_unarchived_change` (`change reopen`): a recovery path, outside this issue.
- `check_project_with_command_output` (`check_project`, and the successor candidates of `audit_project`): the audit surface, where the history walk is meant to stay.

## Refusals for the change being archived

Unchanged. `finalize_change` still runs `load_change` (its own history), the state, workflow-version and materialization checks, acceptance or finalization evidence and the scoped review. `archive_change_with_options` still runs the closing approval, delivery-diff, accepted-transition, integrity preflight and post-move validation for the target. The only refusal that moves is a history failure in an archive the target does not rely on: today it aborts finalize from inside the roster load, and now `change audit --archives` reports it by ID. A relied-on archive with a broken history is refused as a successor through (2), so the target's stale input is still refused, with that successor named among the rejected successors.

## What still scales with archives

The archive-introduction index behind the closing check reads each immutable commit that introduced an archive (`diff-tree`, its `state.json`, its `approvals.json`). That is three Git processes per archive when the persistent read cache is cold and none when it is warm (REQ-change-119 caches them per commit). It is not a history walk, so this change leaves it alone; batching it would touch the partial-clone conclusiveness rules for no proven need.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-124 | `cold_finalize_git_processes_do_not_grow_with_other_archives` failed first on the #842 base (five more archives added 329 Git processes to one cold finalize, about 82 each) and now adds 15, the three reads of each new archive's immutable introducing commit. `warm_finalize_git_processes_do_not_grow_with_other_archives` failed first (five more archives added 20 Git processes to one warm finalize) and now adds none (754 and 754). `finalize_judges_other_archives_from_their_own_bytes` failed first (finalize aborted on another archive's hidden workflow-version downgrade) and passes. `strict_audit_still_refuses_a_hidden_downgrade_in_another_archive` proves `audit_project_with_archives` names the downgraded archive, with a warm cache as well, and that the default audit is unchanged. `finalize_git_processes_do_not_grow_with_other_archives` (REQ-change-121) and the full change suite, including every finalize and archive refusal test, pass unchanged. |
| REQ-cmd-change-027 | `change audit` passes its `--archives` flag to `change::audit_project_with_archives`; `strict_audit_still_refuses_a_hidden_downgrade_in_another_archive` proves the strict report names the archived change and the non-strict report equals `audit_project`'s. |
| REQ-cli-args-022 | `change_audit_archives_is_opt_in_and_independent_of_strict` (src/cli.rs) parses `change audit` with and without `--archives` and `--strict`; `specsync change audit --help` lists `--archives`. |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
