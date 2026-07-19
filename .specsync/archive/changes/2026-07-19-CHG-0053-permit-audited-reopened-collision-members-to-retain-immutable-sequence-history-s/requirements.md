---
change: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
artifact: requirements
---

# Requirements

Acknowledged immutable collisions SHALL support the existing audited delivery-reopen lifecycle
without renumbering, deleting, or weakening either accepted history.

- Only an already-applied record in `verifying` is eligible.
- The append-only reopen event must bind the exact prior verification and closing approval.
- The accepted definition must remain unchanged.
- Reacceptance still requires fresh verification and a new human closing approval.
