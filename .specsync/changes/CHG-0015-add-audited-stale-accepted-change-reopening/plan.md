---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: plan
---

# Plan

1. Add backward-compatible persisted audit and canonical-application state.
2. Implement stale-only accepted-to-verifying transition under the lifecycle lock.
3. Expose required CLI inputs and deterministic text/JSON results.
4. Cover rejection, strict stale behavior, history preservation, fresh verification, reacceptance, and no-double-apply behavior.
5. Synchronize module contracts, workflow documentation, and release notes; run the full repository lane.
