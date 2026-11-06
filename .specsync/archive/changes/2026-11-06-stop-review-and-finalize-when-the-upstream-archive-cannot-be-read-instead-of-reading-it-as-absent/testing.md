---
change: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
artifact: testing
---

# Testing

- `src/change_tests/upstream_archive.rs::an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent`: archive the change, publish it as `origin/main`, branch from before it, then delete the archived `state.json` blob from the clone. `upstream_archive_of` must report it and the message must say it cannot tell, name `origin/main` and `git fetch`. Fails on a6116e0c (returns `None`).
- `tests/integration/archived_upstream.rs::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read`: the auditor's squash fixture with the same missing blob. `change review` and `change finalize` must fail, name the cannot-tell refusal and write no archive. Fails on a6116e0c (review accepts).
- The existing upstream-archive unit and integration tests (found, absent, renamed) and `lifecycle_commands_stay_within_git_spawn_ceilings` stay green.
- `src/change_tests/upstream_archive.rs::upstream_archive_is_found_in_a_project_directory_named_like_pathspec_magic` (#962): the project is renamed to `:!app` on both branches. The remote default's archive must be found with its commit. Fails on a6074bf9 (the plain pathspec listing failed).

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-142 | `an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent` (CannotTell naming `origin/main` and `git fetch`); `review_and_finalize_stop_when_the_upstream_archive_cannot_be_read` (review and finalize refuse, state stays `verifying`, no archive, status not `safe`, status next action and ship-status carry the refusal); `upstream_archive_is_found_in_a_project_directory_named_like_pathspec_magic` (#962) |
