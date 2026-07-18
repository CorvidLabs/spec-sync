---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: testing
---

# Testing

- `REQ-change-029`: an authenticated later owner covers only historical sequence-ledger drift.
- Current-owner exactness: mutating current ledger bytes remains stale.
- Fail closed: missing, mutable, malformed, non-maximum, or unauthenticated owners do not cover drift.
- Repository regression: `specsync check --strict --require-coverage 100 --force` accepts both
  CHG-0048 histories plus CHG-0049 and CHG-0050 after CHG-0052 acceptance.
- Full verification: formatting, Clippy, unit tests, integration tests, release validators, release
  build, and `fledge trust verify`.
