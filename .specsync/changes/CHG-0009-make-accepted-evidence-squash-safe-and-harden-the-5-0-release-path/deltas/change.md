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
- Digest fields are versioned, domain-separated, and length-framed so file boundaries cannot be forged with embedded
  NUL bytes.
- Binary bytes remain exact, while stable file kind and executable-mode evidence invalidate relevant delivery changes.
- Cross-platform topology verification is independent of a runner's global line-ending checkout policy.
