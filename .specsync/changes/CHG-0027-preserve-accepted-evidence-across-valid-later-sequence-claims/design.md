---
change: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
artifact: design
---

# Design

Before hashing acceptance inputs, validate the sequence ledger and compare its owner with the record being evaluated.

- Hash exact ledger bytes for the current owner.
- For a predecessor, reconstruct the canonical historical claim from its own sequence and ID while retaining acknowledged collision evidence.
- Propagate validation failures so malformed, missing-owner, non-maximum, duplicate, or invalid collision claims cannot suppress evidence checks.
- Leave source, canonical spec, change artifacts, and every other covered path unchanged in the digest.
