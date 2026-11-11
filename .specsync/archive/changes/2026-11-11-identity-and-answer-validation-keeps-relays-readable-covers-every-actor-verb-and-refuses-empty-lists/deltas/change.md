---
change: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-150

Interview answers and identity strings SHALL be validated before they are persisted. `change answer` SHALL reject a question key outside the interview set `change answer` accepts, SHALL reject an empty or whitespace-only answer and an answer that parses to no value, and SHALL reject an `affected_specs` entry whose module has no spec at its canonical path, unless the project has no specs at all. `none` SHALL clear `affected_specs` and `dependencies`. Actor identity strings SHALL be trimmed and rejected when empty after trimming or when they contain control characters, Unicode line or paragraph separators, or bidirectional overrides or isolates. This SHALL hold on approve, reopen, send-back, correct, correct-owner, reconcile-history, retire and the doctor late close through one shared `normalize_actor`, and on review through its stricter reviewer-claim check. A relayed approval's decider SHALL be trimmed the same way as the actor it must match.

Acceptance Criteria
- An unknown question key exits 1, stores nothing in `state.json`, and the error lists the accepted question ids; this covers a hyphen/underscore typo such as `acceptance-criteria`.
- An empty or whitespace-only answer, or one that parses to no value (`[]`, `,`), exits 1 and stores nothing; `none` clears `affected_specs` and `dependencies`.
- An `affected_specs` answer naming a module with no spec at its canonical path exits 1, names that path and `specsync new <module>` or the known modules; an unreadable registry is reported as such.
- An `--actor` value with leading or trailing whitespace is stored trimmed; one that is empty after trimming or holds a refused character exits 1 before any ledger write, for explicit, Git- and environment-derived actors.
- A relayed approval with a padded `--actor` records the same trimmed name as actor and decider, and `change status` reads it.
