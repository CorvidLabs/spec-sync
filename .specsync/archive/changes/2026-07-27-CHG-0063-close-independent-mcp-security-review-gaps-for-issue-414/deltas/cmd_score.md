## MODIFIED

### REQUIREMENT REQ-cmd-score-001

The score command SHALL produce deterministic quality scores while honoring filters, formats, and
release gates.

Acceptance Criteria

- Checked coverage discovery succeeds before scoring gates are evaluated.
- Trustworthy warn-mode scoring remains advisory.
- Malformed Gradle/manifest discovery exits nonzero with parseable JSON containing `valid: false`,
  `inconclusive: true`, null score/grade, zero counts/distribution, an empty `specs` collection, and
  an explicit error.

### SPEC SECTION Invariants

5. Checked manifest discovery must succeed before coverage gates are evaluated; malformed Gradle
   settings are inconclusive and exit 1 even though ordinary scoring is advisory.
