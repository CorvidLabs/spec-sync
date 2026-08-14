## ADDED

### REQUIREMENT REQ-cmd-lifecycle-002

The `no_stale` guard SHALL NOT pass when staleness is unverifiable.

Acceptance Criteria
- A promotion gated on absence of staleness is blocked when git cannot answer, and the blocker names the reason.
