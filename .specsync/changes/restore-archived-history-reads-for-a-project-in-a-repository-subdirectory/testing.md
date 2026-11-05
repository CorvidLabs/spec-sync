---
change: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
artifact: testing
---

# Testing

Both tests live in `src/change_tests/fast_archived_reads.rs`. Each builds a Git repository, puts the SpecSync project in `app/`, and asserts `git rev-parse --show-prefix` is `app/`.

`subdirectory_project_refuses_a_hidden_workflow_downgrade_live_and_cached` archives one workflow-v2 change, warms the read cache, then commits a workflow-version downgrade and restores the original bytes. The live status read and a scoped read must both refuse with `workflow-version history changed immutable identity`. On main 40dcf33a the live read returned status JSON instead.

`subdirectory_project_follows_an_earlier_archive_path_through_a_hidden_downgrade` commits that downgrade on the first dated archive path, reopens the package, and rearchives it on a later date. `workflow_version_state_paths` must name both dates, and `load_change` must refuse the downgrade. On main the earlier date was absent from the path list.

The existing repository-root checks stay in place: `workflow_version_downgrade_then_revert_remains_invalid`, `workflow_version_history_follows_cross_date_rearchive_paths`, and `cached_archived_history_still_refuses_a_later_hidden_downgrade`.
