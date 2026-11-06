## MODIFIED

### REQUIREMENT REQ-change-123


`change send-back` SHALL move an approved, implementing or verifying change back to draft so its scope questions can be answered again. It SHALL require a non-empty human actor and reason, SHALL append both, with the prior state, to `approvals.json` under `send_backs`, and SHALL cancel the definition approval recorded before it without removing any recorded approval, verification or other evidence. A definition approval recorded before the latest send-back SHALL NOT satisfy the definition gate or bind a scoped review; only a fresh `change approve` does. A scoped review recorded before the send-back SHALL stay in the append-only review history and SHALL still validate against the approval it bound, but SHALL NOT count as current; it SHALL NOT stop a fresh `change review` after re-approval, and `change status` SHALL report it stale rather than unreadable. A send-back the committed ledger already holds SHALL NOT be removable or editable to bring a cancelled approval back. A send-back committed in any revision of the ledger SHALL NOT be removable or editable later, and the project audit SHALL report one that was. The send-back SHALL list the spec items the change had already applied, including REMOVED items, with each block's text from before the change, and the modules whose Change Log carries the change's row. After a send-back, materialization SHALL converge on what the canonical tree already carries, so a kept REMOVED, ADDED or MODIFIED item is not applied twice and a kept ADDED item may change its body. `change check` and `change accept` SHALL refuse, naming the spec file and the item and saying what to restore, while an item the new scope no longer declares is still applied, or a module it no longer declares still has the change's Change Log row.

#### Acceptance Criteria

- A verifying change sent back can have `affected_paths` answered again, is approved, checked, verified and accepted under its own ID, and an already-applied section is not applied twice.
- A missing actor or reason is refused, naming `--actor` or `--reason`, and nothing is written.
- The send-back record keeps the actor, reason and prior state; earlier approvals and `verification.json` are unchanged; a second send-back appends.
- Until re-approval the definition gate refuses, says the change was sent back, and names `specsync change approve <id>`.
- A draft, accepted or archived change cannot be sent back.
- A sent-back change that owns a spec can be finalized and its archive passes the project audit.
- A re-scope that drops an item the change already applied is refused by check, naming the spec file and the item, until that text is taken out of the spec.
- The definition approval event resolver refuses an approval that a send-back cancelled.
- Removing a committed send-back from `approvals.json` is refused and does not bring the cancelled approval back.
- After a send-back, a change with a REMOVED item materializes and checks again (0xLeif's repro), and a kept ADDED item with a new body replaces the change's earlier body.
- A dropped MODIFIED item is refused with advice to restore its pre-change text, quoting it; a dropped REMOVED item is listed and refused until the block is restored.
- A module dropped from `affected_specs` is refused while its Change Log keeps the change's row.
- A send-back deleted in a later commit is refused by check and reported by the project audit.
- A change reviewed before its send-back is re-approved, re-checked and reviewed again, and finalize archives it with both review attempts in its history, the first unchanged (#914).
- After a send-back, `change status` reports the earlier review stale with the re-review next action, and `change ship` refuses it, even when the re-approved definition is byte-identical.
- A review bound only to an approval a send-back cancelled is never current.
