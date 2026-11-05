# Lesson bundle: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Restore archived history reads for a project in a repository subdirectory
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/fast_archived_reads.rs, specs/change/
- **Acceptance**: A project rooted in a subdirectory of its Git repository refuses a workflow-version downgrade that a later commit restored, both on a live read and from the warm read cache.
- **Acceptance**: That same project still follows an earlier dated archive path, so a downgrade that exists only on that path is refused after reopen and rearchive.

## Evidence

- Verification commit: `cd802912135f000127bec2b5d4ac66d56be6613c`
- Base commit: `40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #851, found while reviewing #840. It is also on main. History queries build a repository-relative path with `git_repo_prefix`, then run Git with the project root as the working directory. Git resolves a pathspec from that directory unless the pathspec carries `:(top)`. When the prefix is non-empty, the prefix is applied twice and the query matches nothing.

Two reads then agree on an empty archived history. `workflow_version_state_paths` never sees earlier dated archive paths, and the walk of the paths it does return never sees the commits that touched them. `lifecycle_paths_touched_since` never sees a later edit, so a cached passing verdict stays current. A workflow-version downgrade that a later commit restored is invisible, live and from the cache.

`archive_introduction_index` already anchors the same kind of path with `:(top,literal)`. `git show <commit>:<path>` is already relative to the repository root and is left alone. The fix is the same anchor: `:(top,glob)` for the dated-archive discovery glob, `:(top,literal)` for the history walk of those paths and for the cache invalidation walk.

A fixture whose project is `app/` inside the repository is what makes the double prefix visible. The same checks at the repository root stay green either way.

## From the change's testing.md

# Testing

Both tests live in `src/change_tests/fast_archived_reads.rs`. Each builds a Git repository, puts the SpecSync project in `app/`, and asserts `git rev-parse --show-prefix` is `app/`.

`subdirectory_project_refuses_a_hidden_workflow_downgrade_live_and_cached` archives one workflow-v2 change, warms the read cache, then commits a workflow-version downgrade and restores the original bytes. The live status read and a scoped read must both refuse with `workflow-version history changed immutable identity`. On main 40dcf33a the live read returned status JSON instead.

`subdirectory_project_follows_an_earlier_archive_path_through_a_hidden_downgrade` commits that downgrade on the first dated archive path, reopens the package, and rearchives it on a later date. `workflow_version_state_paths` must name both dates, and `load_change` must refuse the downgrade. On main the earlier date was absent from the path list.

The existing repository-root checks stay in place: `workflow_version_downgrade_then_revert_remains_invalid`, `workflow_version_history_follows_cross_date_rearchive_paths`, and `cached_archived_history_still_refuses_a_later_hidden_downgrade`.

`glob_metacharacter_project_directory_follows_an_earlier_archive_path` runs the same earlier-archive scenario with the project in `app[1]/`. Before the escape, the dated-archive glob read `[1]` as a character class and the earlier path was missing. With only the escape, `git show <commit>:<path>` for a path absent at that commit fell back to a pathspec, printed nothing and exited 0, so the walk refused with a duplicate-state error instead of the downgrade. The history reads now pass `--` after the object.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-043 | `subdirectory_project_follows_an_earlier_archive_path_through_a_hidden_downgrade`, `glob_metacharacter_project_directory_follows_an_earlier_archive_path`, `subdirectory_project_refuses_a_hidden_workflow_downgrade_live_and_cached`, `workflow_version_history_follows_cross_date_rearchive_paths`, `workflow_version_downgrade_then_revert_remains_invalid` |
| REQ-change-119 | `subdirectory_project_refuses_a_hidden_workflow_downgrade_live_and_cached`, `cached_archived_history_still_refuses_a_later_hidden_downgrade` |

## Where these lessons go

- `specs/change/change.3md`
