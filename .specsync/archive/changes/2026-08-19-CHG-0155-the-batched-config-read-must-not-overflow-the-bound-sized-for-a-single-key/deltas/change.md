## ADDED

### REQUIREMENT REQ-change-077

A bounded Git read SHALL be bounded for the response it can actually receive, not for the response the call it replaced received.

Acceptance Criteria
- Reading the effective checkout overrides succeeds when the four core keys are set in more than one configuration scope, the ordinary layout of a global file plus a repository-local override.
- The values derived equal what a separate per-key query returns for each key, compared against that query rather than against an assumption about which scope takes precedence.
- A genuinely unbounded response is still refused, so the deterministic-output guard is retained rather than removed.
