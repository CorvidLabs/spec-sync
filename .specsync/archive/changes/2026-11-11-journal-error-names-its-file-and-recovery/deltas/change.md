---
module: change
---

## ADDED

### REQUIREMENT REQ-change-151

A corrupt or unreadable transaction journal SHALL be reported with its file
path and the recovery (inspect it, then set it aside when no write was in
flight); a refusal that names `change reopen` SHALL print the complete command,
including `--actor <name>` and `--reason <text>`.

#### Acceptance Criteria

- A corrupt `.specsync/change-transaction.json` produces an error naming the
  file and the set-aside recovery.
- Every printed `specsync change reopen <id>` remediation includes
  `--actor <name> --reason <text>`.
