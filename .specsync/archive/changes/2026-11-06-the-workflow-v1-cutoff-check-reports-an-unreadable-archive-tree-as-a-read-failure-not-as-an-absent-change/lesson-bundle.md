# Lesson bundle: the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The workflow-v1 cutoff check reports an unreadable archive tree as a read failure, not as an absent change
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs, specs/change, CHANGELOG.md, hi/speed.md, INTENT.md
- **Acceptance**: an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change passes: in an offline --filter=tree:0 clone whose cutoff archive tree was never fetched, the cutoff-tree check returns an error saying the archive tree at the cutoff could not be read, instead of Ok(false) (committed failing first in 45ef2006)
- **Acceptance**: A cutoff with no archive root at all still has no archived candidates and gives the same verdict as today
- **Acceptance**: The error is never cached: only a conclusive presence is stored under the cutoff, as before
- **Acceptance**: cutoff_tree_verdict_equals_the_history_walk_verdict and every existing refusal stay unchanged; no gate or bound is weakened

## Evidence

- Verification commit: `f0e3fb5505e8a4ec1d257294ae7c548b0c6d090d`
- Base commit: `45ef2006865ab6154d74c12ea50bea004cd46842`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #939, a Codex P2 on #918 that Leif deferred and then scheduled as its own fix. #918's cutoff-tree check (REQ-change-136) lists `<cutoff>:<archive root>` with `ls-tree` and reads the listing only when that call succeeds. Any failure, including a tree this clone doesn't hold (an offline `--filter=tree:0` partial clone), leaves only the active path as a candidate. The archived workflow-v1 change is then refused as "not present at the trusted pre-v2 cutoff". It fails closed (a refusal, never a false pass), but it names the wrong cause: the history walk it replaced surfaced the read failure.

Leif captured SPEED-1.a.3 for it on 2026-11-06 ("Capture both", relayed by orc t18): "When SpecSync cannot read the history it needs, it says it could not read it, rather than saying the change was not there." The orc assigned REQ-change-145.

## From the change's design.md

# Design

In `workflow_v1_record_present_in_cutoff_tree`, when `ls-tree -z --name-only <cutoff>:<archive root>` fails, tell "no archive root at the cutoff" apart from "an archive tree that can't be read". Ask whether the cutoff commit's tree lists the archive root: `ls-tree --full-tree -z --name-only <cutoff> -- <archive root>`. The command's batch reader already answers that from tree objects (#869), and it falls back to Git when a tree is missing.

- It succeeds and doesn't list the archive root: there is no archive at the cutoff. No archived candidates, the same verdict as today.
- It lists the archive root, or it fails: return `Err("the archive tree at the trusted pre-v2 cutoff <cutoff> could not be read: ...")` with Git's message. The error propagates through `validate_workflow_v2_baseline` as any read failure does, and nothing is cached, because only a conclusive presence is ever stored.

A successful listing takes the same path as today.

## From the change's testing.md

# Testing

- `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` (new; returns `Ok(false)` before the fix, `45ef2006`).
- Unchanged: `cutoff_tree_verdict_equals_the_history_walk_verdict` (a cutoff with legacy archives, a record added after the cutoff), `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`, the partial-clone tests, the integration test `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-145 | change test `an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change` (failing first, `45ef2006`); `cutoff_tree_verdict_equals_the_history_walk_verdict` unchanged |

## Where these lessons go

- `specs/change/change.3md`
