---
id: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
state: archived
type: bug_fix
base_commit: b7c48a4b945ffebda99940ff6b379896083bc73f
---

# Identity and answer validation keeps relays readable, covers every actor verb and refuses empty lists

## Intent

identity and answer validation keeps relays readable, covers every actor verb and refuses empty lists

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a relayed approval with a padded actor stays readable; retire, doctor late close, reopen and reconcile-history normalize actors before any write and reopen does so before moving an archive; Unicode line separators and bidi overrides are refused in actors; empty-list answers are refused and none clears affected_specs and dependencies; affected_specs refusals name the canonical path or the unreadable registry; REQ-change-150, invariant 52 and the lesson match the code

## No-spec Rationale

Not applicable
