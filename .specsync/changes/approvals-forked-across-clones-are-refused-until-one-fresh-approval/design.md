---
change: approvals-forked-across-clones-are-refused-until-one-fresh-approval
artifact: design
---

# Design

`ensure_definition_approval_valid`, which `change check`, `change audit` and the status summary
use, gains `ensure_approval_history_not_forked`, beside the send-back history check:

- `git log --full-history --no-renames HEAD -- :(top,literal)<approvals.json>` lists every commit
  that touched the change's ledger, on both sides of every merge, for its active and archived
  paths. Each version is read, with `relayed_by` left out of every approval.
- Two versions fork when, within their shared length, an approval differs: each holds an approval
  the other lacks. A version that only appends to another is no fork.
- A fork is settled when the definition approval that counts now, the last one in the current
  ledger, appears in neither forked version, so it was recorded after the merge. Otherwise the
  refusal names both commits, the approvers at the point they diverge, and
  `specsync change approve <id> --actor <name>`.
- `change approve` does not run this validation, so the recovery is always available.
- An archived change is not judged again: its finalization evidence binds the ledger it closed
  on and it can no longer be approved, so a fork its history settled must not fail every later
  audit. `change audit --strict --archives` on main passes with this build.

Rejected: a network lock (the issue makes it optional; it cannot cover offline clones), and
refusing at merge time (SpecSync does not run at merge).
