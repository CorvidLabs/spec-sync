---
change: CHG-0111-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
artifact: requirements
---

# Requirements

## REQ-cmd-check-007

Validation of a project with no specs SHALL report the coverage it measured, and SHALL NOT
report a tree containing unmeasured source as clean under strict validation.

Acceptance Criteria
- The coverage figures are printed whenever there are no specs to validate, not only when a gate has already failed.
- Strict validation exits non-zero when the project contains source files and no specs.
- A project with no source files continues to exit zero under strict validation.
- The machine-readable payload carries the source-file count and coverage percent, so a project with unmeasured source is distinguishable from an empty one.
