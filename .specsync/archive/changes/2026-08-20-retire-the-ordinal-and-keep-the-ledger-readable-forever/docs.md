---
change: retire-the-ordinal-and-keep-the-ledger-readable-forever
artifact: docs
---

# Docs

`specsync change new` now prints a slug where it used to print `CHG-NNNN-slug`, and change
directories are named by slug. Existing changes and archives keep the identities they were
created with; nothing is renamed.

The user-visible error for a repeated description changes from *"exhausted change sequence
allocation retries"* to a message naming the existing change, its path and its state.
