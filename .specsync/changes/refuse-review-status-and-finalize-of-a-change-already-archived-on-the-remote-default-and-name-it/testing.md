---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: testing
---

# Testing

New file `tests/integration/archived_upstream.rs`, built from the auditor's squash steps without the audit fixture: a fresh project in a temp repository, one change approved and checked on a feature branch, a bare "origin" whose default branch holds that change archived, and a local `main` reset to the base with the feature squash-merged.

1. Upstream archive present: `change review` exits non-zero, the output names the change, `origin/main` and the archive commit and the merge/rebase step, and the record is still `verifying`. `change status --json` readiness is not `safe`. `change ship-status` names the same cause. `change finalize` exits non-zero with the same cause and writes no archive.
2. No upstream archive (no remote): the same squash, then `change review` and `change finalize` succeed and the package is archived.

Case 1 must fail on main 884a33d3 before the fix. Case 2 must pass before and after.
