---
change: change-ship-status-exits-1-when-it-reports-a-blocker
artifact: delta
module: cmd_change
---

## ADDED

### REQUIREMENT REQ-cmd-change-039

`change ship-status` SHALL exit 1 whenever its report names a blocker, after printing the whole
report, for one change and for the roster. JSON output SHALL stay one document. A report with no
blocker SHALL exit 0 as before (#654).

#### Acceptance Criteria

- An approved change with no verification evidence: `change ship-status <id>` prints its
  `Blocker:` line and exits 1, and `--json` prints one document with a non-empty `blockers`
  list and exits 1.
- `change ship-status` with no id exits 1, in text and in JSON, when any listed change has a
  blocker.
- A change ready to finalize, and a closed-late change, exit 0 as before.
