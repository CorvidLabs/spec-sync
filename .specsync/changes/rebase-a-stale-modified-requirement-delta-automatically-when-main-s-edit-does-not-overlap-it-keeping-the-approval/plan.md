---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: plan
---

# Plan

## Approach

Keep the merge and its safety checks from the earlier rounds; replace the recorded rebase with a proposal written into the delta, refused until approved.

## Out of scope

Rebasing `## MODIFIED` spec sections and `## REMOVED` items; any automatic approval.

## Steps

1. Red tests for the proposal: written and refused; approved then applied; nothing written to the canonical file or the ledger; no proposal in CI; refusal causes named.
2. Remove the records, their readers and the provenance checks; restore the plain approval claims everywhere.
3. Turn the rebase step into the proposal writer; keep the merge, the move rule, the range check, headings, deletion and read failures.
4. Docs, CHANGELOG, invariant and lesson; `change check --commit`, the verify lane, `change audit --strict`, trust; the real-repo run (rule 3).

## Risks

A proposal written while someone is mid-edit of the delta: the merge only runs when the delta matches its approval, so a hand-edited delta is never overwritten.

## Constraints consulted

REQ-change-122, CHANGE-6.a (only a person approves wording), REQ-change-142, and Leif's rule 3 (every refusal names its way out).
