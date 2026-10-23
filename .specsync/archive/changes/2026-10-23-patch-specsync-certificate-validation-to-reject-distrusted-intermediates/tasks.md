---
change: patch-specsync-certificate-validation-to-reject-distrusted-intermediates
artifact: tasks
---

# Tasks

- [x] Identify current advisory and patched release from primary source.
- [x] Resolve and inspect a four-field external preview without changing the source branch.
- [x] Obtain actual human definition approval before Cargo.lock changes.
- [x] Apply and verify the exact minimal lock update.

## Verification and delivery

All current-graph product, Trust, scoped and hosted checks remain required before delivery. Existing old-graph results are not counted as updated-graph proof. Human implementation review, release, merge, deployment and consumer changes remain separate.
