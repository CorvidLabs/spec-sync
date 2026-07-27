## MODIFIED

### REQUIREMENT REQ-cmd-coverage-001

The coverage command SHALL report trustworthy file and LOC coverage and SHALL fail closed when
manifest discovery is inconclusive.

Acceptance Criteria

- Coverage is computed through `compute_coverage_checked`.
- Trustworthy zero-denominator coverage retains the documented 100% behavior.
- Malformed Gradle/manifest discovery exits 1 with valid JSON containing `valid: false`,
  `inconclusive: true`, null percentages, zero counts, empty collections, and an explicit error.

### SPEC SECTION Invariants

4. Checked manifest discovery must succeed before coverage can be trusted; malformed Gradle
   settings are inconclusive and exit 1.
