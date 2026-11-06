# Lesson bundle: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Find an upstream archive by the id in its state record rather than its directory name
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/upstream_archive.rs, CHANGELOG.md
- **Acceptance**: `upstream_archive_of` finds the remote default's archive of a change by the `id` in each package's `state.json`, as `find_change_dir` does, so a renamed or undated package is found like a `YYYY-MM-DD-<id>` one.
- **Acceptance**: The lookup stays bounded: one `git ls-tree -r` of the remote default's archive directory and one `git cat-file --batch` read of the `state.json` blobs, then the adding commit and one ancestry check only for a matching package. It never rebuilds the full archive index, and the #861 lifecycle spawn-ceiling test still passes.
- **Acceptance**: A unit test with a renamed upstream package fails on main first and passes after. The existing upstream-archive tests stay green.
- **Acceptance**: Every other behavior of the upstream-archive refusal is unchanged.

## Evidence

- Verification commit: `5ef5ed1b69515fb03ae75ac7b04158e2b76a17fe`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

spec-sync#907, a review follow-up from #889. `upstream_archive_of` lists the remote default's archive directory and only reads entries named exactly `YYYY-MM-DD-<id>`. `find_change_dir` identifies archives by the `id` inside `state.json`, whatever the directory is called, so a renamed or undated package is missed and its stale active copy gets ordinary status guidance instead of the upstream-archive refusal.

Reproduced on main cd8cab5e by `upstream_archive_is_found_by_its_state_record_not_its_directory_name`: the change is archived, its package renamed to `legacy-package` and published as `origin/main`; from a branch before it, `upstream_archive_of` returns `None`.

Constraint from #889's review: the lookup must not rebuild the full archive index on the status path (lifecycle spawn ceiling, #861).

## From the change's testing.md

# Testing

`src/change_tests/upstream_archive.rs::upstream_archive_is_found_by_its_state_record_not_its_directory_name` fails on main cd8cab5e ("a renamed upstream package must still be found by its state record"). `upstream_archive_is_reported_only_when_head_does_not_contain_it`, the `archived_upstream` integration tests and `lifecycle_commands_stay_within_git_spawn_ceilings` must stay green.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-139 | `upstream_archive_is_found_by_its_state_record_not_its_directory_name` (renamed package found by its state record), `upstream_archive_is_reported_only_when_head_does_not_contain_it`, `archived_upstream` integration tests, `lifecycle_commands_stay_within_git_spawn_ceilings` (bounded Git processes) |

## Where these lessons go

- `specs/change/change.3md`
