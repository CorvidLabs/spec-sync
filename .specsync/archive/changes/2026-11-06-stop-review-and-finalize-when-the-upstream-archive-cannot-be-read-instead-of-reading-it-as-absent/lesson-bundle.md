# Lesson bundle: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Stop review and finalize when the upstream archive cannot be read instead of reading it as absent
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/upstream_archive.rs, tests/integration/archived_upstream.rs, CHANGELOG.md
- **Acceptance**: SHIP-3.c: when a remote default exists and its archived state records cannot be listed or read (a failed or truncated ls-tree or cat-file --batch, or a failed lookup of a matching package's addition or ancestry), `change review` and `change finalize` refuse before acceptance and write no archive.
- **Acceptance**: The refusal names the remote default reference, says the tool cannot tell whether the change is already archived there, and names how to find out: fetch the remote default, then list its archived packages for this id.
- **Acceptance**: `change status` and `change ship-status` report the same refusal, never `safe` or ready to finalize.
- **Acceptance**: A project with no remote default, or a remote default whose archive is read and holds no copy of the change, is unchanged: nothing is reported and review and finalize proceed.
- **Acceptance**: A unit test and an integration test that remove the archived record's blob from the clone fail on the base first and pass after.

## Evidence

- Verification commit: `fd96c9e1a3bfb3d05d3ba0647d91fc3eb2c6cd1f`
- Base commit: `ae3d7bcad107701057bc43667422c5abbed7ce55`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

spec-sync#946, a deferred Codex P2 on #943. `upstream_archive_of` guards against archiving a change a second time when the remote default already holds its archive (REQ-change-129, #889; matched by state record since REQ-change-139, #907). Every Git failure on that path ends in `None`: a failed or truncated `ls-tree`, a failed `cat-file --batch` (for example past the 64 MiB `MAX_GIT_COMMAND_OUTPUT_BYTES` cap, or a missing object in a partial or damaged clone), an unparsable batch header, or a failed `log`/`merge-base` for a matching package. `None` means "no upstream archive", so review, status and finalize carry on and can archive a stale copy again.

Human intent: SHIP-3.c, captured from Leif on 2026-11-06 ("Capture as written"): "When the tool cannot tell whether my change was already archived on the main branch, review and finalize stop and tell me how to find out, instead of archiving it a second time."

Reproduced on a6116e0c (#943's archive tip; this branch stacks on #943 until it merges) by removing the archived `state.json` blob from the clone: `cat-file --batch` answers `missing`, the header parse fails, and `upstream_archive_of` returns `None`; `change review` then accepts.

Scope decisions:
- No remote default (`remote_default_ref` is `None`) stays "nothing to report": there is no main branch to compare with.
- An archived `state.json` that reads but does not parse as a change record is still skipped, as before. It is not this change's record, and refusing on it would block every change in the project.
- Reading the batch in bounded chunks, so a very large archive can be read instead of refused, is not in this change. Past the cap the guard now refuses with a recovery rather than switching off.

## From the change's testing.md

# Testing

- `src/change_tests/upstream_archive.rs::an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent`: archive the change, publish it as `origin/main`, branch from before it, then delete the archived `state.json` blob from the clone. `upstream_archive_of` must report it and the message must say it cannot tell, name `origin/main` and `git fetch`. Fails on a6116e0c (returns `None`).
- `tests/integration/archived_upstream.rs::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read`: the auditor's squash fixture with the same missing blob. `change review` and `change finalize` must fail, name the cannot-tell refusal and write no archive. Fails on a6116e0c (review accepts).
- The existing upstream-archive unit and integration tests (found, absent, renamed) and `lifecycle_commands_stay_within_git_spawn_ceilings` stay green.
- `src/change_tests/upstream_archive.rs::upstream_archive_is_found_in_a_project_directory_named_like_pathspec_magic` (#962): the project is renamed to `:!app` on both branches. The remote default's archive must be found with its commit. Fails on a6074bf9 (the plain pathspec listing failed).

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-142 | `an_unreadable_upstream_archive_listing_is_reported_not_read_as_absent` (CannotTell naming `origin/main` and `git fetch`); `review_and_finalize_stop_when_the_upstream_archive_cannot_be_read` (review and finalize refuse, state stays `verifying`, no archive, status not `safe`, status next action and ship-status carry the refusal); `upstream_archive_is_found_in_a_project_directory_named_like_pathspec_magic` (#962) |

## Where these lessons go

- `specs/change/change.3md`
