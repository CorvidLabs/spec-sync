---
change: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
artifact: testing
---

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
