---
change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
artifact: design
---

# Design

- `archive_candidate_can_ship` returns `false` when `change::upstream_archive_of(root, record)` is `Some`, checked after the existing cheap file tests so it only runs Git for a copy that would otherwise ship. The pull-request sentence then names `specsync change status <id>`, which prints #889's upstream message and recovery.
- `archive-readiness.sh` mirrors `upstream_archive_of` for a row that would otherwise ship: resolve the remote default (`git symbolic-ref --short refs/remotes/origin/HEAD`, else the first of `origin/main`, `origin/master` that `rev-parse --verify` accepts); list `<repo-prefix>.specsync/archive/changes/` on it with `ls-tree --full-tree --name-only`; for a dated package `YYYY-MM-DD-<id>`, read its `state.json` id; find the commit that added it (`log -1 --format=%H --diff-filter=A <ref> -- :(top,literal)<path>`); treat it as upstream when `merge-base --is-ancestor <commit> HEAD` fails. Any Git failure answers "no upstream archive", as in Rust.
- No new public API: `upstream_archive_of` is already `pub(crate)` and the adapter calls it.

Rejected: having the script call the binary (the Archive gate deliberately runs without downloading SpecSync); fetching in the script (network and permissions in a 2-minute gate that other repositories use through the packaged Action).
