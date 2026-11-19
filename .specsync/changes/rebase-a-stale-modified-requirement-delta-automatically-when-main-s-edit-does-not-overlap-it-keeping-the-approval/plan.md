---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: plan
---

# Plan

## Approach

Let `change check` rebase a stale MODIFIED requirement delta when main's edit and the change's edit do not overlap word by word, recording a rebase that every reader re-checks, so the approval stands. Overlaps keep today's refusal. The failing tests come first.

## Out of scope

- `## MODIFIED` spec sections and `## REMOVED` items.
- Workflow-v1 changes, whose definition digest hashes delta bodies.
- Rebasing in acceptance or audit, which only read.
- No tag, release or publish.

## Steps

1. Write the failing tests and record how they fail on 99f61b2b.
2. After scope approval: the word merge and the rebase record with its verification.
3. Effective claims in the binding read, the base check, re-approval and acceptance.
4. The rebase in `change check`, and `change status` output.
5. Specs, a `change.3md` lesson, `site/src/content/docs/deltas.md` and CHANGELOG.
6. `change check --commit`, the verify lane and pre-push.

## Risks

- **Approval laundering.** A record that could carry any body would let unapproved wording ride on an approval. The record holds its inputs, and every read recomputes it.
- **A merge that reads clean but means something else.** Word-level diff3 merges edits that are textually apart; it cannot judge meaning. The rebase is recorded and shown in status, so the reviewer sees it.

## Constraints consulted

- CHANGE-6.a and CHANGE-6.c.
- #843 (REQ-change-122) and #975: the stale-body refusal stays for every case this change does not merge.
