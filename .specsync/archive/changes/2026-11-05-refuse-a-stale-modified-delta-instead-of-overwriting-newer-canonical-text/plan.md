---
change: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
artifact: plan
---

# Plan

## Approach

Record the canonical base of every MODIFIED and REMOVED item at definition approval, and refuse to apply an item whose block has since moved and is not already this change's text.

## Out of scope

- Re-judging approvals recorded before this field (Leif chose to protect only new approvals).
- Re-scoping or withdrawing a change (#844).

## Steps

1. Write failing tests for a stale body applied before and after the first check.
2. Add `approved_delta_bases` and record it in every definition approval writer.
3. Refuse in `change check` and report in `change audit`.
4. Document the refusal in site/src/content/docs/deltas.md, and capture CHANGE-11 in hi.

## Risks

A legitimately rebased delta must be re-approved once. That is the remedy the refusal names, and it is covered by a test.

## Constraints consulted

REQ-change-089/090 (absent evidence is unknown), REQ-change-111 (the edited-body refusal runs first), never weaken a gate.
