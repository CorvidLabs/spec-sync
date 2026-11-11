---
change: approve-accepts-a-sent-back-change-s-revised-body-for-a-requirement-it-added-itself
artifact: testing
---

# Testing

Unit tests in `src/change_tests/send_back_for_rescope.rs` drive approve, materialize,
send-back, a revised delta and re-approval on a temp project. The positive test fails on
the previous code with the #1001 error.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-123 | unit `a_sent_back_change_can_be_re_approved_with_a_revised_requirement_it_added`, `approve_still_refuses_an_added_requirement_whose_canonical_text_someone_else_changed`, and the existing `a_sent_back_change_can_change_the_body_of_an_item_it_added` |
