---
change: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
artifact: design
---

# Design

One rule, one place. `normalize_actor` trims and refuses control characters, U+2028, U+2029,
U+202A to U+202E and U+2066 to U+2069. Each verb calls it at the top, before any lock-protected
move or write: approve (through `resolve_actor`), reopen, send-back, correct, correct-owner,
reconcile-history, retire and the doctor late close. Review keeps
`validate_scoped_reviewer_claim`, which is stricter. `approve_definition_with_relay` trims the
relay's decider so the equality the ledger reader enforces holds. Persisted reconciliation
records are validated as before, with no new character rule, so history is not re-judged.

List answers: `acceptance_criteria` and `affected_paths` refuse a parse with no value.
`affected_specs` and `dependencies` go through `clearable_list_answer`, where `none` clears
and an empty parse is refused with that hint.
