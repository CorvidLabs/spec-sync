---
change: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
artifact: testing
---

# Testing

- Unit `an_unchanged_added_requirement_followed_by_a_section_is_not_a_conflict`: validation
  accepts it and the applier reads it as applied (fails on main with "cannot add existing block").
- Unit `a_changed_added_requirement_followed_by_a_section_is_still_a_conflict`: a different body
  is still refused.
- The existing ADDED-conflict, titled-header (#1024) and send-back tests still pass.
