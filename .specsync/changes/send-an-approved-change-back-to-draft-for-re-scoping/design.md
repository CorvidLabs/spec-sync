---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: design
---

# Design

- `SendBackRecord` is appended to `ApprovalLedger.send_backs` (serde-optional, omitted when empty, so existing ledgers keep their bytes). It holds actor, reason, timestamp, `from_state`, `to_state` (draft), `cancelled_approvals` (the ledger's approval count at send-back) and whether canonical text had been applied.
- `send_back_change` takes the project lock and writes `approvals.json`, `state.json` and `change.md` together. It sets the state to draft and `canonical_applied` to false.
- `effective_definition_approval` refuses when the latest definition approval's index is below the latest send-back's `cancelled_approvals`. Every gate that reads the definition approval therefore sees the cancellation, and nothing is deleted.
- Spec text the change already applied stays in place. The send-back lists it as `applied_items` (module, target, key, body) for every module whose Change Log names the change. The next check converges or applies from the canonical tree.
- `ensure_no_orphaned_applied_text` runs in verify (so in `change check`) and in accept. It refuses while an item listed by a send-back is no longer declared by the delta (same module, target and key) and the canonical tree still carries the body the change applied, naming the spec file and the item.
- `resolve_definition_approval_event` (historical and archived evidence), the scoped-review approver lookup and the scope-expansion summary skip approvals the send-backs cancelled.
- `ensure_committed_send_backs_kept` runs with the definition gate: when `approvals.json` is committed at HEAD, its send-backs must be an unchanged prefix of the current ones, so deleting or editing a committed send-back cannot bring the cancelled approval back.
- CLI: a `ChangeAction::SendBack { id, actor, reason }` command that renders text or JSON.
