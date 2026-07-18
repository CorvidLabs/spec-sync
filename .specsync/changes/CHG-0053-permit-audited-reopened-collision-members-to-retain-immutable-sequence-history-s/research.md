---
change: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
artifact: research
---

# Research

`located_change_sequences` currently derives collision immutability only from the live enum state.
`reopen_change` already preserves the prior verification and superseded closing approval in an
append-only `ReopenRecord`, while `ensure_reopened_definition_unchanged` validates the
definition-bound recovery path. These existing records are sufficient to distinguish an audited
delivery refresh from an unaccepted mutable collision.

The collision check remains an ordering guard, not a replacement for terminal-evidence validation;
forced strict continues to authenticate the reaccepted records after refresh.
