## ADDED

### REQUIREMENT REQ-scoring-002

The freshness dimension SHALL withhold points it could not measure.

Acceptance Criteria
- Unmeasurable git freshness withholds its points rather than awarding them.
- Whether the git half was measured, not applicable, or withheld is recorded so consumers do not withhold twice.
- Removing git history cannot raise a spec's grade.
