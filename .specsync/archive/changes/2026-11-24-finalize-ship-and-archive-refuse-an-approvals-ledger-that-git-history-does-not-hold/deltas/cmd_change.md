---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: delta
module: cmd_change
---

## ADDED

### REQUIREMENT REQ-cmd-change-committed-ledger-001

`change ship-status` SHALL list the refusal of REQ-change-166 as a blocker, with the same message, and SHALL NOT report the change ready to finalize while it holds. `change ship --dry-run` SHALL refuse with that message.

Acceptance Criteria
- With one approval's actor rewritten in the working tree after review, `change ship-status <id>` lists the blocker, reports `ready_to_finalize` false and exits 1, and `change ship <id> --dry-run` refuses with the same message.
- With the file restored, `ready_to_finalize` is true again.
