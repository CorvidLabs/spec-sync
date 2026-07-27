## MODIFIED

### REQUIREMENT REQ-cmd-report-001

The report command SHALL provide a trustworthy project/module health view and SHALL fail closed when
manifest discovery is inconclusive.

Acceptance Criteria

- Overall coverage uses `compute_coverage_checked`.
- Malformed Gradle/manifest discovery exits nonzero before partial report rendering.
- JSON remains parseable with `valid: false`, `inconclusive: true`, null overall coverage, zero
  counts, an empty `modules` collection, and an explicit error.

### SPEC SECTION Invariants

7. Checked manifest discovery must succeed before project coverage is reported; malformed Gradle
   settings are inconclusive and exit 1.
