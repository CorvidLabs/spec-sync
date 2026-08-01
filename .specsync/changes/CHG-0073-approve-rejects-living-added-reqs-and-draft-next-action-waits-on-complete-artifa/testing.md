---
change: CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa
artifact: testing
---

# Testing

- `REQ-change-047`: covered by `draft_next_action_prefers_complete_artifacts_over_approve`.
- `REQ-change-048`: covered by `added_requirement_already_in_living_tree_fails_delta_validation` (validate + approve fail; MODIFIED ok).
- `REQ-cmd-change-006`: covered directly by
  `draft_text_surfaces_require_complete_artifacts_before_approval`, which exercises the shared
  text renderer used by `change status`, `change show`, and `change list` and rejects any approval
  recommendation.
