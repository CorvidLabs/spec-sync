---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: plan
---

# Plan

1. Replace new-change closing approval with one scope approval while preserving historical reads.
2. Implement `change finalize` as a transactional same-PR canonical-apply and archive transition.
3. Add exact archive-only diff classification, parent-check verification, unchanged-tree proof,
   archive/bidirectional validation, and required-gate reporting without product-suite reruns.
4. Add one implementation-parent-bound scoped review check for agent-authored changes and
   newcomer-oriented status guidance.
5. Repair strict path coverage and add an invocation-scoped bounded lifecycle snapshot.
6. Port and independently review the high-confidence schema, ignore, export, and installation
   fixes without importing stale branch history or unrelated canonical files.
7. Update canonical specs, CLI/docs, version surfaces, and release notes through the approved
   semantic deltas.
8. Run focused tests while iterating, resolve independent review findings, then run one final full
   repository/release/trust and CorvidLabs/spec-sync-sandbox validation.
