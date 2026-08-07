---
change: CHG-0094-count-same-pr-archived-changes-toward-path-coverage-after-finalize
artifact: requirements
---

# Requirements

After same-PR `change ship`/`finalize`, Lifecycle gate must not fail path coverage solely because zero actives remain while the archive package on the tip still owns the product paths.

Acceptance: delivery-archived packages cover their affected_paths; archives outside the delivery do not.
