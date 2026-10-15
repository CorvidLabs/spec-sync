---
change: refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1
artifact: requirements
---

# Requirements

Restore existing REQ-validator-044 checked exact-Cargo coverage and fail-closed measurement, together with REQ-validator-008 configured directory fallback. A configured eligible existing regular root file that lacks required checked Cargo identity must not disappear from the coverage denominator. Refuse the incomplete measurement rather than claiming100% from another covered directory.

Preserve valid exact-target counting, explicit directory authority, filters, root confinement, no-follow reads and existing byte/entry budgets. No arbitrary configured-file support or broader traversal is added.
