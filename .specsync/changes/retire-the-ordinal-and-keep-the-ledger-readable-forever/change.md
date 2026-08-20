---
id: retire-the-ordinal-and-keep-the-ledger-readable-forever
state: implementing
type: refactor
base_commit: b3f3201aaa7f924ec4d6e4368b02afa6e2c87ded
---

# Retire the ordinal and keep the ledger readable forever

## Intent

retire the ordinal and keep the ledger readable forever

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Steps one through four carried a slug-only identity all the way through the lifecycle: on the previous binary a hand-converted slug-only workspace already passes list, status, show, answer, approve, check, review and finalize. Three surfaces still fail, and all three through one line in located_change_sequences that turns an ID carrying no ordinal into invalid change ID: change audit cannot even count the change, change new is dead repo-wide, and change status reports a healthy next action while both are bricked because sequence_ledger_freeze_next_action ends Err underscore arrow None. Separately the allocator still mints CHG-NNNN, and its retry loop escapes a taken directory by incrementing the ordinal, so with no ordinal it would retry the same path ten thousand times and report exhausted change sequence allocation retries for what is really a repeated description. And the ordinal was quietly the only guarantee that two changes cannot share an identity: remove it with no replacement and two same-named packages report audit passed. Done when: change new mints a slug; an ID that claims no ordinal takes part in no numeric accounting while one that claims an ordinal badly still fails closed; a repeated description is refused by name rather than by a retry-exhaustion message; an explicit duplicate-identity gate replaces what the numeric gate was doing by accident; the sequence ledger stops growing but stays readable and reconcilable for the hundred and twenty archives that signed it; and no archive changes validity.

## No-spec Rationale

Not applicable
