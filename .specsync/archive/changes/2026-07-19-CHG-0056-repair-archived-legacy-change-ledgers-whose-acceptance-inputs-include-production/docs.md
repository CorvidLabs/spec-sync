---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: docs
---

# Docs

Document that adoption-era (pre-manifest) archived ledgers validate without repair: legacy
acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs
that have no canonical owner, while every newly accepted change still requires deterministic
canonical ownership for production source.
