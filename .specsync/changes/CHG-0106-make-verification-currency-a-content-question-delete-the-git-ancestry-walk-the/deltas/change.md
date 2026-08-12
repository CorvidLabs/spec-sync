## MODIFIED

### REQUIREMENT REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval, and
verification evidence before using it, with one environment-independent verification-freshness
decision.

Acceptance Criteria

- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed evidence, a matching effective contract digest, and a
  matching project-input digest in local and hosted checks.
- Freshness is decided by content equality alone; no commit ancestry, intervening-commit
  inspection, or path allowlist participates in the decision.

### REQUIREMENT REQ-change-016

The lifecycle SHALL preserve accepted closing evidence across repository-integrated commits without
accepting unintegrated or altered evidence, while verifying evidence SHALL be judged on content
alone.

Acceptance Criteria

- Verification currency does not depend on commit ancestry, on inspecting intervening commits, or
  on restricting which paths may change after verification. Provenance of that kind is recorded by
  `attest`, keyed to commit SHAs, and is outside this tool.
- `verification.commit` is retained as an informational correlation key and is never a gate; a
  squash merge that discards the recorded commit does not invalidate the evidence.
- Matching effective contract and project-input digests plus consistent state, verification, and
  latest-attempt evidence remain mandatory.
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an
  unchanged accepted workspace integrated on the remote default branch.
- Changed scoped inputs, stale contracts, and mismatched closing approvals fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and
  executable modes remain exact.
