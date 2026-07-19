---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: testing
---

# Testing

- `REQ-change-038`: a legacy (manifest-less) accepted change whose inputs include unowned
  production source reconstructs its manifest with the exact delivery owner and passes archived
  historical-integrity validation.
- Current acceptance of unowned production source still fails with the established
  deterministic-ownership diagnostic.
- Aggregate mismatch, failed historical verification, and invalid historical closing approval
  reconstructions still yield `found 0` and fail closed.
- Final `specsync check --strict --require-coverage 100 --force` and `fledge trust verify` pass.
