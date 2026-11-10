# Lesson bundle: the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The workflow-v1 cutoff check finds an archive by the record it holds, not only by a directory name ending in its id
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs, specs/change, CHANGELOG.md, hi/speed.md, INTENT.md
- **Acceptance**: cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff passes: a workflow-v1 archive moved before the cutoff to a directory not ending in -<id> is accepted by the cutoff-tree check exactly as the history walk accepts it (committed failing first in 14903d0b), and the change loads
- **Acceptance**: The cutoff-tree candidates include the archived record's own resolved package path (find_change_dir), as workflow_version_state_paths does, alongside the active path and the cutoff-tree entries ending in -<id>
- **Acceptance**: cutoff_tree_verdict_equals_the_history_walk_verdict and every existing refusal stay unchanged: a record absent at the cutoff, added after it, or present only under another id is still refused, and a non-canonical -<id> entry still refuses
- **Acceptance**: The persistent cutoff cache and the warm Git-process guards are unchanged; no gate or bound is weakened

## Evidence

- Verification commit: `a309e0d4b1758e75d98010e9d864ffee39c5a1ab`
- Base commit: `14903d0bb499e0ab6f2d3a83b539cacf026a1ed2`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #940, a Codex P2 on #918 that Leif deferred and then scheduled as its own fix. #918 (REQ-change-136) decides whether a workflow-v1 change existed at the trusted pre-v2 cutoff from the cutoff commit's tree. Its candidates are the active state path and the cutoff archive entries ending in `-<id>`. The history walk it replaced went through `workflow_version_state_paths`, which also includes the archived record's own resolved package path from `find_change_dir`, which finds an archive by the id its `state.json` declares. So a workflow-v1 archive moved before the cutoff to a directory such as `renamed-archive` was accepted before #918 and is refused after it. That is a false refusal, never a false pass.

Leif captured SPEED-1.a.2 for it on 2026-11-06 ("Capture both", relayed by orc t18): "Making a read faster never turns an archived change whose history is intact into a refusal." The orc assigned REQ-change-146.

## From the change's design.md

# Design

In `workflow_v1_record_present_in_cutoff_tree`, an archived record's candidates also include its own resolved package path, `find_change_dir(root, id)/state.json` made repository-relative, when it differs from the active path. That is the same list `workflow_version_state_paths` starts from. The record found there at the cutoff must still have the same id, workflow version 1 and origin `None` or `Some(1)`, so a different change's package can't satisfy it.

Nothing else changes: the cutoff listing, the non-canonical `-<id>` refusal, the persistent cache entry keyed by `(cutoff, id)` and only stored on presence, and the call site in `validate_workflow_v2_baseline`. Equivalence with the history walk: every path the walk could find that also existed at the cutoff is either the active path, a `*-<id>` entry of the cutoff's archive tree, or the record's own resolved path, so the candidate set now covers all three.

## From the change's testing.md

# Testing

- `cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff` (new; fails before the fix at "the cutoff-tree check must accept what the history walk accepts", `14903d0b`).
- Unchanged: `cutoff_tree_verdict_equals_the_history_walk_verdict`, `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`, `cached_archived_history_refuses_a_hidden_downgrade_in_a_renamed_archive`, the integration test `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-146 | change test `cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff` (failing first, `14903d0b`); `cutoff_tree_verdict_equals_the_history_walk_verdict` unchanged |

## Where these lessons go

- `specs/change/change.3md`
