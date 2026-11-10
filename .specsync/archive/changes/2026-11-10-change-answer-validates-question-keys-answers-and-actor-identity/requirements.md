---
change: change-answer-validates-question-keys-answers-and-actor-identity
artifact: requirements
---

# Requirements

- `change answer <id> <unknown-key> <value>` exits 1 and stores nothing; the error lists the advertised question ids.
- An empty or whitespace-only answer exits 1 for every interview question.
- `--actor` (and Git/env-derived actors) are trimmed; empty-after-trim or names with newlines/control characters are rejected before any ledger write.
- `affected_specs` entries must name modules present in the spec registry; unknown modules exit 1 and name the known modules.
