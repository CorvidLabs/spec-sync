## ADDED

### REQUIREMENT REQ-change-069

Declaring an additional affected module SHALL never remove a verification command from what a change receives.

Acceptance Criteria
- The command set selected for a scope is a superset of the set selected for any subset of that scope, so widening declared scope can only add verification.
- A declared module with no component routing entry contributes the project-wide verification commands, because a module nobody routed is not a module that needs no verification.
- A change scoped entirely to routed modules still receives only its component commands, so targeted verification remains available.
- A change declaring no affected module still receives the project-wide verification commands.
- Strict escalation continues to append its own commands without removing any already selected.
