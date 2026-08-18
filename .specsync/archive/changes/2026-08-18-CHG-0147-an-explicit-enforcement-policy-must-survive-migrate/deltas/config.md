## ADDED

### REQUIREMENT REQ-config-012

Serialising configuration SHALL state an explicit enforcement policy rather than omitting it as equal to a default.

Acceptance Criteria
- A configuration written by the tool records its enforcement value in every case, so the effective policy does not depend on which default the reading binary carries.
- A project's effective enforcement is identical before and after migration, observable as an unchanged exit code from a check over a tree that contains a validation error.
- A project that never expressed an enforcement preference is unaffected.
- The documented default matches the value the type declares.
