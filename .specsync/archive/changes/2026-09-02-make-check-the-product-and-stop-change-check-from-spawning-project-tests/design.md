---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: design
---

# Design

No UI. CLI only. `change check` records one `CommandEvidence` row named `specsync check` or
`specsync check --strict`. Status projections list that same command instead of the policy
`verification_commands` list.
