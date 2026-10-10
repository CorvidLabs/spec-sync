## ADDED

### REQUIREMENT REQ-change-107

The approval tests SHALL keep passing after a selected plan must name its sections. A test that sets a principles file SHALL record which constraints apply before approval, and changing that file SHALL still invalidate the approval.

Acceptance Criteria
- `stale_definition_approval_is_rejected` approves a plan that names its sections.
- `project_principles_are_part_of_the_approval_digest` records a constraints answer and still fails approval after the principles file changes.
