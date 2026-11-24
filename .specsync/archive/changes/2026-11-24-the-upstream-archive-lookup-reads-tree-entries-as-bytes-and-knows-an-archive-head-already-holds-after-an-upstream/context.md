---
change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
artifact: context
---

# Context

Two Codex P2s on #943, deferred by Leif's call. Both are in the upstream-archive lookup
(`upstream_archive_of` and `archived_state_paths_for`, `src/change/consider_accepted_evidence_anchor.rs`),
so they are one change.

- #961: `archived_state_paths_for` decoded every `ls-tree -z` entry with `from_utf8(...).ok()?`, so
  one unrelated package with a non-UTF-8 directory name aborted the whole lookup. After #946 that
  reads as cannot-tell, a refusal of every change in the project. Now an entry is split as bytes,
  only the mode, type and object id are decoded, and every candidate's record is still read. A
  non-UTF-8 package is skipped unless its record is this change's; then the lookup still cannot
  tell, because the path cannot be handed on.
- #945: `upstream_addition_outside_head` found the commit that added the matching package with
  `log --no-renames --diff-filter=A`. When HEAD holds the original archive and the remote default
  later renames that package, the rename commit reads as the addition and is not an ancestor of
  HEAD, so the archive HEAD contains read as remote-only and blocked the reopen-and-rearchive path
  REQ-change-129 keeps open. Now, before that log, the lookup asks whether HEAD holds a `state.json`
  blob with the same object id under its archive root. An archived record does not change when its
  directory is renamed. One more `ls-tree` of HEAD runs, only when the remote default holds a
  matching package.

The old `0xleif/fix/g-upstream-945-961` branch (2026-11-06) carried HI SHIP-3.d, which Leif approved
then ("Capture both"). It is not needed here: REQ-change-129 and REQ-change-142 already require
both behaviours, so this stays a bug fix with no new contract text.

Chough: Leif's rule (relayed by t370 on 2026-11-13) is "make sure u use chough 0.6.0 for all
spec-sync changes". spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges, so this
change is built now and goes through a Chough 0.6.0 session before review.
