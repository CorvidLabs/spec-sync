---
change: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
artifact: plan
---

# Plan

## Approach

Keep the five existing interview questions, the single scope approval, and the merge instruction. Add three mechanical pieces beside them.

The domain grows an open-change briefing: substantive lines from each affected module's `context.md`, plus the configured principles file. Text and JSON on `change new`, `change status`, and `change show` render that same briefing. A generated scaffold contributes nothing. The excerpt is capped, and the path is named for the rest. An unreadable file is omitted.

`next_questions` stays the record-only list. A root-aware list, used by every approval and status gate, appends `constraints` only for a draft that already has substantive lessons or a configured principles file, and only after the other questions are answered. The answer is stored in `answers`, so it is already inside the stable scope. "none" is a complete answer. Changing `affected_specs` clears the answer so it is asked again. A change that has left draft without the key is not sent back to the interview.

When a plan is selected, approval fails until Approach, Out of scope, Steps, Risks, and Constraints consulted each have substantive text. Kinds that do not select a plan do not grow one. The incomplete-artifact list uses the same predicate as approval.

`specsync rules` prints the principles path beside the spec-shape rules, and says a lesson becomes a standing rule only when a person writes it there. Finalize and ship text name the lesson-bundle path on their own line. `next_action` stays the merge instruction.

## Out of scope

No model call, no synthesis of lessons, and no automatic promotion of a lesson into a failing check. No new lifecycle state. No bug-fix plan. No change to review-then-ship ordering, the workspace digest, or the rule that a commit between review and ship stales the review. No new field on `ApprovedScopeV1`.

## Steps

1. Add the briefing and the plan-section predicate in the change domain, and thread the root-aware question list through status, handoff, and definition validation.
2. Render the briefing from `change new`, `change status`, and `change show`, in text and JSON. Name the lesson bundle in finalize and ship text without changing `next_action`.
3. Teach `specsync rules` to name `principles_file`.
4. Update the configuration page, the workflow page, and the high-level design so they describe this behavior.
5. Cover the briefing, the draft-only question, the plan gate, the untouched bug-fix artifact set, the scope-digest change, and the merge instruction with tests named in `testing.md`.

## Risks

A question that appears for every historical record would invalidate approvals that never had a chance to answer it. The question is draft-only.

A plan check that ignores the generic `# Complete` bodies in existing tests would fail the suite. Those writers must emit a plan that has the five sections when the artifact is a plan.

Putting the fold back into `next_action` would withhold the merge. The bundle path stays off that line.

An unbounded excerpt of `specs/change/context.md` would bury the interview. The cap is part of the contract.

## Constraints consulted

Do not withhold merge on folding lessons. Do not call a model. Do not fail a lifecycle command when a context or principles file is unreadable. Do not treat a generated scaffold as knowledge. Show a bounded excerpt, and keep spec-shape rules separate from principles.
