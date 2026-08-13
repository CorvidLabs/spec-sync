## ADDED

### REQUIREMENT REQ-types-005

`ValidationResult` SHALL record what validation was able to observe, so a reporter can
distinguish a check that passed from a check that did not run.

Acceptance Criteria
- Whether any mapped source file was present and readable is recorded.
- Whether the spec's Public API names at least one symbol is recorded.
- Both are recorded even when section and export validation are skipped.
