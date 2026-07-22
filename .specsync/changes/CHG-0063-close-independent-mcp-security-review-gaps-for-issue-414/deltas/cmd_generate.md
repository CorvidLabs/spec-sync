## MODIFIED

### REQUIREMENT REQ-cmd-generate-001

The generate command SHALL create deterministic local specs only from trustworthy discovery.

Acceptance Criteria

- All generation modes use checked coverage discovery before selecting output.
- Malformed Gradle/manifest discovery exits nonzero before mutation.
- JSON mode remains parseable with `valid: false`, `inconclusive: true`, an explicit error, and an
  empty `generated` collection.

### SPEC SECTION Invariants

5. Checked manifest discovery must succeed before generation; malformed Gradle settings are
   inconclusive, produce no files, and exit 1.
