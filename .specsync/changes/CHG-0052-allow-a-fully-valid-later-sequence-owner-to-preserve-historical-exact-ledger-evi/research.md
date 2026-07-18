---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: research
---

# Research

`acceptance_manifest_internal` reconstructs historical ledger views, but collision
acknowledgements introduced after an acceptance legitimately change those reconstructed bytes.
`validate_accepted_inputs_recursive` handles semantic successors for canonical owners, yet returns
immediately for every changed `@exact` entry. The safe extension point is therefore the exact-entry
branch, not manifest hashing or collision validation.

The ledger already proves its current owner is the highest located sequence and validates the exact
collision set. Accepted-evidence authentication already proves state, definition, verification,
closing approval, and trusted Git transition. Reusing both checks avoids a second trust model.
