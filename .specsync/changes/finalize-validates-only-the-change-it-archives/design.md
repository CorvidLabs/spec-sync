---
change: finalize-validates-only-the-change-it-archives
artifact: design
---

# Design

1. `list_all_changes_structural` loads the same roster as `list_all_changes_checked`: active records through `list_changes_checked` (their history still validated), each archived `state.json` parsed and passed through `validate_loaded_change`, husks and tombstones skipped, duplicate IDs refused. It skips only `validate_workflow_version_history` and the history prefetch for archived records. Both loaders share one body, `list_all_changes_loaded(root, walk_archived_history)`, so the structural checks cannot drift apart.
2. `AcceptedInputValidationMemo::authenticate` validates an archived record's own workflow-version history before authenticating its accepted evidence. That is the one choke point through which an accepted-input graph pass relies on another archive: a semantic successor in `successor_covers_input`, or a sequence owner in `later_sequence_owner_covers_historical_input`. On the full roster this cannot fail, because loading the roster already proved the same verdict (and memoized it in a read scope). On the structural roster it proves exactly the archives the pass leans on, and no others.
3. `audit_project_with_strict(root, strict)`: under `strict` it adds `archived_workflow_history_errors`, which prefetches and validates every archived record's workflow-version history and reports each failure by change ID. `change audit` passes the global `--strict` flag. `audit_project` and the non-strict audit are unchanged.

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

Unchanged. `finalize_change` still runs `load_change` (its own history), the state, workflow-version and materialization checks, acceptance or finalization evidence and the scoped review. `archive_change_with_options` still runs the closing approval, delivery-diff, accepted-transition, integrity preflight and post-move validation for the target. The only refusal that moves is a history failure in an archive the target does not rely on: today it aborts finalize from inside the roster load, and now `change audit --strict` reports it by ID. A relied-on archive with a broken history is refused as a successor through (2), so the target's stale input is still refused, with that successor named among the rejected successors.

## What still scales with archives

The archive-introduction index behind the closing check reads each immutable commit that introduced an archive (`diff-tree`, its `state.json`, its `approvals.json`). That is three Git processes per archive when the persistent read cache is cold and none when it is warm (REQ-change-119 caches them per commit). It is not a history walk, so this change leaves it alone; batching it would touch the partial-clone conclusiveness rules for no proven need.
