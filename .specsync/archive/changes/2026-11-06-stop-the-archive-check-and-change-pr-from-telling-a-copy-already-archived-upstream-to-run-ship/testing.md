---
change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
artifact: testing
---

# Testing

Failing tests first.

1. Rust unit (`commands::change::tests::an_upstream_archived_copy_is_not_told_to_ship`): a verifying change with a passing review at `HEAD` ships (control); after finalizing it, publishing that as `origin/main` and returning to the reviewed copy with its review files, `upstream_archive_of` reports the archive, `archive_candidate_can_ship` is false, and `pull_request_sentence` names `specsync change status <id>` and never `ship`. A unit test because every command that prints the full sentence (`change pr`, `ship`) pushes or archives; `change status` prints the sentence without its command. Fails on main cd8cab5e.
2. Script (`.github/scripts/test-archive-readiness.sh`): the same shape built with plain Git plus a bare origin: the script prints `Run \`specsync change status <id>\`` with an upstream archive, `ship` without one, and keeps `ship` for an archive `HEAD` already contains. Fails on main.
3. Existing archive-readiness, pull-request-sentence and archived_upstream tests unchanged and green.
4. `specsync change check <id> --commit`, `fledge lanes run verify` (pinned toolchain), pre-push.

## Results

Pending.
