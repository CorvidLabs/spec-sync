---
change: every-actor-accepting-verb-normalizes-identity-the-same-way
artifact: testing
---

# Testing


Integration: `change send-back --actor "Ada\nInjected"` fails and approvals.json is unchanged;
reopen/correct-owner reject the same. Padded actor is stored trimmed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `change_approve_trims_and_validates_actor_identity` (extended: send-back with a newline actor fails and the ledger is byte-unchanged); unit send_back suite 10/10 |
