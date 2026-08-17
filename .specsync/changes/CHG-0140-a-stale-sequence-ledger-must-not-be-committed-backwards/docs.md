---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: docs
---

# Docs

A CHANGELOG entry under `## [Unreleased]` → `### Fixed`.

## What it states

- **What a user saw:** `change check --commit` printing "Verified, committed, and consistent
  with the committed tree", exit 0, having lowered the change-sequence high-water mark.
- **Why that matters beyond bookkeeping:** the mark is what keeps change IDs unique. A
  regressed mark means the next allocation can reissue an ID that is already taken.
- **The mechanism:** `change new` writes the ledger to the working tree only; nothing commits
  it until a later `git add -A`, by which time the branch may have caught up past it.
- **That the allocation-time floor could not have caught it** — the value was correct when
  written and went stale afterwards.
- **How far the damage travelled before it surfaced**, because that is the part that made it
  expensive: nine surfaces stay green, then audit/finalize/ship/new refuse with a diagnostic
  naming neither the command nor the file.

## New behaviour a reader must not be surprised by

`change check --commit` may now print a note on **stderr**:

    note: raised the change sequence ledger from N to the committed M before staging; …

That is not an error and does not change the exit status. It appears whenever a branch has sat
long enough for `main` to move past its allocation — which is ordinary, not exceptional.

## Stated limit

This guards SpecSync's own commit paths. A user who runs `git add -A && git commit` by hand
can still commit a stale ledger; the tool cannot police manual git, and this change does not
claim to.
