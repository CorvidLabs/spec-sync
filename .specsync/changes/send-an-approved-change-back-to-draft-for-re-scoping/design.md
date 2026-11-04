---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: design
---

# Design

- `SendBackRecord` is appended to `ApprovalLedger.send_backs` (serde-optional, omitted when empty, so existing ledgers keep their bytes). It holds actor, reason, timestamp, `from_state`, `to_state` (draft), `cancelled_approvals` (the ledger's approval count at send-back) and whether canonical text had been applied.
- `send_back_change` takes the project lock and writes `approvals.json`, `state.json` and `change.md` together. It sets the state to draft and `canonical_applied` to false.
- `effective_definition_approval` refuses when the latest definition approval's index is below the latest send-back's `cancelled_approvals`. Every gate that reads the definition approval therefore sees the cancellation, and nothing is deleted.
- Spec text the change already applied stays in place. The next check converges or applies from the canonical tree.
- CLI: a `ChangeAction::SendBack { id, actor, reason }` command that renders text or JSON.
