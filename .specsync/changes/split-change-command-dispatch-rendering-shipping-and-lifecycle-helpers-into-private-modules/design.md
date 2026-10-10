---
change: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
artifact: design
---

# Design

## Internal boundaries

The facade retains the complete match dispatch and its read-scope guards. Rendering owns transaction snapshots, text safety, briefing, roster and handoff projections. Ship status owns trust/readiness projections and stage guidance; archive guidance owns ordering and next-action copy. Shipping owns PR preflight, gate-before-push, finalize/archive orchestration and check-run waiting. Repository status owns Git reads and tip classification. Lifecycle commits own verify/commit ordering, literal scoped staging, ledger flooring and warnings. Owner inputs owns manifest parsing. No public type or signature changes.
