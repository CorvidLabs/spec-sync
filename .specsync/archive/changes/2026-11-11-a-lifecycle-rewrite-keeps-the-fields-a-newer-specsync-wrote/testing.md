---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: testing
---

# Testing

Unit tests in `src/change_tests/unknown_members.rs` and one end-to-end CLI test.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-079 | unit `an_older_specsync_appending_to_a_ledger_keeps_what_a_newer_one_wrote`, `an_entry_the_rewrite_changed_or_replaced_never_gains_another_records_members`, `kept_members_keep_their_exact_text_and_known_members_their_order`, `a_rewrite_with_nothing_unknown_writes_exactly_what_it_wrote_before`, `a_known_field_the_rewrite_sets_or_removes_is_never_overridden_by_the_old_text`, `an_empty_object_in_the_new_text_still_takes_its_unknown_members`; integration `lifecycle_rewrites_keep_members_a_newer_specsync_wrote` (fails on main at the first send-back) |
