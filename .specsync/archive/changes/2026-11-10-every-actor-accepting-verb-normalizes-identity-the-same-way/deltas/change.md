## MODIFIED

### REQUIREMENT REQ-change-150

Interview answers and identity strings SHALL be validated before they are persisted: `change answer` SHALL reject a question key outside the advertised interview set, SHALL reject an empty or whitespace-only answer to any question, SHALL reject an `affected_specs` entry that names no registered spec module, and actor identity strings on EVERY lifecycle verb that records them — approve, review, reopen, send-back, correct, correct-owner, reconcile-history — SHALL be trimmed and rejected when empty after trimming or when they contain newlines or control characters.

Acceptance Criteria
- An unknown question key exits 1, stores nothing in `state.json`, and the error lists the advertised question ids; this covers a hyphen/underscore typo such as `acceptance-criteria`.
- An empty or whitespace-only answer to any interview question exits 1 and stores nothing.
- An `affected_specs` answer naming an unregistered module exits 1 and names the registered modules.
- An `--actor` value with leading or trailing whitespace is stored trimmed; one that is empty after trimming or contains a newline or control character exits 1 before any ledger write, on every actor-accepting verb, and this holds for Git- or environment-derived actors as well as explicit flags.
