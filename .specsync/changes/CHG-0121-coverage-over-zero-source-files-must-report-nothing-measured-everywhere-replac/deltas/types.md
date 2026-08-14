## ADDED

### REQUIREMENT REQ-types-008

`CoverageReport` SHALL NOT carry a precomputed coverage percentage.

Acceptance Criteria
- Percentages are exposed as `Option`, and are `None` when the denominator is zero.
- "Nothing was measured" is distinguishable from "measured, and the result is zero".
- No field holds a value that a caller could read without deciding what an unmeasured tree renders as.
