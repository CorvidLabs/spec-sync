## ADDED

### REQUIREMENT REQ-validator-013

Retained configuration discovery SHALL record a parse failure rather than returning defaults
indistinguishable from a successful load.

Acceptance Criteria
- A config file that fails to parse during discovery records the failure alongside the defaults it fell back to.
- Source directory discovery is unaffected.
