## ADDED

### REQUIREMENT REQ-cmd-scaffold-008

`scaffold` SHALL check a new module name for a case collision only against spec folders the project rule reaches (REQ-commands-016).

Acceptance Criteria
- `scaffold auth` with a `specs` link out holding `Auth` reports no collision from outside.
