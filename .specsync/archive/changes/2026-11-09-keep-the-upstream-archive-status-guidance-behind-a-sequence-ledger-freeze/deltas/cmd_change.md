## ADDED

### REQUIREMENT REQ-cmd-change-034

Text `change status` and the `change ship-status` lifecycle next action SHALL name a sequence-ledger freeze before the upstream-archive step, matching JSON status.

Acceptance Criteria
- With an upstream archive and a frozen ledger, the text `Next:` line names the freeze.
