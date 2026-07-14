---
change: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
artifact: testing
---

# Testing

## Requirement Evidence

- `REQ-change-029`: focused unit regressions cover valid later ownership, exact current-owner bytes, and invalid later claims.

## Planned Verification

- Unit: an accepted record remains current after a valid later change advances the ledger.
- Unit: editing the ledger while evaluating its current owner changes the acceptance digest.
- Unit: malformed, orphaned, non-maximum, duplicate, and invalid collision claims remain rejected.
- Full: formatting, lint, unit/integration tests, release build, strict SpecSync, Trust, Augur, and Attest.
