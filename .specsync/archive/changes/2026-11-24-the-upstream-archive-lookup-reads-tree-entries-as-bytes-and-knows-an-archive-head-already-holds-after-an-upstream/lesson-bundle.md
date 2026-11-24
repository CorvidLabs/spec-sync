# Lesson bundle: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream rename
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/consider_accepted_evidence_anchor.rs, src/change_tests/upstream_archive.rs
- **Acceptance**: an unrelated archive package with a non-UTF-8 directory name on the remote default does not stop the lookup, and this change's upstream archive is still found; an archive HEAD already holds is not reported after the remote default renames its package; a non-UTF-8 package that does hold this change's record is still cannot-tell

## Evidence

- Verification commit: `eb48442400078c690fe1db01806ce7ab7835e6cb`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change --strict`

## From the change's context.md

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

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-129 (#945) | `a_reopened_change_reviews_after_the_remote_default_renames_its_archive` in `upstream_archive.rs`, the real flow through the lifecycle functions the commands call (finalize, reopen, check, review) with a bare origin, plus its control with no rename (the tester's BLOCK on f740d0a2: red there, green after containment moved from HEAD's tree to HEAD's history); `an_upstream_rename_of_an_archive_head_already_contains_is_not_reported` and the existing `upstream_archive_is_reported_only_when_head_does_not_contain_it` |
| REQ-change-129, REQ-change-142 (#961) | `an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup` (red before, green after), and the existing cannot-tell tests |
| REQ-change-139 (bounded processes) | the existing upstream-archive tests; one `log -1 --find-object` of HEAD's history runs per matching record, only when the remote default holds a match |

## How it runs

`cargo test --bin specsync upstream_archive` locally (red, then green), and hosted CI's full suite
at the archived head.

## Where these lessons go

- `specs/change/change.3md`
