---
change: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
artifact: testing
---

# Testing

Integration tests drive the built binary on temp projects. The relay test fails with the
decider trim reverted.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `a_relayed_approval_with_a_padded_actor_records_one_decider_and_stays_readable`, `change_answer_refuses_empty_lists_and_none_clears_optional_lists`, `change_approve_trims_and_validates_actor_identity`, `change_new_and_answer_reject_unregistered_affected_spec`, `change_answer_rejects_unknown_question_key_and_persists_nothing`, `change_answer_rejects_empty_and_whitespace_answers` |
