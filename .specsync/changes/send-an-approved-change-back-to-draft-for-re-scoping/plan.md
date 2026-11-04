---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: plan
---

# Plan

## Approach

Add an audited `change send-back` that returns an approved, implementing or verifying change to draft and cancels its earlier approvals without deleting them.

## Out of scope

- Option B, a withdrawn end state and supersede of a withdrawn change. Its criteria are not confirmed yet.
- Undoing spec text the change already applied.
- Accepted or archived changes, which use `change reopen`.

## Steps

1. Write failing tests for CHANGE-12, 12.a, 12.b and 12.c.
2. Add `SendBackRecord`, `send_back_change` and the cancellation check.
3. Wire the CLI and document it in the CLI and workflow guides.
4. Capture CHANGE-12 in hi and record requirement evidence.

## Risks

- A re-scope that drops an item the change already applied would leave orphan text in the spec. Check and accept refuse and name it until the text is taken out or the item is kept (review round on e52f77d). Review round two (6e9abc9): REMOVED items are listed too, the advice restores pre-change text, a dropped module's Change Log row is refused, and materialization converges after a send-back.
- A ledger edited before its first commit cannot be compared with history; the committed-ledger comparison covers every send-back once committed.
- Field growth on `ApprovalLedger` is read-tolerant and omitted when empty.

## Constraints consulted

CHANGE-6.c (changing scope costs a fresh approval), append-only audit evidence as in `reopen`, never weaken a gate.
