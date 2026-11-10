---
change: re-approval-accepts-an-unchanged-delta-body-that-already-carries-the-moved-canonical-text
artifact: testing
---

# Testing

Tests in `src/change_tests/stale_modified_delta.rs`. The git-backed tests commit the
approval-time base, land the other change's edit, and re-approve. The positive test fails when
the three-way acceptance is disabled.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-122 | unit `three_way_keeps_body_accepts_only_bodies_that_carry_the_moved_edits`, `re_approving_an_unchanged_body_that_already_carries_the_moved_text_records_the_new_base`, `re_approving_an_unchanged_body_that_lacks_the_moved_text_is_refused_in_a_repository`, `re_approving_an_unchanged_stale_body_is_refused`, `re_approving_an_unchanged_current_body_keeps_its_base` |
