---
change: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
artifact: context
---

# Context

PR #1026 (Fixes #426), round 4. Leif's quality bar asks for an independent review before
merge. A fresh reviewer at `b7c48a4b` requested changes at about 40% confidence:

- Must-fix: `approve --actor "  Ada  "` with a relayed decision recorded `actor: "Ada"` beside
  `relayed_by.decided_by: "  Ada  "`, a ledger every later read refused. Fixed in
  `approve_definition_with_relay` by trimming the relay's decider the same way.
- Retire and the doctor late close skipped the shared actor rule. Reopen moved an archived
  package before checking the actor. Reconcile-history's control-character check re-judged
  committed records.
- `[]` and `,` answers silently cleared required lists. `dependencies ''` was the only way to
  clear dependencies and is now refused, so `none` clears `affected_specs` and `dependencies`.
- The `affected_specs` refusal gave one misleading message for a missing canonical spec, a
  spec outside its canonical path, and an unreadable registry.
- U+2028 and bidi overrides passed the actor check.
- REQ-change-150, invariant 52 and the z=46 lesson claimed more than the code did.

REQ-change-150 is this PR's own unmerged requirement. Its delta brings the text in line with
the code. The coordinator directed that #1026 continue as a bug fix under Leif's rule.
