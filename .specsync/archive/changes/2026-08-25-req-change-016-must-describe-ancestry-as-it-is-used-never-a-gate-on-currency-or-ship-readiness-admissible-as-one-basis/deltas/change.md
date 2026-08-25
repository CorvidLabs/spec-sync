## MODIFIED

### REQUIREMENT REQ-change-016

The lifecycle SHALL preserve accepted closing evidence across repository-integrated commits without
accepting unintegrated or altered evidence, while verifying evidence SHALL be judged on content
alone.

Acceptance Criteria

- Verification currency does not depend on commit ancestry, on inspecting intervening commits, or
  on restricting which paths may change after verification. Provenance of that kind is recorded by
  `attest`, keyed to commit SHAs, and is outside this tool.
- `verification.commit` is never a gate on verification currency or ship readiness; a squash merge
  that discards the recorded commit does not invalidate the evidence or block delivery. Archival
  authentication of accepted evidence is a separate question — whether the acceptance is anchored
  in history a reader can reach — and MAY consult commit ancestry there, as one basis among the
  integrated accepted workspace and the acceptance recorded on the remote default branch. Ancestry
  MUST NOT be the only basis on which anchoring can be established.
- Matching effective contract and project-input digests plus consistent state, verification, and
  latest-attempt evidence remain mandatory.
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an
  unchanged accepted workspace integrated on the remote default branch.
- Changed scoped inputs, stale contracts, and mismatched closing approvals fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and
  executable modes remain exact.
