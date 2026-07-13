---
spec: change.spec.md
---

# Testing

Unit tests cover IDs, requirement grammar, semantic application, unsafe command rejection, adaptive policy, stale approvals, conflicts, and the full acceptance/archive path. CLI integration tests cover JSON interviews, rationale enforcement, dry-run adoption, and new-project enablement. Release validation runs the full Rust, spec, docs, audit, and build matrix.

## Requirement Evidence

- `REQ-change-001`–`REQ-change-004`: full lifecycle, stale approval, JSON interview, and requirement-evidence unit/integration tests.
- `REQ-change-005`: Markdown boundary, rollback, and interrupted-transaction recovery tests.
- `REQ-change-006`: working-tree digest, Unicode/space path, and failed-evidence tests.
- `REQ-change-007`: malformed policy, unavailable comparison, archive timing, bounded artifact, safe path, and effective-contract tests.
- `REQ-change-008`: dependency ordering, late-gate, component-boundary, concurrent-ID, and feature-branch coverage tests.
- `REQ-change-009`: delta approval, module-scoped IDs, corrupt state, retryable archive, tombstone, and Spec Kit import tests.
- `REQ-change-010`: root Action/manifest/lockfile and component-boundary tests.
- `REQ-change-011`: `effective_contract_workspaces_are_unique` plus the existing effective-contract semantic tests.
- `REQ-change-017`: unit and CLI integration coverage for accepted → stale failure → audited reopen → stale verifying failure → fresh verify → fresh accept, including preserved evidence, required audit fields, non-stale rejection, and deterministic JSON.
