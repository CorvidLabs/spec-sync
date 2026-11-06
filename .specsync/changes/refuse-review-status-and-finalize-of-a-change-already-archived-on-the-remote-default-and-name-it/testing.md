---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: testing
---

# Testing

New file `tests/integration/archived_upstream.rs`, built from the auditor's squash steps without the audit fixture: a fresh project in a temp repository, one change approved and checked on a feature branch, a bare "origin" whose default branch holds that change archived, and a local `main` reset to the base with the feature squash-merged.

1. Upstream archive present: `change review` exits non-zero, the output names the change, `origin/main` and the archive commit and the merge/rebase step, and the record is still `verifying`. `change status --json` readiness is not `safe`. `change ship-status` names the same cause. `change finalize` exits non-zero with the same cause and writes no archive.
2. No upstream archive (no remote): the same squash, then `change review` and `change finalize` succeed and the package is archived.

Case 1 must fail on main 884a33d3 before the fix. Case 2 must pass before and after.

## Results

`tests/integration/archived_upstream.rs`:

- `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` failed on main 884a33d3 (`change review` recorded a pass with handoff `safe`). With the change it passes: review, status, ship-status and finalize all name `origin/main` and the archive commit, the record stays `verifying`, and no second archive is written.
- `a_squash_before_finalize_without_an_upstream_archive_still_reviews_and_finalizes` passed before and after (#690 guard).

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-127 | `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance`, `a_squash_before_finalize_without_an_upstream_archive_still_reviews_and_finalizes` |
| REQ-cmd-change-028 | `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` |
