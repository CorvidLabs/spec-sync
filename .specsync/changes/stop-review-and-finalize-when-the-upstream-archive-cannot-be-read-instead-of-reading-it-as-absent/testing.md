---
change: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
artifact: testing
---

# Testing

- `src/change_tests/upstream_archive.rs::an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent`: archive the change, publish it as `origin/main`, branch from before it, then delete the archived `state.json` blob from the clone. `upstream_archive_of` must report it and the message must say it cannot tell, name `origin/main` and `git fetch`. Fails on a6116e0c (returns `None`).
- `tests/integration/archived_upstream.rs::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read`: the auditor's squash fixture with the same missing blob. `change review` and `change finalize` must fail, name the cannot-tell refusal and write no archive. Fails on a6116e0c (review accepts).
- The existing upstream-archive unit and integration tests (found, absent, renamed) and `lifecycle_commands_stay_within_git_spawn_ceilings` stay green.
