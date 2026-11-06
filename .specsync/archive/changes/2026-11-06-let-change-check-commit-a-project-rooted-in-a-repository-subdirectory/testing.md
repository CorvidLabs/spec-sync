---
change: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory
artifact: testing
---

# Testing

New file `tests/integration/subdirectory_check_commit.rs`: a Git repository whose SpecSync project is `app/`, one change approved for `src/auth.rs` with an `auth` delta, the implementation committed, then `change check --commit`. It must exit 0, commit the materialized `app/specs/auth/auth.spec.md`, and leave the change `verifying`. On main 884a33d3 it fails with the out-of-scope error.

A unit test on the path normalization covers the refusals: a path without the repository prefix, and a prefixed path that is not a candidate.

The repository-root control: the existing `change check --commit` integration tests stay green.

## Results

- `check_commit_completes_for_a_project_rooted_in_a_repository_subdirectory` failed on main 884a33d3 with "Git returned an out-of-scope modified path `app/specs/auth/auth.spec.md`" and passes with the change.
- `a_subdirectory_project_reads_diff_paths_relative_to_itself` and `a_path_outside_the_project_is_still_out_of_scope` cover the normalization and the refusals.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-128 | `check_commit_completes_for_a_project_rooted_in_a_repository_subdirectory`, `a_subdirectory_project_reads_diff_paths_relative_to_itself`, `a_path_outside_the_project_is_still_out_of_scope` |
