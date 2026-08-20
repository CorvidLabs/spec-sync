---
change: CHG-0164-a-change-without-an-ordinal-must-not-brick-the-workspace
artifact: context
---

# Context

A landmine shipped by CHG-0162 (`65755ac7`), found while scoping the sequence-ledger retirement
rather than by anyone hitting it.

CHG-0162 relaxed `validate_change_id` so an identity without the `CHG-NNNN` prefix is legal —
which is the whole point of the identity work. What it did not do is check whether anything
downstream still required the ordinal. One place did, and it is on the path of both
`change audit` and `change new`.

The failure is worse than an error message. `change status` reports every change as healthy,
because `sequence_ledger_freeze_next_action` pattern-matches known error strings and ends
`Err(_) => None` — so an unrecognised error produces a clean next-action. The project looks fine
and cannot create another change.

This is the sibling-site pattern once more: the fix landed where the report pointed
(`validate_change_id`) and a second validator with the same job survived beside it, stricter by
accident rather than by decision.
