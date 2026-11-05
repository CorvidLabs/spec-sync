---
change: send-an-approved-change-back-to-draft-for-re-scoping
artifact: design
---

# Design

- `SendBackRecord` is appended to `ApprovalLedger.send_backs` (serde-optional, omitted when empty, so existing ledgers keep their bytes). It holds actor, reason, timestamp, `from_state`, `to_state` (draft), `cancelled_approvals` (the ledger's approval count at send-back) and whether canonical text had been applied, the items it had applied (`applied_items`) and the modules whose Change Log carried its row (`applied_modules`).
- `send_back_change` takes the project lock and writes `approvals.json`, `state.json` and `change.md` together. It sets the state to draft and `canonical_applied` to false.
- `effective_definition_approval` refuses when the latest definition approval's index is below the latest send-back's `cancelled_approvals`. Every gate that reads the definition approval therefore sees the cancellation, and nothing is deleted.
- Spec text the change already applied stays in place. The send-back lists it as `applied_items` (module, operation, target, key, body, and the block's pre-change text from the parent of the first commit carrying the change's Change Log row, or HEAD before that row is committed) for every module whose Change Log names the change; REMOVED items are listed while their block is still gone. `prepare_module_application` converges whenever the ledger holds a send-back, not only while `canonical_applied` is set, so a kept item already in the tree is skipped and the version bump and Change Log row are never repeated. A kept ADDED item whose block still carries the body a send-back listed is applied as a replacement.
- `ensure_no_orphaned_applied_text` runs in verify (so in `change check`) and in accept. It refuses while an item listed by a send-back is no longer declared by the delta (same module, target and key) and the canonical tree still carries what the change did (its body, or a removed block's absence), naming the spec file and the item and the text to restore. It also refuses a module dropped from `affected_specs` whose Change Log still carries the change's row.
- `ensure_committed_send_backs_kept` reads the ledger history with one cached `git log -G send_backs` per ledger path and requires the change's send-backs in every listed revision to be a prefix of the current ones. The project audit runs it for every change it audits, active or archived.
- `resolve_definition_approval_event` (historical and archived evidence), the scoped-review approver lookup and the scope-expansion summary skip approvals the send-backs cancelled.
- `ensure_committed_send_backs_kept` runs with the definition gate: when `approvals.json` is committed at HEAD, its send-backs must be an unchanged prefix of the current ones, so deleting or editing a committed send-back cannot bring the cancelled approval back.
- CLI: a `ChangeAction::SendBack { id, actor, reason }` command that renders text or JSON.
