---
change: change-answer-validates-question-keys-answers-and-actor-identity
artifact: testing
---

# Testing

- Unit/integration: unknown question key fails and `state.json` is unchanged; a typo like `acceptance-criteria` fails.
- Empty/whitespace answers fail for each question arm.
- Approve with a padded/newline actor fails; a normal actor still works.
- `affected_specs` naming an unregistered module fails and names known modules.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `change_answer_rejects_unknown_question_key_and_persists_nothing`, `change_answer_rejects_empty_and_whitespace_answers`, `change_new_and_answer_reject_unregistered_affected_spec`, `change_approve_trims_and_validates_actor_identity`; full unit suite 2972/2972 and integration 581/581 green |
