---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: docs
---

# Docs

Clarify invariant 14 in `specs/change/change.spec.md`: a fully valid later accepted sequence owner
may cover collision acknowledgements introduced after historical acceptance, while every non-ledger
input and the current owner's ledger bytes remain exact.
