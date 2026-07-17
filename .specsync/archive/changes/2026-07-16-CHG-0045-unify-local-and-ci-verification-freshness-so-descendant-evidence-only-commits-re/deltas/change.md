## MODIFIED

### REQUIREMENT REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval, history, and verification evidence before using it, with one environment-independent verification-freshness decision.

Acceptance Criteria

- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed evidence, a matching effective contract digest, and a matching project-input digest in local and hosted checks.
- A descendant verification commit remains current only when every intervening commit and every parent edge changes exactly `state.json`, `verification.json`, or `verification-attempts.json` under a canonical active-change ID and the persisted state/evidence remains consistent.
- Source-change-then-revert history, ambiguous merges, nonancestor history, malformed paths, and any broader volatile or lifecycle path fail closed.

### REQUIREMENT REQ-change-016

The lifecycle SHALL preserve accepted closing evidence and supported verifying evidence across repository-integrated commits without accepting unintegrated, altered, or historically tainted evidence.

Acceptance Criteria

- Normal verification-commit ancestry remains mandatory proof and uses identical local and CI semantics.
- Every intervening commit is inspected against every parent with NUL-delimited portable paths; a net tree diff cannot hide a governed change and later revert.
- Only supported verification persistence beneath canonical active-change IDs may follow verification without invalidating it; archive, approvals, tasks, definitions, sequence, hashes, locks, configuration, policy, specs, source, tests, build, and cache paths are rejected.
- Matching effective contract and project-input digests plus consistent state, verification, and latest-attempt evidence remain mandatory.
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an unchanged accepted workspace integrated on the remote default branch.
- Unintegrated heads, changed scoped inputs, stale contracts, mismatched closing approvals, nonancestor evidence, and ambiguous merges fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and executable modes remain exact.
