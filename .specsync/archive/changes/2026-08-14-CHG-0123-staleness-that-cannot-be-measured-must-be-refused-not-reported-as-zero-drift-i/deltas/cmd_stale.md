## ADDED

### REQUIREMENT REQ-cmd-stale-003

`stale` SHALL derive its precondition from the shared helper.

Acceptance Criteria
- Existing messages, JSON shape and exit codes are byte-identical to before.
- The duplicated precondition logic is removed, so no reader carries its own copy.
