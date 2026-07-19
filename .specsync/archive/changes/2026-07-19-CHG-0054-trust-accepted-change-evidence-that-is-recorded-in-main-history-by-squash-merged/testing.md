---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: testing
---

# Testing

- `REQ-change-037`: an accepted change whose evidence was refreshed while already accepted and
  squash-merged archives successfully when exactly one in-history commit records it as accepted
  with byte-identical evidence.
- A change with no in-history accepted record matching its current evidence still fails the
  archive preflight with the established `found 0` diagnostic.
- Merge-commit and first-acceptance histories keep passing the existing transition-anchor tests
  unchanged.
- The four blocked production changes archive with the fixed binary, and final
  `specsync check --strict --require-coverage 100 --force` and `fledge trust verify` pass.
