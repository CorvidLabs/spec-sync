---
change: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
artifact: plan
---

# Plan

1. Add a deterministic path classifier and fixture tests.
2. Add job-level conditions without renaming existing checks.
3. Add one stable aggregate required gate.
4. Make PR summaries treat intentional skips as neutral.
5. Make main attestation depend on the aggregate gate.
6. Exercise archive-only, active-change, product, and mixed path cases.
7. Run every local repository lane and inspect the hosted PR job selection.
