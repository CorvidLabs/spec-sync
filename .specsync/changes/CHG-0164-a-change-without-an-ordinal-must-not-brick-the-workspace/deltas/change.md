## ADDED

### REQUIREMENT REQ-change-086

A change whose identity carries no ordinal SHALL be absent from ordinal collision detection rather than fatal to it.

Acceptance Criteria
- A workspace holding a change with no ordinal continues to enumerate, audit, and create further changes, because an identity the tool accepts on load must not be one it refuses on enumeration.
- Two changes claiming the same ordinal are still refused, so tolerating an absent ordinal does not disable the check that reading ordinals exists for.
