## ADDED

### REQUIREMENT REQ-change-016

The lifecycle SHALL preserve accepted closing evidence across repository-integrated squash merges without accepting
unintegrated or altered evidence.

Acceptance Criteria
- Normal verification-commit ancestry remains the primary proof.
- A squash fallback requires matching scoped inputs and an unchanged accepted workspace already integrated on the
  remote default branch.
- Unintegrated heads, changed scoped inputs, stale contracts, and mismatched closing approvals fail closed.
- Squash-integrated accepted workspaces remain archivable.
