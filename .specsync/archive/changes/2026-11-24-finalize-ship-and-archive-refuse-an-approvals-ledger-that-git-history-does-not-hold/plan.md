---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: plan
---

# Plan

## Approach

Add one read-only check that the change's `approvals.json` is held by Git history: the working copy
against `HEAD`, then every committed version along `HEAD`'s first-parent history against the one before
it. Ask it where finalize, archive and ship are about to write, and in ship-status as a blocker. Reuse the
#660 relay-blind projection for the comparison. Each refusal names the file and the way out that fits
what differs.

## Out of scope

- An archive edited after it was archived. Reading commands already call that `corrupt-history`.
- `corrections.json`, `review.json` and the other lifecycle files. Leif's words name the approvals.
- A repair for a committed rewrite. The refusal names the two ways out instead.
- Walking every parent of a merge. Only the first-parent history is one line of versions.

## Steps

1. Failing tests: `src/change_tests/committed_approvals.rs`, starting from the three of `7f15b21a`, then
   one per legitimate writer, the never-committed ledger, the uncommitted append, the accepted-change
   exception and its refusals, the committed rewrite of each list, the delete-and-recreate, the reopen
   out of the archive and the merged-in rewrite; integration tests through the binary for finalize,
   ship, `ship --dry-run`, ship-status, and accept then archive.
2. `src/change/historical_definition_digest_matches.rs`: the check, the extends rule and the refusal
   messages, beside finalize and archive.
3. Ask it in `finalize_change`, `archive_change_with_options` and `ship_status_report`.
4. Existing tests that finalize or archive a ledger that is not committed gain a commit; the one that
   tampers on purpose expects the new message.
5. The `files:` entry for the new test file in the `change` spec; `change check` adds the
   requirements.
6. Docs: `site/src/content/docs/workflow.md` and `cli.md`, `CHANGELOG.md`.
7. `fledge lanes run verify`, then the pre-push gate.

## Risks

- A legitimate rewrite is refused. Held by one test per writer that commits the rewrite and finalizes,
  and by the whole suite, which finalizes and archives in many shapes.
- A change becomes impossible to finish because an older SpecSync rewrote a committed record. The
  refusal names the commit and the two ways out, and the CHANGELOG says to run 6.3.0 or newer.
- Finalize slows down on a long history. The walk is bounded by the existing history bound, and
  only the commits that touched the one file are read.

## Constraints consulted

- CHANGE-16 to CHANGE-16.c, and Leif's choice "Every committed version".
- REQ-change-relay-repair-001: `change repair-relays` restores or removes relays, so `relayed_by` is
  not compared, in an approval and in a reopening's embedded approval.
- REQ-change-131: the open-task refusal runs before acceptance writes; this check sits beside it.
- The change lesson that one rule kept in two places drifts: the comparison reuses the #660 projection.
- The change lesson that a printed recovery command must run as printed: each way out is a command
  that does what the refusal says.
- #1044: lifecycle rewrites keep members they do not know, so a newer member at the top of the ledger
  is not a rewrite.
