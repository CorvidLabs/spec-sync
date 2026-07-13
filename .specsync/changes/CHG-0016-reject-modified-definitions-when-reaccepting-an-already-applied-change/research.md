---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: research
---

# Research

`verify_change` correctly binds evidence to the currently approved definition. That is insufficient for reacceptance because `canonical_applied` deliberately suppresses semantic application: a newly approved definition could pass verification but never reach canonical specs.

`ReopenRecord.prior_verification.contract_digest` is the immutable identity of the definition that originally produced canonical truth. Comparing against the latest event supports repeated audited delivery corrections without inventing mutable state or weakening existing approval checks.
