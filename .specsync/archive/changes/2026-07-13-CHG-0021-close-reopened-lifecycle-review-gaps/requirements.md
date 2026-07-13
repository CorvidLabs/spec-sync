---
change: CHG-0021-close-reopened-lifecycle-review-gaps
artifact: requirements
---

# Requirements

### Reopened lifecycle consistency

Audited reacceptance SHALL enforce the same immutable definition and fresh
evidence requirements in strict checks, definition reapproval, verification,
and closing across root and nested projects.

Acceptance Criteria
- Strict checks reject a reopened definition that closing would reject.
- Reapproval keeps canonical-applied reopened records in the verifying lane.
- Nested history lookup uses repository-top-relative state paths.
- Reopen rejects matching delivery-input digests regardless of another
  closing-validity failure.
