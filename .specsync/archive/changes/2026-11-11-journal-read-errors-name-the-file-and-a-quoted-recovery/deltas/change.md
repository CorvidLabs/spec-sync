---
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-151

A transaction journal that cannot be read or parsed SHALL be reported with its
file path and a command that sets it aside. Every printed `change reopen`
remediation SHALL include `--actor` and `--reason`.

#### Acceptance Criteria

- A corrupt `.specsync/change-transaction.json` produces an error naming the
  file and the set-aside recovery.
- A non-UTF-8 journal is reported the same way.
- The printed recovery command runs verbatim in a project whose path contains
  a space.
- Every printed `specsync change reopen <id>` remediation includes
  `--actor <name> --reason <text>`.
