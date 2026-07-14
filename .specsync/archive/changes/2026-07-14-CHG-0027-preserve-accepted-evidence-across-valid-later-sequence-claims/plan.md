---
change: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
artifact: plan
---

# Plan

1. Validate the current repository-backed sequence ledger before interpreting ownership.
2. Reconstruct canonical historical ledger bytes for predecessor acceptance inputs.
3. Continue hashing exact ledger bytes for the current owner and fail closed for invalid claims.
4. Add regression tests for valid successor creation, current-owner tampering, and invalid ownership.
5. Update the canonical `change` contract and companions through semantic acceptance.
