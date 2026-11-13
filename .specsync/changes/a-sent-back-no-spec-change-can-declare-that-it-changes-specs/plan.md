---
change: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
artifact: plan
---

# Plan

## Approach

Add `no_spec_change` to the answerable interview questions. Only `no` is accepted; it clears the
flag and the rationale on a draft.

## Out of scope

Declaring no spec change after creation (it needs a rationale only `change new` records);
`change correct` on a live change, which Leif did not choose.

## Steps

1. Red tests in send_back_for_rescope.rs.
2. `INTERVIEW_QUESTION_IDS` gains `no_spec_change`; `answer_question_with_snapshot` handles it.
3. cli.md and workflow.md.
4. REQ-change-160 delta once its wording is checked; targeted tests, fmt and clippy.

## Risks

The definition digest covers `no_spec_change`, so clearing it needs a fresh approval, which the
send-back already requires. `next_questions` never asks it, like `dependencies`.

## Constraints consulted

The re-scope route of `change send-back` (CHANGE-12): answers are draft-only, and a definition
approval from before a send-back no longer counts.
