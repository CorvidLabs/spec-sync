## ADDED

### REQUIREMENT REQ-output-005

Text output SHALL state that nothing was measured rather than print a percentage.

Acceptance Criteria
- A zero denominator prints the measured counts and names the reason.
- The renderer derives from the shared accessor rather than re-computing the ratio.
