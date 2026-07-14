---
change: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
artifact: research
---

# Research

`acceptance_input_digest` hashes every covered nonvolatile project path. Because `.specsync/change-sequence.json` is protected, an accepted record binds its then-current bytes even after `change new` legitimately advances the ledger.

`validate_change_sequences` already proves the ledger schema, numeric ID, maximum sequence, owner existence, collision uniqueness, exact collision membership, and historical immutability. Reusing that validation before projecting historical bytes avoids a parallel or weaker trust decision.
