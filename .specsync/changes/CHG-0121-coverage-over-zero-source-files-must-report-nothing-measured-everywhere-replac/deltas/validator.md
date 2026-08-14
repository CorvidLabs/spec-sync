## ADDED

### REQUIREMENT REQ-validator-014

Coverage computation SHALL NOT substitute a value for a ratio whose denominator is zero.

Acceptance Criteria
- A zero denominator yields the absent value, never a default of 100 or 0.
- The substituting expressions are removed rather than corrected, leaving no site to re-derive.
