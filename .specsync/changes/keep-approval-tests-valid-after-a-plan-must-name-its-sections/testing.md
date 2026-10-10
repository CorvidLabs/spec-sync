---
change: keep-approval-tests-valid-after-a-plan-must-name-its-sections
artifact: testing
---

# Testing

REQ-change-107

`project_principles_are_part_of_the_approval_digest` and `stale_definition_approval_is_rejected` both pass. Changing the principles file still invalidates the approval. Changing the approved context still invalidates a workflow-v1 approval.
