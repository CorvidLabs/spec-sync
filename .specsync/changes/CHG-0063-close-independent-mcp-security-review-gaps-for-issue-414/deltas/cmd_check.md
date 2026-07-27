## MODIFIED

### REQUIREMENT REQ-cmd-check-001

Unified JSON checking SHALL preserve the documented top-level check schema when SDD validation or
coverage discovery fails.

Acceptance Criteria

- Failed SDD JSON output includes `passed`, `errors`, `warnings`, `stale`, and `specs_checked`.
- Structured SDD detail remains available as an additive field.
- Malformed manifest discovery exits nonzero and emits valid JSON with `passed: false`,
  `valid: false`, `inconclusive: true`, and an explicit error.

### SPEC SECTION Invariants

9. Coverage uses checked manifest discovery; malformed Gradle settings make the result inconclusive
   and exit 1 instead of producing partial or vacuous coverage.
