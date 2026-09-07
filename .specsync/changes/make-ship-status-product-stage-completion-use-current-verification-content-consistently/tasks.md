---
change: make-ship-status-product-stage-completion-use-current-verification-content-consistently
artifact: tasks
---

# Tasks

- [x] Reproduce the contradictory predicates by reading the current report and stage code.
- [x] Define the narrow public contract and the stale-content negative control.
- [x] Obtain scope approval for this package.
- [x] Add report-level regression coverage and confirm it catches #745.
- [x] Reuse verification currency in the product stage and update explanatory text.
- [x] Prepare the cmd_change semantic delta and synchronize its existing companions.
- [x] Run focused command-module regressions: 16 passed; both discriminating cases failed before the fix.

## Remaining lifecycle gates

The upcoming change check materializes the approved semantic delta. Full verification, Trust and provenance recording remain pending. Scoped human review and finalization must follow before merge; none are claimed complete here.
