# Lesson bundle: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Stop the Archive check and change pr from telling a copy already archived upstream to run ship
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, .github/scripts/archive-readiness.sh, .github/scripts/test-archive-readiness.sh, CHANGELOG.md
- **Acceptance**: With the remote default holding an archive of a change that this branch still holds unarchived, the pull-request sentence printed by `change status` and `change pr` names `specsync change status <id>`, never `specsync change ship <id>`, even when the stale copy carries a passing review whose `implementation_commit` equals `HEAD` and a verification record.
- **Acceptance**: `.github/scripts/archive-readiness.sh` reaches the same verdict and prints the same sentence when the remote default ref is present (`origin/HEAD`, else `origin/main`, else `origin/master`), using the same test as `upstream_archive_of`: a dated package of this change id on that ref whose `state.json` carries the id, added by a commit `HEAD` does not contain.
- **Acceptance**: An archive `HEAD` already contains (the reopen-and-rearchive path) and a repository with no remote default ref keep today's classification; the script never fetches, so the Archive job's single-commit checkout cannot see upstream archives, and that job still fails on any open change.
- **Acceptance**: A copy with no upstream archive and a passing review at `HEAD` is still told `specsync change ship <id>` by both.
- **Acceptance**: A Rust test (`an_upstream_archived_copy_is_not_told_to_ship` in `commands::change`) and a `test-archive-readiness.sh` case each fail on main cd8cab5e and pass with the fix.

## Evidence

- Verification commit: `c21f5356a918c41212d2bce99f31d71a6dbd56f2`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

Issue #902, a follow-up from the #889 review (thread 4). #889 made `change review`, `finalize`, `status` and `ship-status` refuse or report a change whose id the remote default already archived while this branch holds an unarchived copy (`upstream_archive_of`, REQ-change-129, REQ-cmd-change-029). The shared archive-candidate decision, `archive_candidate_can_ship` in `src/commands/change.rs`, and its shell twin `.github/scripts/archive-readiness.sh` were not taught that condition. Both still classify a stale copy with a passing review whose `implementation_commit` equals `HEAD` as shippable, so the pull-request sentence (`change status` without an id, `change pr`) and the Archive check can print `Run specsync change ship <id>`, a command `ship` now refuses.

The CI Archive job checks out a single commit (`actions/checkout` default depth), so no remote default ref exists there. The script cannot see upstream archives in that job; it keeps today's classification there, and the job fails on any open change regardless. Locally, where `change pr` and `ship` run, `origin` is present.

Out of scope: making the Archive job fetch the remote default (a CI cost decision), and any change to `upstream_archive_of` itself.

## From the change's design.md

# Design

- `archive_candidate_can_ship` returns `false` when `change::upstream_archive_of(root, record)` is `Some`, checked after the existing cheap file tests so it only runs Git for a copy that would otherwise ship. The pull-request sentence then names `specsync change status <id>`, which prints #889's upstream message and recovery.
- `archive-readiness.sh` mirrors `upstream_archive_of` for a row that would otherwise ship: resolve the remote default (`git symbolic-ref --short refs/remotes/origin/HEAD`, else the first of `origin/main`, `origin/master` that `rev-parse --verify` accepts); list `<repo-prefix>.specsync/archive/changes/` on it with `ls-tree --full-tree --name-only`; for a dated package `YYYY-MM-DD-<id>`, read its `state.json` id; find the commit that added it (`log -1 --format=%H --diff-filter=A <ref> -- :(top,literal)<path>`); treat it as upstream when `merge-base --is-ancestor <commit> HEAD` fails. Any Git failure answers "no upstream archive", as in Rust.
- No new public API: `upstream_archive_of` is already `pub(crate)` and the adapter calls it.

Rejected: having the script call the binary (the Archive gate deliberately runs without downloading SpecSync); fetching in the script (network and permissions in a 2-minute gate that other repositories use through the packaged Action).

## From the change's testing.md

# Testing

Failing tests first.

1. Rust unit (`commands::change::tests::an_upstream_archived_copy_is_not_told_to_ship`): a verifying change with a passing review at `HEAD` ships (control); after finalizing it, publishing that as `origin/main` and returning to the reviewed copy with its review files, `upstream_archive_of` reports the archive, `archive_candidate_can_ship` is false, and `pull_request_sentence` names `specsync change status <id>` and never `ship`. A unit test because every command that prints the full sentence (`change pr`, `ship`) pushes or archives; `change status` prints the sentence without its command. Fails on main cd8cab5e.
2. Script (`.github/scripts/test-archive-readiness.sh`): the same shape built with plain Git plus a bare origin: the script prints `Run \`specsync change status <id>\`` with an upstream archive, `ship` without one, and keeps `ship` for an archive `HEAD` already contains. Fails on main.
3. Existing archive-readiness, pull-request-sentence and archived_upstream tests unchanged and green.
4. `specsync change check <id> --commit`, `fledge lanes run verify` (pinned toolchain), pre-push.

## Results

Pending.

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
