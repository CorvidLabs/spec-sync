# Change lifecycle hardening delta

## ADDED

### REQUIREMENT REQ-change-005

The system SHALL preserve unrelated canonical Markdown when applying semantic blocks.

Acceptance Criteria
- Modifying or removing the final requirement before a higher-level heading preserves that heading and all following content.
- Failed preparation leaves canonical files byte-for-byte unchanged.
- An interrupted multi-file acceptance is recovered from its transaction journal before the next lifecycle mutation.

### REQUIREMENT REQ-change-006

The system SHALL bind verification evidence to every tested working-tree input.

Acceptance Criteria
- Source, test, configuration, or contract edits after verification invalidate acceptance even when HEAD is unchanged.
- Failed verification remains an error until fresh successful evidence is recorded.

### REQUIREMENT REQ-change-007

The system SHALL fail closed when lifecycle enforcement cannot be evaluated.

Acceptance Criteria
- Malformed policy and unavailable changed-path comparison fail unified checking.
- Effective-contract validation runs during verification and acceptance.
- Oversized lifecycle artifacts and unsafe, traversing, or symlink-escaping project paths are rejected.

### REQUIREMENT REQ-change-008

The system SHALL apply concurrent change semantics in declared dependency order.

Acceptance Criteria
- Effective deltas are topologically ordered regardless of change ID.
- Dependency and conflict gates are rechecked immediately before acceptance.
- Path coverage matches complete path components rather than arbitrary prefixes.
- Lifecycle mutations serialize through an operating-system lock so concurrent creation cannot duplicate IDs.

### REQUIREMENT REQ-change-009

The system SHALL keep definitions and persisted lifecycle state trustworthy through approval, adoption, and archival.

Acceptance Criteria
- Definition approval rejects missing, malformed, or cross-module semantic requirements before recording evidence.
- Corrupt active state fails unified checking instead of disappearing from enforcement.
- Failed archive moves preserve the accepted active workspace so archival can be retried.
- Only accepted or archived requirement removals become permanent tombstones.
- Spec Kit adoption does not classify native companion-only spec directories as feature workspaces.

### REQUIREMENT REQ-change-010

The system SHALL require lifecycle coverage for common root action, manifest, and dependency lock files by default.

Acceptance Criteria
- Root Action configuration and supported ecosystem manifest or lockfile changes are meaningful paths.
- Component-boundary matching continues to exclude similarly prefixed unrelated files.

## MODIFIED

### SPEC SECTION Contract

1. Every meaningful SDD change moves through draft, approved, implementing, verifying, accepted, and archived states without bypasses.
2. Definition and closing approvals are portable records bound to deterministic SHA-256 digests.
3. Approved semantic deltas form the effective future contract without mutating canonical specs before acceptance.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell and rejects unsafe shell syntax.
6. Verification evidence is bound to the tested commit and working-tree inputs, and effective contracts must validate before acceptance.
7. Invalid policy, unavailable coverage comparison, failed evidence, and stale ordering gates fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, corrupt state fails closed, and archival failures remain retryable.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
