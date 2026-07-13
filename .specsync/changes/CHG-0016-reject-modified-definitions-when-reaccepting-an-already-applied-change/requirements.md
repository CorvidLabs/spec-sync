---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: requirements
---

# Requirements

### REQ-change-017

The lifecycle SHALL preserve audited contract identity when reaccepting an already-applied change.

Acceptance Criteria
- A reopened change with an unchanged definition can verify and reaccept without applying its canonical delta twice.
- A reopened change whose definition digest differs from the latest pre-reopen verification contract is rejected even after the modified definition is approved and verified.
- The rejection occurs before a closing approval or canonical write and directs further spec work to a new change workspace.
- A verifying `canonical_applied` record without audited reopen history fails closed.
