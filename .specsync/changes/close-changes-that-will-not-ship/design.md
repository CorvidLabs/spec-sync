---
change: close-changes-that-will-not-ship
artifact: design
---

# Design

- **Where it goes.** Like a late close (REQ-change-143), the package moves byte for byte under
  `.specsync/archive/` but outside `.specsync/archive/changes/`: into
  `.specsync/archive/closed/<date>-<id>/`, with `close.json`. No change state is added, so no
  lifecycle reader and no older SpecSync sees it; its paths, sequence and coverage are released
  because nothing enumerates it.
- **Who and why.** `--actor` and a non-empty `--reason` are required; `--words/--where/
  --recorded-by` record a relayed decision (`relayed_decision`, as approve). `--replaced-by`
  must name another change that exists.
- **What it refuses.** Accepted and archived changes (finalize, reopen or retire them), a change
  already closed or closed late, a package that already holds `close.json`, an ambiguous
  active/archived id, and a change whose materialized spec text has not merged: each canonical
  file of its affected modules must match the remote default branch, or the refusal names
  `git checkout <ref> -- <files>` for files the branch holds and removal for ones the change
  created. With no remote default, a materialized change is refused with that recovery.
- **Batch.** Every id is planned before anything moves; one refusal closes none. The write reuses
  the late close's batch mover, now generic over its record (`PackageCloseRecord`), with the same
  symlink checks, collision-acknowledgement drop and rollback.
- **Reading it back.** `find_change_dir` and `change new`'s ID reservation name the close;
  `change status`, `show` and `ship-status` print it (`summary.state: closed`).

Rejected: a new `Closed` change state (every reader and older SpecSync would meet it), and keeping
closed changes in `.specsync/changes/` (Leif chose the archive).
