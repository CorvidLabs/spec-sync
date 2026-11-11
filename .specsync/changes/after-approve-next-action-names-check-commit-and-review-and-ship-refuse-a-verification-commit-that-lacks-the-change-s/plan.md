---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: plan
---

# Plan

## Approach

Draft the new and changed requirement text now and stop at a READY PR
for Leif's definition approval. After that approval, implement
REQ-change-156 and REQ-change-157 against a red-first integration test
that is the #535 reproduction, insert invariant 59, and re-run the
reproduction on the branch build.

## Out of scope

Making `--commit` the default for `change check`. Restoring the refuted
unrecoverable-hook refusal. Filing or implementing any issue other than
the remaining #535 defect.

## Steps

1. Record the definition (this PR) and wait for Leif to approve it.
2. Write the failing integration tests listed in testing.md.
3. Change domain and text `next_action` to name `check --commit`.
4. Refuse review and ship when the verification commit lacks the work.
5. Insert invariant 59; run the new tests and the reproduction.

## Risks

Parallel open PRs already claim REQ-change-150 through 155 and
invariants 52 through 58. This change claims the next free numbers
(156, 157, and invariant 59) after grepping main and every open PR
head; a later PR that takes the same numbers would collide.

## Constraints consulted

None. The change.3md lesson shown at intake (a damaged active change
directory missing `state.json`) does not constrain this defect. CHANGE-7
is already accepted human intent and is cited only as the existing
"name the next thing to do" requirement, not as new intent.
