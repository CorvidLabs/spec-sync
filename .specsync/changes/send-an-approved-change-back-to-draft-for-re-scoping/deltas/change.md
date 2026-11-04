## ADDED

### REQUIREMENT REQ-change-123

`change send-back` SHALL move an approved, implementing or verifying change back to draft so its scope questions can be answered again. It SHALL require a non-empty human actor and reason, SHALL append both, with the prior state, to `approvals.json` under `send_backs`, and SHALL cancel every approval recorded before it without removing any recorded approval, verification or other evidence. A definition approval recorded before the latest send-back SHALL NOT satisfy the definition gate; only a fresh `change approve` does.

#### Acceptance Criteria

- A verifying change sent back can have `affected_paths` answered again, is approved, checked, verified and accepted under its own ID, and an already-applied section is not applied twice.
- A missing actor or reason is refused, naming `--actor` or `--reason`, and nothing is written.
- The send-back record keeps the actor, reason and prior state; earlier approvals and `verification.json` are unchanged; a second send-back appends.
- Until re-approval the definition gate refuses, says the change was sent back, and names `specsync change approve <id>`.
- A draft, accepted or archived change cannot be sent back.
- A sent-back change can be finalized and its archive passes the project audit.
