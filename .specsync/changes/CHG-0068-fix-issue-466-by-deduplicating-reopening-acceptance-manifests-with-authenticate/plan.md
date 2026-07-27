---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: plan
---

# Plan

1. Add a failing large-manifest characterization that performs repeated reopen cycles and proves
   `approvals.json` duplicates the full payload today.
2. Introduce strict private schema-v1/schema-v2 persistence records and a validated manifest
   reference type while preserving the hydrated public lifecycle view.
3. Add location-aware object path derivation, deterministic object serialization, atomic no-follow
   creation, exact reuse, and strict hydration.
4. Route every approval-ledger read and write through the compatibility layer.
5. Emit compact events from reopen while retaining legacy reads and exact authentication.
6. Cover missing, tampered, malformed, symlinked, wrong-digest, and archive object cases.
7. Cover large A/B/A histories, legacy v1 compatibility, reacceptance, archival, and migration
   idempotence.
8. Update the canonical `change` requirement, public contract text, testing evidence, tasks,
   context, version, and changelog.
9. Run targeted tests, the full repository verification lane, strict 100% spec coverage, score
   gates, Augur, Attest, and independent compatibility/security review before closing approval.
