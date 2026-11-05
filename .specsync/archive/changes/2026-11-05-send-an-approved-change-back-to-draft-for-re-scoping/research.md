---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: research
---

# Research

- `answer_question_with_snapshot` refuses every state but draft, so scope can only change in draft.
- `approve_definition` already accepts approved, implementing and verifying changes and moves them back to approved or implementing, so a draft that was sent back re-enters the normal path.
- For workflow v2, `effective_definition_approval` only demands renewal when scope EXPANDS. A send-back that narrows scope, or changes nothing, would otherwise keep the old approval valid, so cancellation has to be explicit.
- `materialize_change_deltas` computes outstanding work from the canonical tree (#741). After `canonical_applied` is reset, already-applied blocks are converged, not applied twice.
