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
- `REQ-change-018`: squash-history regressions prove recorded acceptance fallback, rejection of overlapping no-spec successors, and successful governance by a later semantic canonical successor.
- `REQ-change-019`: section-only semantic evidence and distinct missing-evidence diagnostics are covered without weakening requirement-ID mappings.
- `REQ-change-020`: reopened reacceptance covers transitional definition evidence, semantic-only successor governance, and validation of current canonical modules without replaying applied deltas.
- Legacy serialization coverage proves false `canonical_applied` values remain absent, omitted and transitional explicit-false definition evidence stays valid, explicit acceptance appends stable evidence for older contract checkers, and true values persist.
- Sequence-integrity coverage rejects unacknowledged active/archive collisions, accepts only the exact immutable historical baseline, and exercises the committed claim update.
- Verification coverage rejects direct and indirect lifecycle recursion, retains failed attempts, and proves a corrected native retry can become current without deleting history.
- Canonical-successor coverage exercises incomplete, stale, failed, implementing, verifying, and accepted successor states against stale predecessor evidence.
- Registry-path coverage applies spec and requirements deltas to a non-conventional registered module and rejects unsafe mappings.
- `REQ-change-026`: sequence coverage includes five-digit IDs, protected ledger paths, exact accepted/archive baselines, removed historical IDs, and rejected mutable collisions.
- `REQ-change-027`: CLI integration runs inherited verification context through `check`, `change`, and `lifecycle` and asserts one contextual failure.
- `REQ-change-028`: registry-backed effective-contract validation succeeds at the registered path, unsafe mappings fail before reads, and successor evaluation reuses one precomputed project digest.
